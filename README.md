# yymicros

An implementation of the barebone on RISC-V architecture microcontroller.

It was first started at the beginning of 2024 and had once stopped.

But now, it reborns again and continues to live.

It'll be a fun project to work on and a composition for the oscomp2025.

## VSCode Configuration

```json
{
    "rust-analyzer.cargo.target": "riscv64gc-unknown-none-elf",
    "rust-analyzer.check.allTargets": false
}
```

## Running Steps

Providing you're to run os1:

```bash
git clone https://github.com/rustsbi/rustsbi-qemu
cd rustsbi-qemu
cargo test
cd ../os1
make run
```

## Contents

rCore experiments.

|Name|Note|
|:-:|:-:|
|os1|hello-world to no-std|
