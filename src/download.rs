use std::fs::File;
use std::io::prelude::*;
use std::net::{Ipv4Addr, TcpStream};

use bytebuffer::ByteBuffer;

use crate::message_handlers::MessageHandler;
use crate::pieces::Pieces;
use crate::queue::Queue;
use crate::utils;
use crate::utils::torrents;

pub fn download(torrent: &torrents::Torrent, dl_file: &mut File, peer: &utils::Peer, handshake: &ByteBuffer, pieces: &mut Pieces) -> anyhow::Result<()> {
    let peer_addr = (Ipv4Addr::from(peer.ip_addr), peer.port);
    let mut queue: Queue = Queue::new(torrent);
    // let mut stream = TcpStream::connect(peer_addr)?;
    let mut stream = TcpStream::connect("127.0.0.1:14082").expect("Unable to connect to peer");

    println!("Connected to Peer!");

    stream.write(&handshake.to_bytes()).expect("Unable to write to peer");

    let mut message_handler = MessageHandler::new(&torrent, &mut stream, dl_file, pieces, &mut queue);

    let mut is_handshake = true;
    loop {

        if is_handshake {
            message_handler.handshake();
            is_handshake = false;
        } else {
            let mut recv_msg = message_handler.get_whole_msg();
            match message_handler.router(&mut recv_msg) {
                Ok(_) => {}
                Err(e) => {
                    println!("{}", e);
                    break;
                }
            }
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




