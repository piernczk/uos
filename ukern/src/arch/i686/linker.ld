ENTRY(_start)
OUTPUT_FORMAT(elf32-i386)

SECTIONS
{
    . = 0x100000;
    KERNEL_START_PHYS = .;
    KERNEL_START = KERNEL_START_PHYS + 0xC0000000;

    .text.init :
    {
        *(.text.init)
    }

    .data.init :
    {
        *(.data.init)
    }

    .bss.init :
    {
        *(.bss.init)
    }

    . += 0xC0000000;
    PAGE_DIRECTORY = PAGE_DIRECTORY_PHYS + 0xC0000000;
    PAGE_TABLES = PAGE_TABLES_PHYS + 0xC0000000;
    GDT = GDT_PHYS + 0xC0000000;
    GDTR = GDTR_PHYS + 0xC0000000;

    .text : AT(ADDR(.text) - 0xC0000000) ALIGN(4K)
    {
        *(.text)
        *(.text.*)
    }

    .rodata : AT(ADDR(.rodata) - 0xC0000000) ALIGN(4K)
    {
        *(.rodata)
        *(.rodata.*)
    }

    .got : AT(ADDR(.got) - 0xC0000000)
    {
        *(.got)
    }

    .got.plt : AT(ADDR(.got.plt) - 0xC0000000)
    {
        *(.got.plt)
    }

    .data : AT(ADDR(.data) - 0xC0000000) ALIGN(4K)
    {
        *(.data)
        *(.data.*)
    }

    .bss : AT(ADDR(.bss) - 0xC0000000) ALIGN(4K)
    {
        *(COMMON)
        *(.bss)
        *(.bss.*)
    }

    .gcc_except_table : AT(ADDR(.gcc_except_table) - 0xC0000000) ALIGN(4K)
    {
        *(.gcc_except_table.*)
    }

    KERNEL_END = .;
    KERNEL_END_PHYS = KERNEL_END - 0xC0000000;
}
