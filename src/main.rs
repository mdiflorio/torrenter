mod utils;
use utils::torrents;

use anyhow;
use std::{net::UdpSocket, time::Duration};
use url::Url;

const PORT: i16 = 6682;

fn main() -> anyhow::Result<()> {
    let torrent = torrents::decode_file("big-buck-bunny.torrent")?;
    torrents::render_torrent(&torrent);

    let tracker_url = Url::parse(&torrent.announce.unwrap()).unwrap();
    let base_tracker_url = format!(
        "{}:{}",
        tracker_url.host_str().unwrap(),
        tracker_url.port().unwrap()
    );

    {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", PORT)).unwrap();
        socket.set_read_timeout(Some(Duration::new(5, 0)))?;

        let conn_req = utils::build_conn_req();

        socket
            .connect("tracker.opentrackr.org:1337")
            .expect("couldn't connect to address");

        socket
            .send(&conn_req.to_bytes())
            .expect("couldn't send message");

        let mut recv_buf = [0; 16];
        match socket.recv(&mut recv_buf) {
            Ok(received) => println!("received {} bytes {:?}", received, &recv_buf[..4]),
            Err(e) => println!("recv function failed: {:?}", e),
        }

        let conn_resp = utils::parse_conn_resp(&recv_buf);
        let peer_id = utils::gen_peer_id();
        let info_hash = utils::hash_torrent_info(&torrent.info);
        let left = utils::calculate_left(&torrent.info);
        let announce_req =
            utils::build_announce_req(info_hash, left, conn_resp.connection_id, peer_id, PORT);
    }
    Ok(())
}
