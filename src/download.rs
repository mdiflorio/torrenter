use std::io;
use std::io::prelude::*;
use std::net::{IpAddr, Ipv4Addr, TcpStream};

use bytebuffer::ByteBuffer;

use crate::{messages, utils};
use crate::messages::build_peer_handshake;
use crate::utils::Peer;

pub fn download(peer: &utils::Peer, handshake: &ByteBuffer) -> anyhow::Result<()> {
    let peer_addr = (Ipv4Addr::from(peer.ip_addr), peer.port);

    // let mut stream = TcpStream::connect(peer_addr)?;
    let mut stream = TcpStream::connect("86.67.191.151:14082")?;

    println!("Connected to Peer wohaaoh!");

    stream.write(&handshake.to_bytes()).expect("Unable to write to peer");

    let mut recv_msg = get_whole_msg(&mut stream);
    msg_handler(&mut stream, &mut recv_msg);


    Ok(())
}


fn get_whole_msg(stream: &mut TcpStream) -> ByteBuffer {
    let mut whole_msg: ByteBuffer = ByteBuffer::new();

    // Loop until we have a full message
    loop {
        let mut buf = vec![];

        let length = stream.read_to_end(&mut buf);

        // println!("Download: {:?}", buf);
        // println!("Size: {:?}", length);
    }

    return whole_msg;
}


fn msg_handler(stream: &mut TcpStream, mut recv_msg: &mut ByteBuffer) {
    if is_handshake(&mut recv_msg) {
        let send_msg = messages::build_interested();
        stream.write(&send_msg.to_bytes()).expect("Unable to write to peer");
    } else {}
}

fn is_handshake(msg: &mut ByteBuffer) -> bool {
    let handshake = msg.len() == (msg.read_u8() + 49) as usize && msg.to_string() == "BitTorrent protocol";
    return handshake;
}


