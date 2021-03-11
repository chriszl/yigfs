use common::uuid;
use metaservice_mgr::types::Block as MetaBlock;
use metaservice_mgr::types::Segment as MetaSegment;

#[derive(Debug, Default)]
pub struct Segment {
    // seg_id will be generated from UUID. And UUID is u128, so we need two i64s.
    pub seg_id0: u64,
    pub seg_id1: u64,
    pub leader: String,
    pub blocks: Vec<Block>,
}

impl Segment {
    pub fn new(leader: &String) -> Self {
        let ids = uuid::uuid_u64_le();
        Segment{
            seg_id0: ids[0],
            seg_id1: ids[1],
            leader: leader.clone(),
            blocks: Vec::<Block>::new(),
        }
    }

    pub fn copy(&self) -> Self{
        let mut s = Segment{
            seg_id0: self.seg_id0,
            seg_id1: self.seg_id1,
            leader: self.leader.clone(),
            blocks: Vec::<Block>::new(),
        };
        for b in &self.blocks{
            s.blocks.push(b.copy());
        }
        return s;
    }

    pub fn add_block(&mut self, ino: u64, offset: u64, seg_start_offset: u64, nwrite: u32) {
        let b = Block{
            ino: ino,
            generation: 0,
            offset: offset,
            seg_start_addr: seg_start_offset,
            seg_end_addr: seg_start_offset+nwrite as u64,
            size: nwrite as i64,
        };
        self.blocks.push(b);
    }

    pub fn usage(&self) -> u64 {
        let mut total : u64 = 0;
        for b in &self.blocks {
            total += b.size as u64;
        }
        total
    }

    pub fn contains(&self, offset: u64) -> bool {
        if self.blocks.is_empty() {
            return false;
        }
        let len = self.blocks.len();
        if self.blocks[0].offset > offset {
            return false;
        }
        if (self.blocks[len -1 ].offset + self.blocks[len-1].size as u64) < offset {
            return false;
        }

        return true;
    }

    pub fn is_empty(&self)->bool {
        if self.blocks.is_empty() {
            return true;
        }
        return false;
    }

    pub fn to_meta_segment(&self) -> MetaSegment {
        let mut meta_seg = MetaSegment {
            seg_id0: self.seg_id0,
            seg_id1: self.seg_id1,
            leader: self.leader.clone(),
            blocks: Vec::new(),
        };
        for b in &self.blocks {
            meta_seg.blocks.push(b.to_meta_block());
        }
        return meta_seg;
    }
}

#[derive(Debug, Default)]
pub struct Block {
    pub ino: u64,
    pub generation: u64,
    // the offset in the file specified by ino & generation
    pub offset: u64,
    // the offset in this segment
    pub seg_start_addr: u64,
    // the end in this segment
    pub seg_end_addr: u64,
    // the size of this block
    pub size: i64,
}

impl Block {
    pub fn copy(&self) -> Self{
        Block{
            ino: self.ino,
            generation: self.generation,
            offset: self.offset,
            seg_start_addr: self.seg_start_addr,
            seg_end_addr: self.seg_end_addr,
            size: self.size,
        }
    }

    pub fn to_meta_block(&self) ->  MetaBlock{
        MetaBlock{
            offset: self.offset,
            seg_start_addr: self.seg_start_addr,
            seg_end_addr: self.seg_end_addr,
            size: self.size,
        }
    }
}

// below structs are for Leader usage.
#[derive(Debug, Default)]
pub struct SegmentIo {
    pub id0: u64,
    pub id1: u64,
    // the folder which segment resides
    pub dir: String,
}

#[derive(Debug, Default)]
pub struct BlockIo {
    pub id0: u64,
    pub id1: u64,
    // note: this offset is the start addr in the segment file.
    pub offset: u64,
    // the size of this block.
    pub size: u32,
}