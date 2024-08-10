use core::{
    fmt::{Arguments, Write},
    panic::PanicInfo,
};

use polyhal::{debug_console::DebugConsole, instruction::Instruction};

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

/// Print arguments to the stdout
pub(crate) fn print_args(args: Arguments) {
    let _ = DebugConsole.write_fmt(args);
}

/// Print formatted arguments to the stdout
#[macro_export]
macro_rules! println {
    () => {
        crate::lang_items::print_args(format_args!("\n"));
    };
    ($fmt: expr $(, $($arg: tt)+)?) => {
        crate::lang_items::print_args(format_args!("{}\n", format_args!($fmt $(, $($arg)+)?)));
    };
}
