//! SPI（串行外设接口）模块
//! 提供串行外设接口的封装和操作，用于与各种串行外设进行通信

// 屏蔽未使用代码警告
#![allow(unused)]

// 使用内部生成的设备驱动库
use library::*;

/// SPI错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiError {
    /// 初始化失败
    InitializationFailed,
    /// 参数无效
    InvalidParameter,
    /// 传输失败
    TransmissionFailed,
    /// 接收失败
    ReceptionFailed,
    /// SPI忙碌
    Busy,
    /// 超时
    Timeout,
    /// 缓冲区太小
    BufferTooSmall,
    /// 无效操作
    InvalidOperation,
    /// 未知错误
    UnknownError,
}

/// SPI状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiStatus {
    /// SPI准备就绪
    Ready,
    /// SPI正在初始化
    Initializing,
    /// SPI出现错误
    Error,
    /// SPI正在发送
    Transmitting,
    /// SPI正在接收
    Receiving,
    /// SPI忙碌
    Busy,
    /// 有数据可用
    DataAvailable,
    /// 发送缓冲区为空
    TxBufferEmpty,
    /// 接收缓冲区满
    RxBufferFull,
    /// SPI禁用
    Disabled,
}

/// SPI枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiNumber {
    SPI1,
    SPI2,
    SPI3,
}

/// SPI模式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiMode {
    Mode0 = 0,    // CPOL=0, CPHA=0
    Mode1 = 1,    // CPOL=0, CPHA=1
    Mode2 = 2,    // CPOL=1, CPHA=0
    Mode3 = 3,    // CPOL=1, CPHA=1
}

/// SPI数据大小枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiDataSize {
    Bits8 = 0,
    Bits16 = 1,
}

/// SPI时钟预分频系数枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiDirection {
    TwoLinesFullDuplex = 0,
    TwoLinesRxOnly = 1,
    OneLineRx = 2,
    OneLineTx = 3,
}

/// SPI NSS管理模式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiNssMode {
    Software = 1,
    Hardware = 0,
}

/// SPI结构体
#[derive(Debug, Clone, Copy)]
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
    
    /// 获取对应的SPI1寄存器块
    unsafe fn get_spi1(&self) -> &'static mut library::spi1::RegisterBlock {
        &mut *(0x40013000 as *mut library::spi1::RegisterBlock)
    }
    
    /// 获取对应的SPI2寄存器块
    unsafe fn get_spi2(&self) -> &'static mut library::spi2::RegisterBlock {
        &mut *(0x40003800 as *mut library::spi2::RegisterBlock)
    }
    
    /// 获取对应的SPI3寄存器块
    unsafe fn get_spi3(&self) -> &'static mut library::spi3::RegisterBlock {
        &mut *(0x40003C00 as *mut library::spi3::RegisterBlock)
    }
    
    /// 获取RCC寄存器块的可变引用
    pub unsafe fn rcc_reg_mut(&self) -> &'static mut library::rcc::RegisterBlock {
        &mut *(0x40021000 as *mut library::rcc::RegisterBlock)
    }
    
    /// 初始化SPI
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// - 调用者必须确保SPI引脚已经正确配置
    /// - 调用者必须确保提供的参数有效
    /// 
    /// # 参数
    /// - `mode`：SPI模式（CPOL和CPHA组合）
    /// - `data_size`：数据大小（8位或16位）
    /// - `baud_rate`：时钟预分频系数
    /// - `direction`：数据传输方向
    /// - `nss_mode`：NSS管理模式
    /// 
    /// # 返回值
    /// - Ok(())：SPI初始化成功
    /// - Err(SpiError)：SPI初始化失败
    pub unsafe fn init(
        &self,
        mode: SpiMode,
        data_size: SpiDataSize,
        baud_rate: SpiBaudRatePrescaler,
        direction: SpiDirection,
        nss_mode: SpiNssMode,
    ) -> Result<(), SpiError> {
        let rcc = self.rcc_reg_mut();
        
        // 启用SPI时钟
        match self.number {
            SpiNumber::SPI1 => {
                rcc.apb2enr().modify(|_, w| w
                    .spi1en().set_bit()
                );
            },
            SpiNumber::SPI2 => {
                rcc.apb1enr().modify(|_, w| w
                    .spi2en().set_bit()
                );
            },
            SpiNumber::SPI3 => {
                rcc.apb1enr().modify(|_, w| w
                    .spi3en().set_bit()
                );
            },
        }
        
        // 配置SPI寄存器
        match self.number {
            SpiNumber::SPI1 => {
                let spi = self.get_spi1();
                // 禁用SPI
                spi.cr1().modify(|_, w| w
                    .spe().clear_bit()
                );
                
                // 配置CR1寄存器
                let mut cr1 = 0;
                
                // 设置主模式
                cr1 |= (1 << 2); // MSTR位
                
                // 设置SPI模式
                cr1 |= ((mode as u32) & 0x03) << 0; // CPOL和CPHA位
                
                // 设置数据大小
                cr1 |= (data_size as u32) << 11; // DFF位
                
                // 设置时钟预分频
                cr1 |= (baud_rate as u32) << 3; // BR[2:0]位
                
                // 设置数据方向
                cr1 |= ((direction as u32) & 0x03) << 14; // BIDIMODE和RXONLY位
                
                // 设置NSS管理模式
                cr1 |= (nss_mode as u32) << 9; // SSM位
                
                // 启用SPI
                cr1 |= (1 << 6); // SPE位
                
                spi.cr1().write(|w| unsafe { w.bits(cr1) });
                
                // 配置CR2寄存器
                let mut cr2 = 0;
                // 启用接收缓冲区非空中断
                cr2 |= (1 << 6); // RXNEIE位
                spi.cr2().write(|w| unsafe { w.bits(cr2) });
            },
            SpiNumber::SPI2 => {
                let spi = self.get_spi2();
                // 禁用SPI
                spi.cr1().modify(|_, w| w
                    .spe().clear_bit()
                );
                
                // 配置CR1寄存器
                let mut cr1 = 0;
                
                // 设置主模式
                cr1 |= (1 << 2); // MSTR位
                
                // 设置SPI模式
                cr1 |= ((mode as u32) & 0x03) << 0; // CPOL和CPHA位
                
                // 设置数据大小
                cr1 |= (data_size as u32) << 11; // DFF位
                
                // 设置时钟预分频
                cr1 |= (baud_rate as u32) << 3; // BR[2:0]位
                
                // 设置数据方向
                cr1 |= ((direction as u32) & 0x03) << 14; // BIDIMODE和RXONLY位
                
                // 设置NSS管理模式
                cr1 |= (nss_mode as u32) << 9; // SSM位
                
                // 启用SPI
                cr1 |= (1 << 6); // SPE位
                
                spi.cr1().write(|w| unsafe { w.bits(cr1) });
                
                // 配置CR2寄存器
                let mut cr2 = 0;
                // 启用接收缓冲区非空中断
                cr2 |= (1 << 6); // RXNEIE位
                spi.cr2().write(|w| unsafe { w.bits(cr2) });
            },
            SpiNumber::SPI3 => {
                let spi = self.get_spi3();
                // 禁用SPI
                spi.cr1().modify(|_, w| w
                    .spe().clear_bit()
                );
                
                // 配置CR1寄存器
                let mut cr1 = 0;
                
                // 设置主模式
                cr1 |= (1 << 2); // MSTR位
                
                // 设置SPI模式
                cr1 |= ((mode as u32) & 0x03) << 0; // CPOL和CPHA位
                
                // 设置数据大小
                cr1 |= (data_size as u32) << 11; // DFF位
                
                // 设置时钟预分频
                cr1 |= (baud_rate as u32) << 3; // BR[2:0]位
                
                // 设置数据方向
                cr1 |= ((direction as u32) & 0x03) << 14; // BIDIMODE和RXONLY位
                
                // 设置NSS管理模式
                cr1 |= (nss_mode as u32) << 9; // SSM位
                
                // 启用SPI
                cr1 |= (1 << 6); // SPE位
                
                spi.cr1().write(|w| unsafe { w.bits(cr1) });
                
                // 配置CR2寄存器
                let mut cr2 = 0;
                // 启用接收缓冲区非空中断
                cr2 |= (1 << 6); // RXNEIE位
                spi.cr2().write(|w| unsafe { w.bits(cr2) });
            },
        }
        
        Ok(())
    }
    

    
    /// 发送数据
    /// 
    /// # 安全
    /// - 调用者必须确保SPI已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `data`：要发送的数据（8位或16位，根据配置）
    /// 
    /// # 返回值
    /// - Ok(())：发送成功
    /// - Err(SpiError)：发送失败
    pub unsafe fn send(&self, data: u16) -> Result<(), SpiError> {
        // 等待发送缓冲区为空
        match self.number {
            SpiNumber::SPI1 => {
                let spi = self.get_spi1();
                // 等待发送缓冲区为空
                let mut timeout = 10000;
                while !spi.sr().read().txe().bit_is_set() {
                    timeout -= 1;
                    if timeout == 0 {
                        return Err(SpiError::Timeout);
                    }
                    core::hint::spin_loop();
                }
                
                // 发送数据
                spi.dr().write(|w| unsafe { w.bits(data as u32) });
                
                // 等待传输完成
                timeout = 10000;
                while !spi.sr().read().txc().bit_is_set() {
                    timeout -= 1;
                    if timeout == 0 {
                        return Err(SpiError::Timeout);
                    }
                    core::hint::spin_loop();
                }
            },
            SpiNumber::SPI2 => {
                let spi = self.get_spi2();
                // 等待发送缓冲区为空
                let mut timeout = 10000;
                while !spi.sr().read().txe().bit_is_set() {
                    timeout -= 1;
                    if timeout == 0 {
                        return Err(SpiError::Timeout);
                    }
                    core::hint::spin_loop();
                }
                
                // 发送数据
                spi.dr().write(|w| unsafe { w.bits(data as u32) });
                
                // 等待传输完成
                timeout = 10000;
                while !spi.sr().read().txc().bit_is_set() {
                    timeout -= 1;
                    if timeout == 0 {
                        return Err(SpiError::Timeout);
                    }
                    core::hint::spin_loop();
                }
            },
            SpiNumber::SPI3 => {
                let spi = self.get_spi3();
                // 等待发送缓冲区为空
                let mut timeout = 10000;
                while !spi.sr().read().txe().bit_is_set() {
                    timeout -= 1;
                    if timeout == 0 {
                        return Err(SpiError::Timeout);
                    }
                    core::hint::spin_loop();
                }
                
                // 发送数据
                spi.dr().write(|w| unsafe { w.bits(data as u32) });
                
                // 等待传输完成
                timeout = 10000;
                while !spi.sr().read().txc().bit_is_set() {
                    timeout -= 1;
                    if timeout == 0 {
                        return Err(SpiError::Timeout);
                    }
                    core::hint::spin_loop();
                }
            },
        }
        
        Ok(())
    }
    
    /// 接收数据
    /// 
    /// # 安全
    /// - 调用者必须确保SPI已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(u16)：接收到的数据
    /// - Err(SpiError)：接收失败
    pub unsafe fn receive(&self) -> Result<u16, SpiError> {
        match self.number {
            SpiNumber::SPI1 => {
                let spi = self.get_spi1();
                // 等待接收缓冲区非空
                let mut timeout = 10000;
                while !spi.sr().read().rxne().bit_is_set() {
                    timeout -= 1;
                    if timeout == 0 {
                        return Err(SpiError::Timeout);
                    }
                    core::hint::spin_loop();
                }
                
                // 读取数据
                Ok(spi.dr().read().bits() as u16)
            },
            SpiNumber::SPI2 => {
                let spi = self.get_spi2();
                // 等待接收缓冲区非空
                let mut timeout = 10000;
                while !spi.sr().read().rxne().bit_is_set() {
                    timeout -= 1;
                    if timeout == 0 {
                        return Err(SpiError::Timeout);
                    }
                    core::hint::spin_loop();
                }
                
                // 读取数据
                Ok(spi.dr().read().bits() as u16)
            },
            SpiNumber::SPI3 => {
                let spi = self.get_spi3();
                // 等待接收缓冲区非空
                let mut timeout = 10000;
                while !spi.sr().read().rxne().bit_is_set() {
                    timeout -= 1;
                    if timeout == 0 {
                        return Err(SpiError::Timeout);
                    }
                    core::hint::spin_loop();
                }
                
                // 读取数据
                Ok(spi.dr().read().bits() as u16)
            },
        }
    }
    
    /// 发送并接收数据（全双工）
    /// 
    /// # 安全
    /// - 调用者必须确保SPI已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `data`：要发送的数据
    /// 
    /// # 返回值
    /// - Ok(u16)：接收到的数据
    /// - Err(SpiError)：传输失败
    pub unsafe fn transfer(&self, data: u16) -> Result<u16, SpiError> {
        // 发送数据
        self.send(data)?;
        
        // 接收数据
        self.receive()
    }
    
    /// 发送数据缓冲区
    /// 
    /// # 安全
    /// - 调用者必须确保SPI已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `buffer`：要发送的数据缓冲区
    /// 
    /// # 返回值
    /// - Ok(())：发送成功
    /// - Err(SpiError)：发送失败
    pub unsafe fn send_buffer(&self, buffer: &[u8]) -> Result<(), SpiError> {
        for &byte in buffer {
            self.send(byte as u16)?;
        }
        Ok(())
    }
    
    /// 接收数据缓冲区
    /// 
    /// # 安全
    /// - 调用者必须确保SPI已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `buffer`：用于接收数据的缓冲区
    /// 
    /// # 返回值
    /// - Ok(())：接收成功
    /// - Err(SpiError)：接收失败
    pub unsafe fn receive_buffer(&self, buffer: &mut [u8]) -> Result<(), SpiError> {
        for byte in buffer {
            *byte = self.receive()? as u8;
        }
        Ok(())
    }
    
    /// 传输数据缓冲区（全双工）
    /// 
    /// # 安全
    /// - 调用者必须确保SPI已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `tx_buffer`：要发送的数据缓冲区
    /// - `rx_buffer`：用于接收数据的缓冲区
    /// 
    /// # 返回值
    /// - Ok(())：传输成功
    /// - Err(SpiError)：传输失败
    pub unsafe fn transfer_buffer(&self, tx_buffer: &[u8], rx_buffer: &mut [u8]) -> Result<(), SpiError> {
        for (i, &byte) in tx_buffer.iter().enumerate() {
            if i < rx_buffer.len() {
                rx_buffer[i] = self.transfer(byte as u16)? as u8;
            } else {
                self.send(byte as u16)?;
            }
        }
        Ok(())
    }
    
    /// 检查SPI是否忙
    /// 
    /// # 安全
    /// - 调用者必须确保SPI已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(bool)：SPI是否忙碌
    /// - Err(SpiError)：检查失败
    pub unsafe fn is_busy(&self) -> Result<bool, SpiError> {
        match self.number {
            SpiNumber::SPI1 => {
                let spi = self.get_spi1();
                Ok(spi.sr().read().bsy().bit_is_set())
            },
            SpiNumber::SPI2 => {
                let spi = self.get_spi2();
                Ok(spi.sr().read().bsy().bit_is_set())
            },
            SpiNumber::SPI3 => {
                let spi = self.get_spi3();
                Ok(spi.sr().read().bsy().bit_is_set())
            },
        }
    }
    
    /// 检查接收缓冲区是否非空
    /// 
    /// # 安全
    /// - 调用者必须确保SPI已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(bool)：接收缓冲区是否非空
    /// - Err(SpiError)：检查失败
    pub unsafe fn is_rx_not_empty(&self) -> Result<bool, SpiError> {
        match self.number {
            SpiNumber::SPI1 => {
                let spi = self.get_spi1();
                Ok(spi.sr().read().rxne().bit_is_set())
            },
            SpiNumber::SPI2 => {
                let spi = self.get_spi2();
                Ok(spi.sr().read().rxne().bit_is_set())
            },
            SpiNumber::SPI3 => {
                let spi = self.get_spi3();
                Ok(spi.sr().read().rxne().bit_is_set())
            },
        }
    }
    
    /// 检查发送缓冲区是否为空
    /// 
    /// # 安全
    /// - 调用者必须确保SPI已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(bool)：发送缓冲区是否为空
    /// - Err(SpiError)：检查失败
    pub unsafe fn is_tx_empty(&self) -> Result<bool, SpiError> {
        match self.number {
            SpiNumber::SPI1 => {
                let spi = self.get_spi1();
                Ok(spi.sr().read().txe().bit_is_set())
            },
            SpiNumber::SPI2 => {
                let spi = self.get_spi2();
                Ok(spi.sr().read().txe().bit_is_set())
            },
            SpiNumber::SPI3 => {
                let spi = self.get_spi3();
                Ok(spi.sr().read().txe().bit_is_set())
            },
        }
    }
    
    /// 启用SPI
    /// 
    /// # 安全
    /// - 调用者必须确保SPI已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(())：启用成功
    /// - Err(SpiError)：启用失败
    pub unsafe fn enable(&self) -> Result<(), SpiError> {
        match self.number {
            SpiNumber::SPI1 => {
                let spi = self.get_spi1();
                spi.cr1().modify(|_, w| w
                    .spe().set_bit()
                );
            },
            SpiNumber::SPI2 => {
                let spi = self.get_spi2();
                spi.cr1().modify(|_, w| w
                    .spe().set_bit()
                );
            },
            SpiNumber::SPI3 => {
                let spi = self.get_spi3();
                spi.cr1().modify(|_, w| w
                    .spe().set_bit()
                );
            },
        }
        Ok(())
    }
    
    /// 禁用SPI
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(())：禁用成功
    /// - Err(SpiError)：禁用失败
    pub unsafe fn disable(&self) -> Result<(), SpiError> {
        match self.number {
            SpiNumber::SPI1 => {
                let spi = self.get_spi1();
                spi.cr1().modify(|_, w| w
                    .spe().clear_bit()
                );
            },
            SpiNumber::SPI2 => {
                let spi = self.get_spi2();
                spi.cr1().modify(|_, w| w
                    .spe().clear_bit()
                );
            },
            SpiNumber::SPI3 => {
                let spi = self.get_spi3();
                spi.cr1().modify(|_, w| w
                    .spe().clear_bit()
                );
            },
        }
        Ok(())
    }
    
    /// 获取SPI状态
    /// 
    /// # 安全
    /// - 调用者必须确保SPI已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(SpiStatus)：SPI当前状态
    /// - Err(SpiError)：获取状态失败
    pub unsafe fn get_status(&self) -> Result<SpiStatus, SpiError> {
        match self.number {
            SpiNumber::SPI1 => {
                let spi = self.get_spi1();
                let sr = spi.sr().read();
                let cr1 = spi.cr1().read();
                
                // 检查是否禁用
                if !cr1.spe().bit_is_set() {
                    return Ok(SpiStatus::Disabled);
                }
                
                // 检查忙碌状态
                if sr.bsy().bit_is_set() {
                    return Ok(SpiStatus::Busy);
                }
                
                // 检查接收缓冲区
                if sr.rxne().bit_is_set() {
                    return Ok(SpiStatus::DataAvailable);
                }
                
                // 检查发送缓冲区
                if sr.txe().bit_is_set() {
                    return Ok(SpiStatus::TxBufferEmpty);
                }
                
                Ok(SpiStatus::Ready)
            },
            SpiNumber::SPI2 => {
                let spi = self.get_spi2();
                let sr = spi.sr().read();
                let cr1 = spi.cr1().read();
                
                // 检查是否禁用
                if !cr1.spe().bit_is_set() {
                    return Ok(SpiStatus::Disabled);
                }
                
                // 检查忙碌状态
                if sr.bsy().bit_is_set() {
                    return Ok(SpiStatus::Busy);
                }
                
                // 检查接收缓冲区
                if sr.rxne().bit_is_set() {
                    return Ok(SpiStatus::DataAvailable);
                }
                
                // 检查发送缓冲区
                if sr.txe().bit_is_set() {
                    return Ok(SpiStatus::TxBufferEmpty);
                }
                
                Ok(SpiStatus::Ready)
            },
            SpiNumber::SPI3 => {
                let spi = self.get_spi3();
                let sr = spi.sr().read();
                let cr1 = spi.cr1().read();
                
                // 检查是否禁用
                if !cr1.spe().bit_is_set() {
                    return Ok(SpiStatus::Disabled);
                }
                
                // 检查忙碌状态
                if sr.bsy().bit_is_set() {
                    return Ok(SpiStatus::Busy);
                }
                
                // 检查接收缓冲区
                if sr.rxne().bit_is_set() {
                    return Ok(SpiStatus::DataAvailable);
                }
                
                // 检查发送缓冲区
                if sr.txe().bit_is_set() {
                    return Ok(SpiStatus::TxBufferEmpty);
                }
                
                Ok(SpiStatus::Ready)
            },
        }
    }
}



/// 预定义的SPI实例
pub const SPI1: Spi = Spi::new(SpiNumber::SPI1);
pub const SPI2: Spi = Spi::new(SpiNumber::SPI2);
pub const SPI3: Spi = Spi::new(SpiNumber::SPI3);

/// 测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试SPI初始化和状态获取
    #[test]
    fn test_spi_init_status() {
        let spi = Spi::new(SpiNumber::SPI1);
        
        // 初始化SPI
        unsafe {
            let init_result = spi.init(
                SpiMode::Mode0,
                SpiDataSize::Bits8,
                SpiBaudRatePrescaler::Div16,
                SpiDirection::TwoLinesFullDuplex,
                SpiNssMode::Software
            );
            assert!(init_result.is_ok(), "SPI初始化应该成功");
            
            let status = spi.get_status();
            assert!(status.is_ok(), "获取SPI状态应该成功");
            assert_eq!(status.unwrap(), SpiStatus::Ready, "SPI初始状态应该是Ready");
        }
    }
    
    /// 测试SPI基本传输
    #[test]
    fn test_spi_basic_transfer() {
        let spi = Spi::new(SpiNumber::SPI1);
        
        unsafe {
            let init_result = spi.init(
                SpiMode::Mode0,
                SpiDataSize::Bits8,
                SpiBaudRatePrescaler::Div16,
                SpiDirection::TwoLinesFullDuplex,
                SpiNssMode::Software
            );
            assert!(init_result.is_ok(), "SPI初始化应该成功");
            
            // 发送数据
            let send_result = spi.send(0x55);
            assert!(send_result.is_ok(), "SPI发送数据应该成功");
            
            // 检查发送缓冲区是否为空
            let is_tx_empty = spi.is_tx_empty();
            assert!(is_tx_empty.is_ok(), "检查发送缓冲区状态应该成功");
            assert!(is_tx_empty.unwrap(), "发送缓冲区应该为空");
        }
    }
    
    /// 测试SPI缓冲区传输
    #[test]
    fn test_spi_buffer_transfer() {
        let spi = Spi::new(SpiNumber::SPI1);
        
        unsafe {
            let init_result = spi.init(
                SpiMode::Mode0,
                SpiDataSize::Bits8,
                SpiBaudRatePrescaler::Div16,
                SpiDirection::TwoLinesFullDuplex,
                SpiNssMode::Software
            );
            assert!(init_result.is_ok(), "SPI初始化应该成功");
            
            // 测试发送缓冲区
            let tx_buffer = [0x01, 0x02, 0x03, 0x04, 0x05];
            let send_result = spi.send_buffer(&tx_buffer);
            assert!(send_result.is_ok(), "SPI发送缓冲区应该成功");
            
            // 测试传输缓冲区
            let mut rx_buffer = [0u8; 5];
            let transfer_result = spi.transfer_buffer(&tx_buffer, &mut rx_buffer);
            assert!(transfer_result.is_ok(), "SPI传输缓冲区应该成功");
        }
    }
    
    /// 测试SPI启用/禁用
    #[test]
    fn test_spi_enable_disable() {
        let spi = Spi::new(SpiNumber::SPI1);
        
        unsafe {
            let init_result = spi.init(
                SpiMode::Mode0,
                SpiDataSize::Bits8,
                SpiBaudRatePrescaler::Div16,
                SpiDirection::TwoLinesFullDuplex,
                SpiNssMode::Software
            );
            assert!(init_result.is_ok(), "SPI初始化应该成功");
            
            // 禁用SPI
            let disable_result = spi.disable();
            assert!(disable_result.is_ok(), "禁用SPI应该成功");
            
            let status = spi.get_status();
            assert!(status.is_ok(), "获取SPI状态应该成功");
            assert_eq!(status.unwrap(), SpiStatus::Disabled, "SPI状态应该是Disabled");
            
            // 启用SPI
            let enable_result = spi.enable();
            assert!(enable_result.is_ok(), "启用SPI应该成功");
            
            let status = spi.get_status();
            assert!(status.is_ok(), "获取SPI状态应该成功");
            assert_eq!(status.unwrap(), SpiStatus::Ready, "SPI状态应该是Ready");
        }
    }
}
