use std::io::prelude::*;
use std::net::{Ipv4Addr, TcpStream};

use bytebuffer::ByteBuffer;

use crate::utils;
use crate::message_handlers::MessageHandler;
use crate::pieces::{Pieces, Queue};
use crate::utils::torrents;

pub fn download(torrent: &torrents::Torrent, peer: &utils::Peer, handshake: &ByteBuffer, pieces: &mut Pieces) -> anyhow::Result<()> {
    let peer_addr = (Ipv4Addr::from(peer.ip_addr), peer.port);


    let mut queue: Queue = Queue::new(pieces.len);
    // let mut stream = TcpStream::connect(peer_addr)?;
    let mut stream = TcpStream::connect("127.0.0.1:14082").expect("Unable to connect to peer");

    println!("Connected to Peer!");

    stream.write(&handshake.to_bytes()).expect("Unable to write to peer");

    let mut message_handler = MessageHandler::new(&mut stream, pieces, &mut queue);

    let mut is_handshake = true;
    loop {
        let mut recv_msg = message_handler.get_whole_msg();

        if is_handshake {
            message_handler.interested();
            is_handshake = false;
        } else {
            message_handler.router(&mut recv_msg);
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




