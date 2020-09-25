use std::io;
use std::io::prelude::*;
use std::net::{IpAddr, TcpStream};

use bytebuffer::ByteBuffer;

use crate::messages::build_peer_handshake;
use crate::utils;
use crate::utils::Peer;

pub fn download(peer: &utils::Peer, handshake: &ByteBuffer) -> anyhow::Result<()> {
    let peer_addr = (IpAddr::from(peer.ip_addr.to_be_bytes()), peer.port);

    let mut stream = TcpStream::connect(peer_addr)?;

    stream.write(&handshake.to_bytes()).expect("Unable to write to peer");


    let mut buf = vec![];
    loop {
        match stream.read_to_end(&mut buf) {
            Ok(_) => break,
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                println!("WouldBlock error");
            },
            Err(e) => panic!("encountered IO error: {}", e)
        }
    }
    println!("Download: {:?}", buf);

    Ok(())
}
