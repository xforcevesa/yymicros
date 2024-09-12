build:
	dd if=/dev/zero of=./disk.img bs=1M count=50
	mkfs.vfat -F 32 ./disk.img
	cd user && bash compile.sh && cd ..
	LOG=TRACE cargo build --release
	rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/os -O binary target/riscv64gc-unknown-none-elf/release/os.bin

run: build
	qemu-system-riscv64 \
    -machine virt \
	-serial mon:stdio \
    -bios ./rustsbi-qemu.bin \
    -kernel target/riscv64gc-unknown-none-elf/release/os.bin \
	-drive file=disk.img,if=none,format=raw,id=x0 \
	-device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
	-device virtio-gpu-device,bus=virtio-mmio-bus.1 \
	-device virtio-mouse-device,bus=virtio-mmio-bus.2 \
	-device virtio-net-device,netdev=net0,bus=virtio-mmio-bus.3 \
	-netdev user,id=net0,hostfwd=tcp::5555-:5555 \
	-device virtio-sound-device,audiodev=audio0,bus=virtio-mmio-bus.4 \
	-audiodev alsa,id=audio0

clean:
	cargo clean
	rm -rf target
