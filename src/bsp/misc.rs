#![allow(unused)]

const SCB_BASE: u32 = 0xE000ED00;
const NVIC_BASE: u32 = 0xE000E100;
const SYSTICK_BASE: u32 = 0xE000E010;

const SCB_AIRCR: *mut u32 = (SCB_BASE + 0x0C) as *mut u32;
const SCB_VTOR: *mut u32 = (SCB_BASE + 0x08) as *mut u32;
const SCB_SCR: *mut u32 = (SCB_BASE + 0x04) as *mut u32;

const NVIC_ISER: *mut u32 = NVIC_BASE as *mut u32;
const NVIC_ICER: *mut u32 = (NVIC_BASE + 0x080) as *mut u32;
const NVIC_IP: *mut u32 = (NVIC_BASE + 0x300) as *mut u32;

const SYSTICK_CTRL: *mut u32 = SYSTICK_BASE as *mut u32;

const AIRCR_VECTKEY_MASK: u32 = 0x05FA0000;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum NvicPriorityGroup {
    Group0 = 0x700,
    Group1 = 0x600,
    Group2 = 0x500,
    Group3 = 0x400,
    Group4 = 0x300,
}

#[derive(Debug, Clone, Copy)]
pub struct NvicInitStruct {
    pub irq_channel: u8,
    pub preemption_priority: u8,
    pub sub_priority: u8,
    pub enable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum LowPowerMode {
    SleepOnExit = 0x02,
    SleepDeep = 0x04,
    SevOnPend = 0x10,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum SysTickClkSource {
    HclkDiv8 = 0xFFFFFFFB,
    Hclk = 0x00000004,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum NvicVectTab {
    Ram = 0x20000000,
    Flash = 0x08000000,
}

pub struct Misc;

impl Misc {
    pub const fn new() -> Self {
        Self
    }

    pub unsafe fn nvic_priority_group_config(&self, priority_group: NvicPriorityGroup) {
        *SCB_AIRCR = AIRCR_VECTKEY_MASK | (priority_group as u32);
    }

    pub unsafe fn nvic_init(&self, init_struct: NvicInitStruct) {
        if init_struct.enable {
            let priority_group = ((*SCB_AIRCR) & 0x700) >> 8;
            let pre_bits = 4 - priority_group;
            let sub_bits = priority_group;
            
            let mut priority = (init_struct.preemption_priority as u32) << sub_bits;
            priority |= (init_struct.sub_priority as u32) & ((1 << sub_bits) - 1);
            priority <<= 4;
            
            let ip_index = init_struct.irq_channel as usize;
            let ip_register = NVIC_IP.add(ip_index / 4);
            let shift = (ip_index % 4) * 8 + 4;
            *ip_register &= !(0xFF << shift);
            *ip_register |= priority << shift;
            
            let iser_index = init_struct.irq_channel as usize / 32;
            let iser_bit = init_struct.irq_channel % 32;
            let iser_register = NVIC_ISER.add(iser_index);
            *iser_register |= 1 << iser_bit;
        } else {
            let icer_index = init_struct.irq_channel as usize / 32;
            let icer_bit = init_struct.irq_channel % 32;
            let icer_register = NVIC_ICER.add(icer_index);
            *icer_register |= 1 << icer_bit;
        }
    }

    pub unsafe fn nvic_set_vector_table(&self, vect_tab: NvicVectTab, offset: u32) {
        *SCB_VTOR = (vect_tab as u32) | (offset & 0x1FFFFF80);
    }

    pub unsafe fn nvic_system_lp_config(&self, low_power_mode: LowPowerMode, new_state: bool) {
        if new_state {
            *SCB_SCR |= low_power_mode as u32;
        } else {
            *SCB_SCR &= !(low_power_mode as u32);
        }
    }

    pub unsafe fn systick_clk_source_config(&self, clk_source: SysTickClkSource) {
        if clk_source == SysTickClkSource::Hclk {
            *SYSTICK_CTRL |= SysTickClkSource::Hclk as u32;
        } else {
            *SYSTICK_CTRL &= SysTickClkSource::HclkDiv8 as u32;
        }
    }
}

pub const MISC: Misc = Misc::new();