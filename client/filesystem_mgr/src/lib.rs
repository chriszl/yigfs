pub mod yigfs;
mod handle;

use std::rc::Rc;
use yigfs::Yigfs;
use metaservice_mgr::mgr::MetaServiceMgr;
use segment_mgr::leader_mgr::LeaderMgr;

pub struct MountOptions{
    // mount point
    pub mnt: String,
}

pub struct FilesystemMgr {
   meta_service_mgr: Rc<dyn MetaServiceMgr>,
   leader_mgr: Option<LeaderMgr>,
}

impl FilesystemMgr{
    pub fn create(meta_service_mgr: Rc<dyn MetaServiceMgr>, leader_mgr: LeaderMgr)->FilesystemMgr{
        FilesystemMgr{
            meta_service_mgr: meta_service_mgr,
            leader_mgr: Some(leader_mgr),
        }
    }

    pub fn mount(&mut self, mount_options : MountOptions) {
        if let Some(leader_mgr) = self.leader_mgr.take() {
            let yfs = Yigfs::create(self.meta_service_mgr.clone(), leader_mgr);
            fuse::mount(yfs, &mount_options.mnt, &[]).unwrap();
        }
    }
}