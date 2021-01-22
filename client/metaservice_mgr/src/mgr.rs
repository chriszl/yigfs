use crate::types::DirEntry;
use crate::types::FileAttr;

pub trait MetaServiceMgr {
    fn read_dir(&self, ino: u64, offset: i64)->Result<Vec<DirEntry>, String>;
    fn read_dir_file_attr(&self, ino: u64, name: &String) -> Result<Vec<FileAttr>, String>;
}