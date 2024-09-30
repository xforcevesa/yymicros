.globl __call
    __call:
        li x9, 8
        sub   sp, sp, x9
        call  green_thread_call_entry
        li x9, 16
        add   sp, sp, x9
        ret

.altmacro
.macro SAVE_SN_GREEN_THREAD n
    sd s\n, (\n+2)*8(a0)
.endm
.macro LOAD_SN_GREEN_THREAD n
    ld s\n, (\n+2)*8(a1)
.endm

.globl __switch_green_thread
    __switch_green_thread:
        # __switch_task(
        #     current_task_cx_ptr: *mut TaskContext,
        #     next_task_cx_ptr: *const TaskContext
        # )
        # save kernel stack of current task
        sd sp, 8(a0)
        # save ra & s0~s11 of current execution
        .set n, 11
        .rept 12
            SAVE_SN_GREEN_THREAD %n
            .set n, n - 1
        .endr
        # restore ra & s0~s11 of next execution
        .set n, 11
        .rept 12
            LOAD_SN_GREEN_THREAD %n
            .set n, n - 1
        .endr
        # restore kernel stack of next task
        ld sp, 8(a1)
        ret

        