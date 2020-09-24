use bytebuffer::ByteBuffer;

pub struct RequestPayload {
    index: u32,
    begin: u32,
    length: u32,
}

pub struct PiecePayload {
    index: u32,
    begin: u32,
    block: ByteBuffer,
}

pub fn build_peer_handshake(info_hash: &[u8; 20], peer_id: &ByteBuffer) -> ByteBuffer {
    // Handshake
    // The handshake is a required message and must be the first message transmitted by the client. It is (49+len(pstr)) bytes long.
    //
    //     handshake: <pstrlen><pstr><reserved><info_hash><peer_id>
    //
    //     pstrlen: string length of <pstr>, as a single raw byte
    //     pstr: string identifier of the protocol
    //     reserved: eight (8) reserved bytes. All current implementations use all zeroes. Each bit in these bytes can be used to change the behavior of the protocol. An email from Bram suggests that trailing bits should be used first, so that leading bits may be used to change the meaning of trailing bits.
    //     info_hash: 20-byte SHA1 hash of the info key in the metainfo file. This is the same info_hash that is transmitted in tracker requests.
    //     peer_id: 20-byte string used as a unique ID for the client. This is usually the same peer_id that is transmitted in tracker requests (but not always e.g. an anonymity option in Azureus).

    // In version 1.0 of the BitTorrent protocol, pstrlen = 19, and pstr = "BitTorrent protocol".

    let mut handshake: ByteBuffer = ByteBuffer::new();
    handshake.write_u8(19);
    handshake.write_string("BitTorrent protocol");
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

pub fn build_bitfield(bitfield: &ByteBuffer) -> ByteBuffer {
    // The bitfield message may only be sent immediately after
    // the handshaking sequence is completed, and before any other
    // messages are sent. It is optional, and need not be sent if
    // a client has no pieces.

    // bitfield: <len=0001+X><id=5><bitfield>

    let mut buf: ByteBuffer = ByteBuffer::new();

    buf.write_u32((bitfield.len() + 1) as u32);
    buf.write_u8(5);
    buf.write_bytes(&bitfield.to_bytes());

    return buf;
}


pub fn build_request(payload: RequestPayload) -> ByteBuffer {
    // request: <len=0013><id=6><index><begin><length>
    // The request message is fixed length, and is used to request a block. The payload contains the following information:
    //
    //   index: integer specifying the zero-based piece index
    //   begin: integer specifying the zero-based byte offset within the piece
    //   length: integer specifying the requested length.

    let mut buf: ByteBuffer = ByteBuffer::new();

    buf.write_u32(13);
    buf.write_u8(6);

    buf.write_u32(payload.index);
    buf.write_u32(payload.begin);
    buf.write_u32(payload.length);

    return buf;
}


pub fn build_piece(payload: PiecePayload) -> ByteBuffer {

    // piece: <len=0009+X><id=7><index><begin><block>
    //     The piece message is variable length, where X is the length of the block. The payload contains the following information:
    //
    //     index: integer specifying the zero-based piece index
    //     begin: integer specifying the zero-based byte offset within the piece
    //     block: block of data, which is a subset of the piece specified by index.

    let mut buf: ByteBuffer = ByteBuffer::new();

    buf.write_u32(9 + payload.block.len() as u32);
    buf.write_u8(7);

    buf.write_u32(payload.index);
    buf.write_u32(payload.begin);
    buf.write_bytes(&payload.block.to_bytes());

    return buf;
}


pub fn build_cancel(payload: RequestPayload) -> ByteBuffer {
    // cancel: <len=0013><id=8><index><begin><length>

    // The cancel message is fixed length, and is used to cancel block requests.
    // The payload is identical to that of the "request" message.
    // It is typically used during "End Game"
    //
    //   index: integer specifying the zero-based piece index
    //   begin: integer specifying the zero-based byte offset within the piece
    //   length: integer specifying the requested length.

    let mut buf: ByteBuffer = ByteBuffer::new();

    buf.write_u32(13);
    buf.write_u8(8);

    buf.write_u32(payload.index);
    buf.write_u32(payload.begin);
    buf.write_u32(payload.length);

    return buf;
}


pub fn build_port(port: u16) -> ByteBuffer {
    // port: <len=0003><id=9><listen-port>

    // The port message is sent by newer versions of
    // the Mainline that implements a DHT tracker.
    // The listen port is the port this peer's DHT node is listening on.
    // This peer should be inserted in the local routing table (if DHT tracker is supported).

    let mut buf: ByteBuffer = ByteBuffer::new();

    buf.write_u32(3);
    buf.write_u8(9);

    buf.write_u16(port);

    return buf;
}
