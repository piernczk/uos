ENTRY(_start)
OUTPUT_FORMAT(elf32-i386)

SECTIONS
{
    .text : ALIGN(4)
    {
        *(.text)
        *(.text.*)
    }

    .rodata :
    {
        *(.rodata)
        *(.rodata.*)
    }

    .data :
    {
        *(.data)
    }

    .bss :
    {
        *(COMMON)
        *(.bss)
    }

    .gcc_except_table :
    {
        *(.gcc_except_table.*)
    }
}

