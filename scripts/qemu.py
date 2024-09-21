import subprocess
from . import config, cargo

mem_size = "1G"
core_num = "4"


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
            "-machine",
            "raspi3b",
            "-kernel",
            cargo.kernel_bin,
        ]
    elif config.arch == "loongarch64":
        qemu_args += ["qemu-system-loongarch64", "-kernel", cargo.kernel_elf]

    # qemu_args += [
    #     "-m",
    #     mem_size,
    #     "-smp",
    #     core_num,
    #     "-D",
    #     "qemu.log",
    #     "-d",
    #     "in_asm,int,pcall,cpu_reset,guest_errors",
    # ]

    # qemu_args += [
    #     "-device",
    #     "sdhci-pci",
    #     "-drive",
    #     "id=mydrive,if=none,format=raw,file=mount.img",
    #     "-device",
    #     "sd-card,drive=mydrive",
    # ]


    qemu_args += [
        "-drive",
        "id=mydrive,if=sd,format=raw,file=mount.img",
    ]

    # Enable E1000 Device.
    # qemu_args += ["-netdev", "user,id=net0", "-device", "e1000,netdev=net0"]

    # Enable USB Device.
    # qemu_args += [
    #     "-usb",
    #     "-device",
    #     "usb-ehci,id=ehci",
    #     "-device",
    #     "usb-tablet,bus=ehci.0",
    # ]
    
    # Configure graphic for qemu
    if config.graphic:
        qemu_args += ["-serial", "stdio"]
    else:
        qemu_args += ["-nographic"]

    # qemu_args += [
    #     "-drive",
    #     "file={},if=none,format=raw,id=x0".format("mount.img"),
    #     "-device",
    #     "virtio-blk-{},drive=x0".format(bus),
    # ]

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
