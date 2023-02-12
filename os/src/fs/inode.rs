use alloc::boxed::Box;
use super::File;
use crate::drivers::QEMU_BLOCK_DEVICE;
use crate::mm::UserBuffer;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::error::Error;
use core::future::Future;
use core::ops::Deref;
use core::pin::Pin;
use bitflags::*;
use cfg_if::cfg_if;
use fat32_trait::{DirectoryLike, FileLike};
use lazy_static::*;
use spin::Mutex;
use crate::fs::dbfs::{FakeMMap};
use crate::fs::FileExt;
use crate::println;
use super::ROOT_DIR;

// cfg_if!(
//     if #[cfg(feature = "db_engine")] {
//         type DiskFile = dbfs::File<FakeMMap>;
//         type Dir = dbfs::Dir<FakeMMap>;
//     } else {
//         type DiskFile = fat32::File;
//         type Dir = fat32::Dir;
//     }
// );
type Dir = dyn DirectoryLike<Error = fat32::OperationError, FError:Error+'static>;
type DiskFile = dyn FileLike<Error= fat32::OperationError>;

pub enum Inode{
    Dir(Arc<Dir>),
    File(Arc<DiskFile>),
}


pub struct OSInode {
    readable: bool,
    writable: bool,
    inner: Mutex<OSInodeInner>,
}

pub struct OSInodeInner {
    offset: usize,
    inode: Arc<Inode>,
}

impl OSInode {
    pub fn new(readable: bool, writable: bool, inode: Arc<Inode>) -> Self {
        Self {
            readable,
            writable,
            inner: unsafe { Mutex::new(OSInodeInner { offset: 0, inode }) },
        }
    }
    pub fn read_all(&self) -> Vec<u8> {
        info!("read_all");
        let mut inner = self.inner.lock();
        let  inode = inner.inode.as_ref();
        let data = match inode {
            Inode::Dir(dir)=>{
                panic!("not a file");
            }
            Inode::File(file) => {
                let size = file.size();
                info!("read size: {}kb  offset: {}",size/1024,inner.offset);
                file.read(inner.offset as u32, size).unwrap()
            }
        };
        data
    }
}

lazy_static! {
    pub static ref ROOT_INODE: Arc<Inode> = {
        let dir = ROOT_DIR.clone();
        let inode = Inode::Dir(dir);
        Arc::new(inode)
    };
}

pub fn list_apps() {
    println!("/**** APPS ****");
    let inode = ROOT_INODE.as_ref();
    match inode {
        Inode::Dir(dir) =>{
            dir.list().unwrap().iter().for_each(|x|{
                println!("{}",x);
            })
        }
        _ => panic!("It is not a dir")
    }
    println!("**************/");
}
pub fn read_all_file(){
    println!("try read all file");
    let inode = ROOT_INODE.as_ref();
    let all_name = match inode {
        Inode::Dir(dir) =>{
            dir.list().unwrap()
        }
        _ => panic!("It is not a dir")
    };
    for name in all_name{
        let file = open_file(name.as_str(),OpenFlags::RDONLY);
        let data = file.unwrap().read_all();
        println!("file {} len is {}",name,data.len());
    }
}
bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
        const TRUNC = 1 << 10;
    }
}

impl OpenFlags {
    /// Do not check validity for simplicity
    /// Return (readable, writable)
    pub fn read_write(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else if self.contains(Self::WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
    }
}

pub fn open_file(name: &str, flags: OpenFlags) -> Option<Arc<OSInode>> {
    let (readable, writable) = flags.read_write();
    let inode = ROOT_INODE.as_ref();
    let inode = match inode {
        Inode::File(file) => {
            panic!("not a dir");
        }
        Inode::Dir(dir) => {
            info!("flag: {:?}",flags);
            if flags.contains(OpenFlags::CREATE){
                if let Some(x) = dir.list().unwrap().iter().find(|f| f.as_str() == name){
                    let file = dir.open(x).unwrap();
                    file.clear();
                    let inode = Arc::new(Inode::File(file));
                    Some(Arc::new(OSInode::new(readable,writable,inode)))
                }else {
                    dir.create_file(name).unwrap();
                    let file = dir.open(name).unwrap();
                    let inode = Arc::new(Inode::File(file));
                    Some(Arc::new(OSInode::new(readable,writable,inode)))
                }
            }else {
                if let Some(x) = dir.list().unwrap().iter().find(|f| f.as_str() == name){
                    info!("try read {}",x);
                    let file = dir.open(x).unwrap();
                    if flags.contains(OpenFlags::TRUNC){
                        file.clear();
                    }
                    let inode = Arc::new(Inode::File(file));
                    Some(Arc::new(OSInode::new(readable,writable,inode)))
                } else { None }
            }
        }
    };
    inode
}

impl FileExt for OSInode{
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }
}
impl File for OSInode {
    fn read(&self, mut buf: UserBuffer) -> Result<usize,isize> {
        let mut inner = self.inner.lock();
        let  inode = inner.inode.as_ref();
        let data = match inode {
            Inode::Dir(dir)=>{
                panic!("not a file");
            }
            Inode::File(file) => {
                let size = file.size();
                file.read(inner.offset as u32, size).unwrap()
            }
        };
        let mut  total_read = 0;
        let mut read_size = 0;
        for slice in buf.buffers.iter_mut() {
            let slice_size = slice.len();
            if read_size + slice_size > data.len() {
                slice[..data.len() - read_size].copy_from_slice(&data[read_size..]);
                total_read += data.len() - read_size;
                read_size = data.len();
                break;
            } else {
                slice.copy_from_slice(&data[read_size..read_size + slice_size]);
                total_read += slice_size;
                read_size += slice_size;
            }
        }
        inner.offset += total_read;
        Ok(total_read)
    }
    fn write(&self, buf: UserBuffer) -> Result<usize,isize>  {
        let mut inner = self.inner.lock();
        let mut inode = inner.inode.as_ref();
        let mut total_write_size = 0usize;
        let offset = inner.offset as u32;
        match inode {
            Inode::Dir(dir)=>{
                panic!("not a file");
            }
            Inode::File(file) => {
                let file = file.clone();
                for slice in buf.buffers.iter() {
                    let write_size = file.write(offset, *slice).unwrap();
                    assert_eq!(write_size as usize, slice.len());
                    inner.offset += write_size as usize;
                    total_write_size += write_size as usize;
                }
            }
        };
        Ok(total_write_size)
    }

    fn aread(&self, buf: UserBuffer, tid: usize, pid: usize, key: usize) -> Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>> {
        todo!("aread")
    }
}
