arch = ""
graphic = False
platform = ""
rootfs = ""
log = ""
rust_target = ""

# Rust target list
target_list = {
    "riscv64": "riscv64gc-unknown-none-elf",
    "x86_64": "x86_64-unknown-none",
    "aarch64": "aarch64-unknown-none-softfloat",
    "loongarch64": "loongarch64-unknown-none"
}

# Build configuration.
def build():
    assert(arch in target_list.keys())
    global rust_target
    rust_target = target_list[arch]
