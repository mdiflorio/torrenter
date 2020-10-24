use core::convert::TryInto;

use anyhow;
use bytebuffer::ByteBuffer;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use rand::Rng;
use serde_bencode::ser;

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

#[test]
fn test_hash_torrent_info() {
    let torrent = torrents::decode_file("big-buck-bunny.torrent")
        .ok()
        .unwrap();

    let _hashed_info = hash_torrent_info(&torrent.info);
    // assert_eq!(hashed_info, "dd8255ecdc7ca55fb0bbf81323d87062db1f6d1c");
}

pub fn hash_torrent_info(torrent_info: &torrents::Info) -> [u8; 20] {
    let _hashed_info: &mut [u8] = &mut [0; 20];

    let mut hasher = Sha1::new();
    let bencoded_info = ser::to_bytes(torrent_info).unwrap();

    hasher.input(&bencoded_info);
    hasher.result(_hashed_info);

    let mut hashed_info: [u8; 20] = [0; 20];
    hashed_info.clone_from_slice(_hashed_info);
    return hashed_info;
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

pub fn build_conn_req() -> ByteBuffer {
    let mut rng = rand::thread_rng();
    let mut buffer = ByteBuffer::new();

    // 0       64-bit integer  protocol_id     0x41727101980 // magic constant
    // 8       32-bit integer  action          0 // connect
    // 12      32-bit integer  transaction_id

    let protocol_id: i64 = 0x41727101980;
    let action: i32 = 0;
    let transaction_id: i32 = rng.gen::<i32>();

    buffer.write_i64(protocol_id);
    buffer.write_i32(action);
    buffer.write_i32(transaction_id);

    return buffer;
}

pub fn parse_conn_resp(buf: &[u8; 16]) -> ConnResp {
    let conn_resp = ConnResp {
        action: i32::from_be_bytes(buf[..4].try_into().unwrap()),
        transaction_id: i32::from_be_bytes(buf[4..8].try_into().unwrap()),
        connection_id: i64::from_be_bytes(buf[8..16].try_into().unwrap()),
    };

    return conn_resp;
}

pub fn calculate_torrent_size(torrent_info: &torrents::Info) -> u64 {
    let mut left: u64 = 0;

    if let &Some(ref files) = &torrent_info.files {
        for f in files {
            left += f.length;
        }
    } else {
        left += &torrent_info.length.unwrap_or_else(|| 0);
    }

    return left;
}

pub fn build_announce_req(
    torrent_info: &torrents::Info,
    hashed_info: &[u8; 20],
    connection_id: i64,
    peer_id: &ByteBuffer,
    port: i16,
) -> ByteBuffer {
    // Offset  Size    Name    Value

    let mut announce_req = ByteBuffer::new();
    let mut rng = rand::thread_rng();

    // 0       64-bit integer  connection_id
    announce_req.write_i64(connection_id);
    // 8       32-bit integer  action          1 // announce
    announce_req.write_i32(1);
    // 12      32-bit integer  transaction_id
    announce_req.write_i32(rng.gen::<i32>());
    // 16      20-byte string  info_hash
    announce_req.write_bytes(hashed_info);

    // 36      20-byte string  peer_id
    announce_req.write_bytes(&peer_id.to_bytes());
    // 56      64-bit integer  downloaded
    announce_req.write_i64(0);
    // 64      64-bit integer  left
    let left = calculate_torrent_size(&torrent_info);
    announce_req.write_u64(left);
    // 72      64-bit integer  uploaded
    announce_req.write_i64(0);
    // 80      32-bit integer  event           0 // 0: none; 1: completed; 2: started; 3: stopped
    announce_req.write_i32(0);
    // 84      32-bit integer  IP address      0 // default
    announce_req.write_i32(0);
    // 88      32-bit integer  key
    announce_req.write_i32(0);
    // 92      32-bit integer  num_want        -1 // default
    announce_req.write_i32(-1);
    // 96      16-bit integer  port
    announce_req.write_i16(port);

    return announce_req;
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
