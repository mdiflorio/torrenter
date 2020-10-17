use std::io::prelude::*;
use std::net::TcpStream;

use bytebuffer::ByteBuffer;

use crate::messages;
use crate::messages::{parse, Payload};

pub(crate) fn router(stream: &mut TcpStream, mut msg: &mut ByteBuffer, requested_pieces: &mut Vec<u32>, queue: &mut Vec<u32>) {
    let parsed_msg = parse(&mut msg);

    match parsed_msg.id {
        0 => choke(stream),
        1 => unchoke(stream),
        4 => have(stream, &parsed_msg.payload.as_ref().unwrap(), requested_pieces, queue),
        5 => bitfield(stream, &parsed_msg.payload.as_ref().unwrap()),
        7 => piece(stream, &parsed_msg.payload.as_ref().unwrap(), requested_pieces, queue),
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

fn unchoke(stream: &mut TcpStream) {
    println!("UNCHOKING");
}

fn have(stream: &mut TcpStream, payload: &Payload, requested_pieces: &mut Vec<u32>, queue: &mut Vec<u32>) {
    let piece_index = payload.index;

    queue.push(piece_index);
    println!("HAVE");
}

fn bitfield(stream: &mut TcpStream, payload: &Payload) {
    println!("BITFIELD");
}

fn piece(stream: &mut TcpStream, payload: &Payload, requested_pieces: &mut Vec<u32>, queue: &mut Vec<u32>) {
    println!("PIECE");
}
