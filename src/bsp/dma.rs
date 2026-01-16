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
        let dma = self.get_dma();
        
        // 根据通道选择对应的CCR寄存器
        match self.channel {
            DmaChannel::Channel1 => {
                dma.ccr1().write(|w| {
                    // 配置数据传输方向
                    match direction {
                        DmaDirection::PeripheralToMemory => w.dir().clear_bit(),
                        DmaDirection::MemoryToPeripheral => w.dir().set_bit(),
                        DmaDirection::MemoryToMemory => {
                            w.dir().set_bit();
                            w.mem2mem().set_bit()
                        },
                    };
                    
                    // 配置外设地址增量模式
                    match peripheral_increment {
                        DmaPeripheralIncrementMode::Disabled => w.pinc().clear_bit(),
                        DmaPeripheralIncrementMode::Enabled => w.pinc().set_bit(),
                    };
                    
                    // 配置内存地址增量模式
                    match memory_increment {
                        DmaMemoryIncrementMode::Disabled => w.minc().clear_bit(),
                        DmaMemoryIncrementMode::Enabled => w.minc().set_bit(),
                    };
                    
                    // 配置外设数据宽度
                    w.psize().bits(peripheral_data_size as u8);
                    
                    // 配置内存数据宽度
                    w.msize().bits(memory_data_size as u8);
                    
                    // 配置通道优先级
                    w.pl().bits(priority as u8);
                    
                    // 配置循环模式
                    match circular_mode {
                        DmaCircularMode::Disabled => w.circ().clear_bit(),
                        DmaCircularMode::Enabled => w.circ().set_bit(),
                    }
                });
            },
            DmaChannel::Channel2 => {
                dma.ccr2().write(|w| {
                    // 配置数据传输方向
                    match direction {
                        DmaDirection::PeripheralToMemory => w.dir().clear_bit(),
                        DmaDirection::MemoryToPeripheral => w.dir().set_bit(),
                        DmaDirection::MemoryToMemory => {
                            w.dir().set_bit();
                            w.mem2mem().set_bit()
                        },
                    };
                    
                    // 配置外设地址增量模式
                    match peripheral_increment {
                        DmaPeripheralIncrementMode::Disabled => w.pinc().clear_bit(),
                        DmaPeripheralIncrementMode::Enabled => w.pinc().set_bit(),
                    };
                    
                    // 配置内存地址增量模式
                    match memory_increment {
                        DmaMemoryIncrementMode::Disabled => w.minc().clear_bit(),
                        DmaMemoryIncrementMode::Enabled => w.minc().set_bit(),
                    };
                    
                    // 配置外设数据宽度
                    w.psize().bits(peripheral_data_size as u8);
                    
                    // 配置内存数据宽度
                    w.msize().bits(memory_data_size as u8);
                    
                    // 配置通道优先级
                    w.pl().bits(priority as u8);
                    
                    // 配置循环模式
                    match circular_mode {
                        DmaCircularMode::Disabled => w.circ().clear_bit(),
                        DmaCircularMode::Enabled => w.circ().set_bit(),
                    }
                });
            },
            DmaChannel::Channel3 => {
                dma.ccr3().write(|w| {
                    // 配置数据传输方向
                    match direction {
                        DmaDirection::PeripheralToMemory => w.dir().clear_bit(),
                        DmaDirection::MemoryToPeripheral => w.dir().set_bit(),
                        DmaDirection::MemoryToMemory => {
                            w.dir().set_bit();
                            w.mem2mem().set_bit()
                        },
                    };
                    
                    // 配置外设地址增量模式
                    match peripheral_increment {
                        DmaPeripheralIncrementMode::Disabled => w.pinc().clear_bit(),
                        DmaPeripheralIncrementMode::Enabled => w.pinc().set_bit(),
                    };
                    
                    // 配置内存地址增量模式
                    match memory_increment {
                        DmaMemoryIncrementMode::Disabled => w.minc().clear_bit(),
                        DmaMemoryIncrementMode::Enabled => w.minc().set_bit(),
                    };
                    
                    // 配置外设数据宽度
                    w.psize().bits(peripheral_data_size as u8);
                    
                    // 配置内存数据宽度
                    w.msize().bits(memory_data_size as u8);
                    
                    // 配置通道优先级
                    w.pl().bits(priority as u8);
                    
                    // 配置循环模式
                    match circular_mode {
                        DmaCircularMode::Disabled => w.circ().clear_bit(),
                        DmaCircularMode::Enabled => w.circ().set_bit(),
                    }
                });
            },
            DmaChannel::Channel4 => {
                dma.ccr4().write(|w| {
                    // 配置数据传输方向
                    match direction {
                        DmaDirection::PeripheralToMemory => w.dir().clear_bit(),
                        DmaDirection::MemoryToPeripheral => w.dir().set_bit(),
                        DmaDirection::MemoryToMemory => {
                            w.dir().set_bit();
                            w.mem2mem().set_bit()
                        },
                    };
                    
                    // 配置外设地址增量模式
                    match peripheral_increment {
                        DmaPeripheralIncrementMode::Disabled => w.pinc().clear_bit(),
                        DmaPeripheralIncrementMode::Enabled => w.pinc().set_bit(),
                    };
                    
                    // 配置内存地址增量模式
                    match memory_increment {
                        DmaMemoryIncrementMode::Disabled => w.minc().clear_bit(),
                        DmaMemoryIncrementMode::Enabled => w.minc().set_bit(),
                    };
                    
                    // 配置外设数据宽度
                    w.psize().bits(peripheral_data_size as u8);
                    
                    // 配置内存数据宽度
                    w.msize().bits(memory_data_size as u8);
                    
                    // 配置通道优先级
                    w.pl().bits(priority as u8);
                    
                    // 配置循环模式
                    match circular_mode {
                        DmaCircularMode::Disabled => w.circ().clear_bit(),
                        DmaCircularMode::Enabled => w.circ().set_bit(),
                    }
                });
            },
            DmaChannel::Channel5 => {
                dma.ccr5().write(|w| {
                    // 配置数据传输方向
                    match direction {
                        DmaDirection::PeripheralToMemory => w.dir().clear_bit(),
                        DmaDirection::MemoryToPeripheral => w.dir().set_bit(),
                        DmaDirection::MemoryToMemory => {
                            w.dir().set_bit();
                            w.mem2mem().set_bit()
                        },
                    };
                    
                    // 配置外设地址增量模式
                    match peripheral_increment {
                        DmaPeripheralIncrementMode::Disabled => w.pinc().clear_bit(),
                        DmaPeripheralIncrementMode::Enabled => w.pinc().set_bit(),
                    };
                    
                    // 配置内存地址增量模式
                    match memory_increment {
                        DmaMemoryIncrementMode::Disabled => w.minc().clear_bit(),
                        DmaMemoryIncrementMode::Enabled => w.minc().set_bit(),
                    };
                    
                    // 配置外设数据宽度
                    w.psize().bits(peripheral_data_size as u8);
                    
                    // 配置内存数据宽度
                    w.msize().bits(memory_data_size as u8);
                    
                    // 配置通道优先级
                    w.pl().bits(priority as u8);
                    
                    // 配置循环模式
                    match circular_mode {
                        DmaCircularMode::Disabled => w.circ().clear_bit(),
                        DmaCircularMode::Enabled => w.circ().set_bit(),
                    }
                });
            },
            DmaChannel::Channel6 => {
                dma.ccr6().write(|w| {
                    // 配置数据传输方向
                    match direction {
                        DmaDirection::PeripheralToMemory => w.dir().clear_bit(),
                        DmaDirection::MemoryToPeripheral => w.dir().set_bit(),
                        DmaDirection::MemoryToMemory => {
                            w.dir().set_bit();
                            w.mem2mem().set_bit()
                        },
                    };
                    
                    // 配置外设地址增量模式
                    match peripheral_increment {
                        DmaPeripheralIncrementMode::Disabled => w.pinc().clear_bit(),
                        DmaPeripheralIncrementMode::Enabled => w.pinc().set_bit(),
                    };
                    
                    // 配置内存地址增量模式
                    match memory_increment {
                        DmaMemoryIncrementMode::Disabled => w.minc().clear_bit(),
                        DmaMemoryIncrementMode::Enabled => w.minc().set_bit(),
                    };
                    
                    // 配置外设数据宽度
                    w.psize().bits(peripheral_data_size as u8);
                    
                    // 配置内存数据宽度
                    w.msize().bits(memory_data_size as u8);
                    
                    // 配置通道优先级
                    w.pl().bits(priority as u8);
                    
                    // 配置循环模式
                    match circular_mode {
                        DmaCircularMode::Disabled => w.circ().clear_bit(),
                        DmaCircularMode::Enabled => w.circ().set_bit(),
                    }
                });
            },
            DmaChannel::Channel7 => {
                dma.ccr7().write(|w| {
                    // 配置数据传输方向
                    match direction {
                        DmaDirection::PeripheralToMemory => w.dir().clear_bit(),
                        DmaDirection::MemoryToPeripheral => w.dir().set_bit(),
                        DmaDirection::MemoryToMemory => {
                            w.dir().set_bit();
                            w.mem2mem().set_bit()
                        },
                    };
                    
                    // 配置外设地址增量模式
                    match peripheral_increment {
                        DmaPeripheralIncrementMode::Disabled => w.pinc().clear_bit(),
                        DmaPeripheralIncrementMode::Enabled => w.pinc().set_bit(),
                    };
                    
                    // 配置内存地址增量模式
                    match memory_increment {
                        DmaMemoryIncrementMode::Disabled => w.minc().clear_bit(),
                        DmaMemoryIncrementMode::Enabled => w.minc().set_bit(),
                    };
                    
                    // 配置外设数据宽度
                    w.psize().bits(peripheral_data_size as u8);
                    
                    // 配置内存数据宽度
                    w.msize().bits(memory_data_size as u8);
                    
                    // 配置通道优先级
                    w.pl().bits(priority as u8);
                    
                    // 配置循环模式
                    match circular_mode {
                        DmaCircularMode::Disabled => w.circ().clear_bit(),
                        DmaCircularMode::Enabled => w.circ().set_bit(),
                    }
                });
            },
        }
    }
    
    /// 配置DMA传输
    pub unsafe fn configure_transfer(&self, peripheral_addr: u32, memory_addr: u32, data_count: u16) {
        let dma = self.get_dma();
        
        // 根据通道配置相应的寄存器
        match self.channel {
            DmaChannel::Channel1 => {
                dma.cpar1().write(|w| unsafe { w.bits(peripheral_addr) });
                dma.cmar1().write(|w| unsafe { w.bits(memory_addr) });
                dma.cndtr1().write(|w| w.ndt().bits(data_count));
            },
            DmaChannel::Channel2 => {
                dma.cpar2().write(|w| unsafe { w.bits(peripheral_addr) });
                dma.cmar2().write(|w| unsafe { w.bits(memory_addr) });
                dma.cndtr2().write(|w| w.ndt().bits(data_count));
            },
            DmaChannel::Channel3 => {
                dma.cpar3().write(|w| unsafe { w.bits(peripheral_addr) });
                dma.cmar3().write(|w| unsafe { w.bits(memory_addr) });
                dma.cndtr3().write(|w| w.ndt().bits(data_count));
            },
            DmaChannel::Channel4 => {
                dma.cpar4().write(|w| unsafe { w.bits(peripheral_addr) });
                dma.cmar4().write(|w| unsafe { w.bits(memory_addr) });
                dma.cndtr4().write(|w| w.ndt().bits(data_count));
            },
            DmaChannel::Channel5 => {
                dma.cpar5().write(|w| unsafe { w.bits(peripheral_addr) });
                dma.cmar5().write(|w| unsafe { w.bits(memory_addr) });
                dma.cndtr5().write(|w| w.ndt().bits(data_count));
            },
            DmaChannel::Channel6 => {
                dma.cpar6().write(|w| unsafe { w.bits(peripheral_addr) });
                dma.cmar6().write(|w| unsafe { w.bits(memory_addr) });
                dma.cndtr6().write(|w| w.ndt().bits(data_count));
            },
            DmaChannel::Channel7 => {
                dma.cpar7().write(|w| unsafe { w.bits(peripheral_addr) });
                dma.cmar7().write(|w| unsafe { w.bits(memory_addr) });
                dma.cndtr7().write(|w| w.ndt().bits(data_count));
            },
        }
    }
    
    /// 启用DMA通道
    pub unsafe fn enable(&self) {
        let dma = self.get_dma();
        
        match self.channel {
            DmaChannel::Channel1 => { dma.ccr1().modify(|_, w| w.en().set_bit()); },
            DmaChannel::Channel2 => { dma.ccr2().modify(|_, w| w.en().set_bit()); },
            DmaChannel::Channel3 => { dma.ccr3().modify(|_, w| w.en().set_bit()); },
            DmaChannel::Channel4 => { dma.ccr4().modify(|_, w| w.en().set_bit()); },
            DmaChannel::Channel5 => { dma.ccr5().modify(|_, w| w.en().set_bit()); },
            DmaChannel::Channel6 => { dma.ccr6().modify(|_, w| w.en().set_bit()); },
            DmaChannel::Channel7 => { dma.ccr7().modify(|_, w| w.en().set_bit()); },
        }
    }
    
    /// 禁用DMA通道
    pub unsafe fn disable(&self) {
        let dma = self.get_dma();
        
        match self.channel {
            DmaChannel::Channel1 => { dma.ccr1().modify(|_, w| w.en().clear_bit()); },
            DmaChannel::Channel2 => { dma.ccr2().modify(|_, w| w.en().clear_bit()); },
            DmaChannel::Channel3 => { dma.ccr3().modify(|_, w| w.en().clear_bit()); },
            DmaChannel::Channel4 => { dma.ccr4().modify(|_, w| w.en().clear_bit()); },
            DmaChannel::Channel5 => { dma.ccr5().modify(|_, w| w.en().clear_bit()); },
            DmaChannel::Channel6 => { dma.ccr6().modify(|_, w| w.en().clear_bit()); },
            DmaChannel::Channel7 => { dma.ccr7().modify(|_, w| w.en().clear_bit()); },
        }
    }
    
    /// 启用中断
    pub unsafe fn enable_interrupt(&self, interrupt: DmaInterrupt) {
        let dma = self.get_dma();
        
        match self.channel {
            DmaChannel::Channel1 => {
                dma.ccr1().modify(|_, w| {
                    match interrupt {
                        DmaInterrupt::TransferComplete => w.tcie().set_bit(),
                        DmaInterrupt::HalfTransfer => w.htie().set_bit(),
                        DmaInterrupt::TransferError => w.teie().set_bit(),
                    }
                });
            },
            DmaChannel::Channel2 => {
                dma.ccr2().modify(|_, w| {
                    match interrupt {
                        DmaInterrupt::TransferComplete => w.tcie().set_bit(),
                        DmaInterrupt::HalfTransfer => w.htie().set_bit(),
                        DmaInterrupt::TransferError => w.teie().set_bit(),
                    }
                });
            },
            DmaChannel::Channel3 => {
                dma.ccr3().modify(|_, w| {
                    match interrupt {
                        DmaInterrupt::TransferComplete => w.tcie().set_bit(),
                        DmaInterrupt::HalfTransfer => w.htie().set_bit(),
                        DmaInterrupt::TransferError => w.teie().set_bit(),
                    }
                });
            },
            DmaChannel::Channel4 => {
                dma.ccr4().modify(|_, w| {
                    match interrupt {
                        DmaInterrupt::TransferComplete => w.tcie().set_bit(),
                        DmaInterrupt::HalfTransfer => w.htie().set_bit(),
                        DmaInterrupt::TransferError => w.teie().set_bit(),
                    }
                });
            },
            DmaChannel::Channel5 => {
                dma.ccr5().modify(|_, w| {
                    match interrupt {
                        DmaInterrupt::TransferComplete => w.tcie().set_bit(),
                        DmaInterrupt::HalfTransfer => w.htie().set_bit(),
                        DmaInterrupt::TransferError => w.teie().set_bit(),
                    }
                });
            },
            DmaChannel::Channel6 => {
                dma.ccr6().modify(|_, w| {
                    match interrupt {
                        DmaInterrupt::TransferComplete => w.tcie().set_bit(),
                        DmaInterrupt::HalfTransfer => w.htie().set_bit(),
                        DmaInterrupt::TransferError => w.teie().set_bit(),
                    }
                });
            },
            DmaChannel::Channel7 => {
                dma.ccr7().modify(|_, w| {
                    match interrupt {
                        DmaInterrupt::TransferComplete => w.tcie().set_bit(),
                        DmaInterrupt::HalfTransfer => w.htie().set_bit(),
                        DmaInterrupt::TransferError => w.teie().set_bit(),
                    }
                });
            },
        }
    }
    
    /// 禁用中断
    pub unsafe fn disable_interrupt(&self, interrupt: DmaInterrupt) {
        let dma = self.get_dma();
        
        match self.channel {
            DmaChannel::Channel1 => {
                dma.ccr1().modify(|_, w| {
                    match interrupt {
                        DmaInterrupt::TransferComplete => w.tcie().clear_bit(),
                        DmaInterrupt::HalfTransfer => w.htie().clear_bit(),
                        DmaInterrupt::TransferError => w.teie().clear_bit(),
                    }
                });
            },
            DmaChannel::Channel2 => {
                dma.ccr2().modify(|_, w| {
                    match interrupt {
                        DmaInterrupt::TransferComplete => w.tcie().clear_bit(),
                        DmaInterrupt::HalfTransfer => w.htie().clear_bit(),
                        DmaInterrupt::TransferError => w.teie().clear_bit(),
                    }
                });
            },
            DmaChannel::Channel3 => {
                dma.ccr3().modify(|_, w| {
                    match interrupt {
                        DmaInterrupt::TransferComplete => w.tcie().clear_bit(),
                        DmaInterrupt::HalfTransfer => w.htie().clear_bit(),
                        DmaInterrupt::TransferError => w.teie().clear_bit(),
                    }
                });
            },
            DmaChannel::Channel4 => {
                dma.ccr4().modify(|_, w| {
                    match interrupt {
                        DmaInterrupt::TransferComplete => w.tcie().clear_bit(),
                        DmaInterrupt::HalfTransfer => w.htie().clear_bit(),
                        DmaInterrupt::TransferError => w.teie().clear_bit(),
                    }
                });
            },
            DmaChannel::Channel5 => {
                dma.ccr5().modify(|_, w| {
                    match interrupt {
                        DmaInterrupt::TransferComplete => w.tcie().clear_bit(),
                        DmaInterrupt::HalfTransfer => w.htie().clear_bit(),
                        DmaInterrupt::TransferError => w.teie().clear_bit(),
                    }
                });
            },
            DmaChannel::Channel6 => {
                dma.ccr6().modify(|_, w| {
                    match interrupt {
                        DmaInterrupt::TransferComplete => w.tcie().clear_bit(),
                        DmaInterrupt::HalfTransfer => w.htie().clear_bit(),
                        DmaInterrupt::TransferError => w.teie().clear_bit(),
                    }
                });
            },
            DmaChannel::Channel7 => {
                dma.ccr7().modify(|_, w| {
                    match interrupt {
                        DmaInterrupt::TransferComplete => w.tcie().clear_bit(),
                        DmaInterrupt::HalfTransfer => w.htie().clear_bit(),
                        DmaInterrupt::TransferError => w.teie().clear_bit(),
                    }
                });
            },
        }
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
        dma.ifcr().write(|w| unsafe { w.bits((interrupt as u32) << channel_offset) });
    }
    
    /// 获取剩余数据计数
    pub unsafe fn get_remaining_count(&self) -> u16 {
        let dma = self.get_dma();
        
        match self.channel {
            DmaChannel::Channel1 => dma.cndtr1().read().ndt().bits(),
            DmaChannel::Channel2 => dma.cndtr2().read().ndt().bits(),
            DmaChannel::Channel3 => dma.cndtr3().read().ndt().bits(),
            DmaChannel::Channel4 => dma.cndtr4().read().ndt().bits(),
            DmaChannel::Channel5 => dma.cndtr5().read().ndt().bits(),
            DmaChannel::Channel6 => dma.cndtr6().read().ndt().bits(),
            DmaChannel::Channel7 => dma.cndtr7().read().ndt().bits(),
        }
    }
    
    /// 检查DMA通道是否正在传输
    pub unsafe fn is_transferring(&self) -> bool {
        let dma = self.get_dma();
        
        match self.channel {
            DmaChannel::Channel1 => {
                dma.ccr1().read().en().bit_is_set() && dma.cndtr1().read().ndt().bits() > 0
            },
            DmaChannel::Channel2 => {
                dma.ccr2().read().en().bit_is_set() && dma.cndtr2().read().ndt().bits() > 0
            },
            DmaChannel::Channel3 => {
                dma.ccr3().read().en().bit_is_set() && dma.cndtr3().read().ndt().bits() > 0
            },
            DmaChannel::Channel4 => {
                dma.ccr4().read().en().bit_is_set() && dma.cndtr4().read().ndt().bits() > 0
            },
            DmaChannel::Channel5 => {
                dma.ccr5().read().en().bit_is_set() && dma.cndtr5().read().ndt().bits() > 0
            },
            DmaChannel::Channel6 => {
                dma.ccr6().read().en().bit_is_set() && dma.cndtr6().read().ndt().bits() > 0
            },
            DmaChannel::Channel7 => {
                dma.ccr7().read().en().bit_is_set() && dma.cndtr7().read().ndt().bits() > 0
            },
        }
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
