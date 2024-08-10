use core::{cell::UnsafeCell, cmp::min};

use alloc::{collections::btree_map::BTreeMap, vec::Vec};
use polyhal::{
    debug_console::DebugConsole,
    pagetable::PAGE_SIZE,
    trap::{run_user_task, EscapeReason},
    trapframe::{TrapFrame, TrapFrameArgs},
    PageTableWrapper, VirtPage,
};
use syscalls::{Errno, Sysno};
use xmas_elf::{program::Type, ElfFile};

use crate::config::{ALIGN_SIZE, DEFAULT_USER_STACK_SIZE, DEFAULT_USER_STACK_TOP};

use super::memset::MemSet;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[allow(non_camel_case_types, dead_code)]
pub enum AuxV {
    /// end of vector
    NULL = 0,
    /// entry should be ignored
    IGNORE = 1,
    /// file descriptor of program
    EXECFD = 2,
    /// program headers for program
    PHDR = 3,
    /// size of program header entry
    PHENT = 4,
    /// number of program headers
    PHNUM = 5,
    /// system page size
    PAGESZ = 6,
    /// base address of interpreter
    BASE = 7,
    /// flags
    FLAGS = 8,
    /// entry point of program
    ENTRY = 9,
    /// program is not ELF
    NOTELF = 10,
    /// real uid
    UID = 11,
    /// effective uid
    EUID = 12,
    /// real gid
    GID = 13,
    /// effective gid
    EGID = 14,
    /// string identifying CPU for optimizations
    PLATFORM = 15,
    /// arch dependent hints at CPU capabilities
    HWCAP = 16,
    /// frequency at which times() increments
    CLKTCK = 17,
    // values 18 through 22 are reserved
    DCACHEBSIZE = 19,
    /// secure mode boolean
    SECURE = 23,
    /// string identifying real platform, may differ from AT_PLATFORM
    BASE_PLATFORM = 24,
    /// address of 16 random bytes
    RANDOM = 25,
    /// extension of AT_HWCAP
    HWCAP2 = 26,
    /// filename of program
    EXECFN = 31,
}

/// Monolithic Task
pub struct Task {
    /// This field records the current state of the user  task.
    pub trap_frame: UnsafeCell<TrapFrame>,
    /// This field records the page table of the user task.
    /// Release the page table includes leaf page table when exiting.
    pub page_table: PageTableWrapper,
    /// Records the used Physical pages.
    pub memset: MemSet,
}

impl Task {
    /// Create a new Monolithic Task from the given elf file.
    pub fn from_elf(elf_data: &[u8], args: &[&str]) -> Self {
        let page_table = PageTableWrapper::alloc();
        let mut task = Task {
            page_table,
            trap_frame: UnsafeCell::new(TrapFrame::new()),
            memset: MemSet::new(),
        };

        let file = ElfFile::new(elf_data).expect("This is not a valid elf file");

        // Load data from elf file.
        file.program_iter()
            .filter(|ph| ph.get_type() == Ok(Type::Load))
            .for_each(|ph| {
                let mut offset = ph.offset() as usize;
                let mut vaddr = ph.virtual_addr() as usize;
                let end = offset + ph.file_size() as usize;
                let vaddr_end = vaddr + ph.mem_size() as usize;

                loop {
                    if vaddr >= vaddr_end {
                        break;
                    }

                    let ppn = task
                        .memset
                        .map_page(task.page_table.0, VirtPage::from_addr(vaddr));

                    // If need to read data from elf file.
                    if offset < end {
                        let rsize = min(PAGE_SIZE - vaddr % PAGE_SIZE, end - offset);
                        // Copy data from elf file's data to the correct position.
                        ppn.get_buffer()[offset % PAGE_SIZE..(offset % PAGE_SIZE) + rsize]
                            .copy_from_slice(&elf_data[offset..offset + rsize]);

                        offset += rsize;
                    }

                    // Calculate offset
                    vaddr += PAGE_SIZE - vaddr % PAGE_SIZE;
                }
            });

        // Map user stack.
        for i in 0..DEFAULT_USER_STACK_SIZE / 0x1000 {
            task.memset.map_page(
                task.page_table.0,
                VirtPage::from_addr(DEFAULT_USER_STACK_TOP - i - 1),
            );
        }

        // FIXME: This is just for debugging, remove it when debug is finished.
        for i in 0..10 {
            task.memset.map_page(
                task.page_table.0,
                VirtPage::from_addr(0x2_0000_0000 + i * PAGE_SIZE),
            );
        }

        let mut stack_ptr = DEFAULT_USER_STACK_TOP;

        let args_ptr: Vec<_> = args
            .iter()
            .map(|arg| {
                // TODO: set end bit was zeroed manually.
                stack_ptr = (stack_ptr - arg.bytes().len() - 1) / ALIGN_SIZE * ALIGN_SIZE;
                task.memset
                    .vpn_to_ppn(VirtPage::from_addr(stack_ptr))
                    .get_buffer()[stack_ptr % PAGE_SIZE..stack_ptr % PAGE_SIZE + arg.bytes().len()]
                    .copy_from_slice(arg.as_bytes());
                stack_ptr
            })
            .collect();

        let mut push_num = |num: usize| {
            stack_ptr = stack_ptr - core::mem::size_of::<usize>();

            task.memset
                .vaddr_to_paddr(stack_ptr.into())
                .write_volatile(num);

            stack_ptr
        };

        log::info!("ph_count: {}", file.header.pt2.ph_count());
        // TODO: Support relocated memory locations.
        let base = 0;
        // let random_ptr = push_bytes(&[0u8; 16], 0);
        let mut auxv = BTreeMap::new();
        // auxv.insert(AuxV::PLATFORM, push_str("riscv"));
        auxv.insert(AuxV::EXECFN, args_ptr[0]);
        // auxv.insert(AuxV::PHNUM, elf_header.pt2.ph_count() as usize);
        // auxv.insert(AuxV::PHNUM, file.header.pt2.ph_count() as _);
        auxv.insert(AuxV::PAGESZ, PAGE_SIZE);
        auxv.insert(AuxV::ENTRY, base + file.header.pt2.entry_point() as usize);
        // auxv.insert(AuxV::PHENT, elf_header.pt2.ph_entry_size() as usize);
        // auxv.insert(AuxV::PHENT, file.header.pt2.ph_entry_size() as _);
        // TODO: Support phdr
        // auxv.insert(AuxV::PHDR, base + elf.get_ph_addr().unwrap_or(0) as usize);
        auxv.insert(AuxV::GID, 0);
        auxv.insert(AuxV::EGID, 0);
        auxv.insert(AuxV::UID, 0);
        auxv.insert(AuxV::EUID, 0);
        auxv.insert(AuxV::NULL, 0);
        // auxv.insert(AuxV::SECURE, 0);
        // auxv.insert(AuxV::RANDOM, random_ptr);

        // push_num(0);

        // auxv top
        push_num(0);
        // TODO: push auxv
        auxv.into_iter().for_each(|(key, v)| {
            push_num(v);
            push_num(key as usize);
        });
        // ENVP TOP
        push_num(0);
        // ARGS TOP
        push_num(0);
        // Args
        args_ptr.iter().rev().for_each(|x| {
            push_num(*x);
        });
        // ARGS_BOTTOM
        push_num(args_ptr.len());

        task.trap_frame.get_mut()[TrapFrameArgs::SEPC] = file.header.pt2.entry_point() as _;
        task.trap_frame.get_mut()[TrapFrameArgs::SP] = stack_ptr;

        task
    }

    /// Get the mutable Trapframe pointer from the reference
    ///
    /// The trapframe will only be used in a thread to drop user.
    #[inline]
    pub fn get_tf_mut_force(&self) -> &'static mut TrapFrame {
        unsafe { self.trap_frame.get().as_mut().unwrap() }
    }

    #[inline]
    pub fn into_user(&self) {
        self.page_table.change();
        let tf = self.get_tf_mut_force();
        loop {
            let reason = run_user_task(tf);
            if reason == EscapeReason::SysCall {
                tf.syscall_ok();
                let sysid = match Sysno::new(tf[TrapFrameArgs::SYSCALL]) {
                    Some(sysid) => sysid,
                    _ => {
                        tf[TrapFrameArgs::RET] = (-Errno::EINVAL.into_raw()) as _;
                        return;
                    }
                };
                tf[TrapFrameArgs::RET] = match sysid {
                    Sysno::set_tid_address => 1,
                    Sysno::getuid => 0,
                    Sysno::ioctl => -1 as isize as usize,
                    Sysno::dup3 => 4,
                    Sysno::getpid => 1,
                    Sysno::rt_sigprocmask => 0,
                    Sysno::rt_sigaction => 0,
                    Sysno::getppid => 1,
                    Sysno::uname => 0,
                    Sysno::getcwd => 0,
                    #[cfg(target_arch = "x86_64")]
                    Sysno::arch_prctl => 0,
                    Sysno::brk => {
                        if tf.args()[0] == 0 {
                            0x2_0000_0000
                        } else {
                            tf.args()[0]
                        }
                    }
                    Sysno::write => {
                        if tf.args()[0] == 2 || tf.args()[0] == 1 {
                            unsafe {
                                core::slice::from_raw_parts_mut(
                                    tf.args()[1] as *mut u8,
                                    tf.args()[2],
                                )
                                .iter()
                                .map(u8::clone)
                                .for_each(DebugConsole::putchar);
                            }
                        }
                        tf.args()[2]
                    }
                    _ => todo!("Syscall {sysid:?} is not implemented"),
                };
                // log::debug!("3");
                // tf.syscall_ok();
                // log::debug!("sys trapframe: {:#x?}", tf);
            }
        }
    }
}
