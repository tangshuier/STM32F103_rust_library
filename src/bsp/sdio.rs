//! SDIO模块
//! 提供安全数字输入/输出功能封装

#![allow(unused)]

// 导入内部生成的设备驱动库
use stm32f103::*;

/// SDIO时钟频率枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SdioClockFreq {
    Freq400kHz = 0,    // 400kHz (初始化频率)
    Freq25MHz = 1,     // 25MHz
    Freq50MHz = 2,     // 50MHz
}

/// SDIO响应类型枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SdioResponseType {
    NoResponse = 0,    // 无响应
    ShortResponse = 1, // 短响应 (R1, R1b, R2, R3, R6)
    LongResponse = 2,  // 长响应 (R7)
}

/// SDIO数据传输宽度枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SdioDataWidth {
    Width1b = 0,     // 1位
    Width4b = 1,     // 4位
    Width8b = 2,     // 8位
}

/// SDIO结构体
pub struct SdioDriver;

impl SdioDriver {
    /// 创建新的SDIO实例
    pub const fn new() -> Self {
        Self
    }
    
    /// 获取SDIO寄存器块
    unsafe fn sdio() -> &'static mut stm32f103::sdio::RegisterBlock {
        &mut *(0x40012C00 as *mut stm32f103::sdio::RegisterBlock)
    }
    
    /// 初始化SDIO
    /// 
    /// # 参数
    /// * `clock_freq` - 时钟频率
    pub unsafe fn init(&self, clock_freq: SdioClockFreq) {
        let sdio = SdioDriver::sdio();
        
        // 关闭SDIO电源
        sdio.power().write(|w: &mut stm32f103::sdio::power::W| unsafe { w.bits(0x00000000) });
        
        // 重置SDIO
        self.reset();
        
        // 打开SDIO电源
        sdio.power().write(|w: &mut stm32f103::sdio::power::W| unsafe { w.bits(0x00000003) });
        
        // 配置时钟频率
        self.set_clock_frequency(clock_freq);
        
        // 启用SDIO
        sdio.power().write(|w: &mut stm32f103::sdio::power::W| unsafe { w.bits(0x00000003) });
    }
    
    /// 重置SDIO
    pub unsafe fn reset(&self) {
        let sdio = SdioDriver::sdio();
        
        // 重置命令通道（通过重置CPSMEN位）
        sdio.cmd().modify(|_, w: &mut stm32f103::sdio::cmd::W| w
            .cpsmen().clear_bit()
        );
        while !sdio.cmd().read().cpsmen().bit() {
            core::hint::spin_loop();
        }
        
        // 重置数据通道（通过重置DTEN位）
        sdio.dctrl().modify(|_, w: &mut stm32f103::sdio::dctrl::W| w
            .dten().clear_bit()
        );
        while !sdio.dctrl().read().dten().bit() {
            core::hint::spin_loop();
        }
    }
    
    /// 设置SDIO时钟频率
    /// 
    /// # 参数
    /// * `clock_freq` - 时钟频率
    pub unsafe fn set_clock_frequency(&self, clock_freq: SdioClockFreq) {
        let sdio = SdioDriver::sdio();
        
        // 禁用时钟
        sdio.clkcr().modify(|_, w: &mut stm32f103::sdio::clkcr::W| w
            .clken().clear_bit()
        );
        
        // 设置时钟分频系数
        match clock_freq {
            SdioClockFreq::Freq400kHz => {
                // 400kHz = 25MHz / 62.5, 取63
                sdio.clkcr().write(|w: &mut stm32f103::sdio::clkcr::W| unsafe { w.bits((62 << 0) | (1 << 7)) });
            }
            SdioClockFreq::Freq25MHz => {
                // 25MHz = 50MHz / 2
                sdio.clkcr().write(|w: &mut stm32f103::sdio::clkcr::W| unsafe { w.bits((1 << 0) | (1 << 7)) });
            }
            SdioClockFreq::Freq50MHz => {
                // 50MHz = 50MHz / 1
                sdio.clkcr().write(|w: &mut stm32f103::sdio::clkcr::W| unsafe { w.bits((0 << 0) | (1 << 7)) });
            }
        }
        
        // 启用时钟
        sdio.clkcr().modify(|_, w: &mut stm32f103::sdio::clkcr::W| w
            .clken().set_bit()
        );
    }
    
    /// 发送命令
    /// 
    /// # 参数
    /// * `cmd` - 命令号
    /// * `arg` - 命令参数
    /// * `resp_type` - 响应类型
    pub unsafe fn send_command(&self, cmd: u8, arg: u32, resp_type: SdioResponseType) {
        let sdio = SdioDriver::sdio();
        
        // 设置命令参数
        sdio.arg().write(|w: &mut stm32f103::sdio::arg::W| unsafe { w.bits(arg) });
        
        // 配置命令
        let mut cmd_reg = (cmd as u32) << 0;
        cmd_reg |= (resp_type as u32) << 6;
        cmd_reg |= (1 << 10); // 启动命令
        
        sdio.cmd().write(|w: &mut stm32f103::sdio::cmd::W| unsafe { w.bits(cmd_reg) });
        
        // 等待命令完成
        while !sdio.sta().read().cmdrend().bit() {
            core::hint::spin_loop();
        }
        
        // 清除命令完成标志
        sdio.icr().write(|w: &mut stm32f103::sdio::icr::W| unsafe { w.bits(1 << 0) });
    }
    
    /// 读取响应
    /// 
    /// # 参数
    /// * `resp_type` - 响应类型
    /// 
    /// # 返回值
    /// 响应数据
    pub unsafe fn read_response(&self, resp_type: SdioResponseType) -> [u32; 4] {
        let sdio = SdioDriver::sdio();
        let mut resp = [0u32; 4];
        
        match resp_type {
            SdioResponseType::ShortResponse => {
                resp[0] = sdio.respi1().read().bits();
            }
            SdioResponseType::LongResponse => {
                resp[0] = sdio.respi1().read().bits();
                resp[1] = sdio.resp2().read().bits();
                resp[2] = sdio.resp3().read().bits();
                resp[3] = sdio.resp4().read().bits();
            }
            _ => {}
        }
        
        resp
    }
    
    /// 配置数据传输
    /// 
    /// # 参数
    /// * `data_width` - 数据传输宽度
    /// * `block_size` - 块大小 (字节)
    /// * `block_count` - 块数量
    pub unsafe fn configure_data_transfer(
        &self,
        data_width: SdioDataWidth,
        block_size: u16,
        block_count: u16
    ) {
        let sdio = SdioDriver::sdio();
        
        // 设置数据长度
        sdio.dlen().write(|w: &mut stm32f103::sdio::dlen::W| unsafe { w.bits((block_size * block_count) as u32) });
        
        // 配置数据控制寄存器
        let mut dctrl = 0x00000000;
        dctrl |= (data_width as u32) << 0;
        dctrl |= (1 << 4); // 启用数据传输
        dctrl |= (1 << 5); // 块传输模式
        
        sdio.dctrl().write(|w: &mut stm32f103::sdio::dctrl::W| unsafe { w.bits(dctrl) });
    }
    
    /// 启动数据传输
    pub unsafe fn start_data_transfer(&self) {
        let sdio = SdioDriver::sdio();
        
        // 启动数据传输
        sdio.dctrl().modify(|_, w: &mut stm32f103::sdio::dctrl::W| w
            .dten().set_bit()
        );
    }
    
    /// 等待数据传输完成
    pub unsafe fn wait_for_data_transfer_complete(&self) {
        let sdio = SdioDriver::sdio();
        
        while !sdio.sta().read().dataend().bit() {
            core::hint::spin_loop();
        }
        
        // 清除数据传输完成标志
        sdio.icr().write(|w: &mut stm32f103::sdio::icr::W| unsafe { w.bits(1 << 1) });
    }
    
    /// 读取数据
    /// 
    /// # 参数
    /// * `buffer` - 数据缓冲区
    /// * `length` - 数据长度 (字节)
    pub unsafe fn read_data(&self, buffer: &mut [u8], length: usize) {
        let sdio = SdioDriver::sdio();
        let mut index = 0;
        let mut remaining = length;
        
        while remaining > 0 {
            if sdio.sta().read().rxfifohf().bit() {
                if remaining >= 4 {
                    let data = sdio.fifo().read().bits();
                    buffer[index..index+4].copy_from_slice(&data.to_le_bytes());
                    index += 4;
                    remaining -= 4;
                } else {
                    let data = sdio.fifo().read().bits();
                    let data_bytes = data.to_le_bytes();
                    buffer[index..index+remaining].copy_from_slice(&data_bytes[0..remaining]);
                    index += remaining;
                    remaining = 0;
                }
            }
        }
    }
    
    /// 写入数据
    /// 
    /// # 参数
    /// * `buffer` - 数据缓冲区
    /// * `length` - 数据长度 (字节)
    pub unsafe fn write_data(&self, buffer: &[u8], length: usize) {
        let sdio = SdioDriver::sdio();
        let mut index = 0;
        let mut remaining = length;
        
        while remaining > 0 {
            if sdio.sta().read().txfifohe().bit() {
                if remaining >= 4 {
                    let data = u32::from_le_bytes(buffer[index..index+4].try_into().unwrap());
                    sdio.fifo().write(|w: &mut stm32f103::sdio::fifo::W| unsafe { w.bits(data) });
                    index += 4;
                    remaining -= 4;
                } else {
                    let mut padding = [0u8; 4];
                    padding[0..remaining].copy_from_slice(&buffer[index..index+remaining]);
                    let data = u32::from_le_bytes(padding);
                    sdio.fifo().write(|w: &mut stm32f103::sdio::fifo::W| unsafe { w.bits(data) });
                    index += remaining;
                    remaining = 0;
                }
            }
        }
    }
    
    /// 启用中断
    /// 
    /// # 参数
    /// * `interrupt_mask` - 中断掩码
    pub unsafe fn enable_interrupts(&self, interrupt_mask: u32) {
        let sdio = SdioDriver::sdio();
        sdio.mask().modify(|_, w: &mut stm32f103::sdio::mask::W| unsafe {
            w.bits(sdio.mask().read().bits() | interrupt_mask)
        });
    }
    
    /// 禁用中断
    /// 
    /// # 参数
    /// * `interrupt_mask` - 中断掩码
    pub unsafe fn disable_interrupts(&self, interrupt_mask: u32) {
        let sdio = SdioDriver::sdio();
        sdio.mask().modify(|_, w: &mut stm32f103::sdio::mask::W| unsafe {
            w.bits(sdio.mask().read().bits() & !interrupt_mask)
        });
    }
    
    /// 获取状态
    /// 
    /// # 返回值
    /// SDIO状态
    pub unsafe fn get_status(&self) -> u32 {
        let sdio = SdioDriver::sdio();
        sdio.sta().read().bits()
    }
    
    /// 清除状态标志
    /// 
    /// # 参数
    /// * `flags` - 要清除的标志
    pub unsafe fn clear_status_flags(&self, flags: u32) {
        let sdio = SdioDriver::sdio();// 清除状态标志
        sdio.icr().write(|w: &mut stm32f103::sdio::icr::W| unsafe { w.bits(flags) });
    }
    
    /// 禁用SDIO
    pub unsafe fn disable(&self) {
        let sdio = SdioDriver::sdio();
        
        // 关闭SDIO电源
        sdio.power().write(|w: &mut stm32f103::sdio::power::W| unsafe { w.bits(0x00000000) });
    }
}

/// SDIO中断枚举
pub enum SdioInterrupt {
    CMDI = 1 << 0,     // 命令完成中断
    DATI = 1 << 1,     // 数据传输完成中断
    CCRCFAIL = 1 << 2, // 命令CRC失败中断
    DCRCFAIL = 1 << 3, // 数据CRC失败中断
    CTIMEOUT = 1 << 4, // 命令超时中断
    DTIMEOUT = 1 << 5, // 数据超时中断
    TXUNDERR = 1 << 6, // 发送下溢中断
    RXOVERR = 1 << 7,  // 接收溢出中断
    CMDREND = 1 << 8,  // 命令响应结束中断
    CMDSENT = 1 << 9,  // 命令发送中断
    DATAEND = 1 << 10, // 数据传输结束中断
    STBITERR = 1 << 11, // 起始位错误中断
    DBCKEND = 1 << 12, // 数据块结束中断
    CMDACT = 1 << 13,  // 命令激活中断
    TXACT = 1 << 14,   // 发送激活中断
    RXACT = 1 << 15,   // 接收激活中断
    TXFIFOHE = 1 << 16, // 发送FIFO半满中断
    RXFIFOHF = 1 << 17, // 接收FIFO半满中断
    TXFIFOF = 1 << 18, // 发送FIFO满中断
    RXFIFOF = 1 << 19, // 接收FIFO满中断
    TXFIFOE = 1 << 20, // 发送FIFO空中断
    RXFIFOE = 1 << 21, // 接收FIFO空中断
    BUSY = 1 << 22,    // 总线忙中断
    SDIOIT = 1 << 23,  // SDIO中断
}

/// 预定义的SDIO实例
pub const SDIO: SdioDriver = SdioDriver::new();