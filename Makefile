build:
	dd if=/dev/zero of=./disk.img bs=1M count=50
	mkfs.vfat -F 32 ./disk.img
	mkdir -p ./loopback
	sudo mount ./disk.img ./loopback
	cd user && bash compile.sh && cd ..
	sudo mkdir -p ./loopback/bin
	sudo cp user/elf/* ./loopback/bin
	sudo umount ./loopback
	cd os && LOG=TRACE CARGO_BUILD_RUSTFLAGS="-Clink-arg=-Tsrc/linker.ld -Cforce-frame-pointers=yes" \
		cargo build --release --target riscv64gc-unknown-none-elf && cd ..
	rust-objcopy --strip-all os/target/riscv64gc-unknown-none-elf/release/os -O binary os/target/riscv64gc-unknown-none-elf/release/os.bin

run: build
	qemu-system-riscv64 \
        -machine virt \
        -nographic \
        -bios ./rustsbi-qemu.bin \
        -kernel os/target/riscv64gc-unknown-none-elf/release/os.bin \
	    -drive file=disk.img,if=none,format=raw,id=x0 \
	    -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0

clean:
	cargo clean
	rm -rf target
