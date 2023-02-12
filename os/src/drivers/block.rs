use crate::drivers::hal::HalImpl;
use alloc::sync::Arc;
use core::ptr::NonNull;
use fat32_trait::BlockDevice;
use lazy_static::lazy_static;
use spin::Mutex;
use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::transport::Transport;

const VIRTIO0: usize = 0x10_008_000;


pub trait BlockDeviceExt{
    fn handle_irq(&self);
}


pub struct QemuBlockDevice<T: Transport> {
    device: Mutex<VirtIOBlk<HalImpl, T>>,
}
impl<T: Transport> QemuBlockDevice<T> {
    pub fn new(device: VirtIOBlk<HalImpl, T>) -> Self {
        Self {
            device: Mutex::new(device),
        }
    }
}
unsafe impl<T: Transport> Send for QemuBlockDevice<T> {}
unsafe impl<T: Transport> Sync for QemuBlockDevice<T> {}

lazy_static! {
    pub static ref QEMU_BLOCK_DEVICE: Mutex<Arc<dyn NewBlockDevice>> = {
        let header = NonNull::new(VIRTIO0 as *mut VirtIOHeader).unwrap();
        let transport = match unsafe { MmioTransport::new(header) } {
        Err(e) => panic!("Error creating VirtIO MMIO transport: {}", e),
        Ok(transport) => {
            info!(
                    "Detected virtio MMIO device with vendor id {:#X}, device type {:?}, version {:?}",
                    transport.vendor_id(),
                    transport.device_type(),
                    transport.version(),
                );
            transport
        }
        };
        let blk = VirtIOBlk::<HalImpl,_>::new(transport).expect("failed to create blk driver");
        let qemu_block_device = QemuBlockDevice::new(blk);
        Mutex::new(Arc::new(qemu_block_device))
    };
}

impl<T: Transport> BlockDevice for QemuBlockDevice<T> {
    fn read(&self, block: usize, buf: &mut [u8]) -> Result<usize, ()> {
        self.device.lock().read_block(block, buf).unwrap();
        Ok(buf.len())
    }
    fn write(&self, block: usize, buf: &[u8]) -> Result<usize, ()> {
        self.device.lock().write_block(block, buf).unwrap();
        Ok(buf.len())
    }
    fn flush(&self) -> Result<(), ()> {
        Ok(())
    }
}

impl <T:Transport> BlockDeviceExt for QemuBlockDevice<T>{
    fn handle_irq(&self) {
        info!("solve block device irq");
        while self.device.lock().ack_interrupt() {

        }
    }
}

pub trait NewBlockDevice:BlockDevice+BlockDeviceExt{

}

impl <T:Transport> NewBlockDevice for QemuBlockDevice<T>{
}