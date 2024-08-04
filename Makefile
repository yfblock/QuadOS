SHELL := /bin/bash
BIN   :=
quados = $(shell kbuild $(1) quados.yaml $(BIN) $(2))
quados_config = $(call quados,config,get_cfg $(1))
quados_env = $(call quados,config,get_env $(1))
quados_meta = $(call quados,config,get_meta $(1))
quados_triple = $(call quados,config,get_triple $(1))
NET  := off
LOG  := error
RELEASE := release
QEMU_EXEC ?= 
GDB  ?= gdb-multiarch
ARCH := $(call quados_triple,arch)
ROOT_FS := $(call quados_config,root_fs)
TARGET := $(call quados_meta,target)

BUS  := device
ifeq ($(ARCH), x86_64)
  QEMU_EXEC += qemu-system-x86_64 \
				-machine q35 \
				-kernel $(KERNEL_ELF) \
				-cpu IvyBridge-v2
  BUS := pci
else ifeq ($(ARCH), riscv64)
  QEMU_EXEC += qemu-system-$(ARCH) \
				-machine virt \
				-kernel $(KERNEL_BIN)
else ifeq ($(ARCH), aarch64)
  QEMU_EXEC += qemu-system-$(ARCH) \
				-cpu cortex-a72 \
				-machine virt \
				-kernel $(KERNEL_BIN)
else ifeq ($(ARCH), loongarch64)
  QEMU_EXEC += qemu-system-$(ARCH) -kernel $(KERNEL_ELF)
  BUS := pci
else
  $(error "ARCH" must be one of "x86_64", "riscv64", "aarch64" or "loongarch64", Current "$(ARCH)")
endif

KERNEL_ELF = target/$(TARGET)/$(RELEASE)/quados
KERNEL_BIN = $(KERNEL_ELF).bin
FS_IMG  := mount.img
QEMU_EXEC += -m 1G\
			-nographic \
			-smp 1 \
			-D qemu.log -d in_asm,int,pcall,cpu_reset,guest_errors

ifeq ($(NVME), on)
QEMU_EXEC += -drive file=$(FS_IMG),if=none,id=nvm \
				-device nvme,serial=deadbeef,drive=nvm
else
QEMU_EXEC += -drive file=$(FS_IMG),if=none,format=raw,id=x0
	QEMU_EXEC += -device virtio-blk-$(BUS),drive=x0
endif

ifeq ($(NET), on)
QEMU_EXEC += -netdev user,id=net0,hostfwd=tcp::6379-:6379,hostfwd=tcp::2222-:2222,hostfwd=tcp::2000-:2000,hostfwd=tcp::8487-:8487,hostfwd=tcp::5188-:5188,hostfwd=tcp::12000-:12000 -object filter-dump,id=net0,netdev=net0,file=packets.pcap \
	-device virtio-net-$(BUS),netdev=net0
endif

all: build

TESTCASE := testcase-$(ARCH)
fs-img:
	@echo "TESTCASE: $(TESTCASE)"
	@echo "ROOT_FS: $(ROOT_FS)"
	rm -f $(FS_IMG)
	dd if=/dev/zero of=$(FS_IMG) bs=1M count=128
	sync
ifeq ($(ROOT_FS), fat32)
	mkfs.vfat -F 32 $(FS_IMG)
	mkdir mount/ -p
	sudo mount $(FS_IMG) mount/ -o uid=1000,gid=1000
	sudo rm -rf mount/*
else 
	mkfs.ext4  -F -O ^metadata_csum_seed $(FS_IMG)
	mkdir mount/ -p
	sudo mount $(FS_IMG) mount/
endif
	sudo cp -rf resources/$(TESTCASE)/* mount/
	sync
	sudo umount $(FS_IMG)

build:
	kbuild build quados.yaml $(BIN)
	rust-objcopy --binary-architecture=$(ARCH) $(KERNEL_ELF) --strip-all -O binary $(KERNEL_BIN)

justbuild: fs-img build 

run: fs-img build
	time $(QEMU_EXEC)

fdt:
	@qemu-system-riscv64 -M 128m -machine virt,dumpdtb=virt.out
	fdtdump virt.out

justrun: fs-img
	$(QEMU_EXEC)

debug: fs-img build
	@tmux new-session -d \
	"$(QEMU_EXEC) -s -S && echo 'Press any key to continue' && read -n 1" && \
	tmux split-window -h "$(GDB) $(KERNEL_ELF) -ex 'target remote localhost:1234' -ex 'disp /16i $$pc' " && \
	tmux -2 attach-session -d

clean:
	rm -rf target/

iso: build
	cp $(KERNEL_ELF) resources/iso/example
	grub-mkrescue -o bootable.iso resources/iso

boot-iso: iso
	qemu-system-x86_64 -cdrom bootable.iso -serial stdio

.PHONY: all run build clean gdb justbuild iso boot-iso
