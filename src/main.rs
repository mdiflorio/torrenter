#![feature(const_int_pow)]

// TODO: Remove this once finished.
// Don't show warnings for unused code when developping.
#![allow(dead_code)]
#![allow(unused_variables)]


use std::fs::File;

use anyhow;

use utils::torrents;

use crate::download::download;
use crate::messages::build_peer_handshake;
use crate::pieces::Pieces;
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

fn main() -> anyhow::Result<()> {
    let torrent = Torrent::new("Flux.torrent");
    torrent.print_info();

    let mut dl_file = File::create(&torrent.info.name)?;

    let peer_id = gen_peer_id();
    // let peers = get_torrent_peers(&torrent, &hashed_info, &peer_id)?;
    let handshake = build_peer_handshake(&torrent.info_hash.unwrap(), &peer_id);

    // println!("{:?}", peers);
    // [Peer { ip_addr: 1410415827, port: 6682 }]
    let peer = Peer {
        ip_addr: 0,
        port: 0,
    };

    let mut pieces: Pieces = Pieces::new(&torrent);


    download(&torrent, &mut dl_file, &peer, &handshake, &mut pieces)?;

    Ok(())
}




