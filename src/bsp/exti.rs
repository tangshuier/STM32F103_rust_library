//! EXTI模块
//! 提供外部中断功能封装

#![allow(unused)]

// 导入内部生成的设备驱动库
use library::*;

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

/// EXTI触发模式枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExtiTriggerMode {
    None = 0,         // 无触发
    Rising = 1,       // 上升沿触发
    Falling = 2,      // 下降沿触发
    RisingFalling = 3, // 上升沿和下降沿触发
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
        &mut *(0x40010400 as *mut library::exti::RegisterBlock)
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
        exti.rtsr().write(|w: &mut library::exti::rtsr::W| unsafe { w.bits(current_rtsr) });
        exti.ftsr().write(|w: &mut library::exti::ftsr::W| unsafe { w.bits(current_ftsr) });
        exti.imr().write(|w: &mut library::exti::imr::W| unsafe { w.bits(current_imr) });
    }
    
    /// 启用EXTI线中断
    pub unsafe fn enable_interrupt(&self, line: ExtiLine) {
        let exti = self.exti();
        let line = line as u8;
        let line_mask = 1 << line;
        
        let current_imr = exti.imr().read().bits();
        exti.imr().write(|w: &mut library::exti::imr::W| unsafe { w.bits(current_imr | line_mask) });
    }
    
    /// 禁用EXTI线中断
    pub unsafe fn disable_interrupt(&self, line: ExtiLine) {
        let exti = self.exti();
        let line = line as u8;
        let line_mask = 1 << line;
        
        let current_imr = exti.imr().read().bits();
        exti.imr().write(|w: &mut library::exti::imr::W| unsafe { w.bits(current_imr & !line_mask) });
    }
    
    /// 启用EXTI线事件
    pub unsafe fn enable_event(&self, line: ExtiLine) {
        let exti = self.exti();
        let line = line as u8;
        let line_mask = 1 << line;
        
        let current_emr = exti.emr().read().bits();
        exti.emr().write(|w: &mut library::exti::emr::W| unsafe { w.bits(current_emr | line_mask) });
    }
    
    /// 禁用EXTI线事件
    pub unsafe fn disable_event(&self, line: ExtiLine) {
        let exti = self.exti();
        let line = line as u8;
        let line_mask = 1 << line;
        
        let current_emr = exti.emr().read().bits();
        exti.emr().write(|w: &mut library::exti::emr::W| unsafe { w.bits(current_emr & !line_mask) });
    }
    
    /// 启用EXTI线上升沿触发
    pub unsafe fn enable_rising_trigger(&self, line: ExtiLine) {
        let exti = self.exti();
        let line = line as u8;
        let line_mask = 1 << line;
        
        let current_rtsr = exti.rtsr().read().bits();
        exti.rtsr().write(|w: &mut library::exti::rtsr::W| unsafe { w.bits(current_rtsr | line_mask) });
    }
    
    /// 禁用EXTI线上升沿触发
    pub unsafe fn disable_rising_trigger(&self, line: ExtiLine) {
        let exti = self.exti();
        let line = line as u8;
        let line_mask = 1 << line;
        
        let current_rtsr = exti.rtsr().read().bits();
        exti.rtsr().write(|w: &mut library::exti::rtsr::W| unsafe { w.bits(current_rtsr & !line_mask) });
    }
    
    /// 启用EXTI线下降沿触发
    pub unsafe fn enable_falling_trigger(&self, line: ExtiLine) {
        let exti = self.exti();
        let line = line as u8;
        let line_mask = 1 << line;
        
        let current_ftsr = exti.ftsr().read().bits();
        exti.ftsr().write(|w: &mut library::exti::ftsr::W| unsafe { w.bits(current_ftsr | line_mask) });
    }
    
    /// 禁用EXTI线下降沿触发
    pub unsafe fn disable_falling_trigger(&self, line: ExtiLine) {
        let exti = self.exti();
        let line = line as u8;
        let line_mask = 1 << line;
        
        let current_ftsr = exti.ftsr().read().bits();
        exti.ftsr().write(|w: &mut library::exti::ftsr::W| unsafe { w.bits(current_ftsr & !line_mask) });
    }
    
    /// 生成软件中断
    pub unsafe fn generate_software_interrupt(&self, line: ExtiLine) {
        let exti = self.exti();
        let line = line as u8;
        let line_mask = 1 << line;
        
        let current_swier = exti.swier().read().bits();
        exti.swier().write(|w: &mut library::exti::swier::W| unsafe { w.bits(current_swier | line_mask) });
    }
    
    /// 检查EXTI线是否挂起
    pub unsafe fn is_pending(&self, line: ExtiLine) -> bool {
        let exti = self.exti();
        let line = line as u8;
        let line_mask = 1 << line;
        
        (exti.pr().read().bits() & line_mask) != 0
    }
    
    /// 清除EXTI线挂起状态
    pub unsafe fn clear_pending(&self, line: ExtiLine) {
        let exti = self.exti();
        let line = line as u8;
        let line_mask = 1 << line;
        
        // 写入1到PR寄存器的对应位来清除挂起状态
        exti.pr().write(|w: &mut library::exti::pr::W| unsafe { w.bits(line_mask) });
    }
    
    /// 清除所有EXTI线挂起状态
    pub unsafe fn clear_all_pending(&self) {
        let exti = self.exti();
        
        // 写入1到所有位来清除中断标志
        exti.pr().write(|w: &mut library::exti::pr::W| unsafe { w.bits(0x00FFFFFF) });
    }
}

/// 预定义的EXTI实例
pub const EXTI: Exti = Exti::new();
