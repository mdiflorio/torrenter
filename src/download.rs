use std::fs::{File, OpenOptions};
use std::fs;
use std::intrinsics::write_bytes;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::net::{Ipv4Addr, TcpStream};
use std::num::Wrapping;
use std::sync::{Arc, Mutex};

use bytebuffer::ByteBuffer;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

use crate::message_handlers::{MessageHandler, PieceChannelPayload};
use crate::messages::build_peer_handshake;
use crate::pieces::Pieces;
use crate::queue::{PieceBlock, Queue};
use crate::utils;
use crate::utils::{Peer, torrents};
use crate::utils::torrents::{DlFile, Torrent};

pub type PiecesManager = Arc<Mutex<Pieces>>;

pub async fn download_torrent(peer_id: ByteBuffer, file_path: &str) -> anyhow::Result<()> {
    let torrent = Arc::new(Torrent::new(file_path));
    torrent.print();

    let handshake = Arc::new(build_peer_handshake(&torrent.info_hash.unwrap(), &peer_id).to_bytes());


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
            download_from_peer(torrent, file_sender, peer, hs, pm).await;
        });
    }

    while let Some(payload) = rx.recv().await {
        write_block_to_file(&torrent.info.files.as_ref().unwrap(), payload)
    }

    Ok(())
}

fn write_block_to_file(files: &Vec<DlFile>, payload: PieceChannelPayload) {

    // Loop through all files in the torrent
    let mut file_offset = 0;
    let mut offset = payload.offset;
    let mut bytes_to_write = payload.block.clone();


    for file in files {
        let for_this_file = offset < file.length + file_offset;

        let write_len = if file.length + file_offset - offset < bytes_to_write.len() as u64 {
            file.length + file_offset - offset
        } else {
            bytes_to_write.len() as u64
        } as usize;


        if for_this_file {
            let mut dl_file = OpenOptions::new().write(true).create(true).open(&file.path.join("/")).expect("Unable to open file");
            dl_file.seek(SeekFrom::Start(offset - file_offset)).expect("Unable to set offset on file");
            dl_file.write(&bytes_to_write[0..write_len]).expect("Unable to write to file");
            bytes_to_write.drain(0..write_len);


            offset += write_len as u64;
            file_offset += file.length;
        }

        if bytes_to_write.len() == 0 {
            break;
        }
    }
}


#[test]
fn test_write_block_to_file() {
    match fs::remove_dir_all("test-files/") {
        Ok(_) => {},
        Err(_) => {}
    };

    match fs::create_dir("test-files/") {
        Ok(_) => {},
        Err(_) => {}
    };

    // Setup
    let f1 = DlFile {
        path: vec!["test-files".to_owned(), "file1.txt".to_owned()],
        length: 5,
        md5sum: None,
    };

    let f2 = DlFile {
        path: vec!["test-files".to_owned(), "file2.txt".to_owned()],
        length: 5,
        md5sum: None,
    };

    let f3 = DlFile {
        path: vec!["test-files".to_owned(), "file3.txt".to_owned()],
        length: 5,
        md5sum: None,
    };

    let mut files: Vec<DlFile> = Vec::new();
    files.push(f1);
    files.push(f2);
    files.push(f3);

    let payload = PieceChannelPayload {
        offset: 4,
        block: vec![1; 8],
    };

    // Logic
    write_block_to_file(&files, payload);

    // Test
    let mut f = File::open("test-files/file1.txt").expect("Couldn't open file");
    let mut buffer = [0; 5];
    f.read(&mut buffer).expect("Couldn't read to buffer");
    println!("{:?}", buffer);
    assert_eq!(vec![0, 0, 0, 0, 1], buffer);


    let mut f = File::open("test-files/file2.txt").expect("Couldn't open file");
    let mut buffer = [0; 5];
    f.read(&mut buffer).expect("Couldn't read to buffer");
    println!("{:?}", buffer);
    assert_eq!(vec![1; 5], buffer);


    let mut f = File::open("test-files/file3.txt").expect("Couldn't open file");
    let mut buffer = [0; 5];
    f.read(&mut buffer).expect("Couldn't read to buffer");
    println!("{:?}", buffer);
    assert_eq!(vec![1, 1, 0, 0, 0], buffer);

    fs::remove_dir_all("test-files/").expect("Couldn't clean up folder");
}


async fn download_from_peer(torrent: Arc<Torrent>, file_sender: Sender<PieceChannelPayload>, peer: Peer, handshake: Arc<Vec<u8>>, pieces: PiecesManager) -> anyhow::Result<()> {
    let peer_addr = (Ipv4Addr::from(peer.ip_addr), peer.port);

    let mut queue: Queue = Queue::new(&torrent);

    // let mut stream = TcpStream::connect(peer_addr)?;
    let mut stream = TcpStream::connect("127.0.0.1:14082").expect("Unable to connect to peer");

    println!("Connected to Peer!");

    stream.write(&handshake).expect("Unable to write to peer");

    let mut message_handler = MessageHandler::new(&torrent, &mut stream, file_sender, pieces, &mut queue);

    let mut is_handshake = true;
    loop {
        if is_handshake {
            message_handler.handshake();
            is_handshake = false;
        } else {
            let recv_msg = message_handler.get_whole_msg();
            message_handler.router(recv_msg).await;
        }
    }

    Ok(())
}


fn check_handshake_msg(msg: &mut ByteBuffer) -> bool {
    if msg.len() < 20 {
        return false;
    }

    let protocol = match String::from_utf8(msg.to_bytes()[1..20].to_owned()) {
        Ok(protocol) => protocol,
        Err(e) => {
            return false;
        }
    };

    let handshake = protocol == "BitTorrent protocol";

    return handshake;
}




