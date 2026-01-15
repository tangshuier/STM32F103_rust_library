//! SPI模块
//! 提供串行外设接口功能封装

#![allow(unused)]

// 使用内部生成的设备驱动库
use stm32f103::*;

/// SPI枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpiNumber {
    SPI1,
    SPI2,
    SPI3,
}

/// SPI模式枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpiMode {
    Mode0 = 0,    // CPOL=0, CPHA=0
    Mode1 = 1,    // CPOL=0, CPHA=1
    Mode2 = 2,    // CPOL=1, CPHA=0
    Mode3 = 3,    // CPOL=1, CPHA=1
}

/// SPI数据大小枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpiDataSize {
    Bits8 = 0,
    Bits16 = 1,
}

/// SPI时钟预分频系数枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpiBaudRatePrescaler {
    Div2 = 0,
    Div4 = 1,
    Div8 = 2,
    Div16 = 3,
    Div32 = 4,
    Div64 = 5,
    Div128 = 6,
    Div256 = 7,
}

/// SPI数据方向枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpiDirection {
    TwoLinesFullDuplex = 0,
    TwoLinesRxOnly = 1,
    OneLineRx = 2,
    OneLineTx = 3,
}

/// SPI NSS管理模式枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpiNssMode {
    Software = 1,
    Hardware = 0,
}

/// SPI结构体
pub struct Spi {
    number: SpiNumber,
}

impl SpiNumber {
    /// 获取SPI时钟使能位
    const fn clock_en_bit(&self) -> u32 {
        match self {
            SpiNumber::SPI1 => 1 << 12,  // APB2
            SpiNumber::SPI2 => 1 << 14,  // APB1
            SpiNumber::SPI3 => 1 << 15,  // APB1
        }
    }
}

impl Spi {
    /// 创建新的SPI实例
    pub const fn new(number: SpiNumber) -> Self {
        Self {
            number,
        }
    }
    
    /// 获取对应的SPI寄存器块
    unsafe fn get_spi(&self) -> &'static mut stm32f103::spi1::RegisterBlock {
        match self.number {
            SpiNumber::SPI1 => &mut *(0x40013000 as *mut stm32f103::spi1::RegisterBlock),
            SpiNumber::SPI2 => &mut *(0x40003800 as *mut stm32f103::spi2::RegisterBlock),
            SpiNumber::SPI3 => &mut *(0x40003C00 as *mut stm32f103::spi3::RegisterBlock),
        }
    }
    
    /// 初始化SPI
    pub unsafe fn init(
        &self,
        mode: SpiMode,
        data_size: SpiDataSize,
        baud_rate: SpiBaudRatePrescaler,
        direction: SpiDirection,
        nss_mode: SpiNssMode,
    ) {
        // 启用SPI时钟
        let rcc = &mut *(0x40021000 as *mut stm32f103::rcc::RegisterBlock);
        match self.number {
            SpiNumber::SPI1 => {
                let mut value = rcc.apb2enr().read().bits();
                value |= 1 << 12;
                rcc.apb2enr().write(|w: &mut stm32f103::rcc::apb2enr::W| unsafe { w.bits(value) });
            },
            SpiNumber::SPI2 => {
                let mut value = rcc.apb1enr().read().bits();
                value |= 1 << 14;
                rcc.apb1enr().write(|w: &mut stm32f103::rcc::apb1enr::W| unsafe { w.bits(value) });
            },
            SpiNumber::SPI3 => {
                let mut value = rcc.apb1enr().read().bits();
                value |= 1 << 15;
                rcc.apb1enr().write(|w: &mut stm32f103::rcc::apb1enr::W| unsafe { w.bits(value) });
            },
        }
        
        let spi = self.get_spi();
        
        // 禁用SPI
        spi.cr1().write(|w: &mut stm32f103::spi1::cr1::W| unsafe { w.bits(spi.cr1().read().bits() & !(1 << 6)) });
        
        // 配置SPI
        let mut cr1 = 0;
        
        // 设置主模式
        cr1 |= (1 << 2);
        
        // 设置SPI模式
        cr1 |= ((mode as u32) & 0x03) << 0;
        
        // 设置数据大小
        cr1 |= (data_size as u32) << 11;
        
        // 设置时钟预分频
        cr1 |= (baud_rate as u32) << 3;
        
        // 设置数据方向
        cr1 |= ((direction as u32) & 0x03) << 14;
        
        // 设置NSS管理模式
        cr1 |= (nss_mode as u32) << 9;
        
        // 启用SPI
        cr1 |= (1 << 6);
        
        spi.cr1().write(|w: &mut stm32f103::spi1::cr1::W| unsafe { w.bits(cr1) });
        
        // 配置CR2
        let mut cr2 = 0;
        // 启用接收缓冲区非空中断
        cr2 |= (1 << 6);
        spi.cr2().write(|w: &mut stm32f103::spi1::cr2::W| unsafe { w.bits(cr2) });
    }
    
    /// 发送数据
    pub unsafe fn send(&self, data: u16) {
        let spi = self.get_spi();
        // 等待发送缓冲区为空
        while (spi.sr().read().bits() & (1 << 1)) == 0 {
            core::hint::spin_loop();
        }
        
        // 发送数据
        spi.dr().write(|w: &mut stm32f103::spi1::dr::W| unsafe { w.bits(data as u32) });
        
        // 等待传输完成
        while (spi.sr().read().bits() & (1 << 7)) == 0 {
            core::hint::spin_loop();
        }
    }
    
    /// 接收数据
    pub unsafe fn receive(&self) -> u16 {
        let spi = self.get_spi();
        // 等待接收缓冲区非空
        while (spi.sr().read().bits() & (1 << 0)) == 0 {
            core::hint::spin_loop();
        }
        
        // 读取数据
        spi.dr().read().bits() as u16
    }
    
    /// 发送并接收数据（全双工）
    pub unsafe fn transfer(&self, data: u16) -> u16 {
        // 发送数据
        self.send(data);
        
        // 接收数据
        self.receive()
    }
    
    /// 发送数据缓冲区
    pub unsafe fn send_buffer(&self, buffer: &[u8]) {
        for &byte in buffer {
            self.send(byte as u16);
        }
    }
    
    /// 接收数据缓冲区
    pub unsafe fn receive_buffer(&self, buffer: &mut [u8]) {
        for byte in buffer {
            *byte = self.receive() as u8;
        }
    }
    
    /// 传输数据缓冲区（全双工）
    pub unsafe fn transfer_buffer(&self, tx_buffer: &[u8], rx_buffer: &mut [u8]) {
        for (i, &byte) in tx_buffer.iter().enumerate() {
            if i < rx_buffer.len() {
                rx_buffer[i] = self.transfer(byte as u16) as u8;
            } else {
                self.send(byte as u16);
            }
        }
    }
    
    /// 检查SPI是否忙
    pub unsafe fn is_busy(&self) -> bool {
        let spi = self.get_spi();
        (spi.sr().read().bits() & (1 << 7)) != 0
    }
    
    /// 检查接收缓冲区是否非空
    pub unsafe fn is_rx_not_empty(&self) -> bool {
        let spi = self.get_spi();
        (spi.sr().read().bits() & (1 << 0)) != 0
    }
    
    /// 检查发送缓冲区是否为空
    pub unsafe fn is_tx_empty(&self) -> bool {
        let spi = self.get_spi();
        (spi.sr().read().bits() & (1 << 1)) != 0
    }
    
    /// 启用SPI
    pub unsafe fn enable(&self) {
        let spi = self.get_spi();
        spi.cr1().write(|w: &mut stm32f103::spi1::cr1::W| unsafe { w.bits(spi.cr1().read().bits() | (1 << 6)) });
    }
    
    /// 禁用SPI
    pub unsafe fn disable(&self) {
        let spi = self.get_spi();
        spi.cr1().write(|w: &mut stm32f103::spi1::cr1::W| unsafe { w.bits(spi.cr1().read().bits() & !(1 << 6)) });
    }
}

/// 预定义的SPI实例
pub const SPI1: Spi = Spi::new(SpiNumber::SPI1);
pub const SPI2: Spi = Spi::new(SpiNumber::SPI2);
pub const SPI3: Spi = Spi::new(SpiNumber::SPI3);
