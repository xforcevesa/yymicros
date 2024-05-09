build:
	RUSTFLAGS="-Clink-arg=-Tsrc/linker.ld -Cforce-frame-pointers=yes" cargo build \
		--target riscv64gc-unknown-none-elf \
		--release
	rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/yymicros \
		 -O binary target/riscv64gc-unknown-none-elf/release/yymicros.bin

run: build
	qemu-system-riscv64 \
    	-machine virt \
    	-nographic \
    	-bios ../../bootloader/rustsbi-qemu.bin \
		-device loader,file=target/riscv64gc-unknown-none-elf/release/yymicros.bin,addr=0x80200000

clean:
	cargo clean
