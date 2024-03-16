# Trap

## Process

1. We save the context including general registers, program counter and program state word to the stack.
2. Disable non-critical IRQs.
3. Get into trap.
4. Restore the trap.

## Context Manager

General Registers are saved before trap, in which gp and tp are only valid for user traps.

Trap Frames are composed of Grneral Registers, Supervisor Exception Program Counter and Supervisor Status Register.

For individual tasks, there are also context information including return address, data registers and so on.

When initializing Task Context, we pass the entry point(Program Counter), stack top pointer and thread pointer register to the initializer and switch the tasks easily.

## Trap Handler

Traps are as such:
1. Exception: Invalid arithmetics or instructions.
2. Interrupt: IRQs.
3. System Call: Not implemented yet.


