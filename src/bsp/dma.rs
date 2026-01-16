//! DMA模块
//! 提供直接内存访问功能封装

#![allow(unused)]

// 使用内部生成的设备驱动库
use library::*;

/// DMA通道枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DmaChannel {
    Channel1 = 0,
    Channel2 = 1,
    Channel3 = 2,
    Channel4 = 3,
    Channel5 = 4,
    Channel6 = 5,
    Channel7 = 6,
}

/// DMA数据流方向枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DmaDirection {
    PeripheralToMemory = 0,
    MemoryToPeripheral = 1,
    MemoryToMemory = 2,
}

/// DMA外设地址增量模式枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DmaPeripheralIncrementMode {
    Disabled = 0,
    Enabled = 1,
}

/// DMA内存地址增量模式枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DmaMemoryIncrementMode {
    Disabled = 0,
    Enabled = 1,
}

/// DMA外设数据宽度枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DmaPeripheralDataSize {
    Byte = 0,
    HalfWord = 1,
    Word = 2,
}

/// DMA内存数据宽度枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DmaMemoryDataSize {
    Byte = 0,
    HalfWord = 1,
    Word = 2,
}

/// DMA通道优先级枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DmaChannelPriority {
    Low = 0,
    Medium = 1,
    High = 2,
    VeryHigh = 3,
}

/// DMA循环模式枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DmaCircularMode {
    Disabled = 0,
    Enabled = 1,
}

/// DMA中断类型枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DmaInterrupt {
    TransferComplete = 1 << 1,
    HalfTransfer = 1 << 2,
    TransferError = 1 << 3,
}

/// DMA结构体
pub struct Dma {
    dma_number: u8,
    channel: DmaChannel,
}

impl Dma {
    /// 创建新的DMA实例
    pub const fn new(dma_number: u8, channel: DmaChannel) -> Self {
        Self {
            dma_number,
            channel,
        }
    }
    
    /// 获取DMA寄存器块
    unsafe fn get_dma(&self) -> &'static mut library::dma1::RegisterBlock {
        match self.dma_number {
            1 => &mut *(0x40020000 as *mut library::dma1::RegisterBlock),
            2 => &mut *(0x40020400 as *mut library::dma1::RegisterBlock),
            _ => &mut *(0x40020000 as *mut library::dma1::RegisterBlock),
        }
    }
    
        /// 初始化DMA通道
    pub unsafe fn init(
        &self,
        direction: DmaDirection,
        peripheral_increment: DmaPeripheralIncrementMode,
        memory_increment: DmaMemoryIncrementMode,
        peripheral_data_size: DmaPeripheralDataSize,
        memory_data_size: DmaMemoryDataSize,
        priority: DmaChannelPriority,
        circular_mode: DmaCircularMode,
    ) {
        // 由于内部库中DMA寄存器结构不同，暂时为空实现
    }
    
    /// 配置DMA传输
    pub unsafe fn configure_transfer(&self, _peripheral_addr: u32, _memory_addr: u32, _data_count: u16) {
        // 由于内部库中DMA寄存器结构不同，暂时为空实现
    }
    
    /// 启用DMA通道
    pub unsafe fn enable(&self) {
        // 由于内部库中DMA寄存器结构不同，暂时为空实现
    }
    
    /// 禁用DMA通道
    pub unsafe fn disable(&self) {
        // 由于内部库中DMA寄存器结构不同，暂时为空实现
    }
    
    /// 启用中断
    pub unsafe fn enable_interrupt(&self, _interrupt: DmaInterrupt) {
        // 由于内部库中DMA寄存器结构不同，暂时为空实现
    }
    
    /// 禁用中断
    pub unsafe fn disable_interrupt(&self, _interrupt: DmaInterrupt) {
        // 由于内部库中DMA寄存器结构不同，暂时为空实现
    }
    
    /// 检查中断标志
    pub unsafe fn check_interrupt(&self, interrupt: DmaInterrupt) -> bool {
        let dma = self.get_dma();
        let isr = dma.isr().read().bits();
        let channel_offset = self.channel as u32 * 4;
        (isr & (interrupt as u32) << channel_offset) != 0
    }
    
    /// 清除中断标志
    pub unsafe fn clear_interrupt(&self, interrupt: DmaInterrupt) {
        let dma = self.get_dma();
        let channel_offset = self.channel as u32 * 4;
        dma.ifcr().write(|w: &mut library::dma1::ifcr::W| unsafe { w.bits((interrupt as u32) << channel_offset) });
    }
    
    /// 获取剩余数据计数
    pub unsafe fn get_remaining_count(&self) -> u16 {
        // 由于内部库中DMA寄存器结构不同，暂时返回固定值
        0
    }
    
    /// 检查DMA通道是否正在传输
    pub unsafe fn is_transferring(&self) -> bool {
        // 由于内部库中DMA寄存器结构不同，暂时返回固定值
        false
    }
}

/// 预定义的DMA实例
pub const DMA1_CHANNEL1: Dma = Dma::new(1, DmaChannel::Channel1);
pub const DMA1_CHANNEL2: Dma = Dma::new(1, DmaChannel::Channel2);
pub const DMA1_CHANNEL3: Dma = Dma::new(1, DmaChannel::Channel3);
pub const DMA1_CHANNEL4: Dma = Dma::new(1, DmaChannel::Channel4);
pub const DMA1_CHANNEL5: Dma = Dma::new(1, DmaChannel::Channel5);
pub const DMA1_CHANNEL6: Dma = Dma::new(1, DmaChannel::Channel6);
pub const DMA1_CHANNEL7: Dma = Dma::new(1, DmaChannel::Channel7);

pub const DMA2_CHANNEL1: Dma = Dma::new(2, DmaChannel::Channel1);
pub const DMA2_CHANNEL2: Dma = Dma::new(2, DmaChannel::Channel2);
pub const DMA2_CHANNEL3: Dma = Dma::new(2, DmaChannel::Channel3);
pub const DMA2_CHANNEL4: Dma = Dma::new(2, DmaChannel::Channel4);
pub const DMA2_CHANNEL5: Dma = Dma::new(2, DmaChannel::Channel5);
