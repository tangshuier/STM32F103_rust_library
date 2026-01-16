//! SDIO（安全数字输入/输出）模块
//! 提供安全数字输入/输出的封装和操作，用于连接SD卡和SDIO设备

// 屏蔽未使用代码警告
#![allow(unused)]

// 使用内部生成的设备驱动库
use library::*;

/// SDIO错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdioError {
    /// 初始化失败
    InitializationFailed,
    /// 参数无效
    InvalidParameter,
    /// 命令执行失败
    CommandFailed,
    /// 响应超时
    ResponseTimeout,
    /// 数据传输失败
    DataTransferFailed,
    /// CRC错误
    CrcError,
    /// 超时错误
    TimeoutError,
    /// FIFO错误
    FifoError,
    /// SDIO忙碌
    Busy,
    /// 电源错误
    PowerError,
    /// 重置失败
    ResetFailed,
    /// 未知错误
    UnknownError,
}

/// SDIO状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdioStatus {
    /// SDIO准备就绪
    Ready,
    /// SDIO正在初始化
    Initializing,
    /// SDIO出现错误
    Error,
    /// 命令执行中
    CommandInProgress,
    /// 数据传输中
    DataTransfer,
    /// SDIO忙碌
    Busy,
    /// 命令完成
    CommandComplete,
    /// 数据传输完成
    DataComplete,
    /// SDIO禁用
    Disabled,
}

/// SDIO时钟频率枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdioClockFreq {
    Freq400kHz = 0,    // 400kHz (初始化频率)
    Freq25MHz = 1,     // 25MHz
    Freq50MHz = 2,     // 50MHz
}

/// SDIO响应类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdioResponseType {
    NoResponse = 0,    // 无响应
    ShortResponse = 1, // 短响应 (R1, R1b, R2, R3, R6)
    LongResponse = 2,  // 长响应 (R7)
}

/// SDIO数据传输宽度枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdioDataWidth {
    Width1b = 0,     // 1位
    Width4b = 1,     // 4位
    Width8b = 2,     // 8位
}

/// SDIO中断枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// SDIO结构体
#[derive(Debug, Clone, Copy)]
pub struct SdioDriver;

impl SdioDriver {
    /// 创建新的SDIO实例
    pub const fn new() -> Self {
        Self
    }
    
    /// 获取SDIO寄存器块的不可变引用
    pub unsafe fn sdio_reg(&self) -> &'static sdio::RegisterBlock {
        &*(0x40012C00 as *const sdio::RegisterBlock)
    }
    
    /// 获取SDIO寄存器块的可变引用
    pub unsafe fn sdio_reg_mut(&self) -> &'static mut sdio::RegisterBlock {
        &mut *(0x40012C00 as *mut sdio::RegisterBlock)
    }
    
    /// 获取RCC寄存器块的可变引用
    pub unsafe fn rcc_reg_mut(&self) -> &'static mut rcc::RegisterBlock {
        &mut *(0x40021000 as *mut rcc::RegisterBlock)
    }
    
    /// 初始化SDIO
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// - 调用者必须确保SDIO时钟已经启用
    /// 
    /// # 参数
    /// - `clock_freq`：时钟频率
    /// 
    /// # 返回值
    /// - Ok(())：SDIO初始化成功
    /// - Err(SdioError)：SDIO初始化失败
    pub unsafe fn init(&self, clock_freq: SdioClockFreq) -> Result<(), SdioError> {
        let sdio = self.sdio_reg_mut();
        
        // 关闭SDIO电源
        sdio.power().write(|w| unsafe { w.bits(0x00000000) });
        
        // 重置SDIO
        self.reset()?;
        
        // 打开SDIO电源
        sdio.power().write(|w| unsafe { w.bits(0x00000003) });
        
        // 配置时钟频率
        self.set_clock_frequency(clock_freq)?;
        
        // 启用SDIO
        sdio.power().write(|w| unsafe { w.bits(0x00000003) });
        
        Ok(())
    }
    
    /// 重置SDIO
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(())：SDIO重置成功
    /// - Err(SdioError)：SDIO重置失败
    pub unsafe fn reset(&self) -> Result<(), SdioError> {
        let sdio = self.sdio_reg_mut();
        
        // 重置命令通道（通过重置CPSMEN位）
        sdio.cmd().modify(|_, w| w
            .cpsmen().clear_bit()
        );
        
        // 等待命令通道重置完成
        let mut timeout = 1000;
        while !sdio.cmd().read().cpsmen().bit() {
            timeout -= 1;
            if timeout == 0 {
                return Err(SdioError::ResetFailed);
            }
            core::hint::spin_loop();
        }
        
        // 重置数据通道（通过重置DTEN位）
        sdio.dctrl().modify(|_, w| w
            .dten().clear_bit()
        );
        
        // 等待数据通道重置完成
        timeout = 1000;
        while !sdio.dctrl().read().dten().bit() {
            timeout -= 1;
            if timeout == 0 {
                return Err(SdioError::ResetFailed);
            }
            core::hint::spin_loop();
        }
        
        Ok(())
    }
    
    /// 设置SDIO时钟频率
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `clock_freq`：时钟频率
    /// 
    /// # 返回值
    /// - Ok(())：时钟频率设置成功
    /// - Err(SdioError)：时钟频率设置失败
    pub unsafe fn set_clock_frequency(&self, clock_freq: SdioClockFreq) -> Result<(), SdioError> {
        let sdio = self.sdio_reg_mut();
        
        // 禁用时钟
        sdio.clkcr().modify(|_, w| w
            .clken().clear_bit()
        );
        
        // 设置时钟分频系数
        match clock_freq {
            SdioClockFreq::Freq400kHz => {
                // 400kHz = 25MHz / 62.5, 取63
                sdio.clkcr().write(|w| unsafe { w.bits((62 << 0) | (1 << 7)) });
            }
            SdioClockFreq::Freq25MHz => {
                // 25MHz = 50MHz / 2
                sdio.clkcr().write(|w| unsafe { w.bits((1 << 0) | (1 << 7)) });
            }
            SdioClockFreq::Freq50MHz => {
                // 50MHz = 50MHz / 1
                sdio.clkcr().write(|w| unsafe { w.bits((0 << 0) | (1 << 7)) });
            }
        }
        
        // 启用时钟
        sdio.clkcr().modify(|_, w| w
            .clken().set_bit()
        );
        
        Ok(())
    }
    
    /// 发送命令
    /// 
    /// # 安全
    /// - 调用者必须确保SDIO已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `cmd`：命令号
    /// - `arg`：命令参数
    /// - `resp_type`：响应类型
    /// 
    /// # 返回值
    /// - Ok(())：命令发送成功
    /// - Err(SdioError)：命令发送失败
    pub unsafe fn send_command(&self, cmd: u8, arg: u32, resp_type: SdioResponseType) -> Result<(), SdioError> {
        let sdio = self.sdio_reg_mut();
        
        // 检查命令号范围
        if cmd > 63 {
            return Err(SdioError::InvalidParameter);
        }
        
        // 设置命令参数
        sdio.arg().write(|w| unsafe { w.bits(arg) });
        
        // 配置命令
        let mut cmd_reg = (cmd as u32) << 0;
        cmd_reg |= (resp_type as u32) << 6;
        cmd_reg |= (1 << 10); // 启动命令
        
        sdio.cmd().write(|w| unsafe { w.bits(cmd_reg) });
        
        // 等待命令完成
        let mut timeout = 10000;
        while !sdio.sta().read().cmdrend().bit() {
            timeout -= 1;
            if timeout == 0 {
                return Err(SdioError::TimeoutError);
            }
            core::hint::spin_loop();
        }
        
        // 检查命令状态
        let status = sdio.sta().read().bits();
        if (status & (1 << 2)) != 0 { // CCRCFAIL
            return Err(SdioError::CrcError);
        }
        if (status & (1 << 4)) != 0 { // CTIMEOUT
            return Err(SdioError::TimeoutError);
        }
        
        // 清除命令完成标志
        sdio.icr().write(|w| unsafe { w.bits(1 << 0) });
        
        Ok(())
    }
    
    /// 读取响应
    /// 
    /// # 安全
    /// - 调用者必须确保SDIO已经初始化
    /// - 调用者必须确保命令已经完成
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `resp_type`：响应类型
    /// 
    /// # 返回值
    /// - Ok([u32; 4])：响应数据
    /// - Err(SdioError)：读取响应失败
    pub unsafe fn read_response(&self, resp_type: SdioResponseType) -> Result<[u32; 4], SdioError> {
        let sdio = self.sdio_reg();
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
            SdioResponseType::NoResponse => {
                return Err(SdioError::InvalidParameter);
            }
        }
        
        Ok(resp)
    }
    
    /// 配置数据传输
    /// 
    /// # 安全
    /// - 调用者必须确保SDIO已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `data_width`：数据传输宽度
    /// - `block_size`：块大小 (字节)
    /// - `block_count`：块数量
    /// 
    /// # 返回值
    /// - Ok(())：数据传输配置成功
    /// - Err(SdioError)：数据传输配置失败
    pub unsafe fn configure_data_transfer(
        &self,
        data_width: SdioDataWidth,
        block_size: u16,
        block_count: u16
    ) -> Result<(), SdioError> {
        let sdio = self.sdio_reg_mut();
        
        // 检查块大小范围
        if block_size < 1 || block_size > 512 {
            return Err(SdioError::InvalidParameter);
        }
        
        // 设置数据长度
        sdio.dlen().write(|w| unsafe { w.bits((block_size * block_count) as u32) });
        
        // 配置数据控制寄存器
        let mut dctrl = 0x00000000;
        dctrl |= (data_width as u32) << 0;
        dctrl |= (1 << 4); // 启用数据传输
        dctrl |= (1 << 5); // 块传输模式
        
        sdio.dctrl().write(|w| unsafe { w.bits(dctrl) });
        
        Ok(())
    }
    
    /// 启动数据传输
    /// 
    /// # 安全
    /// - 调用者必须确保SDIO已经初始化
    /// - 调用者必须确保数据传输已经配置
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(())：数据传输启动成功
    /// - Err(SdioError)：数据传输启动失败
    pub unsafe fn start_data_transfer(&self) -> Result<(), SdioError> {
        let sdio = self.sdio_reg_mut();
        
        // 启动数据传输
        sdio.dctrl().modify(|_, w| w
            .dten().set_bit()
        );
        
        Ok(())
    }
    
    /// 等待数据传输完成
    /// 
    /// # 安全
    /// - 调用者必须确保SDIO已经初始化
    /// - 调用者必须确保数据传输已经启动
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(())：数据传输完成
    /// - Err(SdioError)：数据传输失败
    pub unsafe fn wait_for_data_transfer_complete(&self) -> Result<(), SdioError> {
        let sdio = self.sdio_reg_mut();
        
        // 等待数据传输完成
        let mut timeout = 100000;
        while !sdio.sta().read().dataend().bit() {
            timeout -= 1;
            if timeout == 0 {
                return Err(SdioError::TimeoutError);
            }
            core::hint::spin_loop();
        }
        
        // 检查数据传输状态
        let status = sdio.sta().read().bits();
        if (status & (1 << 3)) != 0 { // DCRCFAIL
            return Err(SdioError::CrcError);
        }
        if (status & (1 << 5)) != 0 { // DTIMEOUT
            return Err(SdioError::TimeoutError);
        }
        if (status & (1 << 6)) != 0 { // TXUNDERR
            return Err(SdioError::FifoError);
        }
        if (status & (1 << 7)) != 0 { // RXOVERR
            return Err(SdioError::FifoError);
        }
        
        // 清除数据传输完成标志
        sdio.icr().write(|w| unsafe { w.bits(1 << 1) });
        
        Ok(())
    }
    
    /// 读取数据
    /// 
    /// # 安全
    /// - 调用者必须确保SDIO已经初始化
    /// - 调用者必须确保数据传输已经配置
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `buffer`：数据缓冲区
    /// - `length`：数据长度 (字节)
    /// 
    /// # 返回值
    /// - Ok(())：数据读取成功
    /// - Err(SdioError)：数据读取失败
    pub unsafe fn read_data(&self, buffer: &mut [u8], length: usize) -> Result<(), SdioError> {
        let sdio = self.sdio_reg();
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
            } else {
                // 检查FIFO错误
                let status = sdio.sta().read().bits();
                if (status & (1 << 7)) != 0 { // RXOVERR
                    return Err(SdioError::FifoError);
                }
                if (status & (1 << 5)) != 0 { // DTIMEOUT
                    return Err(SdioError::TimeoutError);
                }
            }
        }
        
        Ok(())
    }
    
    /// 写入数据
    /// 
    /// # 安全
    /// - 调用者必须确保SDIO已经初始化
    /// - 调用者必须确保数据传输已经配置
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `buffer`：数据缓冲区
    /// - `length`：数据长度 (字节)
    /// 
    /// # 返回值
    /// - Ok(())：数据写入成功
    /// - Err(SdioError)：数据写入失败
    pub unsafe fn write_data(&self, buffer: &[u8], length: usize) -> Result<(), SdioError> {
        let sdio = self.sdio_reg();
        let mut index = 0;
        let mut remaining = length;
        
        while remaining > 0 {
            if sdio.sta().read().txfifohe().bit() {
                if remaining >= 4 {
                    let data = u32::from_le_bytes(buffer[index..index+4].try_into().unwrap());
                    self.sdio_reg_mut().fifo().write(|w| unsafe { w.bits(data) });
                    index += 4;
                    remaining -= 4;
                } else {
                    let mut padding = [0u8; 4];
                    padding[0..remaining].copy_from_slice(&buffer[index..index+remaining]);
                    let data = u32::from_le_bytes(padding);
                    self.sdio_reg_mut().fifo().write(|w| unsafe { w.bits(data) });
                    index += remaining;
                    remaining = 0;
                }
            } else {
                // 检查FIFO错误
                let status = sdio.sta().read().bits();
                if (status & (1 << 6)) != 0 { // TXUNDERR
                    return Err(SdioError::FifoError);
                }
                if (status & (1 << 5)) != 0 { // DTIMEOUT
                    return Err(SdioError::TimeoutError);
                }
            }
        }
        
        Ok(())
    }
    
    /// 启用中断
    /// 
    /// # 安全
    /// - 调用者必须确保SDIO已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `interrupt`：中断类型
    /// 
    /// # 返回值
    /// - Ok(())：中断启用成功
    /// - Err(SdioError)：中断启用失败
    pub unsafe fn enable_interrupt(&self, interrupt: SdioInterrupt) -> Result<(), SdioError> {
        let sdio = self.sdio_reg_mut();
        sdio.mask().modify(|_, w| unsafe {
            w.bits(sdio.mask().read().bits() | (interrupt as u32))
        });
        
        Ok(())
    }
    
    /// 禁用中断
    /// 
    /// # 安全
    /// - 调用者必须确保SDIO已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `interrupt`：中断类型
    /// 
    /// # 返回值
    /// - Ok(())：中断禁用成功
    /// - Err(SdioError)：中断禁用失败
    pub unsafe fn disable_interrupt(&self, interrupt: SdioInterrupt) -> Result<(), SdioError> {
        let sdio = self.sdio_reg_mut();
        sdio.mask().modify(|_, w| unsafe {
            w.bits(sdio.mask().read().bits() & !(interrupt as u32))
        });
        
        Ok(())
    }
    
    /// 获取状态
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(SdioStatus)：SDIO当前状态
    /// - Err(SdioError)：获取状态失败
    pub unsafe fn get_status(&self) -> Result<SdioStatus, SdioError> {
        let sdio = self.sdio_reg();
        let status = sdio.sta().read().bits();
        
        // 检查错误状态
        if (status & (1 << 2)) != 0 || // CCRCFAIL
           (status & (1 << 3)) != 0 || // DCRCFAIL
           (status & (1 << 4)) != 0 || // CTIMEOUT
           (status & (1 << 5)) != 0 || // DTIMEOUT
           (status & (1 << 6)) != 0 || // TXUNDERR
           (status & (1 << 7)) != 0 {   // RXOVERR
            return Ok(SdioStatus::Error);
        }
        
        // 检查命令状态
        if (status & (1 << 0)) != 0 { // CMDI
            return Ok(SdioStatus::CommandComplete);
        }
        
        // 检查数据状态
        if (status & (1 << 1)) != 0 { // DATI
            return Ok(SdioStatus::DataComplete);
        }
        
        // 检查忙碌状态
        if (status & (1 << 22)) != 0 { // BUSY
            return Ok(SdioStatus::Busy);
        }
        
        // 检查命令执行中
        if (status & (1 << 13)) != 0 { // CMDACT
            return Ok(SdioStatus::CommandInProgress);
        }
        
        // 检查数据传输中
        if (status & (1 << 14)) != 0 || (status & (1 << 15)) != 0 { // TXACT or RXACT
            return Ok(SdioStatus::DataTransfer);
        }
        
        // 检查电源状态
        let power_status = sdio.power().read().bits();
        if power_status == 0 {
            return Ok(SdioStatus::Disabled);
        }
        
        return Ok(SdioStatus::Ready);
    }
    
    /// 清除状态标志
    /// 
    /// # 安全
    /// - 调用者必须确保SDIO已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `flags`：要清除的标志
    /// 
    /// # 返回值
    /// - Ok(())：状态标志清除成功
    /// - Err(SdioError)：状态标志清除失败
    pub unsafe fn clear_status_flags(&self, flags: u32) -> Result<(), SdioError> {
        let sdio = self.sdio_reg_mut();
        // 清除状态标志
        sdio.icr().write(|w| unsafe { w.bits(flags) });
        
        Ok(())
    }
    
    /// 禁用SDIO
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(())：SDIO禁用成功
    /// - Err(SdioError)：SDIO禁用失败
    pub unsafe fn disable(&self) -> Result<(), SdioError> {
        let sdio = self.sdio_reg_mut();
        
        // 关闭SDIO电源
        sdio.power().write(|w| unsafe { w.bits(0x00000000) });
        
        Ok(())
    }
}

/// 预定义的SDIO实例
pub const SDIO: SdioDriver = SdioDriver::new();

/// 测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试SDIO初始化和状态获取
    #[test]
    fn test_sdio_init_status() {
        let sdio = SdioDriver::new();
        
        // 初始化SDIO
        unsafe {
            let init_result = sdio.init(SdioClockFreq::Freq400kHz);
            assert!(init_result.is_ok(), "SDIO初始化应该成功");
            
            let status = sdio.get_status();
            assert!(status.is_ok(), "获取SDIO状态应该成功");
            assert_eq!(status.unwrap(), SdioStatus::Ready, "SDIO初始状态应该是Ready");
        }
    }
    
    /// 测试SDIO重置
    #[test]
    fn test_sdio_reset() {
        let sdio = SdioDriver::new();
        
        unsafe {
            let init_result = sdio.init(SdioClockFreq::Freq400kHz);
            assert!(init_result.is_ok(), "SDIO初始化应该成功");
            
            let reset_result = sdio.reset();
            assert!(reset_result.is_ok(), "SDIO重置应该成功");
            
            let status = sdio.get_status();
            assert!(status.is_ok(), "获取SDIO状态应该成功");
        }
    }
    
    /// 测试SDIO时钟频率设置
    #[test]
    fn test_sdio_clock_frequency() {
        let sdio = SdioDriver::new();
        
        unsafe {
            let init_result = sdio.init(SdioClockFreq::Freq400kHz);
            assert!(init_result.is_ok(), "SDIO初始化应该成功");
            
            // 设置为25MHz
            let set_result = sdio.set_clock_frequency(SdioClockFreq::Freq25MHz);
            assert!(set_result.is_ok(), "设置SDIO时钟频率为25MHz应该成功");
            
            // 设置为50MHz
            let set_result = sdio.set_clock_frequency(SdioClockFreq::Freq50MHz);
            assert!(set_result.is_ok(), "设置SDIO时钟频率为50MHz应该成功");
            
            // 设置回400kHz
            let set_result = sdio.set_clock_frequency(SdioClockFreq::Freq400kHz);
            assert!(set_result.is_ok(), "设置SDIO时钟频率为400kHz应该成功");
        }
    }
    
    /// 测试SDIO中断控制
    #[test]
    fn test_sdio_interrupt_control() {
        let sdio = SdioDriver::new();
        
        unsafe {
            let init_result = sdio.init(SdioClockFreq::Freq400kHz);
            assert!(init_result.is_ok(), "SDIO初始化应该成功");
            
            // 启用命令完成中断
            let enable_result = sdio.enable_interrupt(SdioInterrupt::CMDI);
            assert!(enable_result.is_ok(), "启用SDIO中断应该成功");
            
            // 禁用命令完成中断
            let disable_result = sdio.disable_interrupt(SdioInterrupt::CMDI);
            assert!(disable_result.is_ok(), "禁用SDIO中断应该成功");
        }
    }
    
    /// 测试SDIO数据传输配置
    #[test]
    fn test_sdio_data_transfer_config() {
        let sdio = SdioDriver::new();
        
        unsafe {
            let init_result = sdio.init(SdioClockFreq::Freq400kHz);
            assert!(init_result.is_ok(), "SDIO初始化应该成功");
            
            // 配置数据传输
            let config_result = sdio.configure_data_transfer(
                SdioDataWidth::Width4b,
                512, // 512字节块
                1    // 1个块
            );
            assert!(config_result.is_ok(), "配置SDIO数据传输应该成功");
        }
    }
}
