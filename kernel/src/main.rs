#![no_std]
#![no_main]
#![feature(panic_info_message)]

extern crate alloc;

use core::ffi::CStr;

use alloc::{boxed::Box, vec::Vec};
use fs_base::{FSPage, FSTrait, FileTree, FileType, OpenFlags};
use mem::frames::{self, alloc_pages_raw, dealloc_pages_raw};
use polyhal::{
    common::{get_fdt, PageAlloc},
    consts::VIRT_ADDR_START,
    pagetable::PAGE_SIZE,
    trap::TrapType::{self, *},
    trapframe::{TrapFrame, TrapFrameArgs},
    utils::LazyInit,
    PhysPage,
};
use spin::{Mutex, RwLock};

mod config;
mod lang_items;
mod mem;
mod pci;
mod syscall;
mod task;
mod utils;

struct PageAllocator;

impl PageAlloc for PageAllocator {
    /// Allocate a Physical Page.
    fn alloc(&self) -> polyhal::PhysPage {
        unsafe { frames::alloc_page_raw() }
    }

    /// Deallocate a Physical Page.
    fn dealloc(&self, ppn: polyhal::PhysPage) {
        unsafe {
            frames::dealloc_pages_raw(ppn, 1);
        }
    }
}

impl drivers_base::DAlloc for PageAllocator {
    fn alloc(pages: usize) -> usize {
        unsafe { frames::alloc_pages_raw(pages).to_addr() }
    }

    fn dealloc(paddr: usize, pages: usize) -> i32 {
        unsafe {
            frames::dealloc_pages_raw(PhysPage::from_addr(paddr), pages);
            0
        }
    }

    fn phys_to_virt(paddr: usize) -> usize {
        paddr | VIRT_ADDR_START
    }

    fn virt_to_phys(vaddr: usize) -> usize {
        vaddr & !VIRT_ADDR_START
    }
}

/// Kernel Trap Handler
#[polyhal::arch_interrupt]
fn trap_handler(ctx: &mut TrapFrame, trap_type: TrapType) {
    // log::debug!("trap_type @ {:x?} {:#x?}", trap_type, ctx);
    match trap_type {
        Breakpoint => return,
        SysCall => {}
        StorePageFault(paddr) | LoadPageFault(paddr) | InstructionPageFault(paddr) => {
            log::info!("PageFault@{:#x}  {:#x}", ctx[TrapFrameArgs::SEPC], paddr);
        }
        IllegalInstruction(_) => {
            log::info!("illegal instruction");
        }
        Timer => {}
        _ => {
            log::warn!("unsuspended trap type: {:?}", trap_type);
        }
    }
}

pub struct FSTraitImpl;

impl FSTrait for FSTraitImpl {
    fn alloc_page(count: usize) -> FSPage<Self> {
        FSPage::new(unsafe { alloc_pages_raw(count).to_addr() }, count)
    }

    fn dealloc_page(addr: usize, count: usize) {
        unsafe { dealloc_pages_raw(PhysPage::from_addr(addr), count) }
    }

    fn phys_to_virt(phys: usize) -> usize {
        phys | VIRT_ADDR_START
    }

    fn virt_to_phys(virt: usize) -> usize {
        virt & !VIRT_ADDR_START
    }
}

static FILE_TREE: LazyInit<FileTree<Mutex<()>, RwLock<()>, FSTraitImpl>> = LazyInit::new();

/// Kernel Entry Point
#[polyhal::arch_entry]
fn main(hart_id: usize) {
    log::info!("hart_id: {}", hart_id);

    polyhal::common::init(&PageAllocator);
    mem::init();

    println!(r"     ____                     _   ____    _____ ");
    println!(r"    / __ \                   | | / __ \  / ____|");
    println!(r"   | |  | | _   _   __ _   __| || |  | || (___  ");
    println!(r"   | |  | || | | | / _` | / _` || |  | | \___ \ ");
    println!(r"   | |__| || |_| || (_| || (_| || |__| | ____) |");
    println!(r"    \___\_\ \__,_| \__,_| \__,_| \____/ |_____/ ");
    println!();

    // Probe the device from the device tree.
    if let Some(fdt) = get_fdt() {
        for node in fdt.all_nodes() {
            node.compatible().inspect(|x| {
                x.all().find(|n| *n == "virtio,mmio").inspect(|_| {
                    let addr = node.reg().unwrap().next().unwrap().starting_address;
                    let dri = drivers_virtio::probe::<Mutex<()>, PageAllocator>(
                        addr as usize | VIRT_ADDR_START,
                        Vec::new(),
                    );
                    dri.inspect(|dri| log::debug!("{:?}", dri));
                });
            });
        }
        fdt.chosen()
            .bootargs()
            .inspect(|x| log::info!("BootArgs: {}", x));

        let chosen = fdt.find_node("/chosen").unwrap();
        log::info!(
            "RamStart: {:#x?}",
            chosen.property("linux,initrd-start").unwrap()
        );
        let start = usize::from_be_bytes(
            chosen
                .property("linux,initrd-start")
                .unwrap()
                .value
                .try_into()
                .unwrap(),
        );
        log::info!("Initrd Start: {:#x}", start);
    }

    pci::init();

    /* Test File System begin */
    FILE_TREE.init_by(FileTree::new());

    FILE_TREE
        .mount("/", fs_ramfs::RamFs::<Mutex<()>, FSTraitImpl>::new())
        .unwrap();
    FILE_TREE.root().mkdir("hello").unwrap();
    FILE_TREE
        .mount("/", fs_ramfs::RamFs::<Mutex<()>, FSTraitImpl>::new())
        .expect("can't mount /");
    FILE_TREE.root().mkdir("test").unwrap();
    FILE_TREE
        .root()
        .open("/FileTree", OpenFlags::CREAT)
        .unwrap();
    FILE_TREE
        .mount("/test", fs_ramfs::RamFs::<Mutex<()>, FSTraitImpl>::new())
        .expect("can't mount /test");
    FILE_TREE
        .root()
        .open("/test/123", OpenFlags::CREAT | OpenFlags::RDWR)
        .unwrap()
        .writeat(0, b"Hello world!")
        .unwrap();
    let mut buffer = [0u8; PAGE_SIZE];
    assert_eq!(
        FILE_TREE
            .root()
            .open("/test/123", OpenFlags::CREAT | OpenFlags::RDWR)
            .unwrap()
            .readat(0, &mut buffer)
            .unwrap(),
        b"Hello world!".len()
    );
    println!(
        "read data from /test/123: {}",
        CStr::from_bytes_until_nul(&buffer)
            .unwrap()
            .to_str()
            .unwrap()
    );

    const TRUNCATE_LEN: usize = 5;
    buffer.fill(0);
    FILE_TREE
        .root()
        .open("/test/123", OpenFlags::RDWR)
        .unwrap()
        .truncate(TRUNCATE_LEN)
        .unwrap();
    println!("truncate file /test/123 with 5");

    assert_eq!(
        FILE_TREE
            .root()
            .open("/test/123", OpenFlags::RDWR)
            .unwrap()
            .readat(0, &mut buffer)
            .unwrap(),
        TRUNCATE_LEN
    );
    println!(
        "read data from /test/123 after truncate: {}",
        CStr::from_bytes_until_nul(&buffer)
            .unwrap()
            .to_str()
            .unwrap()
    );

    for i in FILE_TREE.root().read_dir().expect("can't read directory") {
        println!("file: {:#x?} type: {:#x?}", i.filename, i.file_type);
        if i.file_type == FileType::Directory {
            for i in FILE_TREE
                .root()
                .open(&i.filename, OpenFlags::DIRECTORY)
                .unwrap()
                .read_dir()
                .unwrap()
            {
                println!("\tfile: {:#x?} type: {:#x?}", i.filename, i.file_type);
            }
        }
    }
    /* Test File System end */

    // Test map elf
    #[cfg(target_arch = "riscv64")]
    let file_data = include_bytes_align_as!(u128, "../../resources/testcase-riscv64/bin/busybox");
    #[cfg(target_arch = "x86_64")]
    let file_data = include_bytes_align_as!(u128, "../../resources/testcase-x86_64/bin/busybox");
    #[cfg(target_arch = "aarch64")]
    let file_data = include_bytes_align_as!(u128, "../../resources/testcase-aarch64/bin/busybox");

    let task = Box::new(task::task::Task::from_elf(
        file_data,
        &["/busybox", "echo", "123"],
    ));
    task.into_user();
}
