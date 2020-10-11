use std::io;
use std::io::Error;
use std::io::prelude::*;
use std::net::{IpAddr, Ipv4Addr, TcpStream};
use std::str;

use bytebuffer::ByteBuffer;

use crate::{messages, utils};
use crate::messages::{build_peer_handshake, get_msg_id, parse};
use crate::utils::Peer;

pub fn download(peer: &utils::Peer, handshake: &ByteBuffer) -> anyhow::Result<()> {
    let peer_addr = (Ipv4Addr::from(peer.ip_addr), peer.port);

    // let mut stream = TcpStream::connect(peer_addr)?;
    let mut stream = TcpStream::connect("127.0.0.1:14082").expect("Unable to connect to peer");

    println!("Connected to Peer!");

    stream.write(&handshake.to_bytes()).expect("Unable to write to peer");


    loop {
        let mut recv_msg = get_whole_msg(&mut stream);
        let msg_id = get_msg_id(&recv_msg);
        println!("Length: {:?} - ID: {:?} - Data: {:?} ", recv_msg.len(), msg_id, recv_msg);
        msg_handler(&mut stream, &mut recv_msg);
    }


    Ok(())
}


fn msg_handler(stream: &mut TcpStream, mut recv_msg: &mut ByteBuffer) {
    if is_handshake(&mut recv_msg) {
        let send_msg = messages::build_interested();
        stream.write(&send_msg.to_bytes()).expect("Unable to write to peer");
    } else {

        // let parsed_msg = parse(msg_id, recv_msg);

        // println!("{:?}", parsed_msg);
    }
}

fn is_handshake(msg: &mut ByteBuffer) -> bool {
    let mut protocol: String;

    match String::from_utf8(msg.to_bytes()[1..20].to_owned()) {
        Ok(pstr) => protocol = pstr,
        Err(e) => {
            return false;
        }
    }

    let id = get_msg_id(msg);
    let handshake = id == 19 && protocol == "BitTorrent protocol";

    return handshake;
}

fn get_whole_msg(stream: &mut TcpStream) -> ByteBuffer {
    let mut whole_msg: ByteBuffer = ByteBuffer::new();

    let mut buf: &mut [u8; 1028] = &mut [0; 1028];

    let len = stream.read(buf).expect("Unable to receive from peer");
    whole_msg.write_bytes(&buf[0..len]);

    return whole_msg;
}



