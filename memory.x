MEMORY
{
  FLASH (rx) : ORIGIN = 0x08000000, LENGTH = 512K
  RAM (rwx) : ORIGIN = 0x20000000, LENGTH = 112K
  RAM2 (rwx) : ORIGIN = 0x2001C000, LENGTH = 16K
  BACKUP_RAM (rwx) : ORIGIN = 0x40024000, LENGTH = 4K
}

_stack_size = 0x1000;

SECTIONS {
  .stack (NOLOAD) : {
    . += _stack_size;
  } > RAM
}
