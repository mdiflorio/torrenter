use std::fmt::Debug;
use std::fs::File;
use std::io::Read;

use anyhow;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use serde_bencode::{de, ser};
use serde_bytes::ByteBuf;
use serde_derive::{Deserialize, Serialize};

pub static BLOCK_LEN: u64 = 2_u64.pow(14) as u64;

#[derive(Debug, Deserialize, Clone)]
struct Node(String, i64);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DlFile {
    pub(crate) path: Vec<String>,
    pub length: u64,
    #[serde(default)]
    pub(crate) md5sum: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Info {
    pub(crate) name: String,
    pub(crate) pieces: ByteBuf,
    #[serde(rename = "piece length")]
    pub(crate) piece_length: u64,
    #[serde(default)]
    md5sum: Option<String>,
    #[serde(default)]
    pub length: Option<u64>,
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

#[derive(Debug, Deserialize, Default, Clone)]
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
    pub(crate) size: Option<u64>,
    pub(crate) info_hash: Option<[u8; 20]>
}

impl Torrent {
    /// Take a torrent file path and convert it into a Torrent struct.
    pub fn new(file_path: &str) -> Torrent {
        let mut handle = File::open(file_path).expect("Could not load the file");
        let mut buffer = Vec::new();

        handle.read_to_end(&mut buffer).expect("Couldn't read all of the torrent file");

        let mut torrent = de::from_bytes::<Torrent>(&buffer).expect("Couldn't load the torrent into the torrent struct");
        torrent.size = Some(calculate_torrent_size(&torrent.info));
        torrent.info_hash = Some(hash_torrent_info(&torrent.info));

        return torrent;
    }


    /// Calculate the size of a piece by looking at the piece index within the torrent file
    /// If it's not the last piece, we return the length,
    /// Otherwise it might be smaller.
    pub fn get_piece_len(&self, piece_index: u64) -> u64 {
        let total_length = self.size.unwrap();
        let piece_length = self.info.piece_length;

        let last_piece_length = total_length % piece_length;
        let last_piece_index = total_length / piece_length;

        return if last_piece_index == piece_index { last_piece_length } else { piece_length };
    }


    /// Calculate the amount of blocks for each piece
    pub fn get_blocks_per_piece(&self, piece_index: u64) -> u64 {
        let piece_len = self.get_piece_len(piece_index);

        let mut blocks_per_piece: u64;

        // Round up if it's the last piece
        if piece_len % BLOCK_LEN > 0 {
            blocks_per_piece = piece_len / BLOCK_LEN + 1;
        } else {
            blocks_per_piece = piece_len / BLOCK_LEN;
        }

        return blocks_per_piece;
    }


    /// Get the block length which is based off of the piece index and the block index.
    /// If it's not the last piece, we return the length,
    /// Otherwise it might be smaller.
    pub fn get_block_len(&self, piece_index: u64, block_index: u64) -> u64 {
        let piece_len = self.get_piece_len(piece_index);

        let last_piece_len = piece_len % BLOCK_LEN;
        let last_piece_index = piece_len / BLOCK_LEN;

        return if block_index == last_piece_index { last_piece_len } else { BLOCK_LEN };
    }

    pub fn print(&self) {
        println!("name:\t\t{}", self.info.name);
        println!("announce:\t{:?}", self.announce);
        println!("nodes:\t\t{:?}", self.nodes);
        if let &Some(ref al) = &self.announce_list {
            for a in al {
                println!("announce list:\t{}", a[0]);
            }
        }
        println!("httpseeds:\t{:?}", self.httpseeds);
        println!("creation date:\t{:?}", self.creation_date);
        println!("comment:\t{:?}", self.comment);
        println!("created by:\t{:?}", self.created_by);
        println!("encoding:\t{:?}", self.encoding);
        println!("piece length:\t{:?}", self.info.piece_length);
        println!("private:\t{:?}", self.info.private);
        println!("root hash:\t{:?}", self.info.root_hash);
        println!("md5sum:\t\t{:?}", self.info.md5sum);
        println!("path:\t\t{:?}", self.info.path);
        if let &Some(ref files) = &self.info.files {
            for f in files {
                println!("file path:\t{:?}", f.path);
                println!("file length:\t{}", f.length);
                println!("file md5sum:\t{:?}", f.md5sum);
            }
        }
        println!("size:\t\t{:?}", self.size);
    }

    pub fn print_info(&self) {
        println!("name:\t{:?}", self.info.name);
        println!("piece_length:\t{:?}", self.info.piece_length);
        println!("md5sum:\t{:?}", self.info.md5sum);
        println!("length:\t{:?}", self.info.length);
        println!("root_hash:\t{:?}", self.info.root_hash);
    }
}


#[test]
fn test_get_piece_len() {
    let torrent = Torrent::new("test-tor.torrent");

    // Test length of last piece
    let piece_len = torrent.get_piece_len(14);
    assert_eq!(piece_len, 20750);

    // Test length of all the other pieces
    let piece_len = torrent.get_piece_len(1);
    assert_eq!(piece_len, 32768);
}


#[test]
fn test_get_block_len() {
    let torrent = Torrent::new("test-tor.torrent");

    // Test length of all the other pieces
    let piece_len = torrent.get_block_len(14, 0);
    assert_eq!(piece_len, 16384);

    // Test length of last block for the last piece
    let piece_len = torrent.get_block_len(14, 1);
    assert_eq!(piece_len, 4366);
}


#[test]
fn test_blocks_per_piece() {

    // Test that the last piece has two blocks and isn't missing a block.
    let torrent = Torrent::new("test-tor.torrent");
    let piece_len = torrent.get_blocks_per_piece(14);
    assert_eq!(piece_len, 2);


    let torrent = Torrent::new("test-tor.torrent");
    let piece_len = torrent.get_blocks_per_piece(13);
    assert_eq!(piece_len, 2);
}


/// Calculate the size of the torrent.
///
/// If many files add up the length of each of each file
/// otherwise, take the length of a single file.
pub fn calculate_torrent_size(torrent_info: &Info) -> u64 {
    let mut size: u64 = 0;

    if let &Some(ref files) = &torrent_info.files {
        for f in files {
            size += f.length;
        }
    } else {
        size += &torrent_info.length.unwrap_or_else(|| 0);
    }
    return size;
}

#[test]
fn test_calculate_torrent_size() {
    let torrent = Torrent::new("test-tor.torrent");
    let torrent_size = calculate_torrent_size(&torrent.info);
    assert_eq!(torrent_size, 479502);
}


/// Create a hash of the torrent info.
///
///     This is used to create the announce that is sent to the tracker
///     and to the peers.
pub fn hash_torrent_info(torrent_info: &Info) -> [u8; 20] {
    let _hashed_info: &mut [u8] = &mut [0; 20];

    let mut hasher = Sha1::new();
    let bencoded_info = ser::to_bytes(torrent_info).unwrap();

    hasher.input(&bencoded_info);
    hasher.result(_hashed_info);

    let mut hashed_info: [u8; 20] = [0; 20];
    hashed_info.clone_from_slice(_hashed_info);
    return hashed_info;
}


#[test]
fn test_hash_torrent_info() {
    let torrent = Torrent::new("test-tor.torrent");
    let hashed_info = hash_torrent_info(&torrent.info);

    let expected: [u8; 20] = [0x06, 0xcb, 0x06, 0x12, 0x40, 0xb2, 0x4f, 0x73, 0x0f, 0xbe, 0xf7, 0xea, 0xd1, 0xb3, 0x48, 0xd8, 0x86, 0x52, 0x44, 0xaf];
    assert_eq!(hashed_info, expected);
}
