#![no_std]
#![no_main]
#![feature(panic_info_message)]

#[macro_use]
extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use mem::frames;
use polyhal::{
    common::{get_fdt, PageAlloc},
    consts::VIRT_ADDR_START,
    trap::TrapType::{self, *},
    trapframe::{TrapFrame, TrapFrameArgs},
    PhysPage,
};
use spin::Mutex;

mod config;
mod lang_items;
mod mem;
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

impl base::DAlloc for PageAllocator {
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

/// Kernel Entry Point
#[polyhal::arch_entry]
fn main(hart_id: usize) {
    log::info!("hart_id: {}", hart_id);

    polyhal::common::init(&PageAllocator);
    mem::init();

    // Probe the device from the device tree.
    if let Some(fdt) = get_fdt() {
        for node in fdt.all_nodes() {
            node.compatible().inspect(|x| {
                x.all().find(|n| *n == "virtio,mmio").inspect(|_| {
                    let addr = node.reg().unwrap().next().unwrap().starting_address;
                    let dri = virtio::probe::<Mutex<()>, PageAllocator>(
                        addr as usize | VIRT_ADDR_START,
                        Vec::new(),
                    );
                    log::debug!("driver: {:?}", dri);
                });
            });
        }
    }

    // Test map elf
    let task = Box::new(task::task::Task::from_elf(include_bytes!(
        "../../resources/testcase-riscv64/bin/busybox"
    )));
    task.into_user();
}
