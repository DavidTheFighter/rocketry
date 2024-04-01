/* Linker script for the STM32F429ZIT6 */
MEMORY
{
    /* NOTE 1 K = 1 KiBi = 1024 bytes */
    RAM (xrw)       : ORIGIN = 0x20000000,  LENGTH = 128K
    CCMRAM (xrw)    : ORIGIN = 0x10000000,  LENGTH = 64K
    FLASH (rx)      : ORIGIN = 0x08010000,   LENGTH = 448K
}

/* This is where the call stack will be allocated. */
/* The stack is of the full descending type. */
/* NOTE Do NOT modify `_stack_start` unless you know what you are doing */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);