mod torrents;

use anyhow;

fn main() -> anyhow::Result<()> {
    let torrent = torrents::decode_file("big-buck-bunny.torrent")?;
    torrents::render_torrent(&torrent);
    Ok(())
}
