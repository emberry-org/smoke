use postcard::Error;
use serde::Serialize;

pub trait PostcardSE<T> {
    fn to_bytes<'a>(&self, buf: &'a mut [u8]) -> Result<&'a mut [u8], Error>;

    fn to_cobs_bytes<'a>(&self, buf: &'a mut [u8]) -> Result<&'a mut [u8], Error>;
}

impl<T> PostcardSE<T> for T
where
    T: Serialize,
{
    fn to_bytes<'a>(&self, buf: &'a mut [u8]) -> Result<&'a mut [u8], Error> {
        postcard::to_slice(self, buf)
    }

    fn to_cobs_bytes<'a>(&self, buf: &'a mut [u8]) -> Result<&'a mut [u8], Error> {
        postcard::to_slice_cobs(self, buf)
    }
}
