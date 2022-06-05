MBALIGN   equ 1 << 0 ; align modules on page boundaries (4K)
MEMINFO equ 1 << 1 ; provide memory map or smth
FLAGS   equ MBALIGN | MEMINFO ; Multiboot flags
MAGIC   equ 0x1BADB002
CHEKSUM equ -(MAGIC + FLAGS)

; Provide a 16 KiB stack. It grows downwards on x86. It also must be 16-byte
; aligned according to SysV ABI standard.
SECTION .bss
ALIGN 16
stack_bottom:
    resb 16384
stack_top:

SECTION .text
GLOBAL _start
EXTERN kernel_main

; Declare Multiboot headers which GRUB will search for.
; They have to be in the first 8 KiB of the kernel file and 4-bytes aligned.
; Btw, I wanted it to be in its own .multiboot section to force it but cargo
; didn't let me.
; cargo bad
ALIGN 4
multiboot:
    dd MAGIC
    dd FLAGS
    dd CHEKSUM

ALIGN 16
_start:
    mov esp, stack_top
    call kernel_main
    cli
halt:
    hlt
    jmp halt
