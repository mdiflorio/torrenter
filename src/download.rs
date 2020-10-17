use std::io;
use std::io::Error;
use std::io::prelude::*;
use std::net::{IpAddr, Ipv4Addr, TcpStream};
use std::str;

use bytebuffer::ByteBuffer;

use crate::{message_handlers, messages, utils};
use crate::messages::{build_peer_handshake, get_msg_id, parse};
use crate::pieces::{Pieces, Queue};
use crate::utils::Peer;

pub fn download(peer: &utils::Peer, handshake: &ByteBuffer, pieces: &mut Pieces) -> anyhow::Result<()> {
    let peer_addr = (Ipv4Addr::from(peer.ip_addr), peer.port);

    let mut queue: Queue = Queue::new(pieces.len);

    // let mut stream = TcpStream::connect(peer_addr)?;
    let mut stream = TcpStream::connect("127.0.0.1:14082").expect("Unable to connect to peer");

    println!("Connected to Peer!");

    stream.write(&handshake.to_bytes()).expect("Unable to write to peer");


    let mut is_handshake = true;
    loop {
        let mut recv_msg = get_whole_msg(&mut stream);

        if is_handshake {
            message_handlers::interested(&mut stream);
            is_handshake = false;
        } else {
            message_handlers::router(&mut stream, &mut recv_msg, pieces, &mut queue);
        }
    }

    Ok(())
}


fn check_handshake_msg(msg: &mut ByteBuffer) -> bool {
    if msg.len() < 20 {
        return false;
    }

    let protocol = match String::from_utf8(msg.to_bytes()[1..20].to_owned()) {
        Ok(protocol) => protocol,
        Err(e) => {
            return false;
        }
    };

    let handshake = protocol == "BitTorrent protocol";

    return handshake;
}

fn get_whole_msg(stream: &mut TcpStream) -> ByteBuffer {
    let mut whole_msg: ByteBuffer = ByteBuffer::new();

    let mut buf: &mut [u8; 1028] = &mut [0; 1028];

    let len = stream.read(buf).expect("Unable to receive from peer");
    whole_msg.write_bytes(&buf[0..len]);

    return whole_msg;
}



