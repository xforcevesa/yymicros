# Bootstrap

## In linker.ld

1. Base address: every addresses are offsets of this base addr.
2. Entry Point: it defines _start() as its entry function.
3. Stack Segmentation: Symbol _sbss and _ebss marked as respectively stack base and top. User stack segmentations are seperated from bootstrap stack space.

## In _start

BOOT_STACK byte array linked as .bss.stack.

In the entry, the stack spaces and the MMU are initialized. Then call rust_entry.

## In rust_entry

1. Clear the BSS segmentation.
2. Multi-processor initialization.
3. Set the trap_vector_base.
4. Call the rust_main (in the runtime).

