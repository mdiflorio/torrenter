use std::net::UdpSocket;
use std::time::Duration;

use bytebuffer::ByteBuffer;
use url::Url;

use crate::{PORT, utils};
use crate::utils::torrents;

pub fn get_torrent_peers(
    torrent: &torrents::Torrent,
    hashed_info: &[u8; 20],
    peer_id: &ByteBuffer,
) -> anyhow::Result<Vec<utils::Peer>> {
    let tracker_url = Url::parse(&torrent.announce.as_ref().unwrap()).unwrap();
    let base_tracker_url = format!(
        "{}:{}",
        tracker_url.host_str().unwrap(),
        tracker_url.port().unwrap()
    );

    let socket = UdpSocket::bind(format!("0.0.0.0:{}", PORT)).unwrap();
    socket.set_read_timeout(Some(Duration::new(5, 0)))?;

    let conn_resp = connect_tracker(&socket, base_tracker_url);

    let announce_resp = announce_tracker(&socket, &torrent.info, hashed_info, peer_id, conn_resp)
        .expect("Not able to get peers from tracker");

    if announce_resp.seeders == 0 {
        anyhow::bail!("No peers at the moment");
    } else {
        return Ok(announce_resp.peers);
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

    socket.recv(&mut recv_buf).expect("Couldn't receive tracker message");

    return utils::parse_conn_resp(&recv_buf);
}

fn announce_tracker(
    socket: &UdpSocket,
    torrent_info: &torrents::Info,
    hashed_info: &[u8; 20],
    peer_id: &ByteBuffer,
    conn_resp: utils::ConnResp,
) -> anyhow::Result<utils::AnnounceResp> {
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
