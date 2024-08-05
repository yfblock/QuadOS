SHELL := /bin/bash

ifeq ($(NVME), on)
QEMU_EXEC += -drive file=$(FS_IMG),if=none,id=nvm \
				-device nvme,serial=deadbeef,drive=nvm
else
QEMU_EXEC += -drive file=$(FS_IMG),if=none,format=raw,id=x0
	QEMU_EXEC += -device virtio-blk-$(BUS),drive=x0
endif

ifeq ($(NET), on)
QEMU_EXEC += -netdev user,id=net0,hostfwd=tcp::6379-:6379 \
	-object filter-dump,id=net0,netdev=net0,file=packets.pcap \
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
	./quados build

justbuild: fs-img build 

run: fs-img build
	time $(QEMU_EXEC)

fdt:
	@qemu-system-riscv64 -M 128m -machine virt,dumpdtb=virt.out
	fdtdump virt.out

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
