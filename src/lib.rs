struct DleEncoder {
    escape_stx_etx: bool,
    escape_cr: bool
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
    pub fn encode() {

    }

    pub fn decode() {

    }
}