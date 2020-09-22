mod utils;
use utils::torrents;

use anyhow;
use std::{time::Duration};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, UdpSocket, TcpStream};
use std::io::prelude::*;

use url::Url;
use crate::utils::{hash_torrent_info, gen_peer_id};
use bytebuffer::ByteBuffer;

const PORT: i16 = 6682;

fn main() -> anyhow::Result<()> {
    let torrent = torrents::decode_file("big-buck-bunny.torrent")?;
    torrents::render_torrent(&torrent);

    let hashed_info = hash_torrent_info(&torrent.info);
    let peer_id = gen_peer_id();
    let peers = get_torrent_peers(&torrent, &hashed_info, &peer_id)?;
    let peer_handshake = build_peer_handshake(&hashed_info, &peer_id);


    println!("{:?}", peers);



    connect_peer(&peers[1], &peer_handshake);

    Ok(())
}

fn connect_peer(peer: &utils::SeederInfo, handshake: &ByteBuffer) -> anyhow::Result<()> {
    let peer_addr = IpAddr::from(peer.ip_addr.to_be_bytes());

    println!("{:?}", &handshake.len());

    let mut stream = TcpStream::connect((peer_addr, peer.port))?;

    stream.write(&handshake.to_bytes()).expect("Unable to write to peer");

    let buf = &mut [0; 256];
    stream
        .read(buf)
        .expect("Unable to recieve from peer");

    println!("{:?}", buf);
    Ok(())
}


fn build_peer_handshake(info_hash: &[u8; 20], peer_id: &ByteBuffer) -> ByteBuffer {
    // Handshake
    // The handshake is a required message and must be the first message transmitted by the client. It is (49+len(pstr)) bytes long.
    //
    //     handshake: <pstrlen><pstr><reserved><info_hash><peer_id>
    //
    //     pstrlen: string length of <pstr>, as a single raw byte
    //     pstr: string identifier of the protocol
    //     reserved: eight (8) reserved bytes. All current implementations use all zeroes. Each bit in these bytes can be used to change the behavior of the protocol. An email from Bram suggests that trailing bits should be used first, so that leading bits may be used to change the meaning of trailing bits.
    //     info_hash: 20-byte SHA1 hash of the info key in the metainfo file. This is the same info_hash that is transmitted in tracker requests.
    //     peer_id: 20-byte string used as a unique ID for the client. This is usually the same peer_id that is transmitted in tracker requests (but not always e.g. an anonymity option in Azureus).

    // In version 1.0 of the BitTorrent protocol, pstrlen = 19, and pstr = "BitTorrent protocol".

    let mut handshake: ByteBuffer = ByteBuffer::new();
    handshake.write_i8(19);
    handshake.write_string("BitTorrent protocol");
    handshake.write_u64(0);
    handshake.write_bytes(info_hash);
    handshake.write_bytes(&peer_id.to_bytes());

    return handshake;
}


fn get_torrent_peers(
    torrent: &torrents::Torrent,
    hashed_info: &[u8; 20],
    peer_id: &ByteBuffer,
) -> anyhow::Result<Vec<utils::SeederInfo>> {

    let tracker_url = Url::parse(&torrent.announce.as_ref().unwrap()).unwrap();
    let base_tracker_url = format!(
        "{}:{}",
        tracker_url.host_str().unwrap(),
        tracker_url.port().unwrap()
    );

    let socket = UdpSocket::bind(format!("0.0.0.0:{}", PORT)).unwrap();
    socket.set_read_timeout(Some(Duration::new(5, 0)));

    let conn_resp = connect_tracker(&socket, base_tracker_url);
    println!("{:?}", conn_resp);
    let announce_resp = announce_tracker(&socket, &torrent.info,hashed_info, peer_id, conn_resp)
        .expect("Not able to get peers from tracker");

    if announce_resp.seeders == 0 {
        anyhow::bail!("No peers at the moment");
    } else {
        return Ok(announce_resp.seeder_info);
    }
}

fn connect_tracker(socket: &UdpSocket, tracker_url: String) -> utils::ConnResp {
    let conn_req = utils::build_conn_req();

    socket
        .connect(tracker_url)
        .expect("couldn't connect to address");

    socket
        .send(&conn_req.to_bytes())
        .expect("couldn't send message");

    let mut recv_buf = [0; 16];
    match socket.recv(&mut recv_buf) {
        Ok(received) => println!("received {} bytes {:?}", received, &recv_buf[..4]),
        Err(e) => {
            println!("recv function failed: {:?}", e);
            panic!();
        }
    }

    return utils::parse_conn_resp(&recv_buf);
}

fn announce_tracker(
    socket: &UdpSocket,
    torrent_info: &torrents::Info,
    hashed_info: &[u8; 20],
    peer_id: &ByteBuffer,
    conn_resp: utils::ConnResp,
) -> anyhow::Result<utils::AnnounceResp> {
    let peer_id = utils::gen_peer_id();
    let announce_req =
        utils::build_announce_req(torrent_info, hashed_info, conn_resp.connection_id, &peer_id, PORT);

    socket
        .send(&announce_req.to_bytes())
        .expect("Couldn't send annouce req");

    let mut recv_buf = [0; 1000];
    let recieved = socket
        .recv(&mut recv_buf)
        .expect("Couldn't recieve announce response");

    let announce_resp =
        utils::parse_announce_resp(&recv_buf, recieved).expect("Couldn't parse the announce resp");

    Ok(announce_resp)
}
