#![feature(const_int_pow)]

// TODO: Remove this once finished.
// Don't show warnings for unused code when developping.
#![allow(dead_code)]
#![allow(unused_variables)]

use crate::download::download_torrent;
use crate::utils::gen_peer_id;

mod utils;
mod messages;
mod download;
mod tracker;
mod message_handlers;
mod pieces;
mod queue;

const PORT: i16 = 6682;


#[tokio::main]
async fn main() {
    let peer_id = gen_peer_id();
    download_torrent(peer_id, "test-tor.torrent").await;
}




