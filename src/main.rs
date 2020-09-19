mod utils;
use utils::torrents;

use anyhow;
use std::{net::UdpSocket, time::Duration};
use url::Url;

const PORT: i16 = 6682;

fn main() -> anyhow::Result<()> {
    let torrent = torrents::decode_file("big-buck-bunny.torrent")?;
    torrents::render_torrent(&torrent);

    let mut peers: Vec<utils::SeederInfo> = Vec::new();
    get_torrent_peers(&torrent, &mut peers);
    println!("{:#?}", peers);

    Ok(())
}

fn get_torrent_peers(torrent: &torrents::Torrent, peers: &mut Vec<utils::SeederInfo>) {
    let announce = &torrent.announce.clone().unwrap();
    let tracker_url = Url::parse(announce).unwrap();
    let base_tracker_url = format!(
        "{}:{}",
        tracker_url.host_str().unwrap(),
        tracker_url.port().unwrap()
    );

    let socket = UdpSocket::bind(format!("0.0.0.0:{}", PORT)).unwrap();
    socket
        .set_read_timeout(Some(Duration::new(1, 0)))
        .expect("Couldn't set UDP socket read timeout");

    let conn_resp = connect_tracker(&socket, base_tracker_url);
    let announce_resp = announce_tracker(&socket, &torrent.info, conn_resp)
        .expect("Not able to get peers from tracker");

    if announce_resp.seeders == 0 {
        panic!("No peers at the moment");
    } else {
        peers.extend_from_slice(&announce_resp.seeder_info);
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
    conn_resp: utils::ConnResp,
) -> anyhow::Result<utils::AnnounceResp> {
    let peer_id = utils::gen_peer_id();
    let announce_req =
        utils::build_announce_req(torrent_info, conn_resp.connection_id, peer_id, PORT);

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
