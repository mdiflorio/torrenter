use std::io::prelude::*;
use std::net::{Shutdown, TcpStream};

use anyhow::{anyhow, Result};
use bytebuffer::ByteBuffer;

use crate::messages;
use crate::messages::{parse, Payload};
use crate::pieces::Pieces;
use crate::queue::Queue;

pub struct MessageHandler<'a> {
    stream: &'a mut TcpStream,
    pieces: &'a mut Pieces,
    queue: &'a mut Queue<'a>,
}

impl MessageHandler<'_> {
    pub fn new<'a>(stream: &'a mut TcpStream, pieces: &'a mut Pieces, queue: &'a mut Queue<'a>) -> MessageHandler<'a> {
        MessageHandler {
            stream,
            pieces,
            queue,
        }
    }

    pub fn router(&mut self, mut msg: &mut ByteBuffer) -> Result<()> {
        if msg.len() == 0 {
            return Err(anyhow!("Peer connection closed"));
        }

        let parsed_msg = parse(&mut msg);

        match parsed_msg.id {
            0 => self.choke(),
            1 => self.unchoke(),
            4 => self.have(&parsed_msg.payload.as_ref().unwrap()),
            5 => self.bitfield(&parsed_msg.payload.as_ref().unwrap()),
            7 => self.piece(&parsed_msg.payload.as_ref().unwrap()),
            _ => {
                println!("Unknown message ID: {:?}", parsed_msg.id);
            }
        }

        println!("PARSED MESSAGE: {:?}", parsed_msg);
        return Ok(());
    }


    pub fn get_whole_msg(&mut self) -> ByteBuffer {
        let mut whole_msg: ByteBuffer = ByteBuffer::new();

        let buf: &mut [u8; 1028] = &mut [0; 1028];

        let len = self.stream.read(buf).expect("Unable to receive from peer");
        whole_msg.write_bytes(&buf[0..len]);

        return whole_msg;
    }

    pub fn interested(&mut self) {
        let send_msg = messages::build_interested();
        self.stream.write(&send_msg.to_bytes()).expect("Unable to write to peer");
        println!("SENT INTERESTED!");
    }

    fn choke(&mut self) {
        println!("CHOKED");
        self.stream.shutdown(Shutdown::Both);
    }

    fn unchoke(&mut self) {
        println!("UNCHOKING");
        self.queue.choked = false;
        self.request_piece();
    }

    // function haveHandler(socket, pieces, queue, payload) {
    //     const pieceIndex = payload.readUInt32BE(0);
    //     const queueEmpty = queue.length === 0;
    //     queue.queue(pieceIndex);
    //     if (queueEmpty) requestPiece(socket, pieces, queue);
    // }

    fn have(&mut self, payload: &Payload) {
        println!("HAVE");
        let piece_index = payload.index;
        let queue_empty = self.queue.len() == 0;

        self.queue.queue(piece_index as u64);
        if queue_empty {
            self.request_piece()
        }
    }

    fn bitfield(&self, payload: &Payload) {
        println!("BITFIELD");
    }

    fn piece(&self, payload: &Payload) {
        println!("PIECE");
    }


    fn request_piece(&mut self) {
        if self.queue.choked {
            println!("We're choked!");
            return;
        }

        println!("REQUESTING PIECES");

        while self.queue.len() > 0 {
            let piece_block = self.queue.deque().unwrap();

            if self.pieces.needed(piece_block) {
                let request = messages::build_request(piece_block);
                self.stream.write(&*request.to_bytes());
                self.pieces.add_requested(piece_block);
                break;
            }
        }
    }
}

