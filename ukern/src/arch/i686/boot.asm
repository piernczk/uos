MBALIGN equ 1 << 0 ; align modules on page boundaries (4K)
MEMINFO equ 1 << 1 ; provide memory map or smth
FLAGS   equ MBALIGN | MEMINFO ; Multiboot flags
MAGIC   equ 0x1BADB002
CHEKSUM equ -(MAGIC + FLAGS)

SECTION .text.init progbits alloc exec nowrite align=16
GLOBAL _start
GLOBAL load_gdtr
EXTERN KERNEL_START_PHYS
EXTERN KERNEL_END_PHYS

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

_start:
    ; init GDTR
    mov eax, GDT_PHYS
    mov [GDTR_PHYS + 2], eax

    lgdt [GDTR_PHYS]
    jmp 0x08:reload_segments_init ; far jump to the kernel code segment 0x08
reload_segments_init:
    ; set data registers to point at the kernel data segment 0x10
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    ; how many pages to allocate at 0xC0000000, including 0x0000-0xFFFF
    mov eax, KERNEL_END_PHYS
    add eax, 4095
    mov ecx, 4096
    mov edx, 0
    div ecx
    mov ecx, eax

tables:
    dec ecx

    ; ebx - the entry's value
    mov ebx, ecx
    shl ebx, 12
    or ebx, 0b011

    ; eax - the entry's address
    mov eax, ecx
    mov edx, 4
    mul edx
    add eax, PAGE_TABLES_PHYS

    mov [eax], ebx
    add eax, 768 * 1024 * 4
    mov [eax], ebx

    test ecx, ecx
    jnz tables

    ; how many pages tables to allocate
    mov eax, KERNEL_END_PHYS
    add eax, 4096 * 1024 - 1
    mov ecx, 4096 * 1024
    mov edx, 0
    div ecx
    mov ecx, eax

directory:
    dec ecx

    ; identity entry
    ; ebx - the entry's value
    mov ebx, ecx
    shl ebx, 12
    add ebx, PAGE_TABLES_PHYS
    or ebx, 0b011

    ; eax - the entry's address
    mov eax, ecx
    mov edx, 4
    mul edx
    add eax, PAGE_DIRECTORY_PHYS

    mov [eax], ebx

    ; higher half entry
    add eax, 768 * 4
    add ebx, 768 * 1024 * 4
    mov [eax], ebx

    test ecx, ecx
    jnz directory

    ; set the CR3 register to point at the page directory
    mov eax, PAGE_DIRECTORY_PHYS
    mov cr3, eax

    ; set the CR0.PG (paging), CR0.WP (write protected) and CR0.PE (protected
    ; mode) bits
    mov eax, cr0
    or eax, 0x80010001
    mov cr0, eax

    jmp main_wrapper

SECTION .data.init progbits alloc noexec write align=4
GLOBAL GDTR_PHYS
GLOBAL GDT_PHYS

GDTR_PHYS:
    dw 0x0027
    dd 0
    dw 0

GDT_PHYS:
    dq 0x0000000000000000 ; null segment
    dq 0x00CF9A000000FFFF ; kernel code
    dq 0x00CF92000000FFFF ; kernel data
    dq 0x00CFFA000000FFFF ; user code
    dq 0x00CFF2000000FFFF ; user data
    dq 0x0000890000000000 ; TSS

SECTION .bss.init nobits alloc noexec write align=4
GLOBAL PAGE_DIRECTORY_PHYS
GLOBAL PAGE_TABLES_PHYS

ALIGN 4096
PAGE_DIRECTORY_PHYS:
    resd 1024
PAGE_TABLES_PHYS:
    resd 1023 * 1024 ; the last PDE is used for recursive paging

SECTION .text
EXTERN kernel_main
EXTERN GDTR
GLOBAL main_wrapper

ALIGN 16
main_wrapper: 
    mov esp, stack_top
    call kernel_main
    cli
halt:
    hlt
    jmp halt

load_gdtr:
    lgdt [GDTR]
    jmp 0x08:reload_segments ; far jump to the kernel code segment 0x08
reload_segments:
    ; set data registers to point at the kernel data segment 0x10
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    ret

SECTION .bss
; Provide a 16 KiB stack. It grows downwards on x86. It also must be 16-byte
; aligned according to SysV ABI standard.
ALIGN 16
stack_bottom:
    resb 16384
stack_top:
