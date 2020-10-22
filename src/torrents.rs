use std::fmt::Debug;
use std::fs::File;
use std::io::Read;

use anyhow;
use serde_bencode::de;
use serde_bytes::ByteBuf;
use serde_derive::{Deserialize, Serialize};

use crate::utils::calculate_torrent_size;

static BLOCK_LEN: i64 = 2_i64.pow(14) as i64;

#[derive(Debug, Deserialize)]
struct Node(String, i64);

#[derive(Debug, Serialize, Deserialize)]
pub struct DlFile {
    path: Vec<String>,
    pub length: i64,
    #[serde(default)]
    md5sum: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Info {
    name: String,
    pub(crate) pieces: ByteBuf,
    #[serde(rename = "piece length")]
    piece_length: i64,
    #[serde(default)]
    md5sum: Option<String>,
    #[serde(default)]
    pub length: Option<i64>,
    #[serde(default)]
    pub files: Option<Vec<DlFile>>,
    #[serde(default)]
    private: Option<u8>,
    #[serde(default)]
    path: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "root hash")]
    root_hash: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Torrent {
    pub info: Info,
    #[serde(default)]
    pub announce: Option<String>,
    #[serde(default)]
    nodes: Option<Vec<Node>>,
    #[serde(default)]
    encoding: Option<String>,
    #[serde(default)]
    httpseeds: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "announce-list")]
    announce_list: Option<Vec<Vec<String>>>,
    #[serde(default)]
    #[serde(rename = "creation date")]
    creation_date: Option<i64>,
    #[serde(rename = "comment")]
    comment: Option<String>,
    #[serde(default)]
    #[serde(rename = "created by")]
    created_by: Option<String>,
}

pub fn render_torrent(torrent: &Torrent) {
    println!("name:\t\t{}", torrent.info.name);
    println!("announce:\t{:?}", torrent.announce);
    println!("nodes:\t\t{:?}", torrent.nodes);
    if let &Some(ref al) = &torrent.announce_list {
        for a in al {
            println!("announce list:\t{}", a[0]);
        }
    }
    println!("httpseeds:\t{:?}", torrent.httpseeds);
    println!("creation date:\t{:?}", torrent.creation_date);
    println!("comment:\t{:?}", torrent.comment);
    println!("created by:\t{:?}", torrent.created_by);
    println!("encoding:\t{:?}", torrent.encoding);
    println!("piece length:\t{:?}", torrent.info.piece_length);
    println!("private:\t{:?}", torrent.info.private);
    println!("root hash:\t{:?}", torrent.info.root_hash);
    println!("md5sum:\t\t{:?}", torrent.info.md5sum);
    println!("path:\t\t{:?}", torrent.info.path);
    if let &Some(ref files) = &torrent.info.files {
        for f in files {
            println!("file path:\t{:?}", f.path);
            println!("file length:\t{}", f.length);
            println!("file md5sum:\t{:?}", f.md5sum);
        }
    }
}

/// Take a torrent file path and convert it into a Torrent struct.
pub fn decode_file(file_path: &str) -> anyhow::Result<Torrent> {
    let mut handle = File::open(file_path)?;
    let mut buffer = Vec::new();

    handle.read_to_end(&mut buffer)?;
    let torrent = de::from_bytes::<Torrent>(&buffer)?;
    return Ok(torrent);
}

/// Calculate the size of a piece by looking at the piece index within the torrent file
/// If it's not the last piece, we return the length,
/// Otherwise it might be smaller.
pub fn get_piece_len(torrent: &Torrent, piece_index: usize) -> i64 {
    let total_length = calculate_torrent_size(&torrent.info);
    let piece_length = torrent.info.piece_length;


    let last_piece_length = total_length % piece_index as i64;
    let last_piece_index = total_length / piece_length;

    return if last_piece_index == piece_index as i64 { last_piece_length } else { piece_length };
}


/// Calculate the amount of blocks for each piece
pub fn get_blocks_per_piece(torrent: &Torrent, piece_index: usize) -> i64 {
    let piece_len = get_piece_len(torrent, piece_index);
    return piece_len / BLOCK_LEN;
}


/// Get the block length which is based off of the piece index and the block index.
/// If it's not the last piece, we return the length,
/// Otherwise it might be smaller.
pub fn get_block_len(torrent: &Torrent, piece_index: usize, block_index: usize) -> i64 {
    let piece_len = get_piece_len(torrent, piece_index);

    let last_piece_len = piece_len % BLOCK_LEN;
    let last_piece_index = piece_len / BLOCK_LEN;

    return if block_index == last_piece_index as usize { last_piece_len } else { BLOCK_LEN };
}


