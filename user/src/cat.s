	.file	"cat.c"
	.option nopic
	.attribute arch, "rv64i2p1_m2p0_a2p1_f2p2_d2p2_c2p0_zicsr2p0"
	.attribute unaligned_access, 0
	.attribute stack_align, 16
	.text
	.align	1
	.type	syscall_read, @function
syscall_read:
	addi	sp,sp,-64
	sd	ra,56(sp)
	sd	s0,48(sp)
	addi	s0,sp,64
	mv	a5,a0
	sd	a1,-48(s0)
	sd	a2,-56(s0)
	sw	a5,-36(s0)
	li	a5,63
	lw	a4,-36(s0)
	mv	a6,a4
	ld	a4,-48(s0)
	ld	a3,-56(s0)
 #APP
# 18 "syscall_test.h" 1
	mv a7, a5
mv a0, a6
mv a1, a4
mv a2, a3
ecall
mv a5, a0

# 0 "" 2
 #NO_APP
	sd	a5,-24(s0)
	ld	a5,-24(s0)
	mv	a0,a5
	ld	ra,56(sp)
	ld	s0,48(sp)
	addi	sp,sp,64
	jr	ra
	.size	syscall_read, .-syscall_read
	.align	1
	.type	syscall_write, @function
syscall_write:
	addi	sp,sp,-64
	sd	ra,56(sp)
	sd	s0,48(sp)
	addi	s0,sp,64
	mv	a5,a0
	sd	a1,-48(s0)
	sd	a2,-56(s0)
	sw	a5,-36(s0)
	li	a5,64
	lw	a4,-36(s0)
	mv	a6,a4
	ld	a4,-48(s0)
	ld	a3,-56(s0)
 #APP
# 35 "syscall_test.h" 1
	mv a7, a5
mv a0, a6
mv a1, a4
mv a2, a3
ecall
mv a5, a0

# 0 "" 2
 #NO_APP
	sd	a5,-24(s0)
	ld	a5,-24(s0)
	mv	a0,a5
	ld	ra,56(sp)
	ld	s0,48(sp)
	addi	sp,sp,64
	jr	ra
	.size	syscall_write, .-syscall_write
	.align	1
	.type	syscall_exit, @function
syscall_exit:
	addi	sp,sp,-32
	sd	ra,24(sp)
	sd	s0,16(sp)
	addi	s0,sp,32
	mv	a5,a0
	sw	a5,-20(s0)
	li	a5,93
	lw	a4,-20(s0)
 #APP
# 99 "syscall_test.h" 1
	mv a7, a5
mv a0, a4
ecall

# 0 "" 2
 #NO_APP
	nop
	ld	ra,24(sp)
	ld	s0,16(sp)
	addi	sp,sp,32
	jr	ra
	.size	syscall_exit, .-syscall_exit
	.align	1
	.type	syscall_open, @function
syscall_open:
	addi	sp,sp,-48
	sd	ra,40(sp)
	sd	s0,32(sp)
	addi	s0,sp,48
	sd	a0,-40(s0)
	mv	a5,a1
	sw	a5,-44(s0)
	li	a5,56
	ld	a4,-40(s0)
	lw	a3,-44(s0)
 #APP
# 123 "syscall_test.h" 1
	mv a7, a5
mv a0, a4
mv a1, a3
ecall
mv a5, a0

# 0 "" 2
 #NO_APP
	sd	a5,-24(s0)
	ld	a5,-24(s0)
	sext.w	a5,a5
	mv	a0,a5
	ld	ra,40(sp)
	ld	s0,32(sp)
	addi	sp,sp,48
	jr	ra
	.size	syscall_open, .-syscall_open
	.align	1
	.type	syscall_close, @function
syscall_close:
	addi	sp,sp,-48
	sd	ra,40(sp)
	sd	s0,32(sp)
	addi	s0,sp,48
	mv	a5,a0
	sw	a5,-36(s0)
	li	a5,57
	lw	a4,-36(s0)
 #APP
# 138 "syscall_test.h" 1
	mv a7, a5
mv a0, a4
ecall
mv a5, a0

# 0 "" 2
 #NO_APP
	sd	a5,-24(s0)
	ld	a5,-24(s0)
	sext.w	a5,a5
	mv	a0,a5
	ld	ra,40(sp)
	ld	s0,32(sp)
	addi	sp,sp,48
	jr	ra
	.size	syscall_close, .-syscall_close
	.section	.rodata
	.align	3
.LC1:
	.string	"Open file: "
	.align	3
.LC2:
	.string	"\n"
	.align	3
.LC3:
	.string	"Failed to open file\n"
	.align	3
.LC0:
	.string	"/yes/no3"
	.text
	.align	1
	.globl	_start
	.type	_start, @function
_start:
	addi	sp,sp,-48
	sd	ra,40(sp)
	sd	s0,32(sp)
	addi	s0,sp,48
	lui	a5,%hi(.LC0)
	addi	a5,a5,%lo(.LC0)
	ld	a4,0(a5)
	sd	a4,-32(s0)
	lbu	a5,8(a5)
	sb	a5,-24(s0)
	li	a2,12
	lui	a5,%hi(.LC1)
	addi	a1,a5,%lo(.LC1)
	li	a0,1
	call	syscall_write
	addi	a5,s0,-32
	li	a2,9
	mv	a1,a5
	li	a0,1
	call	syscall_write
	li	a2,1
	lui	a5,%hi(.LC2)
	addi	a1,a5,%lo(.LC2)
	li	a0,1
	call	syscall_write
	addi	a5,s0,-32
	li	a1,512
	mv	a0,a5
	call	syscall_open
	mv	a5,a0
	sw	a5,-20(s0)
	lw	a5,-20(s0)
	sext.w	a5,a5
	bge	a5,zero,.L11
	li	a2,20
	lui	a5,%hi(.LC3)
	addi	a1,a5,%lo(.LC3)
	li	a0,1
	call	syscall_write
	li	a0,1
	call	syscall_exit
.L11:
	addi	a4,s0,-32
	lw	a5,-20(s0)
	li	a2,9
	mv	a1,a4
	mv	a0,a5
	call	syscall_write
	lw	a4,-20(s0)
	li	a2,1
	lui	a5,%hi(.LC2)
	addi	a1,a5,%lo(.LC2)
	mv	a0,a4
	call	syscall_write
	sd	zero,-48(s0)
	sh	zero,-40(s0)
	addi	a4,s0,-48
	lw	a5,-20(s0)
	li	a2,9
	mv	a1,a4
	mv	a0,a5
	call	syscall_read
	addi	a5,s0,-48
	li	a2,9
	mv	a1,a5
	li	a0,1
	call	syscall_write
	lw	a5,-20(s0)
	mv	a0,a5
	call	syscall_close
	li	a0,0
	call	syscall_exit
	nop
	ld	ra,40(sp)
	ld	s0,32(sp)
	addi	sp,sp,48
	jr	ra
	.size	_start, .-_start
	.ident	"GCC: (g04696df0963) 14.2.0"
	.section	.note.GNU-stack,"",@progbits
