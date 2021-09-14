pub struct DleEncoder {
    escape_stx_etx: bool,
    escape_cr: bool
}

pub enum DleError {
    StreamTooShort,
    DecodingError
}

impl Default for DleEncoder {
    fn default() -> DleEncoder {
        DleEncoder {
            escape_stx_etx: true,
            escape_cr: false
        }
    }
}

impl DleEncoder {
    /// This method encodes a given byte stream with ASCII based DLE encoding
    pub fn encode(
        source_stream: &[u8], source_len: usize, dest_stream: &mut[u8],
        max_dest_len: usize
    ) -> Result<usize, DleError> {
        Ok(0)
    }

    /// This method decodes an ASCII DLE encoded byte stream
    pub fn decode() {

    }
}