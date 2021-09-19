use std::io::Read;

const STX_CHAR: u8 = 0x02;
const ETX_CHAR: u8 = 0x03;
const DLE_CHAR: u8 = 0x10;
const CR_CHAR: u8 = 0x0d;

pub struct DleEncoder {
    pub escape_stx_etx: bool,
    pub escape_cr: bool,
    pub add_stx_etx: bool
}

pub enum DleError {
    StreamTooShort,
    DecodingError
}

impl Default for DleEncoder {
    fn default() -> DleEncoder {
        DleEncoder {
            escape_stx_etx: true,
            escape_cr: false,
            add_stx_etx: true
        }
    }
}

impl DleEncoder {

    /// This method encodes a given byte stream with ASCII based DLE encoding
    pub fn encode(&self,
        source_stream: &[u8], source_len: usize, dest_stream: &mut[u8],
        max_dest_len: usize
    ) -> Result<usize, DleError> {
        if self.escape_stx_etx {
            return self.encode_escaped(
                source_stream, source_len, dest_stream, max_dest_len
            )
        }
        else {
            return self.encode_non_escaped(
                source_stream, source_len, dest_stream, max_dest_len
            )
        }
    }

    pub fn encode_escaped(&self,
        source_stream: &[u8], source_len: usize, dest_stream: &mut[u8],
        max_dest_len: usize
    ) -> Result<usize, DleError> {
        let mut encoded_idx = 0;
        let mut source_idx = 0;
        if self.add_stx_etx {
            if max_dest_len < 1 {
                return Err(DleError::StreamTooShort)
            }
            dest_stream[encoded_idx] = STX_CHAR
        }
        while encoded_idx < max_dest_len && source_idx < source_len {
            let next_byte = source_stream[source_idx];
            if next_byte == STX_CHAR || next_byte == ETX_CHAR || 
                (self.escape_cr && next_byte == CR_CHAR) {
                if encoded_idx + 1 > max_dest_len {
                    return Err(DleError::StreamTooShort)
                }
                else {
                    dest_stream[encoded_idx] = DLE_CHAR;
                    encoded_idx += 1;
                    dest_stream[encoded_idx] = DLE_CHAR;
                }
            }
            else {
                dest_stream[encoded_idx] = next_byte;
            }
            encoded_idx += 1;
            source_idx += 1;
        }

        if source_idx == source_len {
            if self.add_stx_etx {
                if encoded_idx + 1 >= max_dest_len {
                    return Err(DleError::StreamTooShort)
                }
                dest_stream[encoded_idx] = ETX_CHAR;
                encoded_idx += 1
            }
            Ok(encoded_idx)
        }
        else {
            return Err(DleError::StreamTooShort)
        }
    }

    pub fn encode_non_escaped(&self, 
        source_stream: &[u8], source_len: usize, dest_stream: &mut[u8],
        max_dest_len: usize
    )-> Result<usize, DleError> {
        let mut encoded_idx = 0;
        let mut source_idx = 0;
        if self.add_stx_etx {
            if max_dest_len < 2 {
                return Err(DleError::StreamTooShort)
            }
            dest_stream[encoded_idx] = DLE_CHAR;
            encoded_idx += 1;
            dest_stream[encoded_idx] = DLE_CHAR;
            encoded_idx += 1;
        }

        while encoded_idx < max_dest_len && source_idx < source_len {
            let next_byte = source_stream[source_idx];
            if next_byte == DLE_CHAR {
                if encoded_idx + 1 >= max_dest_len {
                    return Err(DleError::StreamTooShort)
                }
                else {
                    dest_stream[encoded_idx] = DLE_CHAR;
                    encoded_idx += 1;
                    dest_stream[encoded_idx] = DLE_CHAR;
                }
            }
            else {
                dest_stream[encoded_idx] = next_byte;
            }
            encoded_idx += 1;
            source_idx += 1;
        }

        if source_idx == source_len {
            if self.add_stx_etx {
                if encoded_idx + 2 >= max_dest_len {
                    return Err(DleError::StreamTooShort)
                }
                dest_stream[encoded_idx] = DLE_CHAR;
                encoded_idx += 1;
                dest_stream[encoded_idx] = DLE_CHAR;
                encoded_idx += 1;
            }
            Ok(encoded_idx)
        }
        else {
            Err(DleError::StreamTooShort)
        }
    }

    /// This method decodes an ASCII DLE encoded byte stream
    pub fn decode(&self,
        source_stream: &[u8], source_len: usize, dest_stream: &mut[u8],
        max_dest_size: usize
    ) -> Result<(usize, usize), DleError> {
        if self.escape_stx_etx {
            return self.decode_escaped_stream(
                source_stream, source_len, dest_stream, max_dest_size
            )
        }
        else {
            return self.decode_non_escaped_stream(
                source_stream, source_len, dest_stream, max_dest_size
            )
        }
    }

    pub fn decode_escaped_stream(&self, 
        source_stream: &[u8], source_len: usize, dest_stream: &mut[u8],
        max_dest_size: usize
    ) -> Result<(usize, usize), DleError> {
        let mut encoded_idx = 0;
        let mut decoded_idx = 0;
        if max_dest_size < 1 {
            return Err(DleError::StreamTooShort)
        }
        if source_stream[encoded_idx] != STX_CHAR {
            return Err(DleError::DecodingError)
        }
        encoded_idx += 1;
        while encoded_idx < source_len &&
                decoded_idx < max_dest_size &&
                source_stream[encoded_idx] != ETX_CHAR &&
                source_stream[encoded_idx] != STX_CHAR {
            
        }
        Ok((0,0))
    }

    pub fn decode_non_escaped_stream(&self, 
        source_stream: &[u8], source_len: usize, dest_stream: &mut[u8],
        max_dest_size: usize
    ) -> Result<(usize, usize), DleError> {
        Ok((0,0))
    }

    pub fn decode_from_reader(source: &impl std::io::Read) {

    }
}