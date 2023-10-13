MBALIGN equ 1 << 0 ; align modules on page boundaries (4K)
MEMINFO equ 1 << 1 ; provide memory map or smth
FLAGS   equ MBALIGN | MEMINFO ; Multiboot flags
MAGIC   equ 0x1BADB002
CHEKSUM equ -(MAGIC + FLAGS)

SECTION .text.init progbits alloc exec nowrite align=16
GLOBAL _start
GLOBAL load_gdtr
EXTERN KERNEL_START_PHYS
EXTERN KERNEL_INIT_ALLOC

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


    ; ; TODO shr INSTEAD OF div
    ; ; which lower half pages to use for the stack
    ; mov eax, STACK_TOP_PHYS
    ; mov ecx, 4096
    ; mov edx, 0
    ; div ecx
    ; mov edx, 4
    ; mul edx
    ; add eax, PAGE_TABLES_PHYS
    ; mov [FIRST_STACK_PAGE_LOW], eax

    ; ; which higher half pages to use for the stack
    ; mov eax, STACK_SIZE
    ; add eax, 4095
    ; mov ecx, 4096
    ; mov edx, 0
    ; div ecx
    ; mov ebx, 1023 * 1024
    ; sub ebx, eax
    ; mov eax, ebx
    ; mov ebx, 4
    ; mul ebx
    ; add eax, PAGE_TABLES_PHYS
    ; ; mov eax, STACK_TOP
    ; ; mov ecx, 4096
    ; ; mov edx, 0
    ; ; div ecx
    ; ; mov edx, 4
    ; ; mul edx
    ; ; add eax, PAGE_TABLES_PHYS
    ; mov [FIRST_STACK_PAGE_HIGH], eax


    ; ; which lower half page tables to use for the stack
    ; mov eax, STACK_TOP_PHYS
    ; mov ecx, 1024 * 4096
    ; mov edx, 0
    ; div ecx
    ; mov edx, 4
    ; mul edx
    ; add eax, PAGE_DIRECTORY_PHYS
    ; mov [FIRST_STACK_PAGE_TABLE_LOW], eax

    ; ; which higher half page tables to use for the stack
    ; mov eax, STACK_SIZE
    ; add eax, 1024 * 4096 - 1
    ; mov ecx, 1024 * 4096
    ; mov edx, 0
    ; div ecx
    ; mov ebx, 1023
    ; sub ebx, eax
    ; mov eax, ebx
    ; mov ebx, 4
    ; mul ebx
    ; add eax, PAGE_DIRECTORY_PHYS
    ; ; mov eax, STACK_TOP
    ; ; add eax, 1024 * 4096 - 1
    ; ; mov ecx, 1024 * 4096
    ; ; mov edx, 0
    ; ; div ecx
    ; ; mov ebx, 4
    ; ; mul ebx
    ; ; add eax, PAGE_DIRECTORY_PHYS
    ; mov [FIRST_STACK_PAGE_TABLE_HIGH], eax


    ; how many pages to allocate at 0xC0000000, including 0x0000-0xFFFF
    mov eax, KERNEL_INIT_ALLOC
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


    ; how many page tables to allocate
    mov eax, KERNEL_INIT_ALLOC
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


    ; how many pages to allocate for the stack
    mov eax, STACK_SIZE
    add eax, 4095
    mov ecx, 4096
    mov edx, 0
    div ecx
    mov ecx, eax

tables_stack:
    dec ecx

    ; ebx - the entry's value
    mov ebx, ecx
    shl ebx, 12
    add ebx, STACK_TOP_PHYS
    or ebx, 0b011

    ; eax - the entry's address
    mov eax, ecx
    mov edx, 4
    mul edx

    ; add eax, STACK_TOP_PHYS_PAGE
    ; mov [eax], ebx
    ; sub eax, STACK_TOP_PHYS_PAGE
    add eax, STACK_TOP_PAGE
    mov [eax], ebx

    test ecx, ecx
    jnz tables_stack


    ; how many page tables to allocate for the stack
    mov eax, STACK_SIZE
    add eax, 4096 * 1024 - 1
    mov ecx, 4096 * 1024
    mov edx, 0
    div ecx
    mov ecx, eax

directory_stack:
    dec ecx

    ; identity entry
    ; ebx - the entry's value
    mov ebx, ecx
    shl ebx, 12
    ; add ebx, stack_top
    or ebx, 0b011

    ; eax - the entry's address
    mov eax, ecx
    mov edx, 4
    mul edx

    ; lower half entry
    ; add ebx, STACK_TOP_PHYS_PAGE
    ; add eax, STACK_TOP_PHYS_PAGE_TABLE
    ; mov [eax], ebx
    ; sub ebx, STACK_TOP_PHYS_PAGE
    ; sub eax, STACK_TOP_PHYS_PAGE_TABLE

    ; higher half entry
    add ebx, STACK_TOP_PAGE
    add eax, STACK_TOP_PAGE_TABLE
    mov [eax], ebx

    test ecx, ecx
    jnz directory_stack


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
    mov ebp, STACK_BOTTOM
    mov esp, ebp
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

SECTION .stack nobits alloc noexec write align=4096
GLOBAL STACK_TOP_PHYS
GLOBAL STACK_BOTTOM_PHYS
EXTERN STACK_TOP
EXTERN STACK_BOTTOM
EXTERN STACK_TOP_PHYS_PAGE
EXTERN STACK_TOP_PHYS_PAGE_TABLE
EXTERN STACK_TOP_PAGE
EXTERN STACK_TOP_PAGE_TABLE
STACK_SIZE equ 16384
ALIGN 4096
STACK_TOP_PHYS:
    resb STACK_SIZE
STACK_BOTTOM_PHYS:
