/// Simple cursor struct for bytes, somewhat similar to `std::io::Cursor`
/// but with nicer methods for our use case (no read/write abstraction pain)
#[derive(Debug)]
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
        self.inner
            .get(self.pos..self.pos + by)
            .inspect(|_| self.pos += by)
            .ok_or_else(|| anyhow::anyhow!("Failed to capture {by} bytes, out of bounds"))
    }

    pub fn try_seek(&mut self, by: usize) -> anyhow::Result<&'a [u8]> {
        self.inner
            .get(self.pos..self.pos + by)
            .ok_or_else(|| anyhow::anyhow!("Failed to seek {by} bytes, out of bounds"))
    }

    pub fn try_capture_exact<const BY: usize>(&mut self) -> anyhow::Result<[u8; BY]> {
        let mut bytes_exact: [u8; BY] = [0; BY];
        bytes_exact.copy_from_slice(self.try_capture(BY)?);
        Ok(bytes_exact)
    }
}
