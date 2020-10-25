use std::collections::hash_map::OccupiedEntry;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::net::{Shutdown, TcpStream};

use anyhow::{anyhow, Result};
use bytebuffer::ByteBuffer;

use crate::messages;
use crate::messages::{GenericPayload, parse};
use crate::pieces::Pieces;
use crate::queue::{PieceBlock, Queue};
use crate::utils::torrents::Torrent;

pub struct MessageHandler<'a> {
    torrent: &'a Torrent,
    stream: &'a mut TcpStream,
    file: &'a mut File,
    pieces: &'a mut Pieces,
    queue: &'a mut Queue<'a>,
}

impl MessageHandler<'_> {
    pub fn new<'a>(torrent: &'a Torrent, stream: &'a mut TcpStream, file: &'a mut File, pieces: &'a mut Pieces, queue: &'a mut Queue<'a>) -> MessageHandler<'a> {
        MessageHandler {
            torrent,
            stream,
            file,
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
    pub fn router(&mut self, mut msg: &mut ByteBuffer) -> Result<()> {
        if msg.len() == 0 {
            return Err(anyhow!("Peer connection closed"));
        }

        let parsed_msg = parse(&mut msg);

        match parsed_msg.id {
            0 => self.choke(),
            1 => self.unchoke(),
            4 => self.have(&parsed_msg.payload),
            5 => self.bitfield(&parsed_msg.payload),
            7 => self.piece(&parsed_msg.payload),
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
        let buf: &mut [u8; 1028 * 16] = &mut [0; 1028 * 16];
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
    fn have(&mut self, payload: &GenericPayload) {
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
    fn bitfield(&mut self, payload: &GenericPayload) {
        println!("BITFIELD");

        let bf = payload.bitfield.as_ref().unwrap().to_bytes();
        let queue_empty = self.queue.len() == 0;

        // Iterate over all bytes and each bit in the bytes
        for (i, b) in bf.iter().enumerate() {
            let mut byte = b.clone();


            for j in 0..8 {
                // Add the pieces to the job queue.
                if byte % 2 > 0 {
                    self.queue.queue((i * 8 + 7 - j) as u64)
                }
                byte = byte / 2;
            }
        }

    }

    /// Handle piece message
    ///
    /// - Add piece to the recieved vec
    /// - Write to file
    /// - Request new pieces if not finished
    fn piece(&mut self, payload: &GenericPayload) {
        println!("PIECE");
        let piece_block = PieceBlock {
            index: payload.index as u64,
            begin: payload.begin as u64,
            length: None,
        };
        self.pieces.add_received(piece_block);

        // Calculate the index offset on where we have to write the received piece.
        let offset = payload.index as u64 * self.torrent.info.piece_length + payload.begin as u64;

        // Write to file
        self.file.seek(SeekFrom::Start(offset)).expect("Unable to set offset on file");
        self.file.write(&*payload.block.as_ref().unwrap().to_bytes()).expect("Unable to write to file");

        // Shutdown if finished
        if self.pieces.is_done() {
            println!("File downloaded!");
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

        println!("REQUESTING PIECES");

        while self.queue.len() > 0 {

            // Grab the first piece in the queue
            let piece_block = self.queue.deque().unwrap();
            println!("Requesting: {}", piece_block.index);

            // Check if that piece is still needed and request if so
            if self.pieces.needed(piece_block) {
                let request = messages::build_request(piece_block);
                self.stream.write(&*request.to_bytes());
                self.pieces.add_requested(piece_block);

                break;
            }
        }
    }
}

