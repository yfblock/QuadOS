use core::panic::PanicInfo;

use polyhal::instruction::Instruction;

/// Rust lang panic handler.
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    log::error!("panic: {:#x?}", info);
    Instruction::shutdown();
}
