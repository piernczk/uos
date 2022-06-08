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

; let's hope it just works for now
gdtr:
    ; null
    dq 0

    ; kernel code
    dw 0xFFFF
    dw 0
    db 0
    db 0x9A
    db 0xCF
    db 0

    ; kernel data
    dw 0xFFFF
    dw 0
    db 0
    db 0x92
    db 0xCF
    db 0

    ; user code
    dw 0xFFFF
    dw 0
    db 0
    db 0xFA
    db 0xCF
    db 0

    ; user data
    dw 0xFFFF
    dw 0
    db 0
    db 0xF2
    db 0xCF
    db 0

    ; tss
    dw 0x6B
    dw TSS
    db TSS >> 16
    db 0x89
    db 0
    db TSS >> 24

tss:
TSS equ $-$$
    resb 0x6C

ALIGN 16
_start:
    mov esp, stack_top
    lgdt [gdtr]
    call kernel_main
    cli
halt:
    hlt
    jmp halt
