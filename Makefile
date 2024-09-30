prepare-fatfs:
	dd if=/dev/zero of=./disk.img bs=1M count=50
	mkfs.vfat -F 32 ./disk.img

prepare-ext4:
	dd if=/dev/zero of=./disk.img bs=1M count=50
	mkfs.ext4 ./disk.img
	
prepare:
	mkdir -p ./loopback
	sudo mount ./disk.img ./loopback
	cd user && bash compile.sh && cd ..
	sudo mkdir -p ./loopback/bin
	sudo cp user/elf/* ./loopback/bin
	sudo umount ./loopback

build-fatfs: prepare-fatfs prepare
	cd os && LOG=TRACE CARGO_BUILD_RUSTFLAGS="-Clink-arg=-Tsrc/linker.ld -Cforce-frame-pointers=yes" \
		cargo build --release --target riscv64gc-unknown-none-elf && cd ..
	rust-objcopy --strip-all os/target/riscv64gc-unknown-none-elf/release/os -O binary os/target/riscv64gc-unknown-none-elf/release/os.bin

build-ext4: prepare-ext4 prepare
	cd os && LOG=TRACE CARGO_BUILD_RUSTFLAGS="-Clink-arg=-Tsrc/linker.ld -Cforce-frame-pointers=yes" \
		cargo build --release --features ext4 --target riscv64gc-unknown-none-elf && cd ..
	rust-objcopy --strip-all os/target/riscv64gc-unknown-none-elf/release/os -O binary os/target/riscv64gc-unknown-none-elf/release/os.bin


run-fatfs: build-fatfs
	qemu-system-riscv64 \
        -machine virt \
        -nographic \
        -bios ./rustsbi-qemu.bin \
        -kernel os/target/riscv64gc-unknown-none-elf/release/os.bin \
	    -drive file=disk.img,if=none,format=raw,id=x0 \
	    -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0

run-ext4: build-ext4
	qemu-system-riscv64 \
        -machine virt \
        -nographic \
        -bios ./rustsbi-qemu.bin \
        -kernel os/target/riscv64gc-unknown-none-elf/release/os.bin \
	    -drive file=disk.img,if=none,format=raw,id=x0 \
	    -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0

run: run-fatfs

run1: run-ext4

clean:
	cd os && cargo clean && cd ..
	rm -rf target user/elf/* disk.img

total_lines:
	bash total_lines.sh
