/* Linker script for the STM32F429ZIT6 */
MEMORY
{
    /* NOTE 1 K = 1 KiBi = 1024 bytes */
    FLASH (rx)      : ORIGIN = 0x8000000, LENGTH = 64K
    RAM (xrw)       : ORIGIN = 0x20000000, LENGTH = 64K
    CCMRAM (xrw)    : ORIGIN = 0x10000000, LENGTH = 64K
    BKPSRAM (rw)    : ORIGIN = 0x40024000, LENGTH = 4K
}

/* This is where the call stack will be allocated. */
/* The stack is of the full descending type. */
/* NOTE Do NOT modify `_stack_start` unless you know what you are doing */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);