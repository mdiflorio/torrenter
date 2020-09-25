use std::io::prelude::*;
use std::net::{IpAddr, TcpStream};

use bytebuffer::ByteBuffer;

use crate::utils;

pub fn connect_peer(peer: &utils::SeederInfo, handshake: &ByteBuffer) -> anyhow::Result<()> {
    let peer_addr = IpAddr::from(peer.ip_addr.to_be_bytes());


    let mut stream = TcpStream::connect((peer_addr, 6881))?;

    stream.write(&handshake.to_bytes()).expect("Unable to write to peer");

    loop {
        let buf = &mut [0; 256];
        stream
            .read(buf)
            .expect("Unable to recieve from peer");

        println!("{:?}", buf);
    }

    Ok(())
}
