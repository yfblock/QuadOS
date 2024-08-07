use core::panic::PanicInfo;

use polyhal::instruction::Instruction;

/// Rust lang panic handler.
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    if let Some(message) = info.message() {
        log::error!("panic: {:#x?}", message);
    }
    if let Some(location) = info.location() {
        log::error!(
            "location: {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    }
    Instruction::shutdown();
}
