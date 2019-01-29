use std::fmt;

/// Wraps a byte slice to enable lowcast hex display formatting
pub(crate) struct Bytes<'a>(pub(crate) &'a [u8]);

impl fmt::LowerHex for Bytes<'_> {
    fn fmt(
        &self,
        fmt: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        for byte in self.0 {
            fmt.write_fmt(format_args!("{:02x}", byte))?
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Bytes;
    #[test]
    fn formats_lowerhex() {
        assert_eq!(format!("{:x}", Bytes(b"test")), "74657374")
    }
}
