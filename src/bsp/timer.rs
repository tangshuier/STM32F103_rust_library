//! 定时器模块
//! 提供基本的定时器功能

// 屏蔽未使用代码警告
#![allow(unused)]

// 使用内部生成的设备驱动库
use stm32f103::*;
use core::ops::DerefMut;
use crate::bsp::rcc::RccDriver;

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
    /// 获取定时器对应的APB总线
    pub const fn get_apb_bus(&self) -> ApbBus {
        match self {
            TimerNumber::TIM1 => ApbBus::APB2,
            TimerNumber::TIM2 | TimerNumber::TIM3 | TimerNumber::TIM4 => ApbBus::APB1,
        }
    }
    
    /// 获取定时器对应的外设枚举值
    pub const fn get_peripheral(&self) -> TimerPeripheral {
        match self {
            TimerNumber::TIM1 => TimerPeripheral::TIM1,
            TimerNumber::TIM2 => TimerPeripheral::TIM2,
            TimerNumber::TIM3 => TimerPeripheral::TIM3,
            TimerNumber::TIM4 => TimerPeripheral::TIM4,
        }
    }
    
    /// 获取定时器基地址
    pub const fn get_base_address(&self) -> usize {
        match self {
            TimerNumber::TIM1 => 0x4001_2C00,
            TimerNumber::TIM2 => 0x4000_0000,
            TimerNumber::TIM3 => 0x4000_0400,
            TimerNumber::TIM4 => 0x4000_0800,
        }
    }
}

/// APB总线枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ApbBus {
    APB1,
    APB2,
}

/// 定时器外设枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimerPeripheral {
    TIM1,
    TIM2,
    TIM3,
    TIM4,
}

impl Timer {
    /// 创建新的定时器实例
    pub const fn new(number: TimerNumber) -> Self {
        Self {
            number,
        }
    }
    
    /// 获取TIM1寄存器块
    unsafe fn get_tim1(&self) -> &'static mut tim1::RegisterBlock {
        &mut *(TimerNumber::TIM1.get_base_address() as *mut tim1::RegisterBlock)
    }
    
    /// 获取TIM2寄存器块
    unsafe fn get_tim2(&self) -> &'static mut tim2::RegisterBlock {
        &mut *(TimerNumber::TIM2.get_base_address() as *mut tim2::RegisterBlock)
    }
    
    /// 获取TIM3寄存器块
    unsafe fn get_tim3(&self) -> &'static mut tim3::RegisterBlock {
        &mut *(TimerNumber::TIM3.get_base_address() as *mut tim3::RegisterBlock)
    }
    
    /// 获取TIM4寄存器块
    unsafe fn get_tim4(&self) -> &'static mut tim4::RegisterBlock {
        &mut *(TimerNumber::TIM4.get_base_address() as *mut tim4::RegisterBlock)
    }
    
    /// 启用定时器时钟
    unsafe fn enable_clock(&self) {
        let rcc = &mut *(0x4002_1000 as *mut rcc::RegisterBlock);
        
        match self.number {
            TimerNumber::TIM1 => {
                rcc.apb2enr().modify(|_, w| w.tim1en().set_bit());
            },
            TimerNumber::TIM2 => {
                rcc.apb1enr().modify(|_, w| w.tim2en().set_bit());
            },
            TimerNumber::TIM3 => {
                rcc.apb1enr().modify(|_, w| w.tim3en().set_bit());
            },
            TimerNumber::TIM4 => {
                rcc.apb1enr().modify(|_, w| w.tim4en().set_bit());
            },
        };
    }
    
    /// 获取定时器时钟频率
    unsafe fn get_timer_clock(&self) -> u32 {
        // 使用RCC驱动获取时钟频率
        let rcc_driver = RccDriver::new();
        let clocks = rcc_driver.get_clocks_freq();
        
        match self.number {
            TimerNumber::TIM1 => {
                // TIM1位于APB2，当APB2预分频系数为1时，定时器时钟 = PCLK2
                // 否则，定时器时钟 = PCLK2 * 2
                if (clocks.sysclk_frequency / clocks.hclk_frequency) == 1 {
                    clocks.pclk2_frequency
                } else {
                    clocks.pclk2_frequency * 2
                }
            },
            TimerNumber::TIM2 | TimerNumber::TIM3 | TimerNumber::TIM4 => {
                // TIM2-TIM4位于APB1，当APB1预分频系数为1时，定时器时钟 = PCLK1
                // 否则，定时器时钟 = PCLK1 * 2
                if (clocks.hclk_frequency / clocks.pclk1_frequency) == 1 {
                    clocks.pclk1_frequency
                } else {
                    clocks.pclk1_frequency * 2
                }
            },
        }
    }
    
    /// 初始化定时器
    /// 
    /// # 参数
    /// * `prescaler` - 预分频器值（0-65535）
    /// * `period` - 自动重装载值（0-65535）
    pub unsafe fn init(&self, prescaler: u16, period: u16) {
        // 参数有效性验证
        assert!(prescaler > 0 || prescaler == 0, "Prescaler value is valid");
        assert!(period > 0 || period == 0, "Period value is valid");
        
        // 1. 启用定时器时钟
        self.enable_clock();
        
        // 2. 配置定时器
        match self.number {
            TimerNumber::TIM1 => {
                let tim = self.get_tim1();
                tim.cr1().write(|w| w.cen().clear_bit());  // 禁用定时器
                tim.psc().write(|w| w.psc().bits(prescaler));  // 预分频器
                tim.arr().write(|w| w.arr().bits(period));  // 自动重装载值
                tim.cnt().write(|w| w.cnt().bits(0));  // 清零计数器
                tim.egr().write(|w| w.ug().set_bit());  // 生成更新事件
                tim.sr().write(|w| w.uif().clear_bit());  // 清除更新中断标志
            },
            TimerNumber::TIM2 => {
                let tim = self.get_tim2();
                tim.cr1().write(|w| w.cen().clear_bit());  // 禁用定时器
                tim.psc().write(|w| w.psc().bits(prescaler));  // 预分频器
                tim.arr().write(|w| w.arr().bits(period));  // 自动重装载值
                tim.cnt().write(|w| w.cnt().bits(0));  // 清零计数器
                tim.egr().write(|w| w.ug().set_bit());  // 生成更新事件
                tim.sr().write(|w| w.uif().clear_bit());  // 清除更新中断标志
            },
            TimerNumber::TIM3 => {
                let tim = self.get_tim3();
                tim.cr1().write(|w| w.cen().clear_bit());  // 禁用定时器
                tim.psc().write(|w| w.psc().bits(prescaler));  // 预分频器
                tim.arr().write(|w| w.arr().bits(period));  // 自动重装载值
                tim.cnt().write(|w| w.cnt().bits(0));  // 清零计数器
                tim.egr().write(|w| w.ug().set_bit());  // 生成更新事件
                tim.sr().write(|w| w.uif().clear_bit());  // 清除更新中断标志
            },
            TimerNumber::TIM4 => {
                let tim = self.get_tim4();
                tim.cr1().write(|w| w.cen().clear_bit());  // 禁用定时器
                tim.psc().write(|w| w.psc().bits(prescaler));  // 预分频器
                tim.arr().write(|w| w.arr().bits(period));  // 自动重装载值
                tim.cnt().write(|w| w.cnt().bits(0));  // 清零计数器
                tim.egr().write(|w| w.ug().set_bit());  // 生成更新事件
                tim.sr().write(|w| w.uif().clear_bit());  // 清除更新中断标志
            },
        }
    }
    
    /// 启动定时器
    pub unsafe fn start(&self) {
        match self.number {
            TimerNumber::TIM1 => { self.get_tim1().cr1().modify(|_, w| w.cen().set_bit()); },
            TimerNumber::TIM2 => { self.get_tim2().cr1().modify(|_, w| w.cen().set_bit()); },
            TimerNumber::TIM3 => { self.get_tim3().cr1().modify(|_, w| w.cen().set_bit()); },
            TimerNumber::TIM4 => { self.get_tim4().cr1().modify(|_, w| w.cen().set_bit()); },
        }
    }
    
    /// 停止定时器
    pub unsafe fn stop(&self) {
        match self.number {
            TimerNumber::TIM1 => { self.get_tim1().cr1().modify(|_, w| w.cen().clear_bit()); },
            TimerNumber::TIM2 => { self.get_tim2().cr1().modify(|_, w| w.cen().clear_bit()); },
            TimerNumber::TIM3 => { self.get_tim3().cr1().modify(|_, w| w.cen().clear_bit()); },
            TimerNumber::TIM4 => { self.get_tim4().cr1().modify(|_, w| w.cen().clear_bit()); },
        }
    }
    
    /// 重置定时器
    pub unsafe fn reset(&self) {
        match self.number {
            TimerNumber::TIM1 => {
                let tim = self.get_tim1();
                tim.cnt().write(|w| w.cnt().bits(0));
                tim.sr().write(|w| w.uif().clear_bit());
            },
            TimerNumber::TIM2 => {
                let tim = self.get_tim2();
                tim.cnt().write(|w| w.cnt().bits(0));
                tim.sr().write(|w| w.uif().clear_bit());
            },
            TimerNumber::TIM3 => {
                let tim = self.get_tim3();
                tim.cnt().write(|w| w.cnt().bits(0));
                tim.sr().write(|w| w.uif().clear_bit());
            },
            TimerNumber::TIM4 => {
                let tim = self.get_tim4();
                tim.cnt().write(|w| w.cnt().bits(0));
                tim.sr().write(|w| w.uif().clear_bit());
            },
        }
    }
    
    /// 检查更新中断标志
    pub unsafe fn has_update(&self) -> bool {
        match self.number {
            TimerNumber::TIM1 => self.get_tim1().sr().read().uif().bit_is_set(),
            TimerNumber::TIM2 => self.get_tim2().sr().read().uif().bit_is_set(),
            TimerNumber::TIM3 => self.get_tim3().sr().read().uif().bit_is_set(),
            TimerNumber::TIM4 => self.get_tim4().sr().read().uif().bit_is_set(),
        }
    }
    
    /// 清除更新中断标志
    pub unsafe fn clear_update(&self) {
        match self.number {
            TimerNumber::TIM1 => { self.get_tim1().sr().write(|w| w.uif().clear_bit()); },
            TimerNumber::TIM2 => { self.get_tim2().sr().write(|w| w.uif().clear_bit()); },
            TimerNumber::TIM3 => { self.get_tim3().sr().write(|w| w.uif().clear_bit()); },
            TimerNumber::TIM4 => { self.get_tim4().sr().write(|w| w.uif().clear_bit()); },
        }
    }
    
    /// 获取当前计数值
    pub unsafe fn get_count(&self) -> u16 {
        match self.number {
            TimerNumber::TIM1 => self.get_tim1().cnt().read().cnt().bits(),
            TimerNumber::TIM2 => self.get_tim2().cnt().read().cnt().bits(),
            TimerNumber::TIM3 => self.get_tim3().cnt().read().cnt().bits(),
            TimerNumber::TIM4 => self.get_tim4().cnt().read().cnt().bits(),
        }
    }
    
    /// 设置计数值
    pub unsafe fn set_count(&self, count: u16) {
        // 参数有效性验证
        assert!(count > 0 || count == 0, "Count value is valid");
        
        match self.number {
            TimerNumber::TIM1 => { self.get_tim1().cnt().write(|w| w.cnt().bits(count)); },
            TimerNumber::TIM2 => { self.get_tim2().cnt().write(|w| w.cnt().bits(count)); },
            TimerNumber::TIM3 => { self.get_tim3().cnt().write(|w| w.cnt().bits(count)); },
            TimerNumber::TIM4 => { self.get_tim4().cnt().write(|w| w.cnt().bits(count)); },
        }
    }
    
    /// 使能更新中断
    pub unsafe fn enable_update_interrupt(&self) {
        match self.number {
            TimerNumber::TIM1 => { self.get_tim1().dier().modify(|_, w| w.uie().set_bit()); },
            TimerNumber::TIM2 => { self.get_tim2().dier().modify(|_, w| w.uie().set_bit()); },
            TimerNumber::TIM3 => { self.get_tim3().dier().modify(|_, w| w.uie().set_bit()); },
            TimerNumber::TIM4 => { self.get_tim4().dier().modify(|_, w| w.uie().set_bit()); },
        }
    }
    
    /// 禁用更新中断
    pub unsafe fn disable_update_interrupt(&self) {
        match self.number {
            TimerNumber::TIM1 => { self.get_tim1().dier().modify(|_, w| w.uie().clear_bit()); },
            TimerNumber::TIM2 => { self.get_tim2().dier().modify(|_, w| w.uie().clear_bit()); },
            TimerNumber::TIM3 => { self.get_tim3().dier().modify(|_, w| w.uie().clear_bit()); },
            TimerNumber::TIM4 => { self.get_tim4().dier().modify(|_, w| w.uie().clear_bit()); },
        }
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
        // 参数有效性验证
        assert!(prescaler > 0 || prescaler == 0, "Prescaler value is valid");
        assert!(period > 0 || period == 0, "Period value is valid");
        assert!(initial_duty <= period, "Initial duty value out of range (0-period)");
        
        // 1. 启用定时器时钟
        self.enable_clock();
        
        // 2. 配置PWM通用设置
        match self.number {
            TimerNumber::TIM1 => {
                let tim = self.get_tim1();
                self.config_pwm_channel_tim1(tim, channel, mode, polarity, period, prescaler, initial_duty);
                
                // 对于高级定时器TIM1，需要启用主输出
                tim.bdtr().modify(|_, w| w.moe().set_bit());
                
                // 生成更新事件，更新影子寄存器
                tim.egr().write(|w| w.ug().set_bit());
                // 清除更新中断标志
                tim.sr().write(|w| w.uif().clear_bit());
                // 启用定时器
                tim.cr1().write(|w| w.cen().set_bit());
            },
            TimerNumber::TIM2 => {
                let tim = self.get_tim2();
                self.config_pwm_channel_tim2(tim, channel, mode, polarity, period, prescaler, initial_duty);
                
                // 生成更新事件，更新影子寄存器
                tim.egr().write(|w| w.ug().set_bit());
                // 清除更新中断标志
                tim.sr().write(|w| w.uif().clear_bit());
                // 启用定时器
                tim.cr1().write(|w| w.cen().set_bit());
            },
            TimerNumber::TIM3 => {
                let tim = self.get_tim3();
                self.config_pwm_channel_tim3(tim, channel, mode, polarity, period, prescaler, initial_duty);
                
                // 生成更新事件，更新影子寄存器
                tim.egr().write(|w| w.ug().set_bit());
                // 清除更新中断标志
                tim.sr().write(|w| w.uif().clear_bit());
                // 启用定时器
                tim.cr1().write(|w| w.cen().set_bit());
            },
            TimerNumber::TIM4 => {
                let tim = self.get_tim4();
                self.config_pwm_channel_tim4(tim, channel, mode, polarity, period, prescaler, initial_duty);
                
                // 生成更新事件，更新影子寄存器
                tim.egr().write(|w| w.ug().set_bit());
                // 清除更新中断标志
                tim.sr().write(|w| w.uif().clear_bit());
                // 启用定时器
                tim.cr1().write(|w| w.cen().set_bit());
            },
        }
    }
    
    /// 配置PWM通道（针对TIM1）
    unsafe fn config_pwm_channel_tim1(
        &self, 
        tim: &mut tim1::RegisterBlock, 
        channel: PwmChannel, 
        mode: PwmMode, 
        polarity: PwmPolarity,
        period: u16,
        prescaler: u16,
        initial_duty: u16
    ) {
        // 禁用定时器
        tim.cr1().write(|w| w.cen().clear_bit());
        // 配置预分频器和自动重装载值
        tim.psc().write(|w| w.psc().bits(prescaler));
        tim.arr().write(|w| w.arr().bits(period));
        
        // 配置PWM通道
        self.config_pwm_channel_tim1_inner(tim, channel, mode, polarity, initial_duty);
    }
    
    /// 配置PWM通道的内部方法（针对TIM1）
    unsafe fn config_pwm_channel_tim1_inner(
        &self, 
        tim: &mut tim1::RegisterBlock, 
        channel: PwmChannel, 
        mode: PwmMode, 
        polarity: PwmPolarity,
        initial_duty: u16
    ) {
        match channel {
            PwmChannel::Channel1 => {
                // 配置CCMR1寄存器：PWM模式，使能预加载
                tim.ccmr1_output().write(|w| {
                    // PWM模式1：0b110，PWM模式2：0b111
                    let mode_bits = match mode {
                        PwmMode::Mode1 => w.oc1m().bits(0b110),
                        PwmMode::Mode2 => w.oc1m().bits(0b111),
                    };
                    mode_bits
                        .oc1pe().set_bit()  // 使能预加载
                });
                
                // 配置CCER寄存器：配置极性，使能通道
                tim.ccer().modify(|_, w| {
                    let polarity_bit = match polarity {
                        PwmPolarity::High => w.cc1p().clear_bit(),
                        PwmPolarity::Low => w.cc1p().set_bit(),
                    };
                    polarity_bit
                        .cc1e().set_bit()  // 使能通道
                });
                
                // 设置初始占空比
                tim.ccr1().write(|w| w.ccr1().bits(initial_duty));
            },
            PwmChannel::Channel2 => {
                // 配置CCMR1寄存器：PWM模式，使能预加载
                tim.ccmr1_output().write(|w| {
                    // PWM模式1：0b110，PWM模式2：0b111
                    let mode_bits = match mode {
                        PwmMode::Mode1 => w.oc2m().bits(0b110),
                        PwmMode::Mode2 => w.oc2m().bits(0b111),
                    };
                    mode_bits
                        .oc2pe().set_bit()  // 使能预加载
                });
                
                // 配置CCER寄存器：配置极性，使能通道
                tim.ccer().modify(|_, w| {
                    let polarity_bit = match polarity {
                        PwmPolarity::High => w.cc2p().clear_bit(),
                        PwmPolarity::Low => w.cc2p().set_bit(),
                    };
                    polarity_bit
                        .cc2e().set_bit()  // 使能通道
                });
                
                // 设置初始占空比
                tim.ccr2().write(|w| w.ccr2().bits(initial_duty));
            },
            PwmChannel::Channel3 => {
                // 配置CCMR2寄存器：PWM模式，使能预加载
                tim.ccmr2_output().write(|w| {
                    // PWM模式1：0b110，PWM模式2：0b111
                    let mode_bits = match mode {
                        PwmMode::Mode1 => w.oc3m().bits(0b110),
                        PwmMode::Mode2 => w.oc3m().bits(0b111),
                    };
                    mode_bits
                        .oc3pe().set_bit()  // 使能预加载
                });
                
                // 配置CCER寄存器：配置极性，使能通道
                tim.ccer().modify(|_, w| {
                    let polarity_bit = match polarity {
                        PwmPolarity::High => w.cc3p().clear_bit(),
                        PwmPolarity::Low => w.cc3p().set_bit(),
                    };
                    polarity_bit
                        .cc3e().set_bit()  // 使能通道
                });
                
                // 设置初始占空比
                tim.ccr3().write(|w| w.ccr3().bits(initial_duty));
            },
            PwmChannel::Channel4 => {
                // 配置CCMR2寄存器：PWM模式，使能预加载
                tim.ccmr2_output().write(|w| {
                    // PWM模式1：0b110，PWM模式2：0b111
                    let mode_bits = match mode {
                        PwmMode::Mode1 => w.oc4m().bits(0b110),
                        PwmMode::Mode2 => w.oc4m().bits(0b111),
                    };
                    mode_bits
                        .oc4pe().set_bit()  // 使能预加载
                });
                
                // 配置CCER寄存器：配置极性，使能通道
                tim.ccer().modify(|_, w| {
                    let polarity_bit = match polarity {
                        PwmPolarity::High => w.cc4p().clear_bit(),
                        PwmPolarity::Low => w.cc4p().set_bit(),
                    };
                    polarity_bit
                        .cc4e().set_bit()  // 使能通道
                });
                
                // 设置初始占空比
                tim.ccr4().write(|w| w.ccr4().bits(initial_duty));
            },
        }
    }
    
    /// 配置PWM通道（针对TIM2）
    unsafe fn config_pwm_channel_tim2(
        &self, 
        tim: &mut tim2::RegisterBlock, 
        channel: PwmChannel, 
        mode: PwmMode, 
        polarity: PwmPolarity,
        period: u16,
        prescaler: u16,
        initial_duty: u16
    ) {
        // 禁用定时器
        tim.cr1().write(|w| w.cen().clear_bit());
        // 配置预分频器和自动重装载值
        tim.psc().write(|w| w.psc().bits(prescaler));
        tim.arr().write(|w| w.arr().bits(period));
        
        // 配置PWM通道
        self.config_pwm_channel_tim2_inner(tim, channel, mode, polarity, initial_duty);
    }
    
    /// 配置PWM通道的内部方法（针对TIM2）
    unsafe fn config_pwm_channel_tim2_inner(
        &self, 
        tim: &mut tim2::RegisterBlock, 
        channel: PwmChannel, 
        mode: PwmMode, 
        polarity: PwmPolarity,
        initial_duty: u16
    ) {
        match channel {
            PwmChannel::Channel1 => {
                // 配置CCMR1寄存器：PWM模式，使能预加载
                tim.ccmr1_output().write(|w| {
                    // PWM模式1：0b110，PWM模式2：0b111
                    let mode_bits = match mode {
                        PwmMode::Mode1 => w.oc1m().bits(0b110),
                        PwmMode::Mode2 => w.oc1m().bits(0b111),
                    };
                    mode_bits
                        .oc1pe().set_bit()  // 使能预加载
                });
                
                // 配置CCER寄存器：配置极性，使能通道
                tim.ccer().modify(|_, w| {
                    let polarity_bit = match polarity {
                        PwmPolarity::High => w.cc1p().clear_bit(),
                        PwmPolarity::Low => w.cc1p().set_bit(),
                    };
                    polarity_bit
                        .cc1e().set_bit()  // 使能通道
                });
                
                // 设置初始占空比
                tim.ccr1().write(|w| w.ccr1().bits(initial_duty));
            },
            PwmChannel::Channel2 => {
                // 配置CCMR1寄存器：PWM模式，使能预加载
                tim.ccmr1_output().write(|w| {
                    // PWM模式1：0b110，PWM模式2：0b111
                    let mode_bits = match mode {
                        PwmMode::Mode1 => w.oc2m().bits(0b110),
                        PwmMode::Mode2 => w.oc2m().bits(0b111),
                    };
                    mode_bits
                        .oc2pe().set_bit()  // 使能预加载
                });
                
                // 配置CCER寄存器：配置极性，使能通道
                tim.ccer().modify(|_, w| {
                    let polarity_bit = match polarity {
                        PwmPolarity::High => w.cc2p().clear_bit(),
                        PwmPolarity::Low => w.cc2p().set_bit(),
                    };
                    polarity_bit
                        .cc2e().set_bit()  // 使能通道
                });
                
                // 设置初始占空比
                tim.ccr2().write(|w| w.ccr2().bits(initial_duty));
            },
            PwmChannel::Channel3 => {
                // 配置CCMR2寄存器：PWM模式，使能预加载
                tim.ccmr2_output().write(|w| {
                    // PWM模式1：0b110，PWM模式2：0b111
                    let mode_bits = match mode {
                        PwmMode::Mode1 => w.oc3m().bits(0b110),
                        PwmMode::Mode2 => w.oc3m().bits(0b111),
                    };
                    mode_bits
                        .oc3pe().set_bit()  // 使能预加载
                });
                
                // 配置CCER寄存器：配置极性，使能通道
                tim.ccer().modify(|_, w| {
                    let polarity_bit = match polarity {
                        PwmPolarity::High => w.cc3p().clear_bit(),
                        PwmPolarity::Low => w.cc3p().set_bit(),
                    };
                    polarity_bit
                        .cc3e().set_bit()  // 使能通道
                });
                
                // 设置初始占空比
                tim.ccr3().write(|w| w.ccr3().bits(initial_duty));
            },
            PwmChannel::Channel4 => {
                // 配置CCMR2寄存器：PWM模式，使能预加载
                tim.ccmr2_output().write(|w| {
                    // PWM模式1：0b110，PWM模式2：0b111
                    let mode_bits = match mode {
                        PwmMode::Mode1 => w.oc4m().bits(0b110),
                        PwmMode::Mode2 => w.oc4m().bits(0b111),
                    };
                    mode_bits
                        .oc4pe().set_bit()  // 使能预加载
                });
                
                // 配置CCER寄存器：配置极性，使能通道
                tim.ccer().modify(|_, w| {
                    let polarity_bit = match polarity {
                        PwmPolarity::High => w.cc4p().clear_bit(),
                        PwmPolarity::Low => w.cc4p().set_bit(),
                    };
                    polarity_bit
                        .cc4e().set_bit()  // 使能通道
                });
                
                // 设置初始占空比
                tim.ccr4().write(|w| w.ccr4().bits(initial_duty));
            },
        }
    }
    
    /// 配置PWM通道（针对TIM3）
    unsafe fn config_pwm_channel_tim3(
        &self, 
        tim: &mut tim3::RegisterBlock, 
        channel: PwmChannel, 
        mode: PwmMode, 
        polarity: PwmPolarity,
        period: u16,
        prescaler: u16,
        initial_duty: u16
    ) {
        // 禁用定时器
        tim.cr1().write(|w| w.cen().clear_bit());
        // 配置预分频器和自动重装载值
        tim.psc().write(|w| w.psc().bits(prescaler));
        tim.arr().write(|w| w.arr().bits(period));
        
        // 配置PWM通道
        self.config_pwm_channel_tim3_inner(tim, channel, mode, polarity, initial_duty);
    }
    
    /// 配置PWM通道的内部方法（针对TIM3）
    unsafe fn config_pwm_channel_tim3_inner(
        &self, 
        tim: &mut tim3::RegisterBlock, 
        channel: PwmChannel, 
        mode: PwmMode, 
        polarity: PwmPolarity,
        initial_duty: u16
    ) {
        match channel {
            PwmChannel::Channel1 => {
                // 配置CCMR1寄存器：PWM模式，使能预加载
                tim.ccmr1_output().write(|w| {
                    // PWM模式1：0b110，PWM模式2：0b111
                    let mode_bits = match mode {
                        PwmMode::Mode1 => w.oc1m().bits(0b110),
                        PwmMode::Mode2 => w.oc1m().bits(0b111),
                    };
                    mode_bits
                        .oc1pe().set_bit()  // 使能预加载
                });
                
                // 配置CCER寄存器：配置极性，使能通道
                tim.ccer().modify(|_, w| {
                    let polarity_bit = match polarity {
                        PwmPolarity::High => w.cc1p().clear_bit(),
                        PwmPolarity::Low => w.cc1p().set_bit(),
                    };
                    polarity_bit
                        .cc1e().set_bit()  // 使能通道
                });
                
                // 设置初始占空比
                tim.ccr1().write(|w| w.ccr1().bits(initial_duty));
            },
            PwmChannel::Channel2 => {
                // 配置CCMR1寄存器：PWM模式，使能预加载
                tim.ccmr1_output().write(|w| {
                    // PWM模式1：0b110，PWM模式2：0b111
                    let mode_bits = match mode {
                        PwmMode::Mode1 => w.oc2m().bits(0b110),
                        PwmMode::Mode2 => w.oc2m().bits(0b111),
                    };
                    mode_bits
                        .oc2pe().set_bit()  // 使能预加载
                });
                
                // 配置CCER寄存器：配置极性，使能通道
                tim.ccer().modify(|_, w| {
                    let polarity_bit = match polarity {
                        PwmPolarity::High => w.cc2p().clear_bit(),
                        PwmPolarity::Low => w.cc2p().set_bit(),
                    };
                    polarity_bit
                        .cc2e().set_bit()  // 使能通道
                });
                
                // 设置初始占空比
                tim.ccr2().write(|w| w.ccr2().bits(initial_duty));
            },
            PwmChannel::Channel3 => {
                // 配置CCMR2寄存器：PWM模式，使能预加载
                tim.ccmr2_output().write(|w| {
                    // PWM模式1：0b110，PWM模式2：0b111
                    let mode_bits = match mode {
                        PwmMode::Mode1 => w.oc3m().bits(0b110),
                        PwmMode::Mode2 => w.oc3m().bits(0b111),
                    };
                    mode_bits
                        .oc3pe().set_bit()  // 使能预加载
                });
                
                // 配置CCER寄存器：配置极性，使能通道
                tim.ccer().modify(|_, w| {
                    let polarity_bit = match polarity {
                        PwmPolarity::High => w.cc3p().clear_bit(),
                        PwmPolarity::Low => w.cc3p().set_bit(),
                    };
                    polarity_bit
                        .cc3e().set_bit()  // 使能通道
                });
                
                // 设置初始占空比
                tim.ccr3().write(|w| w.ccr3().bits(initial_duty));
            },
            PwmChannel::Channel4 => {
                // 配置CCMR2寄存器：PWM模式，使能预加载
                tim.ccmr2_output().write(|w| {
                    // PWM模式1：0b110，PWM模式2：0b111
                    let mode_bits = match mode {
                        PwmMode::Mode1 => w.oc4m().bits(0b110),
                        PwmMode::Mode2 => w.oc4m().bits(0b111),
                    };
                    mode_bits
                        .oc4pe().set_bit()  // 使能预加载
                });
                
                // 配置CCER寄存器：配置极性，使能通道
                tim.ccer().modify(|_, w| {
                    let polarity_bit = match polarity {
                        PwmPolarity::High => w.cc4p().clear_bit(),
                        PwmPolarity::Low => w.cc4p().set_bit(),
                    };
                    polarity_bit
                        .cc4e().set_bit()  // 使能通道
                });
                
                // 设置初始占空比
                tim.ccr4().write(|w| w.ccr4().bits(initial_duty));
            },
        }
    }
    
    /// 配置PWM通道（针对TIM4）
    unsafe fn config_pwm_channel_tim4(
        &self, 
        tim: &mut tim4::RegisterBlock, 
        channel: PwmChannel, 
        mode: PwmMode, 
        polarity: PwmPolarity,
        period: u16,
        prescaler: u16,
        initial_duty: u16
    ) {
        // 禁用定时器
        tim.cr1().write(|w| w.cen().clear_bit());
        // 配置预分频器和自动重装载值
        tim.psc().write(|w| w.psc().bits(prescaler));
        tim.arr().write(|w| w.arr().bits(period));
        
        // 配置PWM通道
        self.config_pwm_channel_tim4_inner(tim, channel, mode, polarity, initial_duty);
    }
    
    /// 配置PWM通道的内部方法（针对TIM4）
    unsafe fn config_pwm_channel_tim4_inner(
        &self, 
        tim: &mut tim4::RegisterBlock, 
        channel: PwmChannel, 
        mode: PwmMode, 
        polarity: PwmPolarity,
        initial_duty: u16
    ) {
        match channel {
            PwmChannel::Channel1 => {
                // 配置CCMR1寄存器：PWM模式，使能预加载
                tim.ccmr1_output().write(|w| {
                    // PWM模式1：0b110，PWM模式2：0b111
                    let mode_bits = match mode {
                        PwmMode::Mode1 => w.oc1m().bits(0b110),
                        PwmMode::Mode2 => w.oc1m().bits(0b111),
                    };
                    mode_bits
                        .oc1pe().set_bit()  // 使能预加载
                });
                
                // 配置CCER寄存器：配置极性，使能通道
                tim.ccer().modify(|_, w| {
                    let polarity_bit = match polarity {
                        PwmPolarity::High => w.cc1p().clear_bit(),
                        PwmPolarity::Low => w.cc1p().set_bit(),
                    };
                    polarity_bit
                        .cc1e().set_bit()  // 使能通道
                });
                
                // 设置初始占空比
                tim.ccr1().write(|w| w.ccr1().bits(initial_duty));
            },
            PwmChannel::Channel2 => {
                // 配置CCMR1寄存器：PWM模式，使能预加载
                tim.ccmr1_output().write(|w| {
                    // PWM模式1：0b110，PWM模式2：0b111
                    let mode_bits = match mode {
                        PwmMode::Mode1 => w.oc2m().bits(0b110),
                        PwmMode::Mode2 => w.oc2m().bits(0b111),
                    };
                    mode_bits
                        .oc2pe().set_bit()  // 使能预加载
                });
                
                // 配置CCER寄存器：配置极性，使能通道
                tim.ccer().modify(|_, w| {
                    let polarity_bit = match polarity {
                        PwmPolarity::High => w.cc2p().clear_bit(),
                        PwmPolarity::Low => w.cc2p().set_bit(),
                    };
                    polarity_bit
                        .cc2e().set_bit()  // 使能通道
                });
                
                // 设置初始占空比
                tim.ccr2().write(|w| w.ccr2().bits(initial_duty));
            },
            PwmChannel::Channel3 => {
                // 配置CCMR2寄存器：PWM模式，使能预加载
                tim.ccmr2_output().write(|w| {
                    // PWM模式1：0b110，PWM模式2：0b111
                    let mode_bits = match mode {
                        PwmMode::Mode1 => w.oc3m().bits(0b110),
                        PwmMode::Mode2 => w.oc3m().bits(0b111),
                    };
                    mode_bits
                        .oc3pe().set_bit()  // 使能预加载
                });
                
                // 配置CCER寄存器：配置极性，使能通道
                tim.ccer().modify(|_, w| {
                    let polarity_bit = match polarity {
                        PwmPolarity::High => w.cc3p().clear_bit(),
                        PwmPolarity::Low => w.cc3p().set_bit(),
                    };
                    polarity_bit
                        .cc3e().set_bit()  // 使能通道
                });
                
                // 设置初始占空比
                tim.ccr3().write(|w| w.ccr3().bits(initial_duty));
            },
            PwmChannel::Channel4 => {
                // 配置CCMR2寄存器：PWM模式，使能预加载
                tim.ccmr2_output().write(|w| {
                    // PWM模式1：0b110，PWM模式2：0b111
                    let mode_bits = match mode {
                        PwmMode::Mode1 => w.oc4m().bits(0b110),
                        PwmMode::Mode2 => w.oc4m().bits(0b111),
                    };
                    mode_bits
                        .oc4pe().set_bit()  // 使能预加载
                });
                
                // 配置CCER寄存器：配置极性，使能通道
                tim.ccer().modify(|_, w| {
                    let polarity_bit = match polarity {
                        PwmPolarity::High => w.cc4p().clear_bit(),
                        PwmPolarity::Low => w.cc4p().set_bit(),
                    };
                    polarity_bit
                        .cc4e().set_bit()  // 使能通道
                });
                
                // 设置初始占空比
                tim.ccr4().write(|w| w.ccr4().bits(initial_duty));
            },
        }
    }
    
    /// 设置PWM占空比
    pub unsafe fn set_pwm_duty(&self, channel: PwmChannel, duty: u16) {
        match self.number {
            TimerNumber::TIM1 => {
                let tim = self.get_tim1();
                self.set_pwm_duty_tim1(tim, channel, duty);
            },
            TimerNumber::TIM2 => {
                let tim = self.get_tim2();
                self.set_pwm_duty_tim2(tim, channel, duty);
            },
            TimerNumber::TIM3 => {
                let tim = self.get_tim3();
                self.set_pwm_duty_tim3(tim, channel, duty);
            },
            TimerNumber::TIM4 => {
                let tim = self.get_tim4();
                self.set_pwm_duty_tim4(tim, channel, duty);
            },
        }
    }
    
    /// 设置PWM占空比（针对TIM1）
    unsafe fn set_pwm_duty_tim1(
        &self, 
        tim: &mut tim1::RegisterBlock, 
        channel: PwmChannel, 
        duty: u16
    ) {
        // 参数有效性验证
        let period = tim.arr().read().arr().bits();
        assert!(duty <= period, "Duty value out of range (0-period)");
        
        match channel {
            PwmChannel::Channel1 => { tim.ccr1().write(|w| w.ccr1().bits(duty)); },
            PwmChannel::Channel2 => { tim.ccr2().write(|w| w.ccr2().bits(duty)); },
            PwmChannel::Channel3 => { tim.ccr3().write(|w| w.ccr3().bits(duty)); },
            PwmChannel::Channel4 => { tim.ccr4().write(|w| w.ccr4().bits(duty)); },
        }
    }
    
    /// 设置PWM占空比（针对TIM2）
    unsafe fn set_pwm_duty_tim2(
        &self, 
        tim: &mut tim2::RegisterBlock, 
        channel: PwmChannel, 
        duty: u16
    ) {
        // 参数有效性验证
        let period = tim.arr().read().arr().bits();
        assert!(duty <= period, "Duty value out of range (0-period)");
        
        match channel {
            PwmChannel::Channel1 => { tim.ccr1().write(|w| w.ccr1().bits(duty)); },
            PwmChannel::Channel2 => { tim.ccr2().write(|w| w.ccr2().bits(duty)); },
            PwmChannel::Channel3 => { tim.ccr3().write(|w| w.ccr3().bits(duty)); },
            PwmChannel::Channel4 => { tim.ccr4().write(|w| w.ccr4().bits(duty)); },
        }
    }
    
    /// 设置PWM占空比（针对TIM3）
    unsafe fn set_pwm_duty_tim3(
        &self, 
        tim: &mut tim3::RegisterBlock, 
        channel: PwmChannel, 
        duty: u16
    ) {
        // 参数有效性验证
        let period = tim.arr().read().arr().bits();
        assert!(duty <= period, "Duty value out of range (0-period)");
        
        match channel {
            PwmChannel::Channel1 => { tim.ccr1().write(|w| w.ccr1().bits(duty)); },
            PwmChannel::Channel2 => { tim.ccr2().write(|w| w.ccr2().bits(duty)); },
            PwmChannel::Channel3 => { tim.ccr3().write(|w| w.ccr3().bits(duty)); },
            PwmChannel::Channel4 => { tim.ccr4().write(|w| w.ccr4().bits(duty)); },
        }
    }
    
    /// 设置PWM占空比（针对TIM4）
    unsafe fn set_pwm_duty_tim4(
        &self, 
        tim: &mut tim4::RegisterBlock, 
        channel: PwmChannel, 
        duty: u16
    ) {
        // 参数有效性验证
        let period = tim.arr().read().arr().bits();
        assert!(duty <= period, "Duty value out of range (0-period)");
        
        match channel {
            PwmChannel::Channel1 => { tim.ccr1().write(|w| w.ccr1().bits(duty)); },
            PwmChannel::Channel2 => { tim.ccr2().write(|w| w.ccr2().bits(duty)); },
            PwmChannel::Channel3 => { tim.ccr3().write(|w| w.ccr3().bits(duty)); },
            PwmChannel::Channel4 => { tim.ccr4().write(|w| w.ccr4().bits(duty)); },
        }
    }
    
    /// 设置PWM频率
    pub unsafe fn set_pwm_frequency(&self, channel: PwmChannel, frequency: u32, duty_percent: u16) {
        // 参数有效性验证
        assert!(frequency > 0, "Frequency must be greater than 0");
        assert!(duty_percent <= 100, "Duty percent must be between 0 and 100");
        
        // 获取定时器时钟频率
        let timer_clock = self.get_timer_clock();
        
        // 计算预分频器和自动重装载值
        // 尝试找到合适的预分频器值，使得ARR在0~65535范围内
        let mut prescaler = 0;
        let mut arr = 0;
        let mut found = false;
        
        // 从预分频器0开始尝试
        for psc in 0..=65535 {
            let psc_val = psc as u32;
            let arr_val = (timer_clock / ((psc_val + 1) * frequency)) as u64 - 1;
            
            if arr_val <= 65535 {
                prescaler = psc_val as u16;
                arr = arr_val as u16;
                found = true;
                break;
            }
        }
        
        assert!(found, "Cannot find valid prescaler and period for the given frequency");
        
        // 计算实际占空比
        let actual_duty = (duty_percent as u32 * arr as u32 / 100) as u16;
        
        // 配置定时器
        match self.number {
            TimerNumber::TIM1 => {
                let tim = self.get_tim1();
                tim.cr1().write(|w| w.cen().clear_bit());  // 禁用定时器
                tim.psc().write(|w| w.psc().bits(prescaler));  // 预分频器
                tim.arr().write(|w| w.arr().bits(arr));  // 自动重装载值
                
                // 设置占空比
                self.set_pwm_duty_tim1(tim, channel, actual_duty);
                
                // 生成更新事件，更新影子寄存器
                tim.egr().write(|w| w.ug().set_bit());
                // 启用定时器
                tim.cr1().write(|w| w.cen().set_bit());
            },
            TimerNumber::TIM2 => {
                let tim = self.get_tim2();
                tim.cr1().write(|w| w.cen().clear_bit());  // 禁用定时器
                tim.psc().write(|w| w.psc().bits(prescaler));  // 预分频器
                tim.arr().write(|w| w.arr().bits(arr));  // 自动重装载值
                
                // 设置占空比
                self.set_pwm_duty_tim2(tim, channel, actual_duty);
                
                // 生成更新事件，更新影子寄存器
                tim.egr().write(|w| w.ug().set_bit());
                // 启用定时器
                tim.cr1().write(|w| w.cen().set_bit());
            },
            TimerNumber::TIM3 => {
                let tim = self.get_tim3();
                tim.cr1().write(|w| w.cen().clear_bit());  // 禁用定时器
                tim.psc().write(|w| w.psc().bits(prescaler));  // 预分频器
                tim.arr().write(|w| w.arr().bits(arr));  // 自动重装载值
                
                // 设置占空比
                self.set_pwm_duty_tim3(tim, channel, actual_duty);
                
                // 生成更新事件，更新影子寄存器
                tim.egr().write(|w| w.ug().set_bit());
                // 启用定时器
                tim.cr1().write(|w| w.cen().set_bit());
            },
            TimerNumber::TIM4 => {
                let tim = self.get_tim4();
                tim.cr1().write(|w| w.cen().clear_bit());  // 禁用定时器
                tim.psc().write(|w| w.psc().bits(prescaler));  // 预分频器
                tim.arr().write(|w| w.arr().bits(arr));  // 自动重装载值
                
                // 设置占空比
                self.set_pwm_duty_tim4(tim, channel, actual_duty);
                
                // 生成更新事件，更新影子寄存器
                tim.egr().write(|w| w.ug().set_bit());
                // 启用定时器
                tim.cr1().write(|w| w.cen().set_bit());
            },
        }
    }
    
    /// 启用PWM通道
    pub unsafe fn enable_pwm_channel(&self, channel: PwmChannel) {
        match self.number {
            TimerNumber::TIM1 => {
                let tim = self.get_tim1();
                match channel {
                    PwmChannel::Channel1 => { tim.ccer().modify(|_, w| w.cc1e().set_bit()); },
                    PwmChannel::Channel2 => { tim.ccer().modify(|_, w| w.cc2e().set_bit()); },
                    PwmChannel::Channel3 => { tim.ccer().modify(|_, w| w.cc3e().set_bit()); },
                    PwmChannel::Channel4 => { tim.ccer().modify(|_, w| w.cc4e().set_bit()); },
                }
            },
            TimerNumber::TIM2 => {
                let tim = self.get_tim2();
                match channel {
                    PwmChannel::Channel1 => { tim.ccer().modify(|_, w| w.cc1e().set_bit()); },
                    PwmChannel::Channel2 => { tim.ccer().modify(|_, w| w.cc2e().set_bit()); },
                    PwmChannel::Channel3 => { tim.ccer().modify(|_, w| w.cc3e().set_bit()); },
                    PwmChannel::Channel4 => { tim.ccer().modify(|_, w| w.cc4e().set_bit()); },
                }
            },
            TimerNumber::TIM3 => {
                let tim = self.get_tim3();
                match channel {
                    PwmChannel::Channel1 => { tim.ccer().modify(|_, w| w.cc1e().set_bit()); },
                    PwmChannel::Channel2 => { tim.ccer().modify(|_, w| w.cc2e().set_bit()); },
                    PwmChannel::Channel3 => { tim.ccer().modify(|_, w| w.cc3e().set_bit()); },
                    PwmChannel::Channel4 => { tim.ccer().modify(|_, w| w.cc4e().set_bit()); },
                }
            },
            TimerNumber::TIM4 => {
                let tim = self.get_tim4();
                match channel {
                    PwmChannel::Channel1 => { tim.ccer().modify(|_, w| w.cc1e().set_bit()); },
                    PwmChannel::Channel2 => { tim.ccer().modify(|_, w| w.cc2e().set_bit()); },
                    PwmChannel::Channel3 => { tim.ccer().modify(|_, w| w.cc3e().set_bit()); },
                    PwmChannel::Channel4 => { tim.ccer().modify(|_, w| w.cc4e().set_bit()); },
                }
            },
        }
    }
    
    /// 禁用PWM通道
    pub unsafe fn disable_pwm_channel(&self, channel: PwmChannel) {
        match self.number {
            TimerNumber::TIM1 => {
                let tim = self.get_tim1();
                match channel {
                    PwmChannel::Channel1 => { tim.ccer().modify(|_, w| w.cc1e().clear_bit()); },
                    PwmChannel::Channel2 => { tim.ccer().modify(|_, w| w.cc2e().clear_bit()); },
                    PwmChannel::Channel3 => { tim.ccer().modify(|_, w| w.cc3e().clear_bit()); },
                    PwmChannel::Channel4 => { tim.ccer().modify(|_, w| w.cc4e().clear_bit()); },
                }
            },
            TimerNumber::TIM2 => {
                let tim = self.get_tim2();
                match channel {
                    PwmChannel::Channel1 => { tim.ccer().modify(|_, w| w.cc1e().clear_bit()); },
                    PwmChannel::Channel2 => { tim.ccer().modify(|_, w| w.cc2e().clear_bit()); },
                    PwmChannel::Channel3 => { tim.ccer().modify(|_, w| w.cc3e().clear_bit()); },
                    PwmChannel::Channel4 => { tim.ccer().modify(|_, w| w.cc4e().clear_bit()); },
                }
            },
            TimerNumber::TIM3 => {
                let tim = self.get_tim3();
                match channel {
                    PwmChannel::Channel1 => { tim.ccer().modify(|_, w| w.cc1e().clear_bit()); },
                    PwmChannel::Channel2 => { tim.ccer().modify(|_, w| w.cc2e().clear_bit()); },
                    PwmChannel::Channel3 => { tim.ccer().modify(|_, w| w.cc3e().clear_bit()); },
                    PwmChannel::Channel4 => { tim.ccer().modify(|_, w| w.cc4e().clear_bit()); },
                }
            },
            TimerNumber::TIM4 => {
                let tim = self.get_tim4();
                match channel {
                    PwmChannel::Channel1 => { tim.ccer().modify(|_, w| w.cc1e().clear_bit()); },
                    PwmChannel::Channel2 => { tim.ccer().modify(|_, w| w.cc2e().clear_bit()); },
                    PwmChannel::Channel3 => { tim.ccer().modify(|_, w| w.cc3e().clear_bit()); },
                    PwmChannel::Channel4 => { tim.ccer().modify(|_, w| w.cc4e().clear_bit()); },
                }
            },
        }
    }
    

}

/// 预定义的定时器常量
pub const TIM1: Timer = Timer::new(TimerNumber::TIM1);
pub const TIM2: Timer = Timer::new(TimerNumber::TIM2);
pub const TIM3: Timer = Timer::new(TimerNumber::TIM3);
pub const TIM4: Timer = Timer::new(TimerNumber::TIM4);
