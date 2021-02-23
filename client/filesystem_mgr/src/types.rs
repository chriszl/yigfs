pub use segment_mgr::types::{Segment};
pub use crossbeam_channel::{Sender};

#[derive(Debug)]
pub struct FileHandle {
    pub ino: u64,
    pub leader: String,
    pub segments: Vec<Segment>,
}

impl FileHandle {
    pub fn copy(&self)->Self {
        let mut handle = FileHandle{
            ino: self.ino,
            leader: self.leader.clone(),
            segments: Vec::<Segment>::new(),
        };
        for s in &self.segments {
            handle.segments.push(s.copy());
        }
        return handle;
    }
    
    pub fn new(ino: u64)->Self{
        FileHandle{
            ino: ino,
            leader: String::from(""),
            segments: Vec::<Segment>::new(),
        }
    }
}

#[derive(Debug)]
pub enum MsgUpdateHandleType {
    // add
    MsgHandleAdd = 0,
    // delete
    MsgHandleDel = 1,
}

#[derive(Debug)]
pub struct MsgUpdateHandle{
    pub update_type: MsgUpdateHandleType,
    pub handle: FileHandle,
}

#[derive(Debug)]
pub struct MsgQueryHandle{
    pub ino: u64,
    pub tx: Sender<Option<FileHandle>>,
}

#[derive(Debug)]
pub enum MsgFileHandleOp{
    Add(FileHandle),
    Del(u64),
    Get(MsgQueryHandle),
}