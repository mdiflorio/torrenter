mod torrents;

use anyhow;
use bytebuffer::ByteBuffer;
use std::net::UdpSocket;
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
        let mut socket = UdpSocket::bind("0.0.0.0:0").unwrap();

        let connReq = buildConnReq();

        socket
            .connect("tracker.leechers-paradise.org:6969")
            .expect("couldn't connect to address");

        socket
            .send(&connReq.to_bytes())
            .expect("couldn't send message");

        let mut buf = [0; 200];
        match socket.recv(&mut buf) {
            Ok(received) => println!("received {} bytes {:?}", received, &buf[..received]),
            Err(e) => println!("recv function failed: {:?}", e),
        }
    }
    Ok(())
}

fn buildConnReq() -> ByteBuffer {
    let mut buffer = ByteBuffer::new();
    buffer.write_u64(0x41727101980);
    buffer.write_u32(0);
    buffer.write_u32(98766687);
    return buffer;
}
