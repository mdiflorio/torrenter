use bytebuffer::ByteBuffer;

use crate::queue::PieceBlock;

#[derive(Debug)]
pub struct GenericPayload {
    pub(crate) index: u32,
    pub(crate) begin: u32,
    pub(crate) length: Option<u32>,
    piece_index: Option<u32>,
    pub(crate) block: Option<ByteBuffer>,
    pub(crate) bitfield: Option<ByteBuffer>,
}

#[derive(Debug)]
pub struct Msg {
    size: u32,
    pub id: u8,
    pub payload: GenericPayload,
}


pub fn get_msg_id(msg: &mut ByteBuffer) -> u8 {
    if msg.len() > 4 {
        msg.to_bytes()[4]
    } else { 0 }
}


pub fn parse(msg: &mut ByteBuffer) -> Msg {
    let mut rest: ByteBuffer = ByteBuffer::new();
    let size = msg.read_u32();
    let mut index: u32 = 0;
    let mut begin: u32 = 0;

    let id = get_msg_id(msg);

    let mut payload_bytes: ByteBuffer = ByteBuffer::new();

    if msg.len() > 5 {
        payload_bytes.write_bytes(&msg.to_bytes()[5..msg.len()]);
    } else {
        payload_bytes.write_u8(0);
    };

    match id {
        // if message request, piece or cancel
        6 | 7 | 8 | 9 => {
            rest.write_bytes(&payload_bytes.to_bytes()[8..payload_bytes.len()]);
            index = payload_bytes.read_u32();
            begin = payload_bytes.read_u32();
        }
        _ => {}
    }

    let mut payload = GenericPayload {
        index,
        begin,
        length: None,
        block: None,
        bitfield: None,
        piece_index: None,
    };

    // Fill payload with different data depending on the message type.
    match id {
        // Choke, unchoke, interested, uninterested.
        0 | 1 | 2 | 3 => payload.length = Some(rest.len() as u32),
        // Have
        4 => payload.piece_index = Some(rest.read_u32()),
        // Bitfield
        5 => payload.bitfield = Some(payload_bytes),
        // Request, cancel
        6 | 8 => payload.length = Some(rest.read_u32()),
        // Piece
        7 => payload.block = Some(rest),
        _ => {}
    };


    return Msg {
        size,
        id,
        payload,
    };
}

/// The handshake is a required message and must be the first message transmitted by the client.
/// It is (49+len(pstr)) bytes long.
///
///     handshake: <pstrlen><pstr><reserved><info_hash><peer_id>
///
///     pstrlen: string length of <pstr>, as a single raw byte
///
///     pstr: string identifier of the protocol
///
///     reserved: eight (8) reserved bytes. All current implementations use all zeroes.
///     Each bit in these bytes can be used to change the behavior of the protocol.
///     An email from Bram suggests that trailing bits should be used first, so that leading bits may be used to change the meaning of trailing bits.
///
///     info_hash: 20-byte SHA1 hash of the info key in the metainfo file. This is the same info_hash that is transmitted in tracker requests.
///
///     peer_id: 20-byte string used as a unique ID for the client.
///     This is usually the same peer_id that is transmitted in tracker requests (but not always e.g. an anonymity option in Azureus).
///
///    In version 1.0 of the BitTorrent protocol, pstrlen = 19, and pstr = "BitTorrent protocol".
pub fn build_peer_handshake(info_hash: &[u8; 20], peer_id: &ByteBuffer) -> ByteBuffer {
    let mut handshake: ByteBuffer = ByteBuffer::new();
    handshake.write_u8(19);
    handshake.write_bytes("BitTorrent protocol".as_bytes());
    handshake.write_u64(0);
    handshake.write_bytes(info_hash);
    handshake.write_bytes(&peer_id.to_bytes());

    return handshake;
}


// Each message has the following format:
// <length prefix><message ID><payload>

pub fn build_keep_alive() -> ByteBuffer {
    // keep-alive: <len=0000>

    let mut buf: ByteBuffer = ByteBuffer::new();

    buf.write_u32(0);

    return buf;
}


pub fn build_choke() -> ByteBuffer {
    // choke: <len=0001><id=0>

    let mut buf: ByteBuffer = ByteBuffer::new();

    buf.write_u32(1);
    buf.write_u8(0);

    return buf;
}

pub fn build_unchoke() -> ByteBuffer {
    // unchoke: <len=0001><id=1>

    let mut buf: ByteBuffer = ByteBuffer::new();

    buf.write_u32(1);
    buf.write_u8(1);

    return buf;
}


pub fn build_interested() -> ByteBuffer {
    // interested: <len=0001><id=2>

    let mut buf: ByteBuffer = ByteBuffer::new();

    buf.write_u32(1);
    buf.write_u8(2);

    return buf;
}


pub fn build_not_interested() -> ByteBuffer {
    // not interested: <len=0001><id=3>

    let mut buf: ByteBuffer = ByteBuffer::new();

    buf.write_u32(1);
    buf.write_u8(3);

    return buf;
}


pub fn build_have(piece_index: u32) -> ByteBuffer {
    // have: <len=0005><id=4><piece index>

    let mut buf: ByteBuffer = ByteBuffer::new();

    buf.write_u32(5);
    buf.write_u8(4);
    buf.write_u32(piece_index);

    return buf;
}

/// The bitfield message may only be sent immediately after
/// the handshaking sequence is completed, and before any other
/// messages are sent. It is optional, and need not be sent if
/// a client has no pieces.
///
/// bitfield: <len=0001+X><id=5><bitfield>
pub fn build_bitfield(bitfield: &ByteBuffer) -> ByteBuffer {
    let mut buf: ByteBuffer = ByteBuffer::new();

    buf.write_u32((bitfield.len() + 1) as u32);
    buf.write_u8(5);
    buf.write_bytes(&bitfield.to_bytes());

    return buf;
}


///
///   The request message is fixed length, and is used to request a block. The payload contains the following information:
///
///   index: integer specifying the zero-based piece index
///   begin: integer specifying the zero-based byte offset within the piece
///   length: integer specifying the requested length.
///
///   request: <len=0013><id=6><index><begin><length>
pub fn build_request(payload: PieceBlock) -> ByteBuffer {
    let mut buf: ByteBuffer = ByteBuffer::new();

    buf.write_u32(13);
    buf.write_u8(6);

    buf.write_u32(payload.index as u32);
    buf.write_u32(payload.begin as u32);
    buf.write_u32(payload.length.unwrap() as u32);

    return buf;
}


///     The piece message is variable length, where X is the length of the block. The payload contains the following information:
///
///     index: integer specifying the zero-based piece index
///     begin: integer specifying the zero-based byte offset within the piece
///     block: block of data, which is a subset of the piece specified by index.
///
///     piece: <len=0009+X><id=7><index><begin><block>
pub fn build_piece(payload: &GenericPayload) -> ByteBuffer {
    let mut buf: ByteBuffer = ByteBuffer::new();

    buf.write_u32(9 + payload.block.as_ref().unwrap().len() as u32);
    buf.write_u8(7);

    buf.write_u32(payload.index);
    buf.write_u32(payload.begin);
    buf.write_bytes(&payload.block.as_ref().unwrap().to_bytes());

    return buf;
}


///  The cancel message is fixed length, and is used to cancel block requests.
///  The payload is identical to that of the "request" message.
///  It is typically used during "End Game"
///
///  index: integer specifying the zero-based piece index
///  begin: integer specifying the zero-based byte offset within the piece
///  length: integer specifying the requested length.
///
///  cancel: <len=0013><id=8><index><begin><length>
///
pub fn build_cancel(payload: GenericPayload) -> ByteBuffer {
    let mut buf: ByteBuffer = ByteBuffer::new();

    buf.write_u32(13);
    buf.write_u8(8);

    buf.write_u32(payload.index);
    buf.write_u32(payload.begin);
    buf.write_u32(payload.length.unwrap_or(0));

    return buf;
}


/// The port message is sent by newer versions of
/// the Mainline that implements a DHT tracker.
/// The listen port is the port this peer's DHT node is listening on.
/// This peer should be inserted in the local routing table (if DHT tracker is supported).
///
/// port: <len=0003><id=9><listen-port>
///
pub fn build_port(port: u16) -> ByteBuffer {
    let mut buf: ByteBuffer = ByteBuffer::new();

    buf.write_u32(3);
    buf.write_u8(9);

    buf.write_u16(port);

    return buf;
}
