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

/// PWM通道枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PwmChannel {
    Channel1,
    Channel2,
    Channel3,
    Channel4,
}

/// PWM模式枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PwmMode {
    Mode1,  // PWM模式1：CNT < CCR时，通道输出有效电平
    Mode2,  // PWM模式2：CNT < CCR时，通道输出无效电平
}

/// PWM极性枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PwmPolarity {
    High,   // 有效电平为高电平
    Low,    // 有效电平为低电平
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
    
    /// 初始化PWM通道
    pub unsafe fn init_pwm(
        &self, 
        channel: PwmChannel, 
        mode: PwmMode, 
        polarity: PwmPolarity,
        period: u16,
        prescaler: u16,
        initial_duty: u16
    ) {
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
        
        // 2. 配置定时器基本参数
        let tim = self.get_timer();
        tim.cr1().write(|w: &mut stm32f103::tim1::cr1::W| unsafe { w.bits(0x0000) });  // 禁用定时器
        tim.psc().write(|w: &mut stm32f103::tim1::psc::W| unsafe { w.bits(prescaler as u32) });  // 预分频器
        tim.arr().write(|w: &mut stm32f103::tim1::arr::W| unsafe { w.bits(period as u32) });  // 自动重装载值
        
        // 3. 配置PWM通道
        match channel {
            PwmChannel::Channel1 => {
                // 配置CCMR1寄存器：PWM模式，使能预加载
                let mode_bits = match mode {
                    PwmMode::Mode1 => 0x60,
                    PwmMode::Mode2 => 0x70,
                };
                tim.ccmr1().write(|w: &mut stm32f103::tim1::ccmr1::W| unsafe { 
                    w.bits((tim.ccmr1().read().bits() & !0x00FF) | mode_bits | 0x0080) 
                });
                
                // 配置CCER寄存器：配置极性，使能通道
                let polarity_bit = match polarity {
                    PwmPolarity::High => 0x0000,
                    PwmPolarity::Low => 0x0002,
                };
                tim.ccer().write(|w: &mut stm32f103::tim1::ccer::W| unsafe { 
                    w.bits((tim.ccer().read().bits() & !0x0003) | polarity_bit | 0x0001) 
                });
                
                // 设置初始占空比
                tim.ccr1().write(|w: &mut stm32f103::tim1::ccr1::W| unsafe { w.bits(initial_duty as u32) });
            },
            PwmChannel::Channel2 => {
                // 配置CCMR1寄存器：PWM模式，使能预加载
                let mode_bits = match mode {
                    PwmMode::Mode1 => 0x6000,
                    PwmMode::Mode2 => 0x7000,
                };
                tim.ccmr1().write(|w: &mut stm32f103::tim1::ccmr1::W| unsafe { 
                    w.bits((tim.ccmr1().read().bits() & !0xFF00) | mode_bits | 0x8000) 
                });
                
                // 配置CCER寄存器：配置极性，使能通道
                let polarity_bit = match polarity {
                    PwmPolarity::High => 0x0000,
                    PwmPolarity::Low => 0x0020,
                };
                tim.ccer().write(|w: &mut stm32f103::tim1::ccer::W| unsafe { 
                    w.bits((tim.ccer().read().bits() & !0x0030) | polarity_bit | 0x0010) 
                });
                
                // 设置初始占空比
                tim.ccr2().write(|w: &mut stm32f103::tim1::ccr2::W| unsafe { w.bits(initial_duty as u32) });
            },
            PwmChannel::Channel3 => {
                // 配置CCMR2寄存器：PWM模式，使能预加载
                let mode_bits = match mode {
                    PwmMode::Mode1 => 0x60,
                    PwmMode::Mode2 => 0x70,
                };
                tim.ccmr2().write(|w: &mut stm32f103::tim1::ccmr2::W| unsafe { 
                    w.bits((tim.ccmr2().read().bits() & !0x00FF) | mode_bits | 0x0080) 
                });
                
                // 配置CCER寄存器：配置极性，使能通道
                let polarity_bit = match polarity {
                    PwmPolarity::High => 0x0000,
                    PwmPolarity::Low => 0x0200,
                };
                tim.ccer().write(|w: &mut stm32f103::tim1::ccer::W| unsafe { 
                    w.bits((tim.ccer().read().bits() & !0x0300) | polarity_bit | 0x0100) 
                });
                
                // 设置初始占空比
                tim.ccr3().write(|w: &mut stm32f103::tim1::ccr3::W| unsafe { w.bits(initial_duty as u32) });
            },
            PwmChannel::Channel4 => {
                // 配置CCMR2寄存器：PWM模式，使能预加载
                let mode_bits = match mode {
                    PwmMode::Mode1 => 0x6000,
                    PwmMode::Mode2 => 0x7000,
                };
                tim.ccmr2().write(|w: &mut stm32f103::tim1::ccmr2::W| unsafe { 
                    w.bits((tim.ccmr2().read().bits() & !0xFF00) | mode_bits | 0x8000) 
                });
                
                // 配置CCER寄存器：配置极性，使能通道
                let polarity_bit = match polarity {
                    PwmPolarity::High => 0x0000,
                    PwmPolarity::Low => 0x2000,
                };
                tim.ccer().write(|w: &mut stm32f103::tim1::ccer::W| unsafe { 
                    w.bits((tim.ccer().read().bits() & !0x3000) | polarity_bit | 0x1000) 
                });
                
                // 设置初始占空比
                tim.ccr4().write(|w: &mut stm32f103::tim1::ccr4::W| unsafe { w.bits(initial_duty as u32) });
            },
        }
        
        // 4. 配置高级定时器特殊设置
        if self.number == TimerNumber::TIM1 {
            // 对于高级定时器TIM1，需要启用主输出
            tim.bdtr().write(|w: &mut stm32f103::tim1::bdtr::W| unsafe { 
                w.bits(tim.bdtr().read().bits() | 0x8000) 
            });
        }
        
        // 5. 生成更新事件，更新影子寄存器
        tim.egr().write(|w: &mut stm32f103::tim1::egr::W| unsafe { w.bits(0x0001) });
        
        // 6. 清除更新中断标志
        tim.sr().write(|w: &mut stm32f103::tim1::sr::W| unsafe { w.bits(tim.sr().read().bits() & !(1 << 0)) });
        
        // 7. 启用定时器
        tim.cr1().write(|w: &mut stm32f103::tim1::cr1::W| unsafe { w.bits(tim.cr1().read().bits() | 0x0001) });
    }
    
    /// 设置PWM占空比
    pub unsafe fn set_pwm_duty(&self, channel: PwmChannel, duty: u16) {
        let tim = self.get_timer();
        match channel {
            PwmChannel::Channel1 => {
                tim.ccr1().write(|w: &mut stm32f103::tim1::ccr1::W| unsafe { w.bits(duty as u32) });
            },
            PwmChannel::Channel2 => {
                tim.ccr2().write(|w: &mut stm32f103::tim1::ccr2::W| unsafe { w.bits(duty as u32) });
            },
            PwmChannel::Channel3 => {
                tim.ccr3().write(|w: &mut stm32f103::tim1::ccr3::W| unsafe { w.bits(duty as u32) });
            },
            PwmChannel::Channel4 => {
                tim.ccr4().write(|w: &mut stm32f103::tim1::ccr4::W| unsafe { w.bits(duty as u32) });
            },
        }
    }
    
    /// 设置PWM频率
    pub unsafe fn set_pwm_frequency(&self, channel: PwmChannel, frequency: u32, duty: u16) {
        let tim = self.get_timer();
        
        // 获取定时器时钟频率
        let timer_clock = match self.number {
            TimerNumber::TIM1 => 72_000_000,  // APB2时钟
            _ => 36_000_000,  // APB1时钟
        };
        
        // 计算预分频器和自动重装载值
        // 尝试找到合适的预分频器值，使得ARR在0~65535范围内
        let mut prescaler = 0;
        let mut arr = 0;
        
        // 从预分频器0开始尝试
        for psc in 0..=65535 {
            let psc_val = psc as u32;
            let arr_val = (timer_clock / ((psc_val + 1) * frequency)) - 1;
            
            if arr_val <= 65535 {
                prescaler = psc_val as u16;
                arr = arr_val as u16;
                break;
            }
        }
        
        // 配置定时器
        tim.cr1().write(|w: &mut stm32f103::tim1::cr1::W| unsafe { w.bits(0x0000) });  // 禁用定时器
        tim.psc().write(|w: &mut stm32f103::tim1::psc::W| unsafe { w.bits(prescaler as u32) });  // 预分频器
        tim.arr().write(|w: &mut stm32f103::tim1::arr::W| unsafe { w.bits(arr as u32) });  // 自动重装载值
        
        // 设置占空比
        self.set_pwm_duty(channel, duty);
        
        // 生成更新事件，更新影子寄存器
        tim.egr().write(|w: &mut stm32f103::tim1::egr::W| unsafe { w.bits(0x0001) });
        
        // 启用定时器
        tim.cr1().write(|w: &mut stm32f103::tim1::cr1::W| unsafe { w.bits(tim.cr1().read().bits() | 0x0001) });
    }
    
    /// 启用PWM通道
    pub unsafe fn enable_pwm_channel(&self, channel: PwmChannel) {
        let tim = self.get_timer();
        match channel {
            PwmChannel::Channel1 => {
                tim.ccer().write(|w: &mut stm32f103::tim1::ccer::W| unsafe { 
                    w.bits(tim.ccer().read().bits() | 0x0001) 
                });
            },
            PwmChannel::Channel2 => {
                tim.ccer().write(|w: &mut stm32f103::tim1::ccer::W| unsafe { 
                    w.bits(tim.ccer().read().bits() | 0x0010) 
                });
            },
            PwmChannel::Channel3 => {
                tim.ccer().write(|w: &mut stm32f103::tim1::ccer::W| unsafe { 
                    w.bits(tim.ccer().read().bits() | 0x0100) 
                });
            },
            PwmChannel::Channel4 => {
                tim.ccer().write(|w: &mut stm32f103::tim1::ccer::W| unsafe { 
                    w.bits(tim.ccer().read().bits() | 0x1000) 
                });
            },
        }
    }
    
    /// 禁用PWM通道
    pub unsafe fn disable_pwm_channel(&self, channel: PwmChannel) {
        let tim = self.get_timer();
        match channel {
            PwmChannel::Channel1 => {
                tim.ccer().write(|w: &mut stm32f103::tim1::ccer::W| unsafe { 
                    w.bits(tim.ccer().read().bits() & !0x0001) 
                });
            },
            PwmChannel::Channel2 => {
                tim.ccer().write(|w: &mut stm32f103::tim1::ccer::W| unsafe { 
                    w.bits(tim.ccer().read().bits() & !0x0010) 
                });
            },
            PwmChannel::Channel3 => {
                tim.ccer().write(|w: &mut stm32f103::tim1::ccer::W| unsafe { 
                    w.bits(tim.ccer().read().bits() & !0x0100) 
                });
            },
            PwmChannel::Channel4 => {
                tim.ccer().write(|w: &mut stm32f103::tim1::ccer::W| unsafe { 
                    w.bits(tim.ccer().read().bits() & !0x1000) 
                });
            },
        }
    }
}

/// 预定义的定时器常量
pub const TIM1: Timer = Timer::new(TimerNumber::TIM1);
pub const TIM2: Timer = Timer::new(TimerNumber::TIM2);
pub const TIM3: Timer = Timer::new(TimerNumber::TIM3);
pub const TIM4: Timer = Timer::new(TimerNumber::TIM4);
