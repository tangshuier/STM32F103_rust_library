/* Linker script for STM32F103C8T6 (64KB flash, 20KB RAM) */
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 64K
  RAM : ORIGIN = 0x20000000, LENGTH = 20K
}

/* Linker script for the STM32F103C8T6 */
SECTIONS
{
  /* The vector table is placed at the beginning of FLASH */
  .vector_table :
  {
    . = ALIGN(4);
    /* Initial stack pointer */
    LONG(0x20005000)
    /* Reset handler */
    LONG(Reset)
    /* NMI handler */
    LONG(DefaultHandler)
    /* Hard fault handler */
    LONG(DefaultHandler)
    /* MPU fault handler */
    LONG(DefaultHandler)
    /* Bus fault handler */
    LONG(DefaultHandler)
    /* Usage fault handler */
    LONG(DefaultHandler)
    /* Reserved */
    LONG(0)
    /* Reserved */
    LONG(0)
    /* Reserved */
    LONG(0)
    /* Reserved */
    LONG(0)
    /* SVCall handler */
    LONG(DefaultHandler)
    /* Debug monitor handler */
    LONG(DefaultHandler)
    /* Reserved */
    LONG(0)
    /* PendSV handler */
    LONG(DefaultHandler)
    /* SysTick handler */
    LONG(DefaultHandler)

    /* STM32F103 specific interrupts */
    LONG(DefaultHandler) /* 0: WWDG Window Watchdog */
    LONG(DefaultHandler) /* 1: PVD through EXTI Line detect */
    LONG(DefaultHandler) /* 2: Tamper */
    LONG(DefaultHandler) /* 3: RTC */
    LONG(DefaultHandler) /* 4: FLASH */
    LONG(DefaultHandler) /* 5: RCC */
    LONG(DefaultHandler) /* 6: EXTI Line 0 */
    LONG(DefaultHandler) /* 7: EXTI Line 1 */
    LONG(DefaultHandler) /* 8: EXTI Line 2 */
    LONG(DefaultHandler) /* 9: EXTI Line 3 */
    LONG(DefaultHandler) /* 10: EXTI Line 4 */
    LONG(DefaultHandler) /* 11: DMA1 Channel 1 */
    LONG(DefaultHandler) /* 12: DMA1 Channel 2 */
    LONG(DefaultHandler) /* 13: DMA1 Channel 3 */
    LONG(DefaultHandler) /* 14: DMA1 Channel 4 */
    LONG(DefaultHandler) /* 15: DMA1 Channel 5 */
    LONG(DefaultHandler) /* 16: DMA1 Channel 6 */
    LONG(DefaultHandler) /* 17: DMA1 Channel 7 */
    LONG(DefaultHandler) /* 18: ADC1 & ADC2 */
    LONG(DefaultHandler) /* 19: CAN1 TX */
    LONG(DefaultHandler) /* 20: CAN1 RX0 */
    LONG(DefaultHandler) /* 21: CAN1 RX1 */
    LONG(DefaultHandler) /* 22: CAN1 SCE */
    LONG(DefaultHandler) /* 23: EXTI Lines 9..5 */
    LONG(DefaultHandler) /* 24: TIM1 Break */
    LONG(DefaultHandler) /* 25: TIM1 Update */
    LONG(DefaultHandler) /* 26: TIM1 Trigger and Commutation */
    LONG(DefaultHandler) /* 27: TIM1 Capture Compare */
    LONG(DefaultHandler) /* 28: TIM2 */
    LONG(DefaultHandler) /* 29: TIM3 */
    LONG(DefaultHandler) /* 30: TIM4 */
    LONG(DefaultHandler) /* 31: I2C1 Event */
    LONG(DefaultHandler) /* 32: I2C1 Error */
    LONG(DefaultHandler) /* 33: I2C2 Event */
    LONG(DefaultHandler) /* 34: I2C2 Error */
    LONG(DefaultHandler) /* 35: SPI1 */
    LONG(DefaultHandler) /* 36: SPI2 */
    LONG(DefaultHandler) /* 37: USART1 */
    LONG(DefaultHandler) /* 38: USART2 */
    LONG(USART3) /* 39: USART3 */
    LONG(DefaultHandler) /* 40: EXTI Lines 15..10 */
    LONG(DefaultHandler) /* 41: RTC Alarm through EXTI Line */
    LONG(DefaultHandler) /* 42: USB Wakeup from suspend */
    . = ALIGN(4);
  } > FLASH

  /* The program code and other data goes into FLASH */
  .text :
  {
    . = ALIGN(4);
    *(.text)           /* .text sections (code) */
    *(.text*)          /* .text* sections (code) */
    *(.rodata)         /* .rodata sections (constants, strings, etc.) */
    *(.rodata*)        /* .rodata* sections (constants, strings, etc.) */
    *(.eh_frame)       /* .eh_frame sections (exception handling) */
    
    KEEP(*(.init))
    KEEP(*(.fini))

    . = ALIGN(4);
    _etext = .;        /* Define a symbol for the end of text section */
  } > FLASH

  /* used by the startup to initialize data */
  _sidata = LOADADDR(.data);

  /* Initialized data sections goes into RAM, load LMA copy after code */
  .data :
  {
    . = ALIGN(4);
    _sdata = .;        /* Define a symbol for the start of the data section */
    *(.data)           /* .data sections */
    *(.data*)          /* .data* sections */

    . = ALIGN(4);
    _edata = .;        /* Define a symbol for the end of the data section */
  } > RAM AT> FLASH

  /* Uninitialized data section */
  .bss :
  {
    . = ALIGN(4);
    _sbss = .;         /* Define a symbol for the start of the BSS section */
    *(.bss)            /* .bss sections */
    *(.bss*)           /* .bss* sections */
    *(COMMON)          /* COMMON sections */

    . = ALIGN(4);
    _ebss = .;         /* Define a symbol for the end of the BSS section */
  } > RAM

  /* User_heap_stack section, used to check that there is enough RAM left */
  ._user_heap_stack :
  {
    . = ALIGN(8);
    PROVIDE ( end = . );
    PROVIDE ( _end = . );
    . = . + 1K; /* Reserve 1KB for heap */
    . = . + 1K; /* Reserve 1KB for stack */
    . = ALIGN(8);
  } > RAM
}