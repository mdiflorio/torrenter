use crate::queue::PieceBlock;
use crate::utils::torrents::{BLOCK_LEN, Torrent};

#[derive(Debug, Clone)]
pub struct Pieces {
    requested: Vec<Vec<bool>>,
    received: Vec<Vec<bool>>,
}

impl Pieces {
    pub fn new(torrent: &Torrent) -> Pieces {
        Pieces {
            requested: build_pieces_vec(torrent),
            received: build_pieces_vec(torrent),
        }
    }

    /// Flag the requested block as true
    pub fn add_requested(&mut self, piece_block: PieceBlock) {
        let block_index = piece_block.begin / BLOCK_LEN;
        self.requested[piece_block.index as usize][block_index as usize] = true;
    }


    /// Flag the received block as true
    pub fn add_received(&mut self, piece_block: PieceBlock) {
        let block_index = piece_block.begin / BLOCK_LEN;
        self.received[piece_block.index as usize][block_index as usize] = true;
    }

    /// Find out of a piece_block as been requested.
    ///
    /// If the piece has been requested and we still haven't received the piece, it will return false.
    pub fn needed(&mut self, piece_block: PieceBlock) -> bool {
        let mut requested_all_pieces = true;
        let block_index = piece_block.begin / BLOCK_LEN;

        // Check if all pieces have been requested
        for piece in &self.requested {
            for block in piece {
                if !block {
                    requested_all_pieces = false;
                    break;
                }
            }
            if !requested_all_pieces {
                break;
            }
        }

        // If all of the pieces have been requested, replace requested with a copy of received.
        // This is used to refresh the list of requested pieces.
        if requested_all_pieces {
            self.requested = self.received.clone();
        }

        return !self.requested[piece_block.index as usize][block_index as usize];
    }

    /// Check if every piece and block has been received
    pub fn is_done(&self) -> bool {
        let mut received_every_piece = true;

        for piece in &self.received {
            for block in piece {
                if !block {
                    received_every_piece = false;
                    break;
                }
            }

            if !received_every_piece {
                break;
            }
        }
        return received_every_piece;
    }
}


/// Used to init the requested and received vecs.
///
/// - The first vec will be the length of the pieces.
/// - The nested vecs will be the length of the number of blocks per piece.
fn build_pieces_vec(torrent: &Torrent) -> Vec<Vec<bool>> {
    let num_pieces = torrent.info.pieces.len() / 20;

    // Create a vec with the length of the pieces
    let mut vec: Vec<Vec<bool>> = vec![vec![false; 0]; num_pieces];

    // For each piece, fill it with a vec which is the length of blocks for that piece
    for i in 0..num_pieces {
        let blocks_per_piece = torrent.get_blocks_per_piece(i as u64);
        vec[i] = vec![false; blocks_per_piece as usize];
    }

    return vec;
}
