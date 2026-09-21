/*
Memory configuration for the Raspberry Pi 2

*/

MEMORY {
    SDRAM : ORIGIN = 0x8000, LENGTH = 128M
}

REGION_ALIAS("VECTORS", SDRAM);
REGION_ALIAS("CODE", SDRAM);
REGION_ALIAS("DATA", SDRAM);
REGION_ALIAS("STACKS", SDRAM);

PROVIDE(_hyp_stack_size = 16K);
PROVIDE(_und_stack_size = 16K);
PROVIDE(_svc_stack_size = 16K);
PROVIDE(_abt_stack_size = 16K);
PROVIDE(_irq_stack_size = 64);
PROVIDE(_fiq_stack_size = 64);
PROVIDE(_sys_stack_size = 16K);
