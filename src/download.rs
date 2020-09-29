use std::io;
use std::io::prelude::*;
use std::net::{IpAddr, TcpStream};

use bytebuffer::ByteBuffer;

use crate::{messages, utils};
use crate::messages::build_peer_handshake;
use crate::utils::Peer;

pub fn download(peer: &utils::Peer, handshake: &ByteBuffer) -> anyhow::Result<()> {
    let peer_addr = (IpAddr::from(peer.ip_addr.to_be_bytes()), peer.port);

    let mut stream = TcpStream::connect(peer_addr)?;

    stream.write(&handshake.to_bytes()).expect("Unable to write to peer");


    let mut recv_msg = get_whole_msg(&mut stream);
    msg_handler(&mut stream, &mut recv_msg);


    Ok(())
}


fn get_whole_msg(stream: &mut TcpStream) -> ByteBuffer {
    let mut whole_msg: ByteBuffer = ByteBuffer::new();

    let mut handshake = true;

    // Loop until we have a full message
    loop {
        let mut buf = vec![];

        match stream.read_to_end(&mut buf) {
            Ok(_) => {

                // Get the length of the message
                let msg_ln: usize = if handshake {
                    (whole_msg.read_u8() + 49) as usize
                } else {
                    (whole_msg.read_u32() + 4) as usize
                } as usize;

                // Add te new data to the buffer
                whole_msg.write_bytes(&buf);

                // Exit loop if we have the full message
                if whole_msg.len() <= 4 && whole_msg.len() <= msg_ln {
                    break;
                }

                handshake = false;
            },
            Err(e) => {
                println!("Peer stream read error: {}", e);
                break;
            }
        }

        println!("Download: {:?}", buf);
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


