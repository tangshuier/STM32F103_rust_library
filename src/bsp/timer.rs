//! 定时器模块
//! 提供基本的定时器功能

// 屏蔽未使用代码警告
#![allow(unused)]

// 使用内部生成的设备驱动库
use stm32f103::*;

/// 定时器枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimerNumber {
    TIM1,  // 高级定时器（APB2）
    TIM2,  // 通用定时器（APB1）
    TIM3,  // 通用定时器（APB1）
    TIM4,  // 通用定时器（APB1）
}

/// 定时器结构体
pub struct Timer {
    number: TimerNumber,
}

impl TimerNumber {
    /// 获取定时器时钟使能位
    const fn clock_en_bit(&self) -> u32 {
        match self {
            TimerNumber::TIM1 => 1 << 11,  // APB2
            TimerNumber::TIM2 => 1 << 0,   // APB1
            TimerNumber::TIM3 => 1 << 1,   // APB1
            TimerNumber::TIM4 => 1 << 2,   // APB1
        }
    }
}

impl Timer {
    /// 创建新的定时器实例
    pub const fn new(number: TimerNumber) -> Self {
        Self {
            number,
        }
    }
    
    /// 获取对应的定时器寄存器块
    unsafe fn get_timer(&self) -> &'static mut Tim1 {
        match self.number {
            TimerNumber::TIM1 => &mut *(0x40012C00 as *mut Tim1),
            TimerNumber::TIM2 => &mut *(0x40000000 as *mut Tim1),
            TimerNumber::TIM3 => &mut *(0x40000400 as *mut Tim1),
            TimerNumber::TIM4 => &mut *(0x40000800 as *mut Tim1),
        }
    }
    
    /// 初始化定时器
    /// 
    /// # 参数
    /// * `prescaler` - 预分频器值（0-65535）
    /// * `period` - 自动重装载值（0-65535）
    pub unsafe fn init(&self, prescaler: u16, period: u16) {
        // 1. 启用定时器时钟
        let rcc = &mut *(0x40021000 as *mut Rcc);
        match self.number {
            TimerNumber::TIM1 => {
                let mut value = rcc.apb2enr().read().bits();
                value |= 1 << 11;
                rcc.apb2enr().write(|w: &mut stm32f103::rcc::apb2enr::W| unsafe { w.bits(value) });
            },
            TimerNumber::TIM2 => {
                let mut value = rcc.apb1enr().read().bits();
                value |= 1 << 0;
                rcc.apb1enr().write(|w: &mut stm32f103::rcc::apb1enr::W| unsafe { w.bits(value) });
            },
            TimerNumber::TIM3 => {
                let mut value = rcc.apb1enr().read().bits();
                value |= 1 << 1;
                rcc.apb1enr().write(|w: &mut stm32f103::rcc::apb1enr::W| unsafe { w.bits(value) });
            },
            TimerNumber::TIM4 => {
                let mut value = rcc.apb1enr().read().bits();
                value |= 1 << 2;
                rcc.apb1enr().write(|w: &mut stm32f103::rcc::apb1enr::W| unsafe { w.bits(value) });
            },
        }
        
        // 2. 配置定时器
        let tim = self.get_timer();
        tim.cr1().write(|w: &mut stm32f103::tim1::cr1::W| unsafe { w.bits(0x0000) });  // 禁用定时器
        tim.psc().write(|w: &mut stm32f103::tim1::psc::W| unsafe { w.bits(prescaler as u32) });  // 预分频器
        tim.arr().write(|w: &mut stm32f103::tim1::arr::W| unsafe { w.bits(period as u32) });  // 自动重装载值
        tim.cnt().write(|w: &mut stm32f103::tim1::cnt::W| unsafe { w.bits(0x0000) });  // 清零计数器
        tim.egr().write(|w: &mut stm32f103::tim1::egr::W| unsafe { w.bits(0x0001) });  // 生成更新事件
        tim.sr().write(|w: &mut stm32f103::tim1::sr::W| unsafe { w.bits(tim.sr().read().bits() & !(1 << 0)) });  // 清除更新中断标志
    }
    
    /// 启动定时器
    pub unsafe fn start(&self) {
        let tim = self.get_timer();
        tim.cr1().write(|w: &mut stm32f103::tim1::cr1::W| unsafe { w.bits(tim.cr1().read().bits() | (1 << 0)) });  // 启用定时器
    }
    
    /// 停止定时器
    pub unsafe fn stop(&self) {
        let tim = self.get_timer();
        tim.cr1().write(|w: &mut stm32f103::tim1::cr1::W| unsafe { w.bits(tim.cr1().read().bits() & !(1 << 0)) });  // 禁用定时器
    }
    
    /// 重置定时器
    pub unsafe fn reset(&self) {
        let tim = self.get_timer();
        tim.cnt().write(|w: &mut stm32f103::tim1::cnt::W| unsafe { w.bits(0x0000) });  // 清零计数器
        tim.sr().write(|w: &mut stm32f103::tim1::sr::W| unsafe { w.bits(tim.sr().read().bits() & !(1 << 0)) });  // 清除更新中断标志
    }
    
    /// 检查更新中断标志
    pub unsafe fn has_update(&self) -> bool {
        let tim = self.get_timer();
        (tim.sr().read().bits() & (1 << 0)) != 0
    }
    
    /// 清除更新中断标志
    pub unsafe fn clear_update(&self) {
        let tim = self.get_timer();
        tim.sr().write(|w: &mut stm32f103::tim1::sr::W| unsafe { w.bits(tim.sr().read().bits() & !(1 << 0)) });
    }
    
    /// 获取当前计数值
    pub unsafe fn get_count(&self) -> u16 {
        let tim = self.get_timer();
        tim.cnt().read().bits() as u16
    }
    
    /// 设置计数值
    pub unsafe fn set_count(&self, count: u16) {
        let tim = self.get_timer();
        tim.cnt().write(|w: &mut stm32f103::tim1::cnt::W| unsafe { w.bits(count as u32) });
    }
    
    /// 使能更新中断
    pub unsafe fn enable_update_interrupt(&self) {
        let tim = self.get_timer();
        tim.dier().write(|w: &mut stm32f103::tim1::dier::W| unsafe { w.bits(tim.dier().read().bits() | (1 << 0)) });
    }
    
    /// 禁用更新中断
    pub unsafe fn disable_update_interrupt(&self) {
        let tim = self.get_timer();
        tim.dier().write(|w: &mut stm32f103::tim1::dier::W| unsafe { w.bits(tim.dier().read().bits() & !(1 << 0)) });
    }
}

/// 预定义的定时器常量
pub const TIM1: Timer = Timer::new(TimerNumber::TIM1);
pub const TIM2: Timer = Timer::new(TimerNumber::TIM2);
pub const TIM3: Timer = Timer::new(TimerNumber::TIM3);
pub const TIM4: Timer = Timer::new(TimerNumber::TIM4);
