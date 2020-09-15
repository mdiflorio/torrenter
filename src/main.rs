mod torrents;

use anyhow;
use bytebuffer::ByteBuffer;
use core::convert::TryInto;
use rand::Rng;
use std::{net::UdpSocket, time::Duration};
use url::Url;

fn main() -> anyhow::Result<()> {
    let torrent = torrents::decode_file("big-buck-bunny.torrent")?;
    torrents::render_torrent(&torrent);

    // const url = urlParse(torrent.announce.toString('utf8'));

    // // 3
    // const socket = dgram.createSocket('udp4');
    // // 4
    // const myMsg = Buffer.from('hello?', 'utf8');
    // // 5
    // socket.send(myMsg, 0, myMsg.length, url.port, url.host, () => {});
    // // 6
    // socket.on('message', msg => {
    //   console.log('message is', msg);
    // });

    let tracker_url = Url::parse(&torrent.announce.unwrap()).unwrap();
    let base_tracker_url = format!(
        "{}:{}",
        tracker_url.host_str().unwrap(),
        tracker_url.port().unwrap()
    );

    {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        socket.set_read_timeout(Some(Duration::new(5, 0)))?;

        let conn_req = build_conn_req();

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

        let conn_resp = parse_conn_resp(&recv_buf);

        println!("{:?}", conn_resp.action.to_be_bytes());
        println!("{:?}", conn_resp.transaction_id.to_be_bytes());
        println!("{:?}", conn_resp.connection_id.to_be_bytes());
    }
    Ok(())
}

fn build_conn_req() -> ByteBuffer {
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

struct ConnResp {
    action: i32,
    transaction_id: i32,
    connection_id: i64,
}

fn parse_conn_resp(buf: &[u8; 16]) -> ConnResp {
    let conn_resp = ConnResp {
        action: i32::from_be_bytes(buf[..4].try_into().unwrap()),
        transaction_id: i32::from_be_bytes(buf[4..8].try_into().unwrap()),
        connection_id: i64::from_be_bytes(buf[8..16].try_into().unwrap()),
    };

    return conn_resp;
}
