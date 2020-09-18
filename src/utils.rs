#[path = "./torrents.rs"]
pub mod torrents;

use crypto::digest::Digest;
use serde_bencode::ser;

use anyhow;
use bytebuffer::ByteBuffer;
use core::convert::TryInto;
use crypto::sha1::Sha1;
use rand::Rng;
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
    interval: i32,
    leechers: i32,
    seeders: i32,
    ip_addr: i32,
    tcp_port: i16,
}

#[test]
fn test_hash_torrent_info() {
    let torrent = torrents::decode_file("big-buck-bunny.torrent")
        .ok()
        .unwrap();

    let hashed_info = hash_torrent_info(&torrent.info);
    assert_eq!(hashed_info, "dd8255ecdc7ca55fb0bbf81323d87062db1f6d1c");
}

pub fn hash_torrent_info(torrent_info: &torrents::Info) -> String {
    let mut hasher = Sha1::new();
    let bencoded_info = ser::to_bytes(torrent_info).unwrap();
    hasher.input(&bencoded_info);
    return hasher.result_str();
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

pub fn calculate_left(torrent_info: &torrents::Info) -> i64 {
    let mut left: i64 = 0;

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
    connection_id: i64,
    peer_id: ByteBuffer,
    port: i16,
) -> ByteBuffer {
    // Offset  Size    Name    Value
    // 0       64-bit integer  connection_id
    // 8       32-bit integer  action          1 // announce
    // 12      32-bit integer  transaction_id
    // 16      20-byte string  info_hash
    // 36      20-byte string  peer_id
    // 56      64-bit integer  downloaded
    // 64      64-bit integer  left
    // 72      64-bit integer  uploaded
    // 80      32-bit integer  event           0 // 0: none; 1: completed; 2: started; 3: stopped
    // 84      32-bit integer  IP address      0 // default
    // 88      32-bit integer  key
    // 92      32-bit integer  num_want        -1 // default
    // 96      16-bit integer  port

    let mut announce_req = ByteBuffer::new();
    let mut rng = rand::thread_rng();

    // connection_id
    announce_req.write_i64(connection_id);
    // action
    announce_req.write_i32(1);
    // transaction_id
    announce_req.write_i32(rng.gen::<i32>());
    // info`
    let info_hash = hash_torrent_info(&torrent_info);
    announce_req.write_bytes(&info_hash.as_bytes());
    // peer_id
    announce_req.write_bytes(&peer_id.to_bytes());
    // downloaded
    announce_req.write_i64(0);
    // left
    let left = calculate_left(&torrent_info);
    announce_req.write_i64(left);
    // uploaded
    announce_req.write_i64(0);
    // event
    announce_req.write_i32(0);
    // IP address - 0 default
    announce_req.write_i32(0);
    // key
    announce_req.write_i32(0);
    // Wanted
    announce_req.write_i32(-1);
    // port
    announce_req.write_i16(port);

    return announce_req;
}

// #[test]
// fn test_parse_announce_resp() {
//     let buf: &[u8; 100] = &[
//         0, 0, 0, 1, 52, 21, 191, 11, 0, 0, 6, 224, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//     ];

//     let recieved: usize = 26;

//     let announce_resp = parse_announce_resp(buf, recieved).unwrap();

//     assert_eq!(announce_resp.action, 1);
//     assert_eq!(announce_resp.transaction_id, 873840395);
//     assert_eq!(announce_resp.interval, 1760);
//     assert_eq!(announce_resp.leechers, 1);
//     assert_eq!(announce_resp.seeders, 0);
// }

pub fn parse_announce_resp(buf: &[u8; 100], recieved: usize) -> anyhow::Result<AnnounceResp> {
    if recieved < 20 {
        anyhow::bail!("Error: Not able to announce to tracker");
    } else {
        let announce_resp = AnnounceResp {
            action: i32::from_be_bytes(buf[..4].try_into().unwrap()),
            transaction_id: i32::from_be_bytes(buf[4..8].try_into().unwrap()),
            interval: i32::from_be_bytes(buf[8..12].try_into().unwrap()),
            leechers: i32::from_be_bytes(buf[12..16].try_into().unwrap()),
            seeders: i32::from_be_bytes(buf[16..20].try_into().unwrap()),
            ip_addr: i32::from_be_bytes(buf[20..24].try_into().unwrap()),
            tcp_port: i16::from_be_bytes(buf[24..26].try_into().unwrap()),
        };

        println!("{:?}", announce_resp);
        return Ok(announce_resp);
    }
}
