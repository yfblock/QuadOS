import subprocess
from . import config, cargo

mem_size = "1G"
core_num = "1"

# Run qemu command.
def run():
    qemu_args = []
    bus = "device"

    if config.arch == "riscv64":
        qemu_args += [
            "qemu-system-riscv64",
            "-kernel",
            cargo.kernel_bin,
            "-machine",
            "virt",
        ]
    elif config.arch == "x86_64":
        qemu_args += [
            "qemu-system-x86_64",
            "-machine",
            "q35",
            "-kernel",
            cargo.kernel_elf,
            "-cpu",
            "IvyBridge-v2",
        ]
        bus = "pci"
    elif config.arch == "aarch64":
        qemu_args += [
            "qemu-system-aarch64",
            "-cpu",
            "cortex-a72",
            "-machine",
            "virt",
            "-kernel",
            cargo.kernel_bin,
        ]
    elif config.arch == "loongarch64":
        qemu_args += ["qemu-system-loongarch64", "-kernel", cargo.kernel_elf]

    qemu_args += [
        "-m",
        mem_size,
        "-smp",
        core_num,
        "-D",
        "qemu.log",
        "-d",
        "in_asm,int,pcall,cpu_reset,guest_errors",
    ]
    
    if not config.graphic or config.arch != "x86_64":
        qemu_args += ["-nographic"]
    
    qemu_args += [
        "-drive", 
        "file={},if=none,format=raw,id=x0".format("mount.img"),
        "-device",
        "virtio-blk-{},drive=x0".format(bus)
    ]
    
    if config.gdb:
        qemu_args += ["-s", "-S"]

    print(
        [
            "qemu-system-{}".format(config.arch),
            "-kernel",
            cargo.kernel_bin,
        ]
        + qemu_args
    )

    subprocess.run(qemu_args).check_returncode()
