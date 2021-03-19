use common::error::Errno;
use crate::types::BlockIo;
pub trait Leader {
    // open the segment for io
    fn open(&self, ino: u64) -> Errno;
    // write the block into the segment file.
    // all the blocks are appended to the segment file.
    fn write(&self, ino: u64, offset: u64, data: &[u8]) -> Result<BlockIo, Errno>;
    // close the file handle specified by ino.
    fn close(&self, ino: u64) -> Errno;
    // release this leader.
    fn release(&mut self);
}