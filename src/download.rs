use std::fs::File;
use std::io::prelude::*;
use std::net::{Ipv4Addr, TcpStream};
use std::sync::Arc;

use bytebuffer::ByteBuffer;
use tokio::sync::mpsc::Sender;

use crate::{PiecesManager, utils};
use crate::message_handlers::{MessageHandler, PieceChannelPayload};
use crate::pieces::Pieces;
use crate::queue::{PieceBlock, Queue};
use crate::utils::{Peer, torrents};
use crate::utils::torrents::Torrent;

pub async fn download(torrent: Arc<Torrent>, file_sender: Sender<PieceChannelPayload>, peer: Peer, handshake: Arc<Vec<u8>>, pieces: PiecesManager) -> anyhow::Result<()> {
    let peer_addr = (Ipv4Addr::from(peer.ip_addr), peer.port);

    let mut queue: Queue = Queue::new(&torrent);

    // let mut stream = TcpStream::connect(peer_addr)?;
    let mut stream = TcpStream::connect("127.0.0.1:14082").expect("Unable to connect to peer");

    println!("Connected to Peer!");

    stream.write(&handshake).expect("Unable to write to peer");

    let mut message_handler = MessageHandler::new(&torrent, &mut stream, file_sender, pieces, &mut queue);

    let mut is_handshake = true;
    loop {

        if is_handshake {
            message_handler.handshake();
            is_handshake = false;
        } else {
            let recv_msg = message_handler.get_whole_msg();
            message_handler.router(recv_msg).await;
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




