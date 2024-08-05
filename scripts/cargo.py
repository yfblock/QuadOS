import subprocess, os
from . import config

env = os.environ.copy()
kernel_elf = ""
kernel_bin = ""

# Add rust flags to the environment
def add_rust_flags(flags: str):
    global env

    if "RUSTFLAGS" not in env.keys():
        env["RUSTFLAGS"] = ""

    env["RUSTFLAGS"] += " {}".format(flags)

# Add rust cfg to compiler configuration
def add_rust_cfg(key: str, value = None):
    global env
    if value is None:
        add_rust_flags(' --cfg {}'.format(key))
    else:
        add_rust_flags(' --cfg {}="{}"'.format(key, value))

# Build rust program
def build():
    global env
    global kernel_elf
    global kernel_bin
    
    extra_args = []
    kernel_elf = "target/{}/release/quados".format(config.rust_target)
    kernel_bin = kernel_elf + ".bin"
    env["LOG"] = config.log

    # Add rust configuration
    add_rust_cfg("board", "qemu")
    
    # Build x86_64 kernel
    if config.arch == "x86_64":
        add_rust_flags("-Clink-arg=-no-pie")
    
    # loongarch64 build rust std.
    if config.arch == "loongarch64":
        extra_args += ["-Z", "build-std=core,alloc"]
    
    # Run cargo build command
    subprocess.run([
        "cargo",
        "build",
        "--release",
        "--target",
        config.rust_target,
    ] + extra_args, env=env).check_returncode()

    # Convert elf to binary file
    subprocess.run([
        "rust-objcopy",
        "--binary-architecture={}".format(config.arch),
        kernel_elf,
        "--strip-all",
        "-O",
        "binary",
        kernel_bin
    ]).check_returncode()
