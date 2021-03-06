use std::rc::Rc;
use crossbeam_channel::bounded;
use common::runtime::Executor;
use common::error::Errno;
use io_engine::types::{MsgFileOpenOp, MsgFileReadOp, MsgFileOp, MsgFileWriteOp, 
    MsgFileWriteResp, MsgFileCloseOp, MsgFileReadData};
use io_engine::io_thread_pool::IoThreadPool;
use io_engine::disk_io_worker::DiskIoWorkerFactory;
use io_engine::backend_storage::BackendStore;
use crate::leader::Leader;
use crate::file_handle::FileHandleMgr;
use crate::types::{FileHandle, Block, BlockIo, Segment};
use crate::segment_mgr::SegmentMgr;

pub struct LeaderLocal {
    machine: String,
    disk_io_pool: IoThreadPool,
    backend_store: Box<dyn BackendStore>,
    exec: Executor,
    segment_mgr: Rc<SegmentMgr>,
    handle_mgr: FileHandleMgr,
}

impl Leader for LeaderLocal {
    fn open(&self, ino: u64) -> Errno {
        let segments : Vec<Segment>;
        let ret = self.handle_mgr.get(ino);
        match ret {
            Ok(ret) => {
                println!("open: got handle for ino: {}, leader: {}", ino, ret.leader);
                return Errno::Esucc;
            }
            Err(err) => {
                if !err.is_enoent() {
                    println!("open: failed to get file handle for ino: {}, err: {:?}", ino, err);
                    return err;
                }
            }
        }
        let ret = self.segment_mgr.get_file_segments(ino, &self.machine);
        match ret {
            Ok(ret) => {
                segments = ret;
            }
            Err(err) => {
                println!("open: failed to get_file_segments for ino: {}", ino);
                return err;
            }
        }
        for seg in &segments {
            let seg_dir = self.segment_mgr.get_segment_dir(seg.seg_id0, seg.seg_id1);
            let worker = self.disk_io_pool.get_thread(seg.seg_id0, seg.seg_id1);
            let (tx, rx) = bounded::<Errno>(1);
            let msg = MsgFileOpenOp{
                id0: seg.seg_id0,
                id1: seg.seg_id1,
                dir: seg_dir,
                resp_sender: tx,
            };
            let ret = worker.do_io(MsgFileOp::OpOpen(msg));
            if !ret.is_success() {
                println!("open(id0: {}, id1: {}): failed to send open msg, err: {:?}",
                seg.seg_id0, seg.seg_id1, ret);
                return ret;
            }
            let ret = rx.recv();
            match ret {
                Ok(e) => {
                    if !e.is_success() {
                        println!("open(id0: {}, id1: {}) failed with errno: {:?}", seg.seg_id0, seg.seg_id1, e);
                        return e;
                    }
                    continue;
                }
                Err(err) => {
                    println!("open: failed to get response for (id0: {}, id1: {}), err: {}", 
                    seg.seg_id0, seg.seg_id1, err);
                    return Errno::Eintr;
                }
            }
        }

        let file_handle = FileHandle {
            ino: ino,
            leader: self.machine.clone(),
            segments: segments,
        };
        self.handle_mgr.add(&file_handle);

        return Errno::Esucc;
    }

    fn read(&self, ino: u64, offset: u64, size: u32) -> Result<Vec<u8>, Errno> {
        let file_handle : FileHandle;
        let ret = self.handle_mgr.get(ino);
        match ret{
            Ok(ret) => {
                file_handle = ret;
            }
            Err(err) => {
                println!("read: failed to get file_handle for ino: {}, err: {:?}", ino, err);
                return Err(err);
            }
        }
        let mut start = offset;
        let mut total_read = size;
        let mut data = Vec::<u8>::new();
        for s in &file_handle.segments {
            for b in &s.blocks {
                if b.offset <= start && start <= (b.offset + b.size as u64) {
                    let mut to_read = total_read;
                    if to_read >= b.size as u32 {
                        to_read = b.size as u32;
                    }
                    // read the data.
                    let seg_dir = self.segment_mgr.get_segment_dir(s.seg_id0, s.seg_id1);
                    let (tx, rx) = bounded::<MsgFileReadData>(1);
                    let msg = MsgFileReadOp{
                        id0: s.seg_id0,
                        id1: s.seg_id1,
                        dir: seg_dir,
                        offset: start,
                        size: to_read,
                        data_sender: tx,
                    };
                    let worker = self.disk_io_pool.get_thread(s.seg_id0, s.seg_id1);
                    let ret = worker.do_io(MsgFileOp::OpRead(msg));
                    if !ret.is_success(){
                        println!("read: failed to read data for ino: {}, offset: {}, err: {:?}", ino, start, ret);
                        return Err(ret);
                    }
                    let ret = rx.recv();
                    match ret {
                        Ok(ret) => {
                            if ret.err.is_success() {
                                if let Some(d) = ret.data{
                                    let l = d.len() as u32;
                                    total_read -= l as u32;
                                    start += l as u64;
                                    data.extend(d);
                                }
                            } else if ret.err.is_eof() {
                                println!("read: got eof for ino: {}, offset: {}, start: {}", ino, offset, start);
                                return Ok(data);
                            } else {
                                println!("read: got error when read data for ino: {}, offset: {}, start: {}, err: {:?}",
                                ino, offset, start, ret.err);
                                return Err(ret.err);
                            }
                        }
                        Err(err) => {
                            println!("read: failed to got response for ino: {}, offset: {}, start: {}, err: {}", 
                            ino, offset, start, err);
                        }
                    }
                    if total_read == 0 {
                        println!("read: finished for ino: {}, offset: {}", ino, offset);
                        return Ok(data);
                    }
                }
            }
        }
        return Ok(data);
    }

    fn write(&self, ino: u64, offset: u64, data: &[u8]) -> Result<BlockIo, Errno> {
        let last_segment: Vec<u64>;
        let ret = self.handle_mgr.get_last_segment(ino);
        match ret {
            Ok(ret) => {
                last_segment = ret;
            }
            Err(err) => {
                println!("write: failed to get_last_segment for ino: {}, err: {:?}", ino, err);
                return Err(err);
            }
        }
        if last_segment.is_empty() {
            println!("write: failed to get_last_segment for ino: {}, no segments found.", ino);
            return Err(Errno::Enoent);
        }
        let mut id0 = last_segment[0];
        let mut id1 = last_segment[1];
        let mut seg_max_size = last_segment[2];
        //println!("write: seg(id0: {}, id1: {}, max_size: {}, ino: {}, offset: {})", id0, id1, seg_max_size, ino, offset);
        loop {
            //println!("write: seg(id0: {}, id1: {}, max_size: {})", id0, id1, seg_max_size);
            let worker = self.disk_io_pool.get_thread(id0, id1);
            let seg_dir = self.segment_mgr.get_segment_dir(id0, id1);
            let (tx, rx) = bounded::<MsgFileWriteResp>(1);
            let msg = MsgFileWriteOp{
                id0: id0,
                id1: id1,
                max_size: seg_max_size,
                dir: seg_dir.clone(),
                offset: offset, // the file offset is not used currently.
                data: data.to_vec(),
                resp_sender: tx,
            };
            let ret = worker.do_io(MsgFileOp::OpWrite(msg));
            if !ret.is_success() {
                println!("write: failed to send_disk_io for ino: {}, seg(id0: {}, id1: {}), err: {:?}",
                ino, id0, id1, ret);
                return Err(Errno::Eintr);
            }
            let ret = rx.recv();
            match ret {
                Ok(r) => {
                    if !r.err.is_success() {
                        if r.err.is_enospc() {
                            println!("write: segment(id0: {}, id1: {}, dir: {}) has no space left for ino: {} with offset: {}",
                            id0, id1, seg_dir, ino, offset);
                            let seg = self.segment_mgr.new_segment(&String::from(""));
                            self.handle_mgr.add_segment(ino, &seg);
                            id0 = seg.seg_id0;
                            id1 = seg.seg_id1;
                            seg_max_size = seg.max_size;
                            println!("write: add new segment(id0: {}, id1: {}) for ino: {} with offset: {}",
                        id0, id1, ino, offset);
                            continue;
                        }
                        println!("write: failed to write segment(id0: {}, id1: {}) for ino: {} with offset: {}, err: {:?}",
                        id0, id1, ino, offset, r.err);
                        return Err(r.err);
                    }
                    // write block success.
                    let b = Block {
                        ino: ino,
                        generation: 0,
                        offset: offset,
                        seg_start_addr: r.offset,
                        seg_end_addr: r.offset + r.nwrite as u64,
                        size: r.nwrite as i64,
                    };
                    let ret = self.handle_mgr.add_block(ino, id0, id1, &b);
                    if !ret.is_success() {
                        println!("write: failed to add_block{:?} for ino: {} with offset: {}, err: {:?}", b, ino, offset, ret);
                        return Err(ret);
                    }
                    
                    return Ok(BlockIo{
                        id0: id0,
                        id1: id1,
                        offset: r.offset,
                        size: r.nwrite,
                    });
                    // currently, will update the segments in close api.
                    // upload the block to meta server.
                    /*let ret = self.segment_mgr.upload_block(ino, id0, id1, &b);
                    if ret.is_success(){
                        return Ok(BlockIo{
                            id0: id0,
                            id1: id1,
                            offset: r.offset,
                            size: r.nwrite,
                        });
                    }
                    println!("write: failed to upload block{:?} for ino: {}, err: {:?}", b, ino, ret);
                    return Err(ret);*/
                }
                Err(err) => {
                    println!("write: failed to get response for seg(id0: {}, id1: {}) of ino: {} with offset: {}, err: {}", 
                        id0, id1, ino, offset, err);
                    return Err(Errno::Eintr);
                }
            }
            
        }
    }

    fn close(&self, ino: u64) -> Errno {
        // first we should update the segments into meta server.
        // second we should close all the file handles for the ino.
        let handle: FileHandle;
        let ret = self.handle_mgr.get(ino);
        match ret {
            Ok(ret) => {
                handle = ret;
            }
            Err(err) => {
                println!("close: failed to get file handle for ino: {}, err: {:?}", ino, err);
                return err;
            }
        }
        // update the segments into meta server.
        if !handle.segments.is_empty() {
            let ret = self.segment_mgr.update_segments(ino, &handle.segments);
            if !ret.is_success(){
                println!("close: failed to update segments for ino: {}, err: {:?}", ino, ret);
                return ret;
            }
        }
        // close the segments file handles.
        for s in &handle.segments {
            for b in &s.blocks {
                println!("ino: {}, seg: {}, {}, block: offset: {}, size: {}",
                ino, s.seg_id0, s.seg_id1, b.offset, b.size);
            }
            //close the segment.
            let worker = self.disk_io_pool.get_thread(s.seg_id0, s.seg_id1);
            let msg = MsgFileCloseOp{
                id0: s.seg_id0,
                id1: s.seg_id1,
            };
            let ret = worker.do_io(MsgFileOp::OpClose(msg));
            if !ret.is_success(){
                println!("failed to close seg: id0: {}, id1: {} for ino: {}, err: {:?}", 
                    s.seg_id0, s.seg_id1, ino, ret);
                return ret;
            }
        }

        let err = self.handle_mgr.del(ino);
        return err;
    }

    fn release(&mut self) {
        self.handle_mgr.stop();
    }
}

impl LeaderLocal {
    pub fn new(machine: &String, thr_num: u32, exec: &Executor, mgr: Rc<SegmentMgr>, backend: Box<dyn BackendStore>) -> Self {
        LeaderLocal {
            machine: machine.clone(),
            disk_io_pool: IoThreadPool::new(thr_num, &String::from("Disk"), 
            exec, &DiskIoWorkerFactory::new()),
            backend_store: backend,
            exec: exec.clone(),
            segment_mgr: mgr,
            handle_mgr: FileHandleMgr::create(),
        }
    }
}