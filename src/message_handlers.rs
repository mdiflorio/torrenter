use std::io::prelude::*;
use std::net::TcpStream;

use bytebuffer::ByteBuffer;

use crate::messages;
use crate::messages::{parse, Payload};
use crate::pieces::{Pieces, Queue};

pub(crate) fn router(stream: &mut TcpStream, mut msg: &mut ByteBuffer, pieces: &mut Pieces, queue: &mut Queue) {
    let parsed_msg = parse(&mut msg);

    match parsed_msg.id {
        0 => choke(stream),
        1 => unchoke(stream, pieces, queue),
        4 => have(stream, &parsed_msg.payload.as_ref().unwrap(), queue),
        5 => bitfield(stream, &parsed_msg.payload.as_ref().unwrap()),
        7 => piece(stream, &parsed_msg.payload.as_ref().unwrap()),
        _ => {
            println!("Unknown message ID: {:?}", parsed_msg.id);
        }
    }

    println!("PARSED MESSAGE: {:?}", parsed_msg);
}

pub fn interested(stream: &mut TcpStream) {
    let send_msg = messages::build_interested();
    stream.write(&send_msg.to_bytes()).expect("Unable to write to peer");
    println!("SENT INTERESTED!");
}

fn choke(stream: &mut TcpStream) {
    println!("CHOKING");
}

fn unchoke(stream: &mut TcpStream, pieces: &mut Pieces, queue: &mut Queue) {
    println!("UNCHOKING");
    queue.choked = false;
    request_piece(stream, pieces, queue);
}

fn have(stream: &mut TcpStream, payload: &Payload, queue: &mut Queue) {
    let piece_index = payload.index;

    println!("HAVE");
}

fn bitfield(stream: &mut TcpStream, payload: &Payload) {
    println!("BITFIELD");
}

fn piece(stream: &mut TcpStream, payload: &Payload) {
    println!("PIECE");
}


pub fn request_piece(stream: &mut TcpStream, pieces: &mut Pieces, queue: &mut Queue) {
    if queue.choked {
        println!("We're choked!");
        return;
    }

    println!("REQUESTING PIECES");
    while queue.pieces.len() > 0 {
        if let piece_index = queue.pieces.pop_front().unwrap() {
            if pieces.needed(piece_index as usize) {
                // TODO - Implement build request
                // let request = messages::build_request();
                // stream.write(&*request.to_bytes());
                pieces.add_requested(piece_index as usize);
                break;
            }
        }
    }
}