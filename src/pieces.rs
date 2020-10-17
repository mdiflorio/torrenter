#[derive(Debug)]
pub struct Pieces {
    requested: Vec<bool>,
    received: Vec<bool>,
}

impl Pieces {
    pub fn new(size: usize) -> Pieces {
        Pieces {
            requested: vec![false; size],
            received: vec![false; size],
        }
    }

    pub fn add_requested(&mut self, piece_index: usize) {
        self.requested[piece_index] = true;
    }


    pub fn add_received(&mut self, piece_index: usize) {
        self.received[piece_index] = true;
    }

    pub fn needed(&mut self, piece_index: usize) -> bool {
        let mut requested_all_pieces = true;

        for index in &self.requested {
            if !index {
                requested_all_pieces = false;
                break;
            }
        }

        if requested_all_pieces {
            self.requested = self.received.clone();
        }

        return !self.requested[piece_index];
    }

    pub fn is_done(self) -> bool {
        let mut received_every_piece = true;

        for index in self.received {
            if !index {
                received_every_piece = false;
                break;
            }
        }
        return received_every_piece;
    }
}