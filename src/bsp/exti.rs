//! EXTI模块
//! 提供外部中断功能封装

#![allow(unused)]

// 导入内部生成的设备驱动库
use library::*;

/// EXTI线常量定义
pub const EXTI_LINE0: u32 = 0x00000001;  // 外部中断线0
pub const EXTI_LINE1: u32 = 0x00000002;  // 外部中断线1
pub const EXTI_LINE2: u32 = 0x00000004;  // 外部中断线2
pub const EXTI_LINE3: u32 = 0x00000008;  // 外部中断线3
pub const EXTI_LINE4: u32 = 0x00000010;  // 外部中断线4
pub const EXTI_LINE5: u32 = 0x00000020;  // 外部中断线5
pub const EXTI_LINE6: u32 = 0x00000040;  // 外部中断线6
pub const EXTI_LINE7: u32 = 0x00000080;  // 外部中断线7
pub const EXTI_LINE8: u32 = 0x00000100;  // 外部中断线8
pub const EXTI_LINE9: u32 = 0x00000200;  // 外部中断线9
pub const EXTI_LINE10: u32 = 0x00000400; // 外部中断线10
pub const EXTI_LINE11: u32 = 0x00000800; // 外部中断线11
pub const EXTI_LINE12: u32 = 0x00001000; // 外部中断线12
pub const EXTI_LINE13: u32 = 0x00002000; // 外部中断线13
pub const EXTI_LINE14: u32 = 0x00004000; // 外部中断线14
pub const EXTI_LINE15: u32 = 0x00008000; // 外部中断线15
pub const EXTI_LINE16: u32 = 0x00010000; // 外部中断线16（PVD输出）
pub const EXTI_LINE17: u32 = 0x00020000; // 外部中断线17（RTC闹钟事件）
pub const EXTI_LINE18: u32 = 0x00040000; // 外部中断线18（USB唤醒事件）
pub const EXTI_LINE19: u32 = 0x00080000; // 外部中断线19（ETH唤醒事件）

/// EXTI线枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExtiLine {
    Line0 = 0,
    Line1 = 1,
    Line2 = 2,
    Line3 = 3,
    Line4 = 4,
    Line5 = 5,
    Line6 = 6,
    Line7 = 7,
    Line8 = 8,
    Line9 = 9,
    Line10 = 10,
    Line11 = 11,
    Line12 = 12,
    Line13 = 13,
    Line14 = 14,
    Line15 = 15,
    Line16 = 16, // PVD输出
    Line17 = 17, // RTC闹钟事件
    Line18 = 18, // USB唤醒事件
    Line19 = 19, // ETH唤醒事件
}

/// EXTI模式枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExtiMode {
    Interrupt = 0x00, // 中断模式
    Event = 0x04,     // 事件模式
}

/// EXTI触发模式枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExtiTriggerMode {
    None = 0,         // 无触发
    Rising = 1,       // 上升沿触发
    Falling = 2,      // 下降沿触发
    RisingFalling = 3, // 上升沿和下降沿触发
}

/// EXTI初始化结构体
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExtiInitType {
    /// EXTI线
    pub line: u32,
    /// EXTI模式
    pub mode: ExtiMode,
    /// EXTI触发方式
    pub trigger: ExtiTriggerMode,
    /// EXTI线使能状态
    pub line_cmd: bool,
}

/// 功能状态枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FunctionalState {
    Enable = 1,
    Disable = 0,
}

/// 标志状态枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlagStatus {
    Reset = 0,
    Set = 1,
}

/// 中断状态枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ITStatus {
    Reset = 0,
    Set = 1,
}

/// EXTI结构体
pub struct Exti;

impl Exti {
    /// 创建新的EXTI实例
    pub const fn new() -> Self {
        Self
    }
    
    /// 获取EXTI寄存器块
    unsafe fn exti(&self) -> &'static mut library::exti::RegisterBlock {
        &mut *(library::Exti::PTR as *const _ as *mut _)
    }
    
    /// 使用初始化结构体初始化EXTI
    pub unsafe fn exti_init(&self, init: &ExtiInitType) {
        let exti = self.exti();
        let line_mask = init.line;
        
        // 清除之前的配置
        let mut current_imr = exti.imr().read().bits();
        let mut current_emr = exti.emr().read().bits();
        let mut current_rtsr = exti.rtsr().read().bits();
        let mut current_ftsr = exti.ftsr().read().bits();
        
        // 根据模式配置中断或事件
        match init.mode {
            ExtiMode::Interrupt => {
                if init.line_cmd {
                    current_imr |= line_mask;
                } else {
                    current_imr &= !line_mask;
                }
                // 清除事件模式
                current_emr &= !line_mask;
            }
            ExtiMode::Event => {
                if init.line_cmd {
                    current_emr |= line_mask;
                } else {
                    current_emr &= !line_mask;
                }
                // 清除中断模式
                current_imr &= !line_mask;
            }
        }
        
        // 清除之前的触发配置
        current_rtsr &= !line_mask;
        current_ftsr &= !line_mask;
        
        // 配置触发模式
        match init.trigger {
            ExtiTriggerMode::Rising => {
                current_rtsr |= line_mask;
            }
            ExtiTriggerMode::Falling => {
                current_ftsr |= line_mask;
            }
            ExtiTriggerMode::RisingFalling => {
                current_rtsr |= line_mask;
                current_ftsr |= line_mask;
            }
            _ => {}
        }
        
        // 写入配置
        exti.imr().write(|w| unsafe { w.bits(current_imr) });
        exti.emr().write(|w| unsafe { w.bits(current_emr) });
        exti.rtsr().write(|w| unsafe { w.bits(current_rtsr) });
        exti.ftsr().write(|w| unsafe { w.bits(current_ftsr) });
    }
    
    /// 初始化EXTI线
    pub unsafe fn init(&self, line: ExtiLine, trigger_mode: ExtiTriggerMode, enable_interrupt: bool) {
        let exti = self.exti();
        let line = line as u8;
        let line_mask = 1 << line;
        
        // 保存当前配置
        let mut current_rtsr = exti.rtsr().read().bits();
        let mut current_ftsr = exti.ftsr().read().bits();
        let mut current_imr = exti.imr().read().bits();
        
        // 清除之前的配置
        current_rtsr &= !line_mask;
        current_ftsr &= !line_mask;
        
        // 配置触发模式
        match trigger_mode {
            ExtiTriggerMode::Rising => {
                current_rtsr |= line_mask;
            }
            ExtiTriggerMode::Falling => {
                current_ftsr |= line_mask;
            }
            ExtiTriggerMode::RisingFalling => {
                current_rtsr |= line_mask;
                current_ftsr |= line_mask;
            }
            _ => {}
        }
        
        // 配置中断
        if enable_interrupt {
            current_imr |= line_mask;
        } else {
            current_imr &= !line_mask;
        }
        
        // 写入配置
        exti.rtsr().write(|w| unsafe { w.bits(current_rtsr) });
        exti.ftsr().write(|w| unsafe { w.bits(current_ftsr) });
        exti.imr().write(|w| unsafe { w.bits(current_imr) });
    }
    
    /// 初始化EXTI初始化结构体为默认值
    pub fn exti_struct_init(init: &mut ExtiInitType) {
        init.line = 0;
        init.mode = ExtiMode::Interrupt;
        init.trigger = ExtiTriggerMode::None;
        init.line_cmd = false;
    }
    
    /// 启用EXTI线中断
    pub unsafe fn enable_interrupt(&self, line: ExtiLine) {
        let exti = self.exti();
        let line = line as u8;
        let line_mask = 1 << line;
        
        let current_imr = exti.imr().read().bits();
        exti.imr().write(|w| unsafe { w.bits(current_imr | line_mask) });
    }
    
    /// 禁用EXTI线中断
    pub unsafe fn disable_interrupt(&self, line: ExtiLine) {
        let exti = self.exti();
        let line = line as u8;
        let line_mask = 1 << line;
        
        let current_imr = exti.imr().read().bits();
        exti.imr().write(|w| unsafe { w.bits(current_imr & !line_mask) });
    }
    
    /// 启用EXTI线事件
    pub unsafe fn enable_event(&self, line: ExtiLine) {
        let exti = self.exti();
        let line = line as u8;
        let line_mask = 1 << line;
        
        let current_emr = exti.emr().read().bits();
        exti.emr().write(|w| unsafe { w.bits(current_emr | line_mask) });
    }
    
    /// 禁用EXTI线事件
    pub unsafe fn disable_event(&self, line: ExtiLine) {
        let exti = self.exti();
        let line = line as u8;
        let line_mask = 1 << line;
        
        let current_emr = exti.emr().read().bits();
        exti.emr().write(|w| unsafe { w.bits(current_emr & !line_mask) });
    }
    
    /// 启用EXTI线上升沿触发
    pub unsafe fn enable_rising_trigger(&self, line: ExtiLine) {
        let exti = self.exti();
        let line = line as u8;
        let line_mask = 1 << line;
        
        let current_rtsr = exti.rtsr().read().bits();
        exti.rtsr().write(|w| unsafe { w.bits(current_rtsr | line_mask) });
    }
    
    /// 禁用EXTI线上升沿触发
    pub unsafe fn disable_rising_trigger(&self, line: ExtiLine) {
        let exti = self.exti();
        let line = line as u8;
        let line_mask = 1 << line;
        
        let current_rtsr = exti.rtsr().read().bits();
        exti.rtsr().write(|w| unsafe { w.bits(current_rtsr & !line_mask) });
    }
    
    /// 启用EXTI线下降沿触发
    pub unsafe fn enable_falling_trigger(&self, line: ExtiLine) {
        let exti = self.exti();
        let line = line as u8;
        let line_mask = 1 << line;
        
        let current_ftsr = exti.ftsr().read().bits();
        exti.ftsr().write(|w| unsafe { w.bits(current_ftsr | line_mask) });
    }
    
    /// 禁用EXTI线下降沿触发
    pub unsafe fn disable_falling_trigger(&self, line: ExtiLine) {
        let exti = self.exti();
        let line = line as u8;
        let line_mask = 1 << line;
        
        let current_ftsr = exti.ftsr().read().bits();
        exti.ftsr().write(|w| unsafe { w.bits(current_ftsr & !line_mask) });
    }
    
    /// 生成软件中断
    pub unsafe fn generate_sw_interrupt(&self, line: u32) {
        let exti = self.exti();
        exti.swier().write(|w| unsafe { w.bits(line) });
    }
    
    /// 检查EXTI线是否挂起
    pub unsafe fn is_pending(&self, line: ExtiLine) -> bool {
        let exti = self.exti();
        let line = line as u8;
        let line_mask = 1 << line;
        
        (exti.pr().read().bits() & line_mask) != 0
    }
    
    /// 获取EXTI线标志状态
    pub unsafe fn get_flag_status(&self, line: u32) -> FlagStatus {
        let exti = self.exti();
        if (exti.pr().read().bits() & line) != 0 {
            FlagStatus::Set
        } else {
            FlagStatus::Reset
        }
    }
    
    /// 清除EXTI线标志
    pub unsafe fn clear_flag(&self, line: u32) {
        let exti = self.exti();
        exti.pr().write(|w| unsafe { w.bits(line) });
    }
    
    /// 获取EXTI线中断状态
    pub unsafe fn get_it_status(&self, line: u32) -> ITStatus {
        let exti = self.exti();
        if ((exti.imr().read().bits() & line) != 0) && ((exti.pr().read().bits() & line) != 0) {
            ITStatus::Set
        } else {
            ITStatus::Reset
        }
    }
    
    /// 清除EXTI线中断挂起位
    pub unsafe fn clear_it_pending_bit(&self, line: u32) {
        let exti = self.exti();
        exti.pr().write(|w| unsafe { w.bits(line) });
    }
    
    /// 清除EXTI线挂起状态
    pub unsafe fn clear_pending(&self, line: ExtiLine) {
        let exti = self.exti();
        let line = line as u8;
        let line_mask = 1 << line;
        
        // 写入1到PR寄存器的对应位来清除挂起状态
        exti.pr().write(|w| unsafe { w.bits(line_mask) });
    }
    
    /// 清除所有EXTI线挂起状态
    pub unsafe fn clear_all_pending(&self) {
        let exti = self.exti();
        
        // 写入1到所有位来清除中断标志
        exti.pr().write(|w| unsafe { w.bits(0x00FFFFFF) });
    }
    
    /// 将EXTI寄存器重置为默认值
    pub unsafe fn deinit(&self) {
        let exti = self.exti();
        
        // 重置所有EXTI寄存器
        exti.imr().write(|w| unsafe { w.bits(0x00000000) });
        exti.emr().write(|w| unsafe { w.bits(0x00000000) });
        exti.rtsr().write(|w| unsafe { w.bits(0x00000000) });
        exti.ftsr().write(|w| unsafe { w.bits(0x00000000) });
        exti.swier().write(|w| unsafe { w.bits(0x00000000) });
        exti.pr().write(|w| unsafe { w.bits(0x00FFFFFF) });
    }
}

/// 预定义的EXTI实例
pub const EXTI: Exti = Exti::new();