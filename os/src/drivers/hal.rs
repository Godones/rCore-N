use core::intrinsics::forget;
use core::ptr::NonNull;
use virtio_drivers::{Hal, PhysAddr, VirtAddr};
use crate::mm::{frame_alloc_more, frame_dealloc, PhysPageNum};

pub struct HalImpl;

impl Hal for HalImpl {
    fn dma_alloc(pages: usize) -> PhysAddr {
        let trakcers = frame_alloc_more(pages);
        let ppn_base = trakcers.as_ref().unwrap().last().unwrap().ppn;
        // QUEUE_FRAMES.exclusive_access().append(&mut trakcers.unwrap());
        forget(trakcers.unwrap());
        // info!("dma_alloc: {:#x}-{:#x}",ppn_base.0 << 12 ,(ppn_base.0<<12) + pages * 4096);
        let pa =  ppn_base.0 << 12;
        pa
    }

    fn dma_dealloc(paddr: PhysAddr, pages: usize) -> i32 {
        let pa = PhysAddr::from(paddr);
        let mut ppn_base: PhysPageNum = pa.into();
        for _ in 0..pages {
            frame_dealloc(ppn_base);;
            ppn_base  = PhysPageNum::from(ppn_base.0 + 1);
        }
        0
    }


    fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
        paddr
    }

    fn virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
        vaddr
    }
}
