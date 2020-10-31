#![feature(const_int_pow)]

// TODO: Remove this once finished.
// Don't show warnings for unused code when developping.
#![allow(dead_code)]
#![allow(unused_variables)]


use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::{Arc, Mutex};

use anyhow;
use bytebuffer::ByteBuffer;
use tokio::sync::mpsc;

use utils::torrents;

use crate::download::download;
use crate::message_handlers::PieceChannelPayload;
use crate::messages::build_peer_handshake;
use crate::pieces::Pieces;
use crate::queue::PieceBlock;
use crate::utils::{gen_peer_id, Peer};
use crate::utils::torrents::Torrent;

mod utils;
mod messages;
mod download;
mod tracker;
mod message_handlers;
mod pieces;
mod queue;

const PORT: i16 = 6682;

type PiecesManager = Arc<Mutex<Pieces>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let torrent = Arc::new(Torrent::new("Flux.torrent"));

    let peer_id = gen_peer_id();
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
            download(torrent, file_sender, peer, hs, pm).await;
        });
    }

    while let Some(payload) = rx.recv().await {
        println!("GOT = {}", payload.offset);
        dl_file.seek(SeekFrom::Start(payload.offset)).expect("Unable to set offset on file");
        dl_file.write(&payload.block).expect("Unable to write to file");
    }


    Ok(())
}




