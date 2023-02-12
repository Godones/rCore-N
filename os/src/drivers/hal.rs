use core::intrinsics::forget;
use core::ptr::NonNull;
use virtio_drivers::{Hal, PhysAddr, PAGE_SIZE, BufferDirection};
use crate::mm::{frame_alloc_more, frame_dealloc, PhysPageNum};

pub struct HalImpl;

unsafe impl Hal for HalImpl {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let trakcers = frame_alloc_more(pages);
        let ppn_base = trakcers.as_ref().unwrap().last().unwrap().ppn;
        // QUEUE_FRAMES.exclusive_access().append(&mut trakcers.unwrap());
        forget(trakcers.unwrap());
        // info!("dma_alloc: {:#x}-{:#x}",ppn_base.0 << 12 ,(ppn_base.0<<12) + pages * 4096);
        let pa =  ppn_base.0 << 12;
        (pa, NonNull::new(pa as *mut u8).unwrap())
    }

    unsafe fn dma_dealloc(paddr: PhysAddr, vaddr: NonNull<u8>, pages: usize) -> i32 {
        let pa = PhysAddr::from(paddr);
        let mut ppn_base: PhysPageNum = pa.into();
        for _ in 0..pages {
            frame_dealloc(ppn_base);;
            ppn_base  = PhysPageNum::from(ppn_base.0 + 1);
        }
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, size: usize) -> NonNull<u8> {
       NonNull::new(paddr as *mut u8).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, direction: BufferDirection) -> PhysAddr {
        let vaddr = buffer.as_ptr() as *mut u8 as usize;
        // Nothing to do, as the host already has access to all memory.
        vaddr
    }

    unsafe fn unshare(paddr: PhysAddr, buffer: NonNull<[u8]>, direction: BufferDirection) {

    }
}
