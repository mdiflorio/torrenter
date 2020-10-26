use core::convert::TryInto;

use anyhow;
use bytebuffer::ByteBuffer;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use rand::Rng;
use serde_bencode::ser;

use crate::utils::torrents::calculate_torrent_size;

#[path = "./torrents.rs"]
pub mod torrents;

#[derive(Debug)]
pub struct ConnResp {
    action: i32,
    transaction_id: i32,
    pub connection_id: i64,
}

#[derive(Debug)]
pub struct AnnounceResp {
    action: i32,
    transaction_id: i32,
    pub interval: i32,
    pub leechers: i32,
    pub seeders: i32,
    pub peers: Vec<Peer>,
}

#[derive(Debug, Clone)]
pub struct Peer {
    pub ip_addr: u32,
    pub port: u16,
}


pub fn gen_peer_id() -> ByteBuffer {
    let mut peer_id = ByteBuffer::new();
    let peer_prefix = "-R~0001-";
    let mut rng = rand::thread_rng();

    peer_id.write_bytes(peer_prefix.as_bytes());
    peer_id.write_i64(rng.gen::<i64>());
    peer_id.write_i32(rng.gen::<i32>());

    return peer_id;
}


pub fn parse_conn_resp(buf: &[u8; 16]) -> ConnResp {
    let conn_resp = ConnResp {
        action: i32::from_be_bytes(buf[..4].try_into().unwrap()),
        transaction_id: i32::from_be_bytes(buf[4..8].try_into().unwrap()),
        connection_id: i64::from_be_bytes(buf[8..16].try_into().unwrap()),
    };

    return conn_resp;
}


pub fn parse_announce_resp(buf: &[u8; 1000], received: usize) -> anyhow::Result<AnnounceResp> {
    if received < 20 {
        anyhow::bail!("Error: Not able to announce to tracker");
    } else {
        let mut announce_resp = AnnounceResp {
            action: i32::from_be_bytes(buf[..4].try_into().unwrap()),
            transaction_id: i32::from_be_bytes(buf[4..8].try_into().unwrap()),
            interval: i32::from_be_bytes(buf[8..12].try_into().unwrap()),
            leechers: i32::from_be_bytes(buf[12..16].try_into().unwrap()),
            seeders: i32::from_be_bytes(buf[16..20].try_into().unwrap()),
            peers: Vec::new(),
        };

        let mut offset = 20;
        for _ in 0..announce_resp.seeders {
            announce_resp.peers.push(Peer {
                ip_addr: u32::from_be_bytes(buf[offset..offset + 4].try_into().unwrap()),
                port: u16::from_be_bytes(buf[offset + 4..offset + 6].try_into().unwrap()),
            });

            offset += 4;
        }

        return Ok(announce_resp);
    }
}
