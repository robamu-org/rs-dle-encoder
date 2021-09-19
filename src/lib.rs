use std::io::Read;

const STX_CHAR: u8 = 0x02;
const ETX_CHAR: u8 = 0x03;
const DLE_CHAR: u8 = 0x10;
const CR_CHAR: u8 = 0x0d;

#[derive(Copy, Clone)]
pub struct DleEncoder {
    pub escape_stx_etx: bool,
    pub escape_cr: bool,
    pub add_stx_etx: bool
}

#[derive(Debug)]
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

    /// This method encodes a given byte stream with ASCII based DLE encoding.
    /// It returns the number of encoded bytes or a DLE error code.
    /// 
    /// # Arguments
    /// 
    /// * `source_stream` - The stream to encode
    /// * `dest_stream` - Encoded stream will be written here
    /// 
    /// # Examples
    /// 
    /// ```
    /// use rs_dle_encoder::DleEncoder;
    /// 
    /// let dle_encoder = DleEncoder::default();
    /// let mut encoding_buffer: [u8; 16] = [0; 16];
    /// let example_array: [u8; 3] = [0, 0x02, 0x10];
    /// 
    /// let encode_result = dle_encoder.encode(
    ///     &example_array, &mut encoding_buffer
    /// );
    /// assert!(encode_result.is_ok());
    /// let encoded_len = encode_result.unwrap();
    /// assert_eq!(encoded_len, 7);
    /// 
    /// println!("Source buffer: {:?}", example_array);
    /// println!("Encoded stream: {:?}", &encoding_buffer[ .. encoded_len])
    /// ```
    pub fn encode(&self,
        source_stream: &[u8], dest_stream: &mut[u8]
    ) -> Result<usize, DleError> {
        if self.escape_stx_etx {
            return self.encode_escaped(
                source_stream, dest_stream
            )
        }
        else {
            return self.encode_non_escaped(
                source_stream, dest_stream
            )
        }
    }

    pub fn encode_escaped(&self,
        source_stream: &[u8], dest_stream: &mut[u8]
    ) -> Result<usize, DleError> {
        let mut encoded_idx = 0;
        let mut source_idx = 0;
        let max_dest_len = dest_stream.len();
        if self.add_stx_etx {
            if max_dest_len < 1 {
                return Err(DleError::StreamTooShort)
            }
            dest_stream[encoded_idx] = STX_CHAR;
            encoded_idx += 1;
        }
        while encoded_idx < max_dest_len && source_idx < source_stream.len() {
            let next_byte = source_stream[source_idx];
            if next_byte == STX_CHAR || next_byte == ETX_CHAR || 
                (self.escape_cr && next_byte == CR_CHAR) {
                if encoded_idx + 1 > max_dest_len {
                    return Err(DleError::StreamTooShort)
                }
                else {
                    dest_stream[encoded_idx] = DLE_CHAR;
                    encoded_idx += 1;
                    // Next byte will be the actual byte + 0x40. This prevents STX and ETX from
                    // appearin in the encoded data stream at all, so when polling an encoded
                    // stream, the transmission can be stopped at ETX. 0x40 was chose at random
                    // with special requirements:
                    // - Prevent going from one control char to another
                    // - Prevent overflow for common characters
                    dest_stream[encoded_idx] =next_byte + 0x40;
                }
            }
            else if next_byte == DLE_CHAR {
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

        if source_idx == source_stream.len() {
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

    pub fn encode_non_escaped(
        &self, source_stream: &[u8], dest_stream: &mut[u8]
    )-> Result<usize, DleError> {
        let mut encoded_idx = 0;
        let mut source_idx = 0;
        let max_dest_len = dest_stream.len();
        if self.add_stx_etx {
            if max_dest_len < 2 {
                return Err(DleError::StreamTooShort)
            }
            dest_stream[encoded_idx] = DLE_CHAR;
            encoded_idx += 1;
            dest_stream[encoded_idx] = STX_CHAR;
            encoded_idx += 1;
        }

        while encoded_idx < max_dest_len && source_idx < source_stream.len() {
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

        if source_idx == source_stream.len() {
            if self.add_stx_etx {
                if encoded_idx + 2 >= max_dest_len {
                    return Err(DleError::StreamTooShort)
                }
                dest_stream[encoded_idx] = DLE_CHAR;
                encoded_idx += 1;
                dest_stream[encoded_idx] = ETX_CHAR;
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
        max_dest_size: usize, read_len: &mut usize
    ) -> Result<usize, DleError> {
        if self.escape_stx_etx {
            return self.decode_escaped_stream(
                source_stream, source_len, dest_stream, max_dest_size,
                read_len
            )
        }
        else {
            return self.decode_non_escaped_stream(
                source_stream, source_len, dest_stream, max_dest_size,
                read_len
            )
        }
    }

    pub fn decode_escaped_stream(&self, 
        source_stream: &[u8], source_len: usize, dest_stream: &mut[u8],
        max_dest_size: usize, read_len: &mut usize
    ) -> Result<usize, DleError> {
        let mut encoded_idx = 0;
        let mut decoded_idx = 0;
        *read_len = 0;
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
            if source_stream[encoded_idx] == DLE_CHAR {
                if encoded_idx + 1 >= source_len {
                    *read_len = source_len;
                    return Err(DleError::DecodingError)
                }
                let next_byte = source_stream[encoded_idx];
                encoded_idx += 1;
                if next_byte == DLE_CHAR {
                    dest_stream[decoded_idx] = next_byte;
                }
                else {
                    if next_byte == STX_CHAR + 0x40 ||
                            next_byte == ETX_CHAR + 0x40 ||
                            (self.escape_cr && next_byte == CR_CHAR + 0x40) {
                        dest_stream[decoded_idx] = next_byte - 0x40;
                    }
                    else {
                        *read_len = encoded_idx + 2;
                        return Err(DleError::DecodingError)
                    }
                }
                encoded_idx += 1
            }
            else {
                dest_stream[decoded_idx] = source_stream[encoded_idx];
            }
            encoded_idx += 1;
            decoded_idx += 1
        }

        if source_stream[encoded_idx] != ETX_CHAR {
            if decoded_idx == max_dest_size {
                *read_len = 0;
                return Err(DleError::StreamTooShort)
            }
            else {
                *read_len = encoded_idx + 1;
                return Err(DleError::DecodingError)
            }
        }
        else {
            *read_len = encoded_idx + 1;
            Ok(decoded_idx)
        }
    }

    pub fn decode_non_escaped_stream(&self, 
        source_stream: &[u8], source_len: usize, dest_stream: &mut[u8],
        max_dest_size: usize, read_len: &mut usize
    ) -> Result<usize, DleError> {
        let mut encoded_idx = 0;
        let mut decoded_idx = 0;
        *read_len = 0;

        if max_dest_size < 2 {
            return Err(DleError::StreamTooShort)
        }
        if source_stream[encoded_idx] != DLE_CHAR {
            return Err(DleError::DecodingError)
        }
        encoded_idx += 1;
        if source_stream[encoded_idx] != STX_CHAR {
            *read_len = 1;
            return Err(DleError::DecodingError)
        }
        while encoded_idx < source_len && decoded_idx < max_dest_size {
            if source_stream[encoded_idx] == DLE_CHAR {
                if encoded_idx + 1 >= source_len {
                    *read_len = encoded_idx;
                    return Err(DleError::DecodingError)
                }
                let next_byte = source_stream[encoded_idx + 1];
                if next_byte == STX_CHAR {
                    // Set read_len so the DLE/STX char combination is preserved
                    // It could be the start of another frame
                    *read_len = encoded_idx;
                    return Err(DleError::DecodingError)
                }
                else if next_byte == DLE_CHAR {
                    dest_stream[decoded_idx] = next_byte;
                    encoded_idx += 1;
                }
                else if next_byte == ETX_CHAR {
                    // End of stream reached
                    *read_len = encoded_idx + 2;
                    return Ok(decoded_idx)
                }
                else {
                    *read_len = encoded_idx;
                    return Err(DleError::DecodingError)
                }
            }
            else {
                dest_stream[decoded_idx] = source_stream[encoded_idx];
            }
            encoded_idx += 1;
            decoded_idx += 1;
        }

        if decoded_idx == max_dest_size {
            // So far we did not find anything wrong here, let the user try
            // again
            *read_len = 0;
            return Err(DleError::StreamTooShort)
        }
        else {
            *read_len = encoded_idx;
            return Err(DleError::DecodingError)
        }
    }

    pub fn decode_from_reader(source: &impl std::io::Read) {

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ARRAY_0: &[u8] = &[0, 0, 0, 0, 0];
    const TEST_ARRAY_1: &[u8] = &[0, DLE_CHAR, 5];
    const TEST_ARRAY_2: &[u8] = &[0, STX_CHAR, 5];
    const TEST_ARRAY_3: &[u8] = &[0, CR_CHAR, ETX_CHAR];
    const TEST_ARRAY_4: &[u8] = &[DLE_CHAR, ETX_CHAR, STX_CHAR];
    
    const TEST_ARRAY_0_ENCODED_ESCPAED: &[u8] = &[
        STX_CHAR, 0, 0, 0, 0, 0, ETX_CHAR
    ];
    const TEST_ARRAY_0_ENCODED_NON_ESCPAED: &[u8] = &[
        DLE_CHAR, STX_CHAR, 0, 0, 0, 0, 0, DLE_CHAR, ETX_CHAR
    ];

    const TEST_ARRAY_1_ENCODED_ESCPAED: &[u8] = &[
        STX_CHAR, 0, DLE_CHAR, DLE_CHAR, 5, ETX_CHAR
    ];
    const TEST_ARRAY_1_ENCODED_NON_ESCPAED: &[u8] = &[
        DLE_CHAR, STX_CHAR, 0, DLE_CHAR, DLE_CHAR, 5, DLE_CHAR, ETX_CHAR
    ];

    const TEST_ARRAY_2_ENCODED_ESCPAED: &[u8] = &[
        STX_CHAR, 0, DLE_CHAR, STX_CHAR + 0x40, 5, ETX_CHAR
    ];
    const TEST_ARRAY_2_ENCODED_NON_ESCPAED: &[u8] = &[
        DLE_CHAR, STX_CHAR, 0, STX_CHAR, 5, DLE_CHAR, ETX_CHAR
    ];

    const TEST_ARRAY_3_ENCODED_ESCPAED: &[u8] = &[
        STX_CHAR, 0, CR_CHAR, DLE_CHAR, ETX_CHAR + 0x40, ETX_CHAR
    ];
    const TEST_ARRAY_3_ENCODED_NON_ESCPAED: &[u8] = &[
        DLE_CHAR, STX_CHAR, 0, CR_CHAR, ETX_CHAR, DLE_CHAR, ETX_CHAR
    ];

    const TEST_ARRAY_4_ENCODED_ESCPAED: &[u8] = &[
        STX_CHAR, DLE_CHAR, DLE_CHAR, DLE_CHAR, ETX_CHAR + 0x40, DLE_CHAR, 
        STX_CHAR + 0x40, ETX_CHAR
    ];
    const TEST_ARRAY_4_ENCODED_NON_ESCPAED: &[u8] = &[
        DLE_CHAR, STX_CHAR, DLE_CHAR, DLE_CHAR, ETX_CHAR, STX_CHAR, DLE_CHAR, ETX_CHAR
    ];

    #[test]
    fn test_encoder() {
        let mut dle_encoder = DleEncoder::default();
        let mut buffer: [u8; 32] = [0; 32];
        let mut test_encode_closure = |
            dle_encoder: &DleEncoder, buf_to_encode: &[u8], expected_buf: &[u8]| {
            let encode_res = dle_encoder.encode(buf_to_encode, &mut buffer);
            assert!(encode_res.is_ok());
            for (idx, byte) in expected_buf.iter().enumerate() {
                assert_eq!(buffer[idx], *byte);
            }
            assert_eq!(encode_res.unwrap(), expected_buf.len());
        };

        test_encode_closure(&dle_encoder, TEST_ARRAY_0, TEST_ARRAY_0_ENCODED_ESCPAED);
        test_encode_closure(&dle_encoder, TEST_ARRAY_1, TEST_ARRAY_1_ENCODED_ESCPAED);
        test_encode_closure(&dle_encoder, TEST_ARRAY_2, TEST_ARRAY_2_ENCODED_ESCPAED);
        test_encode_closure(&dle_encoder, TEST_ARRAY_3, TEST_ARRAY_3_ENCODED_ESCPAED);
        test_encode_closure(&dle_encoder, TEST_ARRAY_4, TEST_ARRAY_4_ENCODED_ESCPAED);

        dle_encoder.escape_stx_etx = false;
        test_encode_closure(&dle_encoder, TEST_ARRAY_0, TEST_ARRAY_0_ENCODED_NON_ESCPAED);
        test_encode_closure(&dle_encoder, TEST_ARRAY_1, TEST_ARRAY_1_ENCODED_NON_ESCPAED);
        test_encode_closure(&dle_encoder, TEST_ARRAY_2, TEST_ARRAY_2_ENCODED_NON_ESCPAED);
        test_encode_closure(&dle_encoder, TEST_ARRAY_3, TEST_ARRAY_3_ENCODED_NON_ESCPAED);
        test_encode_closure(&dle_encoder, TEST_ARRAY_4, TEST_ARRAY_4_ENCODED_NON_ESCPAED);
    }
}
