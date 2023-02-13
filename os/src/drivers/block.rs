use crate::drivers::hal::HalImpl;
use alloc::sync::Arc;
use core::ptr::NonNull;
use easy_fs::BlockDevice;
use lazy_static::lazy_static;
use spin::Mutex;
use virtio_drivers::VirtIOBlk;
use virtio_drivers::VirtIOHeader;
use crate::println;

const VIRTIO0: usize = 0x10_008_000;


pub struct QemuBlockDevice {
    device: Mutex<VirtIOBlk<'static, HalImpl>>,
}
impl QemuBlockDevice {
    pub fn new(device: VirtIOBlk<'static,HalImpl>) -> Self {
        Self {
            device: Mutex::new(device),
        }
    }
}
unsafe impl Send for QemuBlockDevice {}
unsafe impl Sync for QemuBlockDevice {}

lazy_static! {
    pub static ref QEMU_BLOCK_DEVICE: Arc<dyn BlockDevice> = {
        let blk = unsafe{
            VirtIOBlk::<HalImpl>::new(&mut *(VIRTIO0 as *mut VirtIOHeader)).unwrap()
        };
        let qemu_block_device = QemuBlockDevice::new(blk);
        Arc::new(qemu_block_device)
    };
}

impl BlockDevice for QemuBlockDevice {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        self.device.lock().read_block(block_id,buf).unwrap()
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) {
        self.device.lock().write_block(block_id,buf).unwrap()
    }

    fn handle_irq(&self) {
        while let Ok(token) = self.device.lock().pop_used() {
            println!("token: {}", token);
        }
    }
}

