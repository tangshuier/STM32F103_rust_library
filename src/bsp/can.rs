//! CAN（控制器局域网）模块
//! 提供控制器局域网的封装和操作，用于实现可靠的分布式通信

// 屏蔽未使用代码警告
#![allow(unused)]

// 使用内部生成的设备驱动库
use library::*;

/// CAN错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanError {
    /// 初始化失败
    InitializationFailed,
    /// 无效的模式
    InvalidMode,
    /// 无效的位时序
    InvalidBitTiming,
    /// 无效的过滤器编号
    InvalidFilterNumber,
    /// 发送失败
    SendFailed,
    /// 接收失败
    ReceiveFailed,
    /// 总线离线
    BusOff,
    /// 错误被动
    ErrorPassive,
    /// 错误警告
    ErrorWarning,
    /// 未知错误
    UnknownError,
}

/// CAN状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanStatus {
    /// CAN准备就绪
    Ready,
    /// CAN正在初始化
    Initializing,
    /// CAN出现错误
    Error,
    /// CAN忙碌
    Busy,
    /// 总线离线
    BusOff,
    /// 错误被动
    ErrorPassive,
    /// 错误警告
    ErrorWarning,
}

/// CAN模式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanMode {
    /// 正常模式
    Normal = 0,
    /// 回环模式
    LoopBack = 1,
    /// 静默模式
    Silent = 2,
    /// 静默回环模式
    SilentLoopBack = 3,
}

/// CAN位时序结构体
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanBitTiming {
    pub prescaler: u16,      /// 预分频系数，范围：1-1024
    pub time_segment_1: u8,  /// 时间段1，范围：1-16
    pub time_segment_2: u8,  /// 时间段2，范围：1-8
    pub sjw: u8,             /// 同步跳转宽度，范围：1-4
}

impl CanBitTiming {
    /// 创建新的CAN位时序配置
    pub const fn new(prescaler: u16, time_segment_1: u8, time_segment_2: u8, sjw: u8) -> Self {
        Self {
            prescaler,
            time_segment_1,
            time_segment_2,
            sjw,
        }
    }
    
    /// 检查位时序配置是否有效
    pub fn is_valid(&self) -> bool {
        self.prescaler >= 1 && self.prescaler <= 1024 &&
        self.time_segment_1 >= 1 && self.time_segment_1 <= 16 &&
        self.time_segment_2 >= 1 && self.time_segment_2 <= 8 &&
        self.sjw >= 1 && self.sjw <= 4
    }
}

/// CAN过滤器模式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanFilterMode {
    /// 掩码模式
    MaskMode = 0,
    /// 列表模式
    ListMode = 1,
}

/// CAN过滤器尺度枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanFilterScale {
    /// 16位过滤器
    Scale16Bit = 0,
    /// 32位过滤器
    Scale32Bit = 1,
}

/// CAN过滤器FIFO分配枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanFilterFifo {
    /// 分配到FIFO0
    Fifo0 = 0,
    /// 分配到FIFO1
    Fifo1 = 1,
}

/// CAN消息结构体
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanMessage {
    pub id: u32,         /// 消息ID（标准ID: 0-0x7FF，扩展ID: 0-0x1FFFFFFF）
    pub is_extended: bool, /// 是否为扩展ID
    pub rtr: bool,        /// 是否为远程传输请求
    pub dlc: u8,          /// 数据长度（0-8）
    pub data: [u8; 8],     /// 数据内容
}

impl CanMessage {
    /// 创建新的CAN消息
    pub const fn new(id: u32, is_extended: bool, rtr: bool, dlc: u8, data: [u8; 8]) -> Self {
        Self {
            id,
            is_extended,
            rtr,
            dlc: if dlc > 8 { 8 } else { dlc },
            data,
        }
    }
    
    /// 创建新的标准ID消息
    pub const fn new_standard(id: u16, rtr: bool, dlc: u8, data: [u8; 8]) -> Self {
        Self {
            id: id as u32,
            is_extended: false,
            rtr,
            dlc: if dlc > 8 { 8 } else { dlc },
            data,
        }
    }
    
    /// 创建新的扩展ID消息
    pub const fn new_extended(id: u32, rtr: bool, dlc: u8, data: [u8; 8]) -> Self {
        Self {
            id: id & 0x1FFFFFFF, // 确保扩展ID在有效范围内
            is_extended: true,
            rtr,
            dlc: if dlc > 8 { 8 } else { dlc },
            data,
        }
    }
    
    /// 检查消息是否有效
    pub fn is_valid(&self) -> bool {
        if self.is_extended {
            // 扩展ID范围：0-0x1FFFFFFF
            self.id <= 0x1FFFFFFF
        } else {
            // 标准ID范围：0-0x7FF
            self.id <= 0x7FF
        } && self.dlc <= 8
    }
}

/// CAN结构体
#[derive(Debug, Clone, Copy)]
pub struct Can {
    _marker: core::marker::PhantomData<()>,
}

impl Can {
    /// 创建新的CAN实例
    pub const fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
    
    /// 获取CAN1寄存器块的不可变引用
    pub unsafe fn can1_reg(&self) -> &'static can1::RegisterBlock {
        &*(0x40006400 as *const can1::RegisterBlock)
    }
    
    /// 获取CAN1寄存器块的可变引用
    pub unsafe fn can1_reg_mut(&self) -> &'static mut can1::RegisterBlock {
        &mut *(0x40006400 as *mut can1::RegisterBlock)
    }
    
    /// 获取RCC寄存器块的可变引用
    pub unsafe fn rcc_reg_mut(&self) -> &'static mut rcc::RegisterBlock {
        &mut *(0x40021000 as *mut rcc::RegisterBlock)
    }
    
    /// 获取CAN过滤器寄存器块的可变引用
    pub unsafe fn can_filter_reg_mut(&self) -> &'static mut can1::RegisterBlock {
        // CAN过滤器寄存器与CAN1共享相同的地址空间
        &mut *(0x40006400 as *mut can1::RegisterBlock)
    }
    
    /// 初始化CAN
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// - 调用者必须确保提供的位时序配置有效
    /// 
    /// # 参数
    /// - `mode`：CAN工作模式
    /// - `bit_timing`：位时序配置
    pub unsafe fn init(&self, mode: CanMode, bit_timing: CanBitTiming) -> Result<(), CanError> {
        // 检查位时序配置是否有效
        if !bit_timing.is_valid() {
            return Err(CanError::InvalidBitTiming);
        }
        
        let rcc = self.rcc_reg_mut();
        let can1 = self.can1_reg_mut();
        
        // 启用CAN时钟
        rcc.apb1enr().modify(|_, w| w
            .canen().set_bit()
        );
        
        // 重置CAN
        rcc.apb1rstr().modify(|_, w| w
            .canrst().set_bit()
        );
        rcc.apb1rstr().modify(|_, w| w
            .canrst().clear_bit()
        );
        
        // 进入初始化模式
        can1.mcr().modify(|_, w| w
            .inrq().set_bit()
        );
        
        // 等待初始化模式确认
        while !can1.msr().read().inak().bit_is_set() {
            // 等待初始化确认
        }
        
        // 设置工作模式
        can1.btr().modify(|_, w| w
            .silm().bit((mode as u8 >> 1) & 0x01 != 0)
            .lbkm().bit((mode as u8 & 0x01) != 0)
            .ts2().bits(bit_timing.time_segment_2 - 1)
            .ts1().bits(bit_timing.time_segment_1 - 1)
            .sjw().bits(bit_timing.sjw - 1)
            .brp().bits(bit_timing.prescaler - 1)
        );
        
        // 退出初始化模式
        can1.mcr().modify(|_, w| w
            .inrq().clear_bit()
        );
        
        // 等待正常模式确认
        while can1.msr().read().inak().bit_is_set() {
            // 等待正常模式确认
        }
        
        Ok(())
    }
    
    /// 配置过滤器
    /// 
    /// # 安全
    /// - 调用者必须确保CAN已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `filter_number`：过滤器编号（0-13）
    /// - `mode`：过滤器模式
    /// - `scale`：过滤器尺度
    /// - `fifo`：FIFO分配
    /// - `filter_id`：过滤器ID
    /// - `filter_mask`：过滤器掩码
    /// - `activate`：是否激活过滤器
    pub unsafe fn configure_filter(
        &self, 
        filter_number: u8, 
        mode: CanFilterMode, 
        scale: CanFilterScale, 
        fifo: CanFilterFifo, 
        filter_id: u32, 
        filter_mask: u32, 
        activate: bool
    ) -> Result<(), CanError> {
        // 检查过滤器编号是否有效
        if filter_number > 13 {
            return Err(CanError::InvalidFilterNumber);
        }
        
        let can1 = self.can1_reg_mut();
        
        // 进入过滤器初始化模式
        can1.fmr().modify(|_, w| w
            .finIT().set_bit()
        );
        
        // 设置过滤器编号
        can1.fm1r().modify(|_, w| w
            .fm1r().bits((mode as u8) << filter_number)
        );
        
        // 设置过滤器尺度
        can1.fs1r().modify(|_, w| w
            .fs1r().bits((scale as u8) << filter_number)
        );
        
        // 设置FIFO分配
        can1.ffar().modify(|_, w| w
            .ffar().bits((fifo as u8) << filter_number)
        );
        
        // 配置过滤器ID和掩码
        if scale == CanFilterScale::Scale32Bit {
            // 32位模式
            let filter_addr = 0x40006400 + 0x20 + (filter_number * 8) as u32;
            let filter_id_reg = filter_addr as *mut u32;
            let filter_mask_reg = (filter_addr + 4) as *mut u32;
            
            // 写入ID和掩码
            *filter_id_reg = filter_id;
            *filter_mask_reg = filter_mask;
        } else {
            // 16位模式
            let filter_addr = 0x40006400 + 0x20 + (filter_number * 8) as u32;
            let filter_id_reg = filter_addr as *mut u32;
            let filter_mask_reg = (filter_addr + 4) as *mut u32;
            
            // 写入ID和掩码（低16位）
            *filter_id_reg = filter_id & 0xFFFF;
            *filter_mask_reg = filter_mask & 0xFFFF;
        }
        
        // 激活过滤器
        if activate {
            can1.fa1r().modify(|_, w| w
                .fa1r().bits(1 << filter_number)
            );
        } else {
            can1.fa1r().modify(|_, w| w
                .fa1r().bits(0 << filter_number)
            );
        }
        
        // 退出过滤器初始化模式
        can1.fmr().modify(|_, w| w
            .finIT().clear_bit()
        );
        
        Ok(())
    }
    
    /// 发送消息
    /// 
    /// # 安全
    /// - 调用者必须确保CAN已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `message`：要发送的消息
    /// 
    /// # 返回值
    /// 发送是否成功
    pub unsafe fn send_message(&self, message: &CanMessage) -> Result<bool, CanError> {
        // 检查消息是否有效
        if !message.is_valid() {
            return Err(CanError::InvalidBitTiming);
        }
        
        let can1 = self.can1_reg_mut();
        
        // 查找空的发送邮箱
        let tme_bits = can1.tsr().read().tme().bits();
        let mailboxes = [tme_bits & 0x01, (tme_bits >> 1) & 0x01, (tme_bits >> 2) & 0x01];
        
        let mut mailbox_index: Option<usize> = None;
        for (i, &mailbox) in mailboxes.iter().enumerate() {
            if mailbox != 0 {
                mailbox_index = Some(i);
                break;
            }
        }
        
        if let Some(index) = mailbox_index {
            // 获取对应的发送邮箱寄存器
            let tir = &can1.tmir()[index];
            let tdr = &can1.tmdr()[index];
            let tdlr = &can1.tmdt()[index];
            let tdhr = &can1.tmdt()[index + 1];
            
            // 配置ID和消息类型
            tir.write(|w| w
                .txe().set_bit()
                .rtr().bit(message.rtr)
                .ide().bit(message.is_extended)
                .exid().bits(if message.is_extended {
                    message.id >> 11
                } else {
                    0
                })
                .stdid().bits(if message.is_extended {
                    message.id & 0x7FF
                } else {
                    message.id as u16
                })
            );
            
            // 配置数据长度
            let tdlr_reg = tdlr as *mut u32;
            *tdlr_reg = (*tdlr_reg & !0xF000_0000) | ((message.dlc as u32) << 16);
            
            // 写入数据
            let data_low = (message.data[0] as u32) | 
                          ((message.data[1] as u32) << 8) | 
                          ((message.data[2] as u32) << 16) | 
                          ((message.data[3] as u32) << 24);
            
            let data_high = (message.data[4] as u32) | 
                           ((message.data[5] as u32) << 8) | 
                           ((message.data[6] as u32) << 16) | 
                           ((message.data[7] as u32) << 24);
            
            *tdlr_reg = (*tdlr_reg & !0x0000_FFFF) | data_low;
            let tdhr_reg = tdhr as *mut u32;
            *tdhr_reg = data_high;
            
            // 请求发送
            tir.modify(|_, w| w
                .txe().clear_bit()
            );
            
            Ok(true)
        } else {
            Err(CanError::SendFailed)
        }
    }
    
    /// 接收消息（FIFO 0）
    /// 
    /// # 安全
    /// - 调用者必须确保CAN已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// 接收到的消息，如果没有消息则返回None
    pub unsafe fn receive_message_fifo0(&self) -> Result<Option<CanMessage>, CanError> {
        let can1 = self.can1_reg();
        
        // 检查FIFO0是否有消息
        if !can1.rdfr().read().fmp0().bit_is_set() {
            return Ok(None);
        }
        
        // 读取消息
        let fifo0 = &can1.rfmir()[0];
        let fifo0_data = &can1.rfmdt()[0];
        
        let tir = fifo0.read();
        let is_extended = tir.ide().bit_is_set();
        let rtr = tir.rtr().bit_is_set();
        
        let id = if is_extended {
            (tir.exid().bits() as u32) << 11 | (tir.stdid().bits() as u32)
        } else {
            tir.stdid().bits() as u32
        };
        
        let dlc = (fifo0_data.read().bits() >> 16) & 0x0F;
        
        let data_low = fifo0_data.read().bits() & 0x0000_FFFF;
        let data_high = (&can1.rfmdt()[1]).read().bits();
        
        let mut data = [0u8; 8];
        data[0] = (data_low & 0x0000_00FF) as u8;
        data[1] = ((data_low & 0x0000_FF00) >> 8) as u8;
        data[2] = ((data_low & 0x00FF_0000) >> 16) as u8;
        data[3] = ((data_low & 0xFF00_0000) >> 24) as u8;
        data[4] = (data_high & 0x0000_00FF) as u8;
        data[5] = ((data_high & 0x0000_FF00) >> 8) as u8;
        data[6] = ((data_high & 0x00FF_0000) >> 16) as u8;
        data[7] = ((data_high & 0xFF00_0000) >> 24) as u8;
        
        // 释放FIFO0
        can1.rfomr()[0].write(|w| w
            .rfom().set_bit()
        );
        
        Ok(Some(CanMessage {
            id,
            is_extended,
            rtr,
            dlc: dlc as u8,
            data,
        }))
    }
    
    /// 接收消息（FIFO 1）
    /// 
    /// # 安全
    /// - 调用者必须确保CAN已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// 接收到的消息，如果没有消息则返回None
    pub unsafe fn receive_message_fifo1(&self) -> Result<Option<CanMessage>, CanError> {
        let can1 = self.can1_reg();
        
        // 检查FIFO1是否有消息
        if !can1.rdfr().read().fmp1().bit_is_set() {
            return Ok(None);
        }
        
        // 读取消息
        let fifo1 = &can1.rfmir()[1];
        let fifo1_data = &can1.rfmdt()[2];
        
        let tir = fifo1.read();
        let is_extended = tir.ide().bit_is_set();
        let rtr = tir.rtr().bit_is_set();
        
        let id = if is_extended {
            (tir.exid().bits() as u32) << 11 | (tir.stdid().bits() as u32)
        } else {
            tir.stdid().bits() as u32
        };
        
        let dlc = (fifo1_data.read().bits() >> 16) & 0x0F;
        
        let data_low = fifo1_data.read().bits() & 0x0000_FFFF;
        let data_high = (&can1.rfmdt()[3]).read().bits();
        
        let mut data = [0u8; 8];
        data[0] = (data_low & 0x0000_00FF) as u8;
        data[1] = ((data_low & 0x0000_FF00) >> 8) as u8;
        data[2] = ((data_low & 0x00FF_0000) >> 16) as u8;
        data[3] = ((data_low & 0xFF00_0000) >> 24) as u8;
        data[4] = (data_high & 0x0000_00FF) as u8;
        data[5] = ((data_high & 0x0000_FF00) >> 8) as u8;
        data[6] = ((data_high & 0x00FF_0000) >> 16) as u8;
        data[7] = ((data_high & 0xFF00_0000) >> 24) as u8;
        
        // 释放FIFO1
        can1.rfomr()[1].write(|w| w
            .rfom().set_bit()
        );
        
        Ok(Some(CanMessage {
            id,
            is_extended,
            rtr,
            dlc: dlc as u8,
            data,
        }))
    }
    
    /// 启用中断
    /// 
    /// # 安全
    /// - 调用者必须确保CAN已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `interrupt_mask`：中断掩码，使用CAN_IT_*常量组合
    pub unsafe fn enable_interrupt(&self, interrupt_mask: u32) -> Result<(), CanError> {
        let can1 = self.can1_reg_mut();
        
        // 启用相应的中断
        can1.ier().modify(|_, w| {
            // 注意：这里需要根据实际的寄存器结构调整，暂时使用通用方法
            let mut bits = w.bits();
            bits |= interrupt_mask;
            w.bits(bits)
        });
        
        Ok(())
    }
    
    /// 禁用中断
    /// 
    /// # 安全
    /// - 调用者必须确保CAN已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `interrupt_mask`：中断掩码，使用CAN_IT_*常量组合
    pub unsafe fn disable_interrupt(&self, interrupt_mask: u32) -> Result<(), CanError> {
        let can1 = self.can1_reg_mut();
        
        // 禁用相应的中断
        can1.ier().modify(|_, w| {
            // 注意：这里需要根据实际的寄存器结构调整，暂时使用通用方法
            let mut bits = w.bits();
            bits &= !interrupt_mask;
            w.bits(bits)
        });
        
        Ok(())
    }
    
    /// 获取CAN状态
    /// 
    /// # 安全
    /// - 调用者必须确保CAN已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// CAN当前状态
    pub unsafe fn get_status(&self) -> Result<CanStatus, CanError> {
        let can1 = self.can1_reg();
        let esr = can1.esr().read();
        
        // 检查总线离线状态
        if esr.boff().bit_is_set() {
            return Ok(CanStatus::BusOff);
        }
        
        // 检查错误被动状态
        if esr.epv().bit_is_set() {
            return Ok(CanStatus::ErrorPassive);
        }
        
        // 检查错误警告状态
        if esr.ewg().bit_is_set() {
            return Ok(CanStatus::ErrorWarning);
        }
        
        // 检查CAN是否准备就绪
        let msr = can1.msr().read();
        if msr.inak().bit_is_set() {
            return Ok(CanStatus::Initializing);
        }
        
        Ok(CanStatus::Ready)
    }
    
    /// 进入睡眠模式
    /// 
    /// # 安全
    /// - 调用者必须确保CAN已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    pub unsafe fn enter_sleep_mode(&self) -> Result<(), CanError> {
        let can1 = self.can1_reg_mut();
        
        // 请求进入睡眠模式
        can1.mcr().modify(|_, w| w
            .sleep().set_bit()
        );
        
        // 等待睡眠模式确认
        while !can1.msr().read().slak().bit_is_set() {
            // 等待睡眠确认
        }
        
        Ok(())
    }
    
    /// 唤醒
    /// 
    /// # 安全
    /// - 调用者必须确保CAN已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    pub unsafe fn wakeup(&self) -> Result<(), CanError> {
        let can1 = self.can1_reg_mut();
        
        // 请求唤醒
        can1.mcr().modify(|_, w| w
            .sleep().clear_bit()
        );
        
        // 等待唤醒确认
        while can1.msr().read().slak().bit_is_set() {
            // 等待唤醒确认
        }
        
        Ok(())
    }
    
    /// 检查发送邮箱是否为空
    /// 
    /// # 安全
    /// - 调用者必须确保CAN已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// 是否有可用的发送邮箱
    pub unsafe fn is_transmitter_empty(&self) -> Result<bool, CanError> {
        let can1 = self.can1_reg();
        Ok(can1.tsr().read().tme().bits() != 0)
    }
    
    /// 获取发送邮箱状态
    /// 
    /// # 安全
    /// - 调用者必须确保CAN已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// 发送邮箱状态，每一位代表一个邮箱的状态（1表示空，0表示忙）
    pub unsafe fn get_transmitter_status(&self) -> Result<u8, CanError> {
        let can1 = self.can1_reg();
        Ok(can1.tsr().read().tme().bits() as u8)
    }
}

/// CAN中断掩码常量
pub const CAN_IT_TME: u32 = 1 << 0;    /// 发送邮箱空中断
pub const CAN_IT_FMP0: u32 = 1 << 1;   /// FIFO 0 消息挂起中断
pub const CAN_IT_FMP1: u32 = 1 << 2;   /// FIFO 1 消息挂起中断
pub const CAN_IT_FF0: u32 = 1 << 3;    /// FIFO 0 满中断
pub const CAN_IT_FF1: u32 = 1 << 4;    /// FIFO 1 满中断
pub const CAN_IT_FOV0: u32 = 1 << 5;   /// FIFO 0 溢出中断
pub const CAN_IT_FOV1: u32 = 1 << 6;   /// FIFO 1 溢出中断
pub const CAN_IT_WKU: u32 = 1 << 7;    /// 唤醒中断
pub const CAN_IT_SLK: u32 = 1 << 8;    /// 睡眠中断
pub const CAN_IT_ERR: u32 = 1 << 9;    /// 错误中断
pub const CAN_IT_LEC: u32 = 1 << 10;   /// 最后错误代码中断
pub const CAN_IT_BOF: u32 = 1 << 11;   /// 总线离线中断
pub const CAN_IT_EPV: u32 = 1 << 12;   /// 错误被动中断
pub const CAN_IT_EWG: u32 = 1 << 13;   /// 错误警告中断
pub const CAN_IT_ERRIE: u32 = 1 << 15; /// 错误中断使能

/// 预定义的CAN实例
pub const CAN: Can = Can::new();

/// 测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试CAN位时序配置
    #[test]
    fn test_can_bit_timing() {
        // 测试有效位时序
        let valid_timing = CanBitTiming {
            prescaler: 4,
            time_segment_1: 10,
            time_segment_2: 5,
            sjw: 2,
        };
        assert!(valid_timing.is_valid(), "有效位时序应该通过验证");
        
        // 测试无效位时序
        let invalid_timing = CanBitTiming {
            prescaler: 0, // 无效的预分频系数
            time_segment_1: 10,
            time_segment_2: 5,
            sjw: 2,
        };
        assert!(!invalid_timing.is_valid(), "无效位时序应该不通过验证");
    }
    
    /// 测试CAN消息创建
    #[test]
    fn test_can_message() {
        // 测试标准ID消息
        let std_msg = CanMessage::new_standard(0x123, false, 4, [0x01, 0x02, 0x03, 0x04, 0x00, 0x00, 0x00, 0x00]);
        assert!(std_msg.is_valid(), "标准ID消息应该有效");
        assert_eq!(std_msg.id, 0x123, "标准ID应该正确");
        assert!(!std_msg.is_extended, "标准ID消息不应该是扩展ID");
        assert_eq!(std_msg.dlc, 4, "数据长度应该正确");
        
        // 测试扩展ID消息
        let ext_msg = CanMessage::new_extended(0x123456, false, 8, [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
        assert!(ext_msg.is_valid(), "扩展ID消息应该有效");
        assert_eq!(ext_msg.id, 0x123456, "扩展ID应该正确");
        assert!(ext_msg.is_extended, "扩展ID消息应该是扩展ID");
        assert_eq!(ext_msg.dlc, 8, "数据长度应该正确");
        
        // 测试无效消息
        let invalid_msg = CanMessage::new(0x20000000, true, false, 9, [0x00; 8]); // 无效的扩展ID和数据长度
        assert!(!invalid_msg.is_valid(), "无效消息应该不通过验证");
    }
    
    /// 测试CAN状态获取
    #[test]
    fn test_can_status() {
        let can = Can::new();
        
        // 初始化CAN
        let bit_timing = CanBitTiming {
            prescaler: 4,
            time_segment_1: 10,
            time_segment_2: 5,
            sjw: 2,
        };
        
        unsafe {
            let init_result = can.init(CanMode::LoopBack, bit_timing);
            assert!(init_result.is_ok(), "CAN初始化应该成功");
            
            let status = can.get_status();
            assert!(status.is_ok(), "获取CAN状态应该成功");
            assert_eq!(status.unwrap(), CanStatus::Ready, "CAN状态应该是Ready");
        }
    }
    
    /// 测试CAN发送邮箱状态
    #[test]
    fn test_can_transmitter_status() {
        let can = Can::new();
        
        // 初始化CAN
        let bit_timing = CanBitTiming {
            prescaler: 4,
            time_segment_1: 10,
            time_segment_2: 5,
            sjw: 2,
        };
        
        unsafe {
            let init_result = can.init(CanMode::LoopBack, bit_timing);
            assert!(init_result.is_ok(), "CAN初始化应该成功");
            
            let is_empty = can.is_transmitter_empty();
            assert!(is_empty.is_ok(), "获取发送邮箱状态应该成功");
            assert!(is_empty.unwrap(), "发送邮箱应该为空");
            
            let status = can.get_transmitter_status();
            assert!(status.is_ok(), "获取发送邮箱状态应该成功");
            assert!(status.unwrap() != 0, "发送邮箱状态应该不为0");
        }
    }
}
