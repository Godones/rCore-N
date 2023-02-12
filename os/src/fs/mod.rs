mod mail;
mod pipe;
mod serial;
pub mod stdio;
mod dbfs;
mod fat32;
mod inode;

use crate::mm::UserBuffer;
use alloc::boxed::Box;
use core::{future::Future, pin::Pin};

pub use mail::{MailBox, Socket};
pub use inode::{open_file,OpenFlags,list_apps,read_all_file};

pub trait File: Send + Sync {
    fn read(&self, buf: UserBuffer) -> Result<usize, isize>;
    fn write(&self, buf: UserBuffer) -> Result<usize, isize>;
    fn aread(&self, buf: UserBuffer, tid: usize, pid: usize, key: usize) -> Pin<Box<dyn Future<Output = ()> + 'static + Send + Sync>>;
}
pub trait FileExt:Send+Sync+File {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
}

#[cfg(feature = "db_engine")]
use crate::fs::dbfs::ROOT_DIR;

#[cfg(not(feature = "db_engine"))]
use crate::fs::fat32::ROOT_DIR;


pub use pipe::{make_pipe, Pipe};
pub use serial::Serial;
pub use stdio::{Stdin, Stdout};
