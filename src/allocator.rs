use bootloader::BootInfo;
use x86_64::{
    structures::paging::{
        mapper::MapToError,
        FrameAllocator,
        Mapper,
        Page,
        PageTableFlags,
        Size4KiB,
    },
    VirtAddr,
};

pub mod bump;
pub mod fixed_size_block;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked { inner: spin::Mutex::new(inner), }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> { self.inner.lock() }
}

// use linked_list_allocator::LockedHeap;
// #[global_allocator]
// static ALLOCATOR: LockedHeap = LockedHeap::empty();

use fixed_size_block::FixedSizeBlockAllocator;

#[global_allocator]
static ALLOCATOR: Locked<FixedSizeBlockAllocator> =
    Locked::new(FixedSizeBlockAllocator::new());

pub fn init_heap(mapper: &mut impl Mapper<Size4KiB>,
                 frame_allocator: &mut impl FrameAllocator<Size4KiB>)
                 -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page: Page = Page::containing_address(heap_start);
        let heap_end_page: Page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator.allocate_frame()
                                   .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

pub fn init_kernel_heap(boot_info: &'static BootInfo) {
    use crate::{memory, memory::BootInfoFrameAllocator};
    let phys_mem_offset = match boot_info.physical_memory_offset {
        bootloader::boot_info::Optional::Some(offset) => VirtAddr::new(offset),
        bootloader::boot_info::Optional::None => panic!("No offset given"),
    };

    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };

    // new
    init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

#[test_case]
fn test_align_up() {
    assert_eq!(align_up(1024, 512), 1024);
    assert_eq!(align_up(123, 512), 512);
}
