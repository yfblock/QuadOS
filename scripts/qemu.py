import subprocess
from . import config, cargo

# Run qemu command.
def run():
    qemu_args = []

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
        "1G",
        "-nographic",
        "-smp",
        "1",
        "-D",
        "qemu.log",
        "-d",
        "in_asm,int,pcall,cpu_reset,guest_errors",
    ]

    print(
        [
            "qemu-system-{}".format(config.arch),
            "-kernel",
            cargo.kernel_bin,
        ]
        + qemu_args
    )

    subprocess.run(qemu_args).check_returncode()
