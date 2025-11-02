/// Simple cursor struct, `std::io::Cursor` does not have the methods we want.
pub struct BCursor<'a> {
    inner: &'a [u8],
    pos: usize,
}

impl<'a> BCursor<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            inner: bytes,
            pos: 0,
        }
    }

    pub fn try_capture(&mut self, by: usize) -> anyhow::Result<&'a [u8]> {
        if let Some(slice) = self.inner.get(self.pos..self.pos + by) {
            self.pos += by;
            Ok(slice)
        } else {
            anyhow::bail!("Insufficient bytes")
        }
    }

    pub fn try_capture_exact<const BY: usize>(&mut self) -> anyhow::Result<[u8; BY]> {
        let mut bytes_exact: [u8; BY] = [0; BY];
        bytes_exact.copy_from_slice(self.try_capture(BY)?);
        Ok(bytes_exact)
    }

    pub fn get_rest(self) -> &'a [u8] {
        &self.inner[self.pos..]
    }
}
