use std::env;
use std::io::Result;

#[allow(unused_macros)]
macro_rules! display {
    ($fmt:expr) => (println!("cargo:warning={}", format!($fmt)));
    ($fmt:expr, $($arg:tt)*) => (println!(concat!("cargo:warning=", $fmt), $($arg)*));
}

fn main() {
    gen_linker_script(&env::var("CARGO_CFG_BOARD").expect("can't find board"))
        .expect("can't generate linker script");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ARCH");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=linker/linker.lds.S");
}

fn gen_linker_script(platform: &str) -> Result<()> {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("can't find target");
    let fname = format!("linker/linker_{}_{}.lds", arch, platform);
    let (output_arch, kernel_base) = if arch == "x86_64" {
        ("i386:x86-64", "0xffffff8000200000")
    } else if arch.contains("riscv64") {
        ("riscv", "0xffffffc080200000") // OUTPUT_ARCH of both riscv32/riscv64 is "riscv"
    } else if arch.contains("aarch64") {
        ("aarch64", "0xffffff8040080000")
    } else if arch.contains("loongarch64") {
        ("loongarch64", "0x9000000090000000")
    } else {
        (arch.as_str(), "0")
    };
    let ld_content = std::fs::read_to_string("linker/linker.lds.S")?;
    let ld_content = ld_content.replace("%ARCH%", output_arch);
    let ld_content = ld_content.replace("%KERNEL_BASE%", kernel_base);

    std::fs::write(&fname, ld_content)?;
    println!("cargo:rustc-link-arg=-Tkernel/{}", fname);
    Ok(())
}
