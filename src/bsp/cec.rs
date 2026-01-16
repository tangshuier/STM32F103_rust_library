//! CEC（消费电子控制）模块
//! 提供消费电子控制的封装和操作，用于实现电视和其他消费电子设备之间的通信

// 屏蔽未使用代码警告
#![allow(unused)]

// 使用内部生成的设备驱动库
use library::*;

/// CEC错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CecError {
    /// 初始化失败
    InitializationFailed,
    /// 无效的地址
    InvalidAddress,
    /// 发送失败
    SendFailed,
    /// 接收失败
    ReceiveFailed,
    /// CEC忙碌
    Busy,
    /// 传输错误
    TransmissionError,
    /// 接收错误
    ReceptionError,
    /// 位时间错误
    BitTimeError,
    /// 仲裁错误
    ArbitrationError,
    /// 位错误
    BitError,
    /// 溢出错误
    OverrunError,
    /// 未知错误
    UnknownError,
}

/// CEC状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CecStatus {
    /// CEC准备就绪
    Ready,
    /// CEC正在初始化
    Initializing,
    /// CEC出现错误
    Error,
    /// CEC正在发送
    Transmitting,
    /// CEC正在接收
    Receiving,
    /// CEC忙碌
    Busy,
    /// 位时间错误
    BitTimeError,
    /// 仲裁错误
    ArbitrationError,
    /// 位错误
    BitError,
    /// 溢出错误
    OverrunError,
}

/// CEC位时间配置枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CecBitTiming {
    /// 标准位时间
    Standard = 0,
    /// 快速位时间
    Fast = 1,
}

/// CEC消息结构体
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CecMessage {
    pub source: u8,    /// 源地址（0-14）
    pub destination: u8, /// 目的地址（0-14）
    pub opcode: u8,     /// 操作码（0-255）
    pub data: [u8; 14],  /// 数据（最多14字节）
    pub data_len: u8,   /// 数据长度（0-14）
}

impl CecMessage {
    /// 创建新的CEC消息
    pub const fn new(source: u8, destination: u8, opcode: u8, data: [u8; 14], data_len: u8) -> Self {
        Self {
            source: source & 0x0F,
            destination: destination & 0x0F,
            opcode,
            data,
            data_len: if data_len > 14 { 14 } else { data_len },
        }
    }
    
    /// 检查消息是否有效
    pub fn is_valid(&self) -> bool {
        self.source < 15 && self.destination < 15 && self.data_len <= 14
    }
}

/// CEC结构体
#[derive(Debug, Clone, Copy)]
pub struct Cec;

impl Cec {
    /// 创建新的CEC实例
    pub const fn new() -> Self {
        Self
    }
    
    /// 获取CEC寄存器块的不可变引用
    pub unsafe fn cec_reg(&self) -> &'static cec::RegisterBlock {
        &*(0x40011C00 as *const cec::RegisterBlock)
    }
    
    /// 获取CEC寄存器块的可变引用
    pub unsafe fn cec_reg_mut(&self) -> &'static mut cec::RegisterBlock {
        &mut *(0x40011C00 as *mut cec::RegisterBlock)
    }
    
    /// 获取RCC寄存器块的可变引用
    pub unsafe fn rcc_reg_mut(&self) -> &'static mut rcc::RegisterBlock {
        &mut *(0x40021000 as *mut rcc::RegisterBlock)
    }
    
    /// 初始化CEC
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// - 调用者必须确保提供的地址有效
    /// 
    /// # 参数
    /// - `bit_timing`：位时间配置
    /// - `own_address`：自己的CEC地址（0-14）
    pub unsafe fn init(&self, bit_timing: CecBitTiming, own_address: u8) -> Result<(), CecError> {
        // 检查参数范围
        if own_address >= 15 {
            return Err(CecError::InvalidAddress);
        }
        
        let rcc = self.rcc_reg_mut();
        let cec = self.cec_reg_mut();
        
        // 启用CEC时钟
        rcc.apb2enr().modify(|_, w| w
            .cecen().set_bit()
        );
        
        // 重置CEC
        rcc.apb2rstr().modify(|_, w| w
            .cecrst().set_bit()
        );
        rcc.apb2rstr().modify(|_, w| w
            .cecrst().clear_bit()
        );
        
        // 配置位时间
        cec.cfgr().modify(|_, w| w
            .btem().bit(bit_timing == CecBitTiming::Fast)
        );
        
        // 设置自己的地址
        cec.oar().modify(|_, w| {
            // 首先清除所有地址位
            let mut w = w;
            for i in 0..15 {
                w = w.aen(i).clear_bit();
            }
            // 设置自己的地址
            w.aen(own_address).set_bit()
        });
        
        // 启用CEC
        self.enable()
    }
    
    /// 启用CEC
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    pub unsafe fn enable(&self) -> Result<(), CecError> {
        let cec = self.cec_reg_mut();
        
        // 启用CEC
        cec.cr().modify(|_, w| w
            .cecen().set_bit()
        );
        
        Ok(())
    }
    
    /// 禁用CEC
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    pub unsafe fn disable(&self) -> Result<(), CecError> {
        let cec = self.cec_reg_mut();
        
        // 禁用CEC
        cec.cr().modify(|_, w| w
            .cecen().clear_bit()
        );
        
        Ok(())
    }
    
    /// 发送单个字节
    /// 
    /// # 安全
    /// - 调用者必须确保CEC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `data`：要发送的数据字节
    pub unsafe fn send_byte(&self, data: u8) -> Result<(), CecError> {
        let cec = self.cec_reg_mut();
        
        // 检查是否可以发送
        if !cec.sr().read().txe().bit_is_set() {
            return Err(CecError::Busy);
        }
        
        // 写入数据
        cec.dr().write(|w| w
            .dr().bits(data)
        );
        
        Ok(())
    }
    
    /// 接收单个字节
    /// 
    /// # 安全
    /// - 调用者必须确保CEC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// 接收到的数据字节
    pub unsafe fn receive_byte(&self) -> Result<u8, CecError> {
        let cec = self.cec_reg();
        
        // 检查是否有数据可读
        if !cec.sr().read().rxne().bit_is_set() {
            return Err(CecError::ReceiveFailed);
        }
        
        // 读取数据
        Ok(cec.dr().read().dr().bits())
    }
    
    /// 发送消息
    /// 
    /// # 安全
    /// - 调用者必须确保CEC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// - 调用者必须确保消息有效
    /// 
    /// # 参数
    /// - `message`：要发送的消息
    pub unsafe fn send_message(&self, message: &CecMessage) -> Result<(), CecError> {
        // 检查消息是否有效
        if !message.is_valid() {
            return Err(CecError::InvalidAddress);
        }
        
        // 构建消息头：源地址 << 4 | 目的地址
        let header = (message.source << 4) | message.destination;
        
        // 发送消息头
        self.send_byte(header)?;
        
        // 发送操作码
        self.send_byte(message.opcode)?;
        
        // 发送数据
        for i in 0..message.data_len {
            self.send_byte(message.data[i as usize])?;
        }
        
        Ok(())
    }
    
    /// 接收消息
    /// 
    /// # 安全
    /// - 调用者必须确保CEC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// 接收到的消息
    pub unsafe fn receive_message(&self) -> Result<CecMessage, CecError> {
        // 等待接收完成
        while !self.is_receiving_complete()? {
            // 检查是否有错误
            let status = self.get_status()?;
            if status != CecStatus::Receiving && status != CecStatus::Ready {
                return Err(CecError::ReceptionError);
            }
        }
        
        // 读取消息头
        let header = self.receive_byte()?;
        let source = (header >> 4) & 0x0F;
        let destination = header & 0x0F;
        
        // 读取操作码
        let opcode = self.receive_byte()?;
        
        // 读取数据
        let mut data = [0u8; 14];
        let mut data_len = 0;
        
        while self.can_receive()? && data_len < 14 {
            data[data_len as usize] = self.receive_byte()?;
            data_len += 1;
        }
        
        Ok(CecMessage {
            source,
            destination,
            opcode,
            data,
            data_len,
        })
    }
    
    /// 启用发送
    /// 
    /// # 安全
    /// - 调用者必须确保CEC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    pub unsafe fn start_transmission(&self) -> Result<(), CecError> {
        let cec = self.cec_reg_mut();
        
        // 检查是否正在发送
        if self.is_transmitting()? {
            return Err(CecError::Busy);
        }
        
        // 启用发送
        cec.cr().modify(|_, w| w
            .te().set_bit()
        );
        
        Ok(())
    }
    
    /// 停止发送
    /// 
    /// # 安全
    /// - 调用者必须确保CEC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    pub unsafe fn stop_transmission(&self) -> Result<(), CecError> {
        let cec = self.cec_reg_mut();
        
        // 禁用发送
        cec.cr().modify(|_, w| w
            .te().clear_bit()
        );
        
        Ok(())
    }
    
    /// 检查发送是否正在进行
    /// 
    /// # 安全
    /// - 调用者必须确保CEC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// 发送是否正在进行
    pub unsafe fn is_transmitting(&self) -> Result<bool, CecError> {
        let cec = self.cec_reg();
        Ok(cec.sr().read().txbs().bit_is_set())
    }
    
    /// 检查接收是否正在进行
    /// 
    /// # 安全
    /// - 调用者必须确保CEC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// 接收是否正在进行
    pub unsafe fn is_receiving(&self) -> Result<bool, CecError> {
        let cec = self.cec_reg();
        Ok(cec.sr().read().rxbs().bit_is_set())
    }
    
    /// 检查接收是否完成
    /// 
    /// # 安全
    /// - 调用者必须确保CEC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// 接收是否完成
    pub unsafe fn is_receiving_complete(&self) -> Result<bool, CecError> {
        let cec = self.cec_reg();
        Ok(cec.sr().read().eom().bit_is_set())
    }
    
    /// 检查是否可以发送
    /// 
    /// # 安全
    /// - 调用者必须确保CEC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// 是否可以发送
    pub unsafe fn can_send(&self) -> Result<bool, CecError> {
        let cec = self.cec_reg();
        Ok(cec.sr().read().txe().bit_is_set() && !cec.sr().read().txbs().bit_is_set())
    }
    
    /// 检查是否可以接收
    /// 
    /// # 安全
    /// - 调用者必须确保CEC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// 是否可以接收
    pub unsafe fn can_receive(&self) -> Result<bool, CecError> {
        let cec = self.cec_reg();
        Ok(cec.sr().read().rxne().bit_is_set())
    }
    
    /// 启用中断
    /// 
    /// # 安全
    /// - 调用者必须确保CEC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `interrupt_mask`：中断掩码
    pub unsafe fn enable_interrupts(&self, interrupt_mask: u32) -> Result<(), CecError> {
        let cec = self.cec_reg_mut();
        
        // 启用相应的中断
        cec.cr().modify(|_, w| {
            let mut w = w;
            if (interrupt_mask & (1 << 2)) != 0 { // TXE中断
                w = w.txie().set_bit();
            }
            if (interrupt_mask & (1 << 3)) != 0 { // RXNE中断
                w = w.rxie().set_bit();
            }
            if (interrupt_mask & (1 << 4)) != 0 { // BTE中断
                w = w.bteie().set_bit();
            }
            if (interrupt_mask & (1 << 5)) != 0 { // EOM中断
                w = w.eomie().set_bit();
            }
            if (interrupt_mask & (1 << 6)) != 0 { // ERRA中断
                w = w.erraie().set_bit();
            }
            if (interrupt_mask & (1 << 7)) != 0 { // ERRB中断
                w = w.errbie().set_bit();
            }
            if (interrupt_mask & (1 << 8)) != 0 { // RXOVR中断
                w = w.rxovrie().set_bit();
            }
            w
        });
        
        Ok(())
    }
    
    /// 禁用中断
    /// 
    /// # 安全
    /// - 调用者必须确保CEC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `interrupt_mask`：中断掩码
    pub unsafe fn disable_interrupts(&self, interrupt_mask: u32) -> Result<(), CecError> {
        let cec = self.cec_reg_mut();
        
        // 禁用相应的中断
        cec.cr().modify(|_, w| {
            let mut w = w;
            if (interrupt_mask & (1 << 2)) != 0 { // TXE中断
                w = w.txie().clear_bit();
            }
            if (interrupt_mask & (1 << 3)) != 0 { // RXNE中断
                w = w.rxie().clear_bit();
            }
            if (interrupt_mask & (1 << 4)) != 0 { // BTE中断
                w = w.bteie().clear_bit();
            }
            if (interrupt_mask & (1 << 5)) != 0 { // EOM中断
                w = w.eomie().clear_bit();
            }
            if (interrupt_mask & (1 << 6)) != 0 { // ERRA中断
                w = w.erraie().clear_bit();
            }
            if (interrupt_mask & (1 << 7)) != 0 { // ERRB中断
                w = w.errbie().clear_bit();
            }
            if (interrupt_mask & (1 << 8)) != 0 { // RXOVR中断
                w = w.rxovrie().clear_bit();
            }
            w
        });
        
        Ok(())
    }
    
    /// 获取中断标志
    /// 
    /// # 安全
    /// - 调用者必须确保CEC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// 中断标志
    pub unsafe fn get_interrupt_flags(&self) -> Result<u32, CecError> {
        let cec = self.cec_reg();
        let sr = cec.sr().read();
        
        let mut flags = 0u32;
        if sr.txe().bit_is_set() { flags |= 1 << 2; }
        if sr.rxne().bit_is_set() { flags |= 1 << 3; }
        if sr.bte().bit_is_set() { flags |= 1 << 4; }
        if sr.eom().bit_is_set() { flags |= 1 << 5; }
        if sr.erra().bit_is_set() { flags |= 1 << 6; }
        if sr.errb().bit_is_set() { flags |= 1 << 7; }
        if sr.rxovr().bit_is_set() { flags |= 1 << 8; }
        
        Ok(flags)
    }
    
    /// 清除中断标志
    /// 
    /// # 安全
    /// - 调用者必须确保CEC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `flags`：要清除的标志
    pub unsafe fn clear_interrupt_flags(&self, flags: u32) -> Result<(), CecError> {
        let cec = self.cec_reg_mut();
        
        // 清除相应的标志
        if (flags & (1 << 4)) != 0 { // BTE标志
            cec.sr().write(|w| w.bte().set_bit());
        }
        if (flags & (1 << 5)) != 0 { // EOM标志
            cec.sr().write(|w| w.eom().set_bit());
        }
        if (flags & (1 << 6)) != 0 { // ERRA标志
            cec.sr().write(|w| w.erra().set_bit());
        }
        if (flags & (1 << 7)) != 0 { // ERRB标志
            cec.sr().write(|w| w.errb().set_bit());
        }
        if (flags & (1 << 8)) != 0 { // RXOVR标志
            cec.sr().write(|w| w.rxovr().set_bit());
        }
        
        Ok(())
    }
    
    /// 获取CEC状态
    /// 
    /// # 安全
    /// - 调用者必须确保CEC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// CEC当前状态
    pub unsafe fn get_status(&self) -> Result<CecStatus, CecError> {
        let cec = self.cec_reg();
        let sr = cec.sr().read();
        
        // 检查错误状态
        if sr.bte().bit_is_set() {
            return Ok(CecStatus::BitTimeError);
        }
        
        if sr.erra().bit_is_set() {
            return Ok(CecStatus::ArbitrationError);
        }
        
        if sr.errb().bit_is_set() {
            return Ok(CecStatus::BitError);
        }
        
        if sr.rxovr().bit_is_set() {
            return Ok(CecStatus::OverrunError);
        }
        
        // 检查发送和接收状态
        if sr.txbs().bit_is_set() {
            return Ok(CecStatus::Transmitting);
        }
        
        if sr.rxbs().bit_is_set() {
            return Ok(CecStatus::Receiving);
        }
        
        // 检查CEC是否启用
        let cr = cec.cr().read();
        if !cr.cecen().bit_is_set() {
            return Ok(CecStatus::Initializing);
        }
        
        Ok(CecStatus::Ready)
    }
    
    /// 设置CEC滤波器
    /// 
    /// # 安全
    /// - 调用者必须确保CEC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `filter`：滤波器值
    pub unsafe fn set_filter(&self, filter: u8) -> Result<(), CecError> {
        let cec = self.cec_reg_mut();
        
        // 设置滤波器
        cec.cfgr().modify(|_, w| w
            .sft().bits(filter)
        );
        
        Ok(())
    }
}

/// CEC中断掩码常量
pub const CEC_IT_TXE: u32 = 1 << 2;    /// 发送寄存器空中断
pub const CEC_IT_RXNE: u32 = 1 << 3;   /// 接收寄存器非空中断
pub const CEC_IT_BTE: u32 = 1 << 4;    /// 位时间错误中断
pub const CEC_IT_EOM: u32 = 1 << 5;    /// 消息结束中断
pub const CEC_IT_ERRA: u32 = 1 << 6;   /// 仲裁错误中断
pub const CEC_IT_ERRB: u32 = 1 << 7;   /// 位错误中断
pub const CEC_IT_RXOVR: u32 = 1 << 8;  /// 接收溢出中断

/// CEC状态标志常量
pub const CEC_FLAG_TXBSY: u32 = 1 << 0;  /// 发送忙
pub const CEC_FLAG_RXBSY: u32 = 1 << 1;  /// 接收忙
pub const CEC_FLAG_TXE: u32 = 1 << 2;    /// 发送寄存器空
pub const CEC_FLAG_RXNE: u32 = 1 << 3;   /// 接收寄存器非空
pub const CEC_FLAG_BTE: u32 = 1 << 4;    /// 位时间错误
pub const CEC_FLAG_EOM: u32 = 1 << 5;    /// 消息结束
pub const CEC_FLAG_ERRA: u32 = 1 << 6;   /// 仲裁错误
pub const CEC_FLAG_ERRB: u32 = 1 << 7;   /// 位错误
pub const CEC_FLAG_RXOVR: u32 = 1 << 8;  /// 接收溢出

/// 预定义的CEC实例
pub const CEC: Cec = Cec::new();

/// 测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试CEC消息创建
    #[test]
    fn test_cec_message() {
        // 测试有效消息
        let valid_msg = CecMessage::new(1, 2, 0x82, [0x01, 0x02, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], 3);
        assert!(valid_msg.is_valid(), "有效消息应该通过验证");
        assert_eq!(valid_msg.source, 1, "源地址应该正确");
        assert_eq!(valid_msg.destination, 2, "目的地址应该正确");
        assert_eq!(valid_msg.opcode, 0x82, "操作码应该正确");
        assert_eq!(valid_msg.data_len, 3, "数据长度应该正确");
        
        // 测试无效消息
        let invalid_msg = CecMessage::new(15, 2, 0x82, [0x00; 14], 3); // 无效的源地址
        assert!(!invalid_msg.is_valid(), "无效消息应该不通过验证");
        
        let invalid_msg = CecMessage::new(1, 2, 0x82, [0x00; 14], 15); // 无效的数据长度
        assert!(!invalid_msg.is_valid(), "无效数据长度消息应该不通过验证");
    }
    
    /// 测试CEC状态获取
    #[test]
    fn test_cec_status() {
        let cec = Cec::new();
        
        // 初始化CEC
        unsafe {
            let init_result = cec.init(CecBitTiming::Standard, 1);
            assert!(init_result.is_ok(), "CEC初始化应该成功");
            
            let status = cec.get_status();
            assert!(status.is_ok(), "获取CEC状态应该成功");
            assert_eq!(status.unwrap(), CecStatus::Ready, "CEC状态应该是Ready");
        }
    }
    
    /// 测试CEC中断配置
    #[test]
    fn test_cec_interrupts() {
        let cec = Cec::new();
        
        // 初始化CEC
        unsafe {
            let init_result = cec.init(CecBitTiming::Standard, 1);
            assert!(init_result.is_ok(), "CEC初始化应该成功");
            
            // 启用中断
            let enable_result = cec.enable_interrupts(CEC_IT_TXE | CEC_IT_RXNE);
            assert!(enable_result.is_ok(), "启用CEC中断应该成功");
            
            // 禁用中断
            let disable_result = cec.disable_interrupts(CEC_IT_TXE | CEC_IT_RXNE);
            assert!(disable_result.is_ok(), "禁用CEC中断应该成功");
        }
    }
}
