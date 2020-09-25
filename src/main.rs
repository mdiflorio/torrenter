use anyhow;

use utils::torrents;

use crate::download::connect_peer;
use crate::messages::build_peer_handshake;
use crate::tracker::get_torrent_peers;
use crate::utils::{gen_peer_id, hash_torrent_info};

mod utils;
mod messages;
mod download;
mod tracker;

const PORT: i16 = 6682;

fn main() -> anyhow::Result<()> {
    let torrent = torrents::decode_file("big-buck-bunny.torrent")?;
    torrents::render_torrent(&torrent);

    let hashed_info = hash_torrent_info(&torrent.info);
    let peer_id = gen_peer_id();
    let peers = get_torrent_peers(&torrent, &hashed_info, &peer_id)?;
    let peer_handshake = build_peer_handshake(&hashed_info, &peer_id);


    println!("{:?}", peers);

    connect_peer(&peers[1], &peer_handshake)?;

    Ok(())
}




