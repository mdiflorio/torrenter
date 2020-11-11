use std::collections::hash_map::OccupiedEntry;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::net::{Shutdown, TcpStream};

use anyhow::{anyhow, Result};
use bytebuffer::ByteBuffer;
use tokio::sync::mpsc::Sender;

use crate::download::PiecesManager;
use crate::messages;
use crate::messages::{GenericPayload, parse};
use crate::pieces::Pieces;
use crate::queue::{PieceBlock, Queue};
use crate::utils::torrents::Torrent;

pub struct PieceChannelPayload {
    pub offset: u64,
    pub block: Vec<u8>,
}

pub struct MessageHandler<'a> {
    torrent: &'a Torrent,
    stream: &'a mut TcpStream,
    file_sender: Sender<PieceChannelPayload>,
    pieces: PiecesManager,
    queue: &'a mut Queue<'a>,
}

impl MessageHandler<'_> {
    pub fn new<'a>(torrent: &'a Torrent, stream: &'a mut TcpStream, file_sender: Sender<PieceChannelPayload>, pieces: PiecesManager, queue: &'a mut Queue<'a>) -> MessageHandler<'a> {
        MessageHandler {
            torrent,
            stream,
            file_sender,
            pieces,
            queue,
        }
    }

    /// Route and parse all the messages.
    /// Each message will be routed to their corresponding handler.
    ///
    ///     0 : choke
    ///     1 : unchoke
    ///     4 : have
    ///     5 : bitfield
    ///     7 : piece
    ///
    pub async fn router(&mut self, msg: ByteBuffer) -> Result<()> {
        if msg.len() == 0 {
            return Err(anyhow!("Peer connection closed"));
        }

        let parsed_msg = parse(msg);

        match parsed_msg.id {
            0 => self.choke(),
            1 => self.unchoke(),
            4 => self.have(parsed_msg.payload),
            5 => self.bitfield(parsed_msg.payload),
            7 => {
                self.piece(parsed_msg.payload).await;
            },
            _ => {
                println!("Unknown message ID: {:?}", parsed_msg.id);
            }
        }

        return Ok(());
    }


    /// Get an entire message from a peer.
    ///
    /// Even though pieces should be around 16kB, it's possible that they can be larger.
    /// If this is the case, this function will fail to receive a piece.
    pub fn get_whole_msg(&mut self) -> ByteBuffer {
        let mut whole_msg: ByteBuffer = ByteBuffer::new();

        // Get the length from the first message.
        let buf: &mut [u8; 1028 * 36] = &mut [0; 1028 * 36];
        let len = self.stream.read(buf).expect("Unable to receive from peer");
        whole_msg.write_bytes(&buf[0..len]);

        return whole_msg;
    }


    /// Establish the initial contact with a peer, immediately afterwards we send an intersted message.
    pub fn handshake(&mut self) {
        let buf: &mut [u8; 1028] = &mut [0; 1028];
        self.stream.read(buf).expect("Handshake has failed");
        self.interested();
    }

    /// Let the peer know we're interesting in communicating.
    pub fn interested(&mut self) {
        let send_msg = messages::build_interested();
        self.stream.write(&send_msg.to_bytes()).expect("Unable to send interested");
        println!("SENT INTERESTED!");
    }

    /// The peer has stopped communication with us
    fn choke(&mut self) {
        println!("CHOKED");
        self.stream.shutdown(Shutdown::Both).expect("The peer has choked us");
    }

    /// Start to requst pieces from a peer
    fn unchoke(&mut self) {
        println!("UNCHOKING");
        self.queue.choked = false;
        self.request_piece();
    }


    /// A peer has indicted that they have a certain piece.
    fn have(&mut self, payload: GenericPayload) {
        println!("HAVE");
        let piece_index = payload.index;
        let queue_empty = self.queue.len() == 0;

        self.queue.queue(piece_index as u64);
        if queue_empty {
            self.request_piece()
        }
    }

    /// Handle bitfield messages which indicate which are the pieces that the peer has.
    ///
    /// For example, the a bitfield of 01111 indicates that the peer is missing the first piece but has all the others.
    ///
    fn bitfield(&mut self, payload: GenericPayload) {
        println!("BITFIELD");

        let bf = payload.bitfield.as_ref().unwrap().to_bytes();

        let available_pieces = parse_bitfield(bf);

        // Add piece indexes to the download queue
        for piece_index in available_pieces {
            self.queue.queue(piece_index);
        }
    }


    /// Handle piece message
    ///
    /// - Add piece to the recieved vec
    /// - Write to file
    /// - Request new pieces if not finished
    async fn piece(&mut self, payload: GenericPayload) {

        let piece_block = PieceBlock {
            index: payload.index as u64,
            begin: payload.begin as u64,
            length: None,
        };


        // Calculate the index offset on where we have to write the received piece.
        let offset = payload.index as u64 * self.torrent.info.piece_length + payload.begin as u64;

        let payload = PieceChannelPayload {
            offset,
            block: payload.block.unwrap().to_bytes(),
        };

        let mut download_finished: bool;

        {
            let mut pieces = self.pieces.lock().unwrap();
            pieces.add_received(piece_block.clone());
        }

        {
            // Send message to the channel
            self.file_sender.send(payload).await;
        };

        {
            let mut pieces = self.pieces.lock().unwrap();
            download_finished = pieces.is_done();
        }

        // Shutdown if finished
        if download_finished {
            println!("Torrent downloaded!");
            self.stream.shutdown(Shutdown::Both).expect("Unable to shutdown stream");

            // Otherwise, request new pieces
        } else {
            self.request_piece();
        }
    }


    /// Request the first block in the job queue.
    fn request_piece(&mut self) {

        // Don't request anything if we're choked.
        // TODO: Add error handling to retry if we're choked.
        if self.queue.choked {
            println!("We're choked!");
            return;
        }

        let mut pieces = self.pieces.lock().unwrap();

        while self.queue.len() > 0 {

            // Grab the first piece in the queue
            let piece_block = self.queue.deque().unwrap();

            // Check if that piece is still needed and request if so
            if pieces.needed(piece_block) {
                let request = messages::build_request(piece_block);
                self.stream.write(&*request.to_bytes());
                pieces.add_requested(piece_block);

                break;
            }
        }
    }
}

/// Parse the bitfield.
///
///     For example: a bitfield of 255 is 1111 1111 in binary
///     This means that the peer has the pieces 8 pieces
///
///     A bitfield of 1111 1110 means that the peer has 7 pieces, excluding the last piece.
///     A bitfield of 0111 1111 means that the first piece is missing.
fn parse_bitfield(bitfield: Vec<u8>) -> Vec<u64> {
    let mut piece_indexes: Vec<u64> = Vec::new();

    // Iterate over all bytes
    for (i, b) in bitfield.iter().enumerate() {
        let mut byte = b.clone();

        // Iterate over each bit
        for j in 0..8 {
            // Add the pieces to the job queue.
            if byte % 2 > 0 {
                piece_indexes.push((i * 8 + 7 - j) as u64);
            }
            byte = byte / 2;
        }
    }

    return piece_indexes;
}


#[test]
fn test_parse_bitfield() {
    let bitfield: Vec<u8> = vec![127];
    let piece_indexes = parse_bitfield(bitfield);
    assert_eq!(piece_indexes, vec![7, 6, 5, 4, 3, 2, 1]);


    let bitfield: Vec<u8> = vec![255];
    let piece_indexes = parse_bitfield(bitfield);
    assert_eq!(piece_indexes, vec![7, 6, 5, 4, 3, 2, 1, 0]);
}

