use crate::drivers::QEMU_BLOCK_DEVICE;
use alloc::sync::Arc;
use fat32::{Dir, Fat32};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref ROOT_DIR: Arc<Dir> = {
        let device = QEMU_BLOCK_DEVICE.lock();
        let fs = Fat32::new(device.clone()).unwrap();
        fs.root_dir()
    };
}
