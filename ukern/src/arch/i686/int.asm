SECTION .text
EXTERN irq_handler
EXTERN exception_handler

irq_wrapper:
    ; save the CPU state
    pushad
    mov ax, ds
    push eax

    mov ax, 0x10 ; use the kernel data segment
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    call irq_handler

    pop eax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    popad
    add esp, 0x08 ; pop intno and errno

    sti
    iretd

exception_wrapper:
    ; save the CPU state
    pushad
    mov ax, ds
    push eax

    mov ax, 0x10 ; use the kernel data segment
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    call exception_handler

    pop eax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    popad
    add esp, 0x08 ; pop intno and errno

    sti
    iretd

; generate IRQ0..IRQ15
%assign i 0
%rep 16
    GLOBAL irq%+ i
irq%+ i:
    cli
    push DWORD 0
    push DWORD (%+ i + 32)
    jmp irq_wrapper
    %assign i i + 1
%endrep

%macro EXCEPTION_NOERR 1
    GLOBAL exception%1
exception%1:
    cli
    push DWORD 0
    push DWORD (%1)
    jmp exception_wrapper
%endmacro

%macro EXCEPTION_ERR 1
    GLOBAL exception%1
exception%1:
    cli
    push DWORD (%1)
    jmp exception_wrapper
%endmacro

EXCEPTION_NOERR 0
EXCEPTION_NOERR 1
EXCEPTION_NOERR 2
EXCEPTION_NOERR 3
EXCEPTION_NOERR 4
EXCEPTION_NOERR 5
EXCEPTION_NOERR 6
EXCEPTION_NOERR 7
EXCEPTION_ERR 8
EXCEPTION_NOERR 9
EXCEPTION_ERR 10
EXCEPTION_ERR 11
EXCEPTION_ERR 12
EXCEPTION_ERR 13
EXCEPTION_ERR 14
EXCEPTION_NOERR 15
EXCEPTION_NOERR 16
EXCEPTION_ERR 17
EXCEPTION_NOERR 18
EXCEPTION_NOERR 19
EXCEPTION_NOERR 20
EXCEPTION_ERR 21
EXCEPTION_NOERR 22
EXCEPTION_NOERR 23
EXCEPTION_NOERR 24
EXCEPTION_NOERR 25
EXCEPTION_NOERR 26
EXCEPTION_NOERR 27
EXCEPTION_NOERR 28
EXCEPTION_ERR 29
EXCEPTION_ERR 30
EXCEPTION_NOERR 31

GLOBAL IRQS
IRQS:
    %assign i 0
    %rep 16
        dd irq%+ i
        %assign i i + 1
    %endrep

GLOBAL EXCEPTIONS
EXCEPTIONS:
    %assign i 0
    %rep 32
        dd exception%+ i
        %assign i i + 1
    %endrep
