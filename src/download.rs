use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::net::{Ipv4Addr, TcpStream};
use std::sync::{Arc, Mutex};

use bytebuffer::ByteBuffer;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

use crate::utils;
use crate::message_handlers::{MessageHandler, PieceChannelPayload};
use crate::messages::build_peer_handshake;
use crate::pieces::Pieces;
use crate::queue::{PieceBlock, Queue};
use crate::utils::{Peer, torrents};
use crate::utils::torrents::Torrent;

pub type PiecesManager = Arc<Mutex<Pieces>>;

pub async fn download_torrent(peer_id: ByteBuffer, file_path: &str) -> anyhow::Result<()> {
    let torrent = Arc::new(Torrent::new(file_path));
    let handshake = Arc::new(build_peer_handshake(&torrent.info_hash.unwrap(), &peer_id).to_bytes());

    let mut dl_file = File::create(&torrent.info.name)?;

    // let peers = get_torrent_peers(&torrent, &hashed_info, &peer_id)?;


    // println!("{:?}", peers);
    // [Peer { ip_addr: 1410415827, port: 6682 }]
    let peer = Peer {
        ip_addr: 0,
        port: 0,
    };


    let (tx, mut rx) = mpsc::channel::<PieceChannelPayload>(32);

    let pieces_manager = Arc::new(Mutex::new(Pieces::new(&torrent)));

    for i in 0..1 {
        let file_sender = tx.clone();
        let pm = pieces_manager.clone();
        let torrent = torrent.clone();
        let peer = peer.clone();
        let hs = handshake.clone();

        tokio::spawn(async move {
            download_from_peer(torrent, file_sender, peer, hs, pm).await;
        });
    }

    while let Some(payload) = rx.recv().await {
        println!("GOT = {}", payload.offset);
        dl_file.seek(SeekFrom::Start(payload.offset)).expect("Unable to set offset on file");
        dl_file.write(&payload.block).expect("Unable to write to file");
    }

    Ok(())
}


async fn download_from_peer(torrent: Arc<Torrent>, file_sender: Sender<PieceChannelPayload>, peer: Peer, handshake: Arc<Vec<u8>>, pieces: PiecesManager) -> anyhow::Result<()> {
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




