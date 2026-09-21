/*
Memory configuration for the Raspberry Pi (works on any version)
*/

MEMORY {
    SDRAM : ORIGIN = 0x8000, LENGTH = 192M
}

REGION_ALIAS("VECTORS", SDRAM);
REGION_ALIAS("CODE", SDRAM);
REGION_ALIAS("DATA", SDRAM);
REGION_ALIAS("STACKS", SDRAM);

PROVIDE(_hyp_stack_size = 64);
PROVIDE(_und_stack_size = 64);
PROVIDE(_svc_stack_size = 64);
PROVIDE(_abt_stack_size = 64);
PROVIDE(_irq_stack_size = 64);
PROVIDE(_fiq_stack_size = 64);
PROVIDE(_sys_stack_size = 256K);
