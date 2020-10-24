use std::collections::VecDeque;

use crate::utils::torrents::{BLOCK_LEN, get_block_len, get_blocks_per_piece, Torrent};

#[derive(Debug, Copy, Clone)]
struct PieceBlock {
    index: u64,
    begin: u64,
    length: u64,
}

/// Job queue which tracks the blocks and pieces that need to be downloaded from a given peer
pub struct Queue<'a> {
    torrent: &'a Torrent,
    choked: bool,
    pieces: VecDeque<PieceBlock>,
}

impl Queue<'_> {
    pub fn new(torrent: &Torrent, size: usize) -> Queue {
        Queue {
            choked: true,
            pieces: VecDeque::with_capacity(size),
            torrent,
        }
    }

    /// Add the blocks from a given piece_index into the job queue
    pub fn queue(&mut self, piece_index: u64) {
        let num_blocks = get_blocks_per_piece(self.torrent, piece_index);
        for i in 0..num_blocks {
            let piece_block = PieceBlock {
                index: piece_index,
                begin: i * BLOCK_LEN,
                length: get_block_len(self.torrent, piece_index, i),
            };
        }
    }
}
