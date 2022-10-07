MBALIGN equ 1 << 0 ; align modules on page boundaries (4K)
MEMINFO equ 1 << 1 ; provide memory map or smth
FLAGS   equ MBALIGN | MEMINFO ; Multiboot flags
MAGIC   equ 0x1BADB002
CHEKSUM equ -(MAGIC + FLAGS)

SECTION .text
GLOBAL _start
GLOBAL enable_paging
EXTERN kernel_main
GLOBAL load_gdtr

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

enable_paging:
    ; set the CR3 register to point at the page_directory
    mov eax, PAGE_DIRECTORY
    mov cr3, eax

    ; set the CR0.PE (Paging Enable) bit
    mov eax, cr0
    or eax, 0x80000000
    mov cr0, eax
    ret

load_gdtr:
    lgdt [GDTR]
    jmp 0x08:reload_cs ; far jump to the kernel code segment 0x08
reload_cs:
    ; set data registers to point at the kernel data segment 0x10
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    ret

SECTION .bss
GLOBAL PAGE_DIRECTORY
GLOBAL PAGE_TABLES
GLOBAL ESP0
GLOBAL GDTR
GLOBAL GDT
GLOBAL IDTR
GLOBAL IDT
GLOBAL TSS

ALIGN 4096
PAGE_DIRECTORY:
    resd 1024
PAGE_TABLES:
    resd 1023 * 1024 ; the last PDE is used for recursive paging

; Provide a 16 KiB stack. It grows downwards on x86. It also must be 16-byte
; aligned according to SysV ABI standard.
ALIGN 16
stack_bottom:
    resb 16384
stack_top:
; temporary TSS stack
ESP0:
    resb 16384

TSS:
    resb 0x6C
GDTR:
    resq 1
GDT:
    resq 6
IDTR:
    resq 1
IDT:
    resq 256
