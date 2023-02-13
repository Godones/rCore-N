mod mail;
mod pipe;
mod serial;
pub mod stdio;
mod inode;

use crate::mm::UserBuffer;
use alloc::boxed::Box;
use core::{future::Future, pin::Pin};

pub use mail::{MailBox, Socket};

pub trait File: Send + Sync {
    fn read(&self, buf: UserBuffer) -> Result<usize, isize>;
    fn write(&self, buf: UserBuffer) -> Result<usize, isize>;
    fn aread(&self, buf: UserBuffer, tid: usize, pid: usize, key: usize) -> Pin<Box<dyn Future<Output = ()> + 'static + Send + Sync>>;
}
pub trait FileExt:Send+Sync+File {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
}


pub use pipe::{make_pipe, Pipe};
pub use serial::Serial;
pub use stdio::{Stdin, Stdout};
pub use inode::{list_apps,open_file,OpenFlags};