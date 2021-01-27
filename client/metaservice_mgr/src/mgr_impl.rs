#[path = "./message.rs"]
mod message;

use crate::{mgr, types::FileLeader};
use crate::types::DirEntry;
use crate::types::FileAttr;
use common::http_client;
use common::http_client::RespText;
use common::config;
use common::json;
use common::error::Errno;
use common::http_client::HttpMethod;
use message::{MsgFileAttr, ReqDirFileAttr, ReqFileAttr, ReqFileLeader, ReqMount, ReqReadDir, RespDirFileAttr, RespFileAttr, RespFileLeader, RespReadDir};
pub struct MetaServiceMgrImpl{
    http_client: Box<http_client::HttpClient>,
    meta_server_url: String,
    region: String,
    bucket: String,
    zone: String,
    machine: String,
}

impl mgr::MetaServiceMgr for MetaServiceMgrImpl{
    fn mount(&self) -> Result<(), Errno>{
        let req = ReqMount{
            region: self.region.clone(),
            bucket: self.bucket.clone(),
            zone: self.zone.clone(),
            machine: self.machine.clone(),
        };

        let req_json: String;
        let ret = common::json::encode_to_str::<ReqMount>(&req);
        match ret {
            Ok(ret) => {
                req_json = ret;
            }
            Err(error) => {
                println!("failed to encode {:?}, err: {}", req, error);
                return Err(Errno::Eintr);
            }
        }

        let url = format!("{}/v1/dir", self.meta_server_url);
        let resp : RespText;
        let ret = self.http_client.request(&url, &req_json, &HttpMethod::Put);
        match ret {
            Ok(ret) => {
                resp = ret;
            }
            Err(error) => {
                println!("failed to mount region: {}, bucket: {}, err: {}",
                self.region, self.bucket, error);
                return Err(Errno::Eintr);
            }
        }
        if resp.status >= 300 {
            println!("failed to mount region: {}, bucket: {}, got status: {}, body: {}",
            self.region, self.bucket, resp.status, resp.body);
            return Err(Errno::Eintr);
        }
        Ok(())
    }
    fn read_dir(&self, ino: u64, offset: i64)->Result<Vec<DirEntry>, Errno>{
        let mut entrys = Vec::new();
        let ret = self.read_dir_files(ino, offset);
        match ret {
            Ok(dirs) => {
                if dirs.result.err_code != 0 {
                    if dirs.result.err_code == 40003 {
                        println!("no files found in bucket {} with ino: {}, offset: {}", self.bucket, ino, offset);
                        return Err(Errno::Enoent);
                    }
                    println!("got error when read_dir_files for ino: {}, offset: {}, err: {}",
                    ino, offset, dirs.result.err_msg);
                    return Err(Errno::Eintr);
                }
                for i in dirs.files {
                    let entry = DirEntry{
                        ino: i.ino,
                        file_type: i.dir_entry_type.into(),
                        name: i.name,
                    };
                    entrys.push(entry);
                }
                return Ok(entrys);
            }
            Err(error) => {
                println!("failed to read meta for ino: {}, offset: {}, err: {}",
                ino, offset, error);
                return Err(Errno::Eintr);
            }
        }
    }

    fn read_file_attr(&self, ino: u64) -> Result<FileAttr, Errno>{
        let attr : MsgFileAttr;
        let ret = self.read_file_attr(ino);
        match ret {
            Ok(ret) => {
                attr = ret;
            }
            Err(error) => {
                println!("failed to read_file_attr for ino: {}, err: {}", ino, error);
                return Err(Errno::Eintr);
            }
        }

        let file_attr = self.to_file_attr(&attr);
        Ok(file_attr)
    }

    fn read_dir_file_attr(&self, ino: u64, name: &String) -> Result<FileAttr, Errno>{
        let ret = self.read_dir_file_attr(ino, name);
        match ret {
            Ok(ret) => {
                let file_attr = self.to_file_attr(&ret);
                return Ok(file_attr);
            }
            Err(error) => {
                println!("failed to read_dir_file_attr for ino: {}, name: {}, err: {}", ino, name, error);
                return Err(Errno::Eintr);
            }
        }
    }

    fn get_file_leader(&self, ino: u64, flag: u8) -> Result<FileLeader, Errno>{
        let req_file_leader = ReqFileLeader{
            region: self.region.clone(),
            bucket: self.region.clone(),
            zone: self.zone.clone(),
            machine: self.machine.clone(),
            ino: ino,
            flag: flag,
        };
        let body: String;
        let ret = json::encode_to_str::<ReqFileLeader>(&req_file_leader);
        match ret {
            Ok(ret) => {
                body = ret;
            }
            Err(error) => {
                println!("failed to encode req_file_leader: {:?}, err: {}", req_file_leader, error);
                return Err(Errno::Eintr);
            }
        }
        let url = format!("{}/v1/file/leader", self.meta_server_url);
        let resp : RespText;
        let ret = self.http_client.request(&url, &body,&HttpMethod::Get);
        match ret {
            Ok(ret) => {
                resp = ret;
            }
            Err(error) => {
                println!("failed to get file_leader, req: {}, err: {}", body, error);
                return Err(Errno::Eintr);
            }
        }
        if resp.status >= 300 {
            println!("got status {} for file_leader, req: {}, resp: {}", resp.status, body, resp.body);
            return Err(Errno::Eintr);
        }
        let resp_leader : RespFileLeader;
        let ret = json::decode_from_str::<RespFileLeader>(&resp.body);
        match ret {
            Ok(ret) => {
                resp_leader = ret;
            }
            Err(error) => {
                println!("failed to decode file_leader from {}, err: {}", resp.body, error);
                return Err(Errno::Eintr);
            }
        }
        if resp_leader.result.err_code != 0 {
            println!("failed to get file_leader for {}, err_code: {}, err_msg: {}", 
            body, resp_leader.result.err_code, resp_leader.result.err_msg);
            return Err(Errno::Eintr);
        }
        Ok(FileLeader{
            zone: resp_leader.leader_info.zone,
            leader: resp_leader.leader_info.leader,
            ino: ino,
        })
    }
}

impl MetaServiceMgrImpl {
    pub fn new(meta_cfg: &config::Config) -> Result<MetaServiceMgrImpl, String> {
        let http_client = Box::new(http_client::HttpClient::new(3));
        Ok(MetaServiceMgrImpl{
            http_client: http_client,
            meta_server_url: meta_cfg.metaserver_config.meta_server.clone(),
            region: meta_cfg.s3_config.region.clone(),
            bucket: meta_cfg.s3_config.bucket.clone(),
            zone: meta_cfg.zone_config.zone.clone(),
            machine: meta_cfg.zone_config.machine.clone(),
        })
    }

    fn to_file_attr(&self, msg_attr: &MsgFileAttr) -> FileAttr {
        FileAttr {
            ino: msg_attr.ino,
            generation: msg_attr.generation,
            size: msg_attr.size,
            blocks: msg_attr.blocks,
            atime: msg_attr.atime,
            mtime: msg_attr.mtime,
            ctime: msg_attr.ctime,
            kind: msg_attr.kind.into(),
            perm: msg_attr.perm,
            nlink: msg_attr.nlink,
            uid: msg_attr.uid,
            gid: msg_attr.gid,
            rdev: msg_attr.rdev,
            flags: msg_attr.flags,
        }
    }

    fn read_file_attr(&self, ino: u64) -> Result<MsgFileAttr, String> {
        let req_file_attr = ReqFileAttr{
            region: self.region.clone(),
            bucket: self.bucket.clone(),
            ino: ino,
        };
        let ret = json::encode_to_str::<ReqFileAttr>(&req_file_attr);
        let req_body : String;
        match ret {
            Ok(body) => {
                req_body = body;
            }
            Err(error) => {
                return Err(format!("failed to encode req_file_attr: {:?}, err: {}", req_file_attr, error));
            }
        }
        let resp : RespText;
        let url = format!("{}/v1/file/attr", self.meta_server_url);
        let ret = self.http_client.request(&url, &req_body, &HttpMethod::Get);
        match ret {
            Ok(ret) => {
                resp = ret;
            }
            Err(error) => {
                return Err(error);
            }
        }
        if resp.status >= 300 {
            return Err(format!("failed to read_file_attr from {}, for ino: {}, err: {}",
        url, ino, resp.body));
        }
        let resp_attr: RespFileAttr;
        let ret = json::decode_from_str::<RespFileAttr>(&resp.body);
        match ret {
            Ok(ret) => {
                resp_attr = ret;
            }
            Err(error) => {
                return Err(error);
            }
        }
        if resp_attr.result.err_code != 0 {
            return Err(format!("failed to read_file_attr for ino: {}, err_code: {}, err_msg: {}",
        ino, resp_attr.result.err_code, resp_attr.result.err_msg));
        }

        return Ok(resp_attr.attr);
    }

    fn read_dir_file_attr(&self, ino: u64, name: &String) -> Result<MsgFileAttr, String>{
        let req_dir_file_attr = ReqDirFileAttr{
            region: self.region.clone(),
            bucket: self.bucket.clone(),
            ino: ino,
            name: String::from(name),
        };
        let ret = json::encode_to_str::<ReqDirFileAttr>(&req_dir_file_attr);
        let req_child_file_attr_json: String;
        match ret {
            Ok(body) => {
                req_child_file_attr_json = body;
            }
            Err(error) => {
                return Err(error);
            }
        }
        let resp_text : RespText;
        let url = format!("{}/v1/dir/file/attr", self.meta_server_url);
        let ret = self.http_client.request(&url, &req_child_file_attr_json, &HttpMethod::Get);
        match ret {
            Ok(resp) => {
                resp_text = resp;
            }
            Err(error) => {
                return Err(error);
            }
        }
        if resp_text.status >= 300 {
            return Err(format!("failed to get child file attr from url {}, err: {}", url, resp_text.body));
        }
        let resp_attr : RespDirFileAttr;
        let ret = json::decode_from_str::<RespDirFileAttr>(&resp_text.body);
        match ret {
            Ok(attr) => {
                resp_attr = attr;
            }
            Err(error) => {
                return Err(error);
            }
        }
        if resp_attr.result.err_code != 0 {
            return Err(format!("failed to get child file attrs for ino: {}, name: {}, err: {}", 
            ino, name, resp_attr.result.err_msg));
        }
        return Ok(resp_attr.attr);
    }

    fn read_dir_files(&self, ino: u64, offset: i64) -> Result<Box<RespReadDir>, String>{
        let req_read_dir = ReqReadDir{
            region: self.region.clone(),
            bucket:self.bucket.clone(),
            ino: ino,
            offset: offset,
        };
        let ret = serde_json::to_string(&req_read_dir);
        let req_read_dir_json: String;
        match ret {
            Ok(ret) => {
                //send the req to meta server
                req_read_dir_json = ret;
            }
            Err(error) => {
                return Err(format!("faied to convert {:?} to json, err: {}", req_read_dir, error));
            }
        }

        let resp_body :String;
        let url = format!("{}/v1/dir/files", self.meta_server_url);
        let ret = self.http_client.request(&url, &req_read_dir_json, &HttpMethod::Get);
        match ret {
            Ok(text) => {
                if text.status >= 300 {
                    return Err(format!("got resp {}", text.status));
                }
                resp_body = text.body;
            }
            Err(error) => {
                return Err(format!("failed to get response for {}, err: {}", url, error));
            }
        }
        
        let resp_read_dir = json::decode_from_str::<RespReadDir>(&resp_body);
        match resp_read_dir {
            Ok(resp_read_dir) => {
                return Ok(Box::new(resp_read_dir));
            }
            Err(error) => {
                return Err(format!("failed to decode from {}, err: {}", resp_body, error));
            }
        }
    }
}