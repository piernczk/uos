MBALIGN   equ 1 << 0
MEMINFO equ 1 << 1
FLAGS   equ MBALIGN | MEMINFO
MAGIC   equ 0x1BADB002
CHEKSUM equ -(MAGIC + FLAGS)

SECTION .bss
ALIGN 16
stack_bottom:
    resb 16384
stack_top:

SECTION .text
GLOBAL _start
EXTERN kernel_main

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
