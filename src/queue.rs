use std::collections::VecDeque;

use crate::utils::torrents::{BLOCK_LEN, get_block_len, get_blocks_per_piece, Torrent};

#[derive(Debug, Copy, Clone)]
pub struct PieceBlock {
    pub index: u64,
    pub begin: u64,
    pub length: u64,
}

/// Job queue which tracks the blocks and pieces that need to be downloaded from a given peer
pub struct Queue<'a> {
    torrent: &'a Torrent,
    pub(crate) choked: bool,
    pub(crate) pieces: VecDeque<PieceBlock>,
}

impl Queue<'_> {
    pub fn new(torrent: &Torrent) -> Queue {
        Queue {
            choked: true,
            pieces: VecDeque::new(),
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
            self.pieces.push_back(piece_block);
        }
    }

    /// Remove the first item from the pieces queue.
    pub fn deque(&mut self) {
        self.pieces.pop_front();
    }

    /// Get the first item in pieces queue.
    pub fn peek(self) -> PieceBlock {
        return self.pieces[0];
    }

    /// Get the length of the pieces queue.
    pub fn len(self) -> usize {
        return self.pieces.len();
    }
}
