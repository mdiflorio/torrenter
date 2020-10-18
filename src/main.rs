use anyhow;
use serde_bytes::ByteBuf;

use utils::torrents;

use crate::download::download;
use crate::messages::build_peer_handshake;
use crate::pieces::Pieces;
use crate::tracker::get_torrent_peers;
use crate::utils::{gen_peer_id, hash_torrent_info, Peer};

mod utils;
mod messages;
mod download;
mod tracker;
mod message_handlers;
mod pieces;

const PORT: i16 = 6682;

fn main() -> anyhow::Result<()> {
    let torrent = torrents::decode_file("Flux.torrent")?;
    torrents::render_torrent(&torrent);

    let hashed_info = hash_torrent_info(&torrent.info);
    let peer_id = gen_peer_id();
    // let peers = get_torrent_peers(&torrent, &hashed_info, &peer_id)?;
    let handshake = build_peer_handshake(&hashed_info, &peer_id);

    // println!("{:?}", peers);
    // [Peer { ip_addr: 1410415827, port: 6682 }]
    let peer = Peer {
        ip_addr: 0,
        port: 0,
    };

    let mut pieces: Pieces = Pieces::new(torrent.info.pieces.len() / 20);

    download(&peer, &handshake, &mut pieces)?;

    Ok(())
}

// peer_id
//
// Torrent
//     file
//     hashed_info
//     peers
//     handshake
//     pieces
//
//
// PeerDownloader
//     stream : TcpStream
//     torrent: Torrent
//     message_handler:
//
//     download




