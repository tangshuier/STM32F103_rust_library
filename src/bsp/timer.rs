//! 定时器模块
//! 提供基本的定时器功能

// 屏蔽未使用代码警告
#![allow(unused)]

// 使用内部生成的设备驱动库
use library::*;
use core::ops::DerefMut;
use crate::bsp::rcc::RccDriver;

/// 定时器枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimerNumber {
    TIM1,  // 高级定时器（APB2）
    TIM2,  // 通用定时器（APB1）
    TIM3,  // 通用定时器（APB1）
    TIM4,  // 通用定时器（APB1）
    TIM5,  // 通用定时器（APB1）
    TIM6,  // 基本定时器（APB1）
    TIM7,  // 基本定时器（APB1）
    TIM8,  // 高级定时器（APB2）
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

/// 编码器模式枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EncoderMode {
    /// 仅在TI1边沿计数
    Mode1,
    /// 仅在TI2边沿计数
    Mode2,
    /// 在TI1和TI2边沿计数
    Mode3,
}

/// 编码器计数方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncoderDirection {
    /// 正向计数
    Up,
    /// 反向计数
    Down,
}

/// 输入捕获极性枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputCapturePolarity {
    /// 上升沿触发
    RisingEdge,
    /// 下降沿触发
    FallingEdge,
    /// 双边沿触发
    BothEdges,
}

/// 输入捕获预分频器枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputCapturePrescaler {
    /// 不分频
    Div1,
    /// 2分频
    Div2,
    /// 4分频
    Div4,
    /// 8分频
    Div8,
}

/// 定时器错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerError {
    /// 参数无效
    InvalidParameter,
    /// 占空比超出范围
    DutyCycleOutOfRange,
    /// 频率无效
    InvalidFrequency,
    /// 定时器不支持该功能
    UnsupportedFeature,
    /// 定时器未初始化
    NotInitialized,
    /// 定时器已在运行
    AlreadyRunning,
    /// 定时器未在运行
    NotRunning,
    /// 通道无效
    InvalidChannel,
    /// 未知错误
    Unknown,
}

/// 定时器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerStatus {
    /// 就绪
    Ready,
    /// 初始化中
    Initializing,
    /// 运行中
    Running,
    /// 已停止
    Stopped,
    /// 错误
    Error,
    /// PWM模式
    PwmMode,
}

/// 定时器结构体
pub struct Timer {
    number: TimerNumber,
}

impl TimerNumber {
    /// 获取定时器对应的APB总线
    pub const fn get_apb_bus(&self) -> ApbBus {
        match self {
            TimerNumber::TIM1 | TimerNumber::TIM8 => ApbBus::APB2,
            TimerNumber::TIM2 | TimerNumber::TIM3 | TimerNumber::TIM4 | 
            TimerNumber::TIM5 | TimerNumber::TIM6 | TimerNumber::TIM7 => ApbBus::APB1,
        }
    }
    
    /// 获取定时器对应的外设枚举值
    pub const fn get_peripheral(&self) -> TimerPeripheral {
        match self {
            TimerNumber::TIM1 => TimerPeripheral::TIM1,
            TimerNumber::TIM2 => TimerPeripheral::TIM2,
            TimerNumber::TIM3 => TimerPeripheral::TIM3,
            TimerNumber::TIM4 => TimerPeripheral::TIM4,
            TimerNumber::TIM5 => TimerPeripheral::TIM5,
            TimerNumber::TIM6 => TimerPeripheral::TIM6,
            TimerNumber::TIM7 => TimerPeripheral::TIM7,
            TimerNumber::TIM8 => TimerPeripheral::TIM8,
        }
    }
    
    /// 获取定时器基地址
    pub const fn get_base_address(&self) -> usize {
        match self {
            TimerNumber::TIM1 => 0x4001_2C00,
            TimerNumber::TIM2 => 0x4000_0000,
            TimerNumber::TIM3 => 0x4000_0400,
            TimerNumber::TIM4 => 0x4000_0800,
            TimerNumber::TIM5 => 0x4000_0C00,
            TimerNumber::TIM6 => 0x4000_1000,
            TimerNumber::TIM7 => 0x4000_1400,
            TimerNumber::TIM8 => 0x4001_3400,
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
    TIM5,
    TIM6,
    TIM7,
    TIM8,
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
    
    /// 获取TIM5寄存器块
    unsafe fn get_tim5(&self) -> &'static mut tim5::RegisterBlock {
        &mut *(TimerNumber::TIM5.get_base_address() as *mut tim5::RegisterBlock)
    }
    
    /// 获取TIM6寄存器块
    unsafe fn get_tim6(&self) -> &'static mut tim6::RegisterBlock {
        &mut *(TimerNumber::TIM6.get_base_address() as *mut tim6::RegisterBlock)
    }
    
    /// 获取TIM7寄存器块
    unsafe fn get_tim7(&self) -> &'static mut tim7::RegisterBlock {
        &mut *(TimerNumber::TIM7.get_base_address() as *mut tim7::RegisterBlock)
    }
    
    /// 获取TIM8寄存器块
    unsafe fn get_tim8(&self) -> &'static mut tim8::RegisterBlock {
        &mut *(TimerNumber::TIM8.get_base_address() as *mut tim8::RegisterBlock)
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
            TimerNumber::TIM5 => {
                rcc.apb1enr().modify(|_, w| w.tim5en().set_bit());
            },
            TimerNumber::TIM6 => {
                rcc.apb1enr().modify(|_, w| w.tim6en().set_bit());
            },
            TimerNumber::TIM7 => {
                rcc.apb1enr().modify(|_, w| w.tim7en().set_bit());
            },
            TimerNumber::TIM8 => {
                rcc.apb2enr().modify(|_, w| w.tim8en().set_bit());
            },
        };
    }
    
    /// 获取定时器时钟频率
    unsafe fn get_timer_clock(&self) -> u32 {
        // 使用RCC驱动获取时钟频率
        let rcc_driver = RccDriver::new();
        let clocks = rcc_driver.get_clocks_freq();
        
        match self.number {
            TimerNumber::TIM1 | TimerNumber::TIM8 => {
                // TIM1和TIM8位于APB2，当APB2预分频系数为1时，定时器时钟 = PCLK2
                // 否则，定时器时钟 = PCLK2 * 2
                let apb2_prescaler = clocks.sysclk_frequency / clocks.hclk_frequency;
                if apb2_prescaler == 1 {
                    clocks.pclk2_frequency
                } else {
                    clocks.pclk2_frequency * 2
                }
            },
            TimerNumber::TIM2 | TimerNumber::TIM3 | TimerNumber::TIM4 | 
            TimerNumber::TIM5 | TimerNumber::TIM6 | TimerNumber::TIM7 => {
                // TIM2-TIM7位于APB1，当APB1预分频系数为1时，定时器时钟 = PCLK1
                // 否则，定时器时钟 = PCLK1 * 2
                let apb1_prescaler = clocks.hclk_frequency / clocks.pclk1_frequency;
                if apb1_prescaler == 1 {
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
    /// 
    /// # 返回值
    /// * `Ok(())` - 初始化成功
    /// * `Err(TimerError)` - 初始化失败
    pub unsafe fn init(&self, prescaler: u16, period: u16) -> Result<(), TimerError> {
        // 参数有效性验证
        if period == 0 {
            return Err(TimerError::InvalidParameter);
        }
        
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
            TimerNumber::TIM5 => {
                let tim = self.get_tim5();
                tim.cr1().write(|w| w.cen().clear_bit());  // 禁用定时器
                tim.psc().write(|w| w.psc().bits(prescaler));  // 预分频器
                tim.arr().write(|w| w.arr().bits(period));  // 自动重装载值
                tim.cnt().write(|w| w.cnt().bits(0));  // 清零计数器
                tim.egr().write(|w| w.ug().set_bit());  // 生成更新事件
                tim.sr().write(|w| w.uif().clear_bit());  // 清除更新中断标志
            },
            TimerNumber::TIM6 => {
                let tim = self.get_tim6();
                tim.cr1().write(|w| w.cen().clear_bit());  // 禁用定时器
                tim.psc().write(|w| w.psc().bits(prescaler));  // 预分频器
                tim.arr().write(|w| w.arr().bits(period));  // 自动重装载值
                tim.cnt().write(|w| w.cnt().bits(0));  // 清零计数器
                tim.egr().write(|w| w.ug().set_bit());  // 生成更新事件
                tim.sr().write(|w| w.uif().clear_bit());  // 清除更新中断标志
            },
            TimerNumber::TIM7 => {
                let tim = self.get_tim7();
                tim.cr1().write(|w| w.cen().clear_bit());  // 禁用定时器
                tim.psc().write(|w| w.psc().bits(prescaler));  // 预分频器
                tim.arr().write(|w| w.arr().bits(period));  // 自动重装载值
                tim.cnt().write(|w| w.cnt().bits(0));  // 清零计数器
                tim.egr().write(|w| w.ug().set_bit());  // 生成更新事件
                tim.sr().write(|w| w.uif().clear_bit());  // 清除更新中断标志
            },
            TimerNumber::TIM8 => {
                let tim = self.get_tim8();
                tim.cr1().write(|w| w.cen().clear_bit());  // 禁用定时器
                tim.psc().write(|w| w.psc().bits(prescaler));  // 预分频器
                tim.arr().write(|w| w.arr().bits(period));  // 自动重装载值
                tim.cnt().write(|w| w.cnt().bits(0));  // 清零计数器
                tim.egr().write(|w| w.ug().set_bit());  // 生成更新事件
                tim.sr().write(|w| w.uif().clear_bit());  // 清除更新中断标志
            },
        }
        
        Ok(())
    }
    
    /// 启动定时器
    /// 
    /// # 返回值
    /// * `Ok(())` - 启动成功
    /// * `Err(TimerError)` - 启动失败
    pub unsafe fn start(&self) -> Result<(), TimerError> {
        // 检查定时器是否已经在运行
        let is_running = match self.number {
            TimerNumber::TIM1 => self.get_tim1().cr1().read().cen().bit_is_set(),
            TimerNumber::TIM2 => self.get_tim2().cr1().read().cen().bit_is_set(),
            TimerNumber::TIM3 => self.get_tim3().cr1().read().cen().bit_is_set(),
            TimerNumber::TIM4 => self.get_tim4().cr1().read().cen().bit_is_set(),
            TimerNumber::TIM5 => self.get_tim5().cr1().read().cen().bit_is_set(),
            TimerNumber::TIM6 => self.get_tim6().cr1().read().cen().bit_is_set(),
            TimerNumber::TIM7 => self.get_tim7().cr1().read().cen().bit_is_set(),
            TimerNumber::TIM8 => self.get_tim8().cr1().read().cen().bit_is_set(),
        };
        
        if is_running {
            return Err(TimerError::AlreadyRunning);
        }
        
        // 启动定时器
        match self.number {
            TimerNumber::TIM1 => { self.get_tim1().cr1().modify(|_, w| w.cen().set_bit()); },
            TimerNumber::TIM2 => { self.get_tim2().cr1().modify(|_, w| w.cen().set_bit()); },
            TimerNumber::TIM3 => { self.get_tim3().cr1().modify(|_, w| w.cen().set_bit()); },
            TimerNumber::TIM4 => { self.get_tim4().cr1().modify(|_, w| w.cen().set_bit()); },
            TimerNumber::TIM5 => { self.get_tim5().cr1().modify(|_, w| w.cen().set_bit()); },
            TimerNumber::TIM6 => { self.get_tim6().cr1().modify(|_, w| w.cen().set_bit()); },
            TimerNumber::TIM7 => { self.get_tim7().cr1().modify(|_, w| w.cen().set_bit()); },
            TimerNumber::TIM8 => { self.get_tim8().cr1().modify(|_, w| w.cen().set_bit()); },
        }
        
        Ok(())
    }
    
    /// 停止定时器
    /// 
    /// # 返回值
    /// * `Ok(())` - 停止成功
    /// * `Err(TimerError)` - 停止失败
    pub unsafe fn stop(&self) -> Result<(), TimerError> {
        // 检查定时器是否已经停止
        let is_running = match self.number {
            TimerNumber::TIM1 => self.get_tim1().cr1().read().cen().bit_is_set(),
            TimerNumber::TIM2 => self.get_tim2().cr1().read().cen().bit_is_set(),
            TimerNumber::TIM3 => self.get_tim3().cr1().read().cen().bit_is_set(),
            TimerNumber::TIM4 => self.get_tim4().cr1().read().cen().bit_is_set(),
            TimerNumber::TIM5 => self.get_tim5().cr1().read().cen().bit_is_set(),
            TimerNumber::TIM6 => self.get_tim6().cr1().read().cen().bit_is_set(),
            TimerNumber::TIM7 => self.get_tim7().cr1().read().cen().bit_is_set(),
            TimerNumber::TIM8 => self.get_tim8().cr1().read().cen().bit_is_set(),
        };
        
        if !is_running {
            return Err(TimerError::NotRunning);
        }
        
        // 停止定时器
        match self.number {
            TimerNumber::TIM1 => { self.get_tim1().cr1().modify(|_, w| w.cen().clear_bit()); },
            TimerNumber::TIM2 => { self.get_tim2().cr1().modify(|_, w| w.cen().clear_bit()); },
            TimerNumber::TIM3 => { self.get_tim3().cr1().modify(|_, w| w.cen().clear_bit()); },
            TimerNumber::TIM4 => { self.get_tim4().cr1().modify(|_, w| w.cen().clear_bit()); },
            TimerNumber::TIM5 => { self.get_tim5().cr1().modify(|_, w| w.cen().clear_bit()); },
            TimerNumber::TIM6 => { self.get_tim6().cr1().modify(|_, w| w.cen().clear_bit()); },
            TimerNumber::TIM7 => { self.get_tim7().cr1().modify(|_, w| w.cen().clear_bit()); },
            TimerNumber::TIM8 => { self.get_tim8().cr1().modify(|_, w| w.cen().clear_bit()); },
        }
        
        Ok(())
    }
    
    /// 重置定时器
    /// 
    /// # 返回值
    /// * `Ok(())` - 重置成功
    /// * `Err(TimerError)` - 重置失败
    pub unsafe fn reset(&self) -> Result<(), TimerError> {
        // 重置定时器计数器和状态寄存器
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
            TimerNumber::TIM5 => {
                let tim = self.get_tim5();
                tim.cnt().write(|w| w.cnt().bits(0));
                tim.sr().write(|w| w.uif().clear_bit());
            },
            TimerNumber::TIM6 => {
                let tim = self.get_tim6();
                tim.cnt().write(|w| w.cnt().bits(0));
                tim.sr().write(|w| w.uif().clear_bit());
            },
            TimerNumber::TIM7 => {
                let tim = self.get_tim7();
                tim.cnt().write(|w| w.cnt().bits(0));
                tim.sr().write(|w| w.uif().clear_bit());
            },
            TimerNumber::TIM8 => {
                let tim = self.get_tim8();
                tim.cnt().write(|w| w.cnt().bits(0));
                tim.sr().write(|w| w.uif().clear_bit());
            },
        }
        
        Ok(())
    }
    
    /// 检查更新中断标志
    pub unsafe fn has_update(&self) -> bool {
        match self.number {
            TimerNumber::TIM1 => self.get_tim1().sr().read().uif().bit_is_set(),
            TimerNumber::TIM2 => self.get_tim2().sr().read().uif().bit_is_set(),
            TimerNumber::TIM3 => self.get_tim3().sr().read().uif().bit_is_set(),
            TimerNumber::TIM4 => self.get_tim4().sr().read().uif().bit_is_set(),
            TimerNumber::TIM5 => self.get_tim5().sr().read().uif().bit_is_set(),
            TimerNumber::TIM6 => self.get_tim6().sr().read().uif().bit_is_set(),
            TimerNumber::TIM7 => self.get_tim7().sr().read().uif().bit_is_set(),
            TimerNumber::TIM8 => self.get_tim8().sr().read().uif().bit_is_set(),
        }
    }
    
    /// 清除更新中断标志
    pub unsafe fn clear_update(&self) {
        match self.number {
            TimerNumber::TIM1 => { self.get_tim1().sr().write(|w| w.uif().clear_bit()); },
            TimerNumber::TIM2 => { self.get_tim2().sr().write(|w| w.uif().clear_bit()); },
            TimerNumber::TIM3 => { self.get_tim3().sr().write(|w| w.uif().clear_bit()); },
            TimerNumber::TIM4 => { self.get_tim4().sr().write(|w| w.uif().clear_bit()); },
            TimerNumber::TIM5 => { self.get_tim5().sr().write(|w| w.uif().clear_bit()); },
            TimerNumber::TIM6 => { self.get_tim6().sr().write(|w| w.uif().clear_bit()); },
            TimerNumber::TIM7 => { self.get_tim7().sr().write(|w| w.uif().clear_bit()); },
            TimerNumber::TIM8 => { self.get_tim8().sr().write(|w| w.uif().clear_bit()); },
        }
    }
    
    /// 获取当前计数值
    pub unsafe fn get_count(&self) -> u16 {
        match self.number {
            TimerNumber::TIM1 => self.get_tim1().cnt().read().cnt().bits(),
            TimerNumber::TIM2 => self.get_tim2().cnt().read().cnt().bits(),
            TimerNumber::TIM3 => self.get_tim3().cnt().read().cnt().bits(),
            TimerNumber::TIM4 => self.get_tim4().cnt().read().cnt().bits(),
            TimerNumber::TIM5 => self.get_tim5().cnt().read().cnt().bits(),
            TimerNumber::TIM6 => self.get_tim6().cnt().read().cnt().bits(),
            TimerNumber::TIM7 => self.get_tim7().cnt().read().cnt().bits(),
            TimerNumber::TIM8 => self.get_tim8().cnt().read().cnt().bits(),
        }
    }
    
    /// 设置计数值
    /// 
    /// # 参数
    /// * `count` - 要设置的计数值（0-65535）
    /// 
    /// # 返回值
    /// * `Ok(())` - 设置成功
    /// * `Err(TimerError)` - 设置失败
    pub unsafe fn set_count(&self, count: u16) -> Result<(), TimerError> {
        // 参数有效性验证已由硬件保证（count为u16类型）
        
        match self.number {
            TimerNumber::TIM1 => { self.get_tim1().cnt().write(|w| w.cnt().bits(count)); },
            TimerNumber::TIM2 => { self.get_tim2().cnt().write(|w| w.cnt().bits(count)); },
            TimerNumber::TIM3 => { self.get_tim3().cnt().write(|w| w.cnt().bits(count)); },
            TimerNumber::TIM4 => { self.get_tim4().cnt().write(|w| w.cnt().bits(count)); },
            TimerNumber::TIM5 => { self.get_tim5().cnt().write(|w| w.cnt().bits(count)); },
            TimerNumber::TIM6 => { self.get_tim6().cnt().write(|w| w.cnt().bits(count)); },
            TimerNumber::TIM7 => { self.get_tim7().cnt().write(|w| w.cnt().bits(count)); },
            TimerNumber::TIM8 => { self.get_tim8().cnt().write(|w| w.cnt().bits(count)); },
        }
        
        Ok(())
    }
    
    /// 使能更新中断
    pub unsafe fn enable_update_interrupt(&self) {
        match self.number {
            TimerNumber::TIM1 => { self.get_tim1().dier().modify(|_, w| w.uie().set_bit()); },
            TimerNumber::TIM2 => { self.get_tim2().dier().modify(|_, w| w.uie().set_bit()); },
            TimerNumber::TIM3 => { self.get_tim3().dier().modify(|_, w| w.uie().set_bit()); },
            TimerNumber::TIM4 => { self.get_tim4().dier().modify(|_, w| w.uie().set_bit()); },
            TimerNumber::TIM5 => { self.get_tim5().dier().modify(|_, w| w.uie().set_bit()); },
            TimerNumber::TIM6 => { self.get_tim6().dier().modify(|_, w| w.uie().set_bit()); },
            TimerNumber::TIM7 => { self.get_tim7().dier().modify(|_, w| w.uie().set_bit()); },
            TimerNumber::TIM8 => { self.get_tim8().dier().modify(|_, w| w.uie().set_bit()); },
        }
    }
    
    /// 禁用更新中断
    pub unsafe fn disable_update_interrupt(&self) {
        match self.number {
            TimerNumber::TIM1 => { self.get_tim1().dier().modify(|_, w| w.uie().clear_bit()); },
            TimerNumber::TIM2 => { self.get_tim2().dier().modify(|_, w| w.uie().clear_bit()); },
            TimerNumber::TIM3 => { self.get_tim3().dier().modify(|_, w| w.uie().clear_bit()); },
            TimerNumber::TIM4 => { self.get_tim4().dier().modify(|_, w| w.uie().clear_bit()); },
            TimerNumber::TIM5 => { self.get_tim5().dier().modify(|_, w| w.uie().clear_bit()); },
            TimerNumber::TIM6 => { self.get_tim6().dier().modify(|_, w| w.uie().clear_bit()); },
            TimerNumber::TIM7 => { self.get_tim7().dier().modify(|_, w| w.uie().clear_bit()); },
            TimerNumber::TIM8 => { self.get_tim8().dier().modify(|_, w| w.uie().clear_bit()); },
        }
    }
    
    /// 初始化PWM通道
    /// 
    /// # 参数
    /// * `channel` - PWM通道
    /// * `mode` - PWM模式
    /// * `polarity` - PWM极性
    /// * `period` - 自动重装载值（0-65535）
    /// * `prescaler` - 预分频器值（0-65535）
    /// * `initial_duty` - 初始占空比（0-period）
    /// 
    /// # 返回值
    /// * `Ok(())` - 初始化成功
    /// * `Err(TimerError)` - 初始化失败
    pub unsafe fn init_pwm(
        &self, 
        channel: PwmChannel, 
        mode: PwmMode, 
        polarity: PwmPolarity,
        period: u16,
        prescaler: u16,
        initial_duty: u16
    ) -> Result<(), TimerError> {
        // 参数有效性验证
        if period == 0 {
            return Err(TimerError::InvalidParameter);
        }
        
        if initial_duty > period {
            return Err(TimerError::DutyCycleOutOfRange);
        }
        
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
            TimerNumber::TIM5 => {
                let tim = self.get_tim5();
                self.config_pwm_channel_tim5(tim, channel, mode, polarity, period, prescaler, initial_duty);
                
                // 生成更新事件，更新影子寄存器
                tim.egr().write(|w| w.ug().set_bit());
                // 清除更新中断标志
                tim.sr().write(|w| w.uif().clear_bit());
                // 启用定时器
                tim.cr1().write(|w| w.cen().set_bit());
            },
            TimerNumber::TIM6 | TimerNumber::TIM7 => {
                // 基本定时器不支持PWM功能
                return Err(TimerError::UnsupportedFeature);
            },
            TimerNumber::TIM8 => {
                let tim = self.get_tim8();
                self.config_pwm_channel_tim8(tim, channel, mode, polarity, period, prescaler, initial_duty);
                
                // 对于高级定时器TIM8，需要启用主输出
                tim.bdtr().modify(|_, w| w.moe().set_bit());
                
                // 生成更新事件，更新影子寄存器
                tim.egr().write(|w| w.ug().set_bit());
                // 清除更新中断标志
                tim.sr().write(|w| w.uif().clear_bit());
                // 启用定时器
                tim.cr1().write(|w| w.cen().set_bit());
            },
        }
        
        Ok(())
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
    
    /// 配置PWM通道（针对TIM5）
    unsafe fn config_pwm_channel_tim5(
        &self, 
        tim: &mut tim5::RegisterBlock, 
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
        self.config_pwm_channel_tim5_inner(tim, channel, mode, polarity, initial_duty);
    }
    
    /// 配置PWM通道的内部方法（针对TIM5）
    unsafe fn config_pwm_channel_tim5_inner(
        &self, 
        tim: &mut tim5::RegisterBlock, 
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
    
    /// 配置PWM通道（针对TIM8）
    unsafe fn config_pwm_channel_tim8(
        &self, 
        tim: &mut tim8::RegisterBlock, 
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
        self.config_pwm_channel_tim8_inner(tim, channel, mode, polarity, initial_duty);
    }
    
    /// 配置PWM通道的内部方法（针对TIM8）
    unsafe fn config_pwm_channel_tim8_inner(
        &self, 
        tim: &mut tim8::RegisterBlock, 
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
    /// 
    /// # 参数
    /// * `channel` - PWM通道
    /// * `duty` - 占空比（0-当前周期值）
    /// 
    /// # 返回值
    /// * `Ok(())` - 设置成功
    /// * `Err(TimerError)` - 设置失败
    pub unsafe fn set_pwm_duty(&self, channel: PwmChannel, duty: u16) -> Result<(), TimerError> {
        match self.number {
            TimerNumber::TIM1 => {
                let tim = self.get_tim1();
                self.set_pwm_duty_tim1(tim, channel, duty)
            },
            TimerNumber::TIM2 => {
                let tim = self.get_tim2();
                self.set_pwm_duty_tim2(tim, channel, duty)
            },
            TimerNumber::TIM3 => {
                let tim = self.get_tim3();
                self.set_pwm_duty_tim3(tim, channel, duty)
            },
            TimerNumber::TIM4 => {
                let tim = self.get_tim4();
                self.set_pwm_duty_tim4(tim, channel, duty)
            },
            TimerNumber::TIM5 => {
                let tim = self.get_tim5();
                self.set_pwm_duty_tim5(tim, channel, duty)
            },
            TimerNumber::TIM6 | TimerNumber::TIM7 => {
                // 基本定时器不支持PWM功能
                Err(TimerError::UnsupportedFeature)
            },
            TimerNumber::TIM8 => {
                let tim = self.get_tim8();
                self.set_pwm_duty_tim8(tim, channel, duty)
            },
        }
    }
    
    /// 设置PWM占空比（针对TIM1）
    unsafe fn set_pwm_duty_tim1(
        &self, 
        tim: &mut tim1::RegisterBlock, 
        channel: PwmChannel, 
        duty: u16
    ) -> Result<(), TimerError> {
        // 参数有效性验证
        let period = tim.arr().read().arr().bits();
        if duty > period {
            return Err(TimerError::DutyCycleOutOfRange);
        }
        
        match channel {
            PwmChannel::Channel1 => { tim.ccr1().write(|w| w.ccr1().bits(duty)); },
            PwmChannel::Channel2 => { tim.ccr2().write(|w| w.ccr2().bits(duty)); },
            PwmChannel::Channel3 => { tim.ccr3().write(|w| w.ccr3().bits(duty)); },
            PwmChannel::Channel4 => { tim.ccr4().write(|w| w.ccr4().bits(duty)); },
        }
        
        Ok(())
    }
    
    /// 设置PWM占空比（针对TIM2）
    unsafe fn set_pwm_duty_tim2(
        &self, 
        tim: &mut tim2::RegisterBlock, 
        channel: PwmChannel, 
        duty: u16
    ) -> Result<(), TimerError> {
        // 参数有效性验证
        let period = tim.arr().read().arr().bits();
        if duty > period {
            return Err(TimerError::DutyCycleOutOfRange);
        }
        
        match channel {
            PwmChannel::Channel1 => { tim.ccr1().write(|w| w.ccr1().bits(duty)); },
            PwmChannel::Channel2 => { tim.ccr2().write(|w| w.ccr2().bits(duty)); },
            PwmChannel::Channel3 => { tim.ccr3().write(|w| w.ccr3().bits(duty)); },
            PwmChannel::Channel4 => { tim.ccr4().write(|w| w.ccr4().bits(duty)); },
        }
        
        Ok(())
    }
    
    /// 设置PWM占空比（针对TIM3）
    unsafe fn set_pwm_duty_tim3(
        &self, 
        tim: &mut tim3::RegisterBlock, 
        channel: PwmChannel, 
        duty: u16
    ) -> Result<(), TimerError> {
        // 参数有效性验证
        let period = tim.arr().read().arr().bits();
        if duty > period {
            return Err(TimerError::DutyCycleOutOfRange);
        }
        
        match channel {
            PwmChannel::Channel1 => { tim.ccr1().write(|w| w.ccr1().bits(duty)); },
            PwmChannel::Channel2 => { tim.ccr2().write(|w| w.ccr2().bits(duty)); },
            PwmChannel::Channel3 => { tim.ccr3().write(|w| w.ccr3().bits(duty)); },
            PwmChannel::Channel4 => { tim.ccr4().write(|w| w.ccr4().bits(duty)); },
        }
        
        Ok(())
    }
    
    /// 设置PWM占空比（针对TIM4）
    unsafe fn set_pwm_duty_tim4(
        &self, 
        tim: &mut tim4::RegisterBlock, 
        channel: PwmChannel, 
        duty: u16
    ) -> Result<(), TimerError> {
        // 参数有效性验证
        let period = tim.arr().read().arr().bits();
        if duty > period {
            return Err(TimerError::DutyCycleOutOfRange);
        }
        
        match channel {
            PwmChannel::Channel1 => { tim.ccr1().write(|w| w.ccr1().bits(duty)); },
            PwmChannel::Channel2 => { tim.ccr2().write(|w| w.ccr2().bits(duty)); },
            PwmChannel::Channel3 => { tim.ccr3().write(|w| w.ccr3().bits(duty)); },
            PwmChannel::Channel4 => { tim.ccr4().write(|w| w.ccr4().bits(duty)); },
        }
        
        Ok(())
    }
    
    /// 设置PWM占空比（针对TIM5）
    unsafe fn set_pwm_duty_tim5(
        &self, 
        tim: &mut tim5::RegisterBlock, 
        channel: PwmChannel, 
        duty: u16
    ) -> Result<(), TimerError> {
        // 参数有效性验证
        let period = tim.arr().read().arr().bits();
        if duty > period {
            return Err(TimerError::DutyCycleOutOfRange);
        }
        
        match channel {
            PwmChannel::Channel1 => { tim.ccr1().write(|w| w.ccr1().bits(duty)); },
            PwmChannel::Channel2 => { tim.ccr2().write(|w| w.ccr2().bits(duty)); },
            PwmChannel::Channel3 => { tim.ccr3().write(|w| w.ccr3().bits(duty)); },
            PwmChannel::Channel4 => { tim.ccr4().write(|w| w.ccr4().bits(duty)); },
        }
        
        Ok(())
    }
    
    /// 设置PWM占空比（针对TIM8）
    unsafe fn set_pwm_duty_tim8(
        &self, 
        tim: &mut tim8::RegisterBlock, 
        channel: PwmChannel, 
        duty: u16
    ) -> Result<(), TimerError> {
        // 参数有效性验证
        let period = tim.arr().read().arr().bits();
        if duty > period {
            return Err(TimerError::DutyCycleOutOfRange);
        }
        
        match channel {
            PwmChannel::Channel1 => { tim.ccr1().write(|w| w.ccr1().bits(duty)); },
            PwmChannel::Channel2 => { tim.ccr2().write(|w| w.ccr2().bits(duty)); },
            PwmChannel::Channel3 => { tim.ccr3().write(|w| w.ccr3().bits(duty)); },
            PwmChannel::Channel4 => { tim.ccr4().write(|w| w.ccr4().bits(duty)); },
        }
        
        Ok(())
    }
    
    /// 设置PWM频率
    /// 
    /// # 参数
    /// * `channel` - PWM通道
    /// * `frequency` - 频率（Hz）
    /// * `duty_percent` - 占空比百分比（0-100）
    /// 
    /// # 返回值
    /// * `Ok(())` - 设置成功
    /// * `Err(TimerError)` - 设置失败
    pub unsafe fn set_pwm_frequency(&self, channel: PwmChannel, frequency: u32, duty_percent: u16) -> Result<(), TimerError> {
        // 参数有效性验证
        if frequency == 0 {
            return Err(TimerError::InvalidFrequency);
        }
        
        if duty_percent > 100 {
            return Err(TimerError::DutyCycleOutOfRange);
        }
        
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
        
        if !found {
            return Err(TimerError::InvalidFrequency);
        }
        
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
                self.set_pwm_duty_tim1(tim, channel, actual_duty)?;
                
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
                self.set_pwm_duty_tim2(tim, channel, actual_duty)?;
                
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
                self.set_pwm_duty_tim3(tim, channel, actual_duty)?;
                
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
                self.set_pwm_duty_tim4(tim, channel, actual_duty)?;
                
                // 生成更新事件，更新影子寄存器
                tim.egr().write(|w| w.ug().set_bit());
                // 启用定时器
                tim.cr1().write(|w| w.cen().set_bit());
            },
            TimerNumber::TIM5 => {
                let tim = self.get_tim5();
                tim.cr1().write(|w| w.cen().clear_bit());  // 禁用定时器
                tim.psc().write(|w| w.psc().bits(prescaler));  // 预分频器
                tim.arr().write(|w| w.arr().bits(arr));  // 自动重装载值
                
                // 设置占空比
                self.set_pwm_duty_tim5(tim, channel, actual_duty)?;
                
                // 生成更新事件，更新影子寄存器
                tim.egr().write(|w| w.ug().set_bit());
                // 启用定时器
                tim.cr1().write(|w| w.cen().set_bit());
            },
            TimerNumber::TIM6 | TimerNumber::TIM7 => {
                // 基本定时器不支持PWM功能
                return Err(TimerError::UnsupportedFeature);
            },
            TimerNumber::TIM8 => {
                let tim = self.get_tim8();
                tim.cr1().write(|w| w.cen().clear_bit());  // 禁用定时器
                tim.psc().write(|w| w.psc().bits(prescaler));  // 预分频器
                tim.arr().write(|w| w.arr().bits(arr));  // 自动重装载值
                
                // 设置占空比
                self.set_pwm_duty_tim8(tim, channel, actual_duty)?;
                
                // 生成更新事件，更新影子寄存器
                tim.egr().write(|w| w.ug().set_bit());
                // 启用定时器
                tim.cr1().write(|w| w.cen().set_bit());
            },
        }
        
        Ok(())
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
            TimerNumber::TIM5 => {
                let tim = self.get_tim5();
                match channel {
                    PwmChannel::Channel1 => { tim.ccer().modify(|_, w| w.cc1e().set_bit()); },
                    PwmChannel::Channel2 => { tim.ccer().modify(|_, w| w.cc2e().set_bit()); },
                    PwmChannel::Channel3 => { tim.ccer().modify(|_, w| w.cc3e().set_bit()); },
                    PwmChannel::Channel4 => { tim.ccer().modify(|_, w| w.cc4e().set_bit()); },
                }
            },
            TimerNumber::TIM8 => {
                let tim = self.get_tim8();
                match channel {
                    PwmChannel::Channel1 => { tim.ccer().modify(|_, w| w.cc1e().set_bit()); },
                    PwmChannel::Channel2 => { tim.ccer().modify(|_, w| w.cc2e().set_bit()); },
                    PwmChannel::Channel3 => { tim.ccer().modify(|_, w| w.cc3e().set_bit()); },
                    PwmChannel::Channel4 => { tim.ccer().modify(|_, w| w.cc4e().set_bit()); },
                }
            },
            _ => {}, // 基本定时器不支持PWM通道
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
            TimerNumber::TIM5 => {
                let tim = self.get_tim5();
                match channel {
                    PwmChannel::Channel1 => { tim.ccer().modify(|_, w| w.cc1e().clear_bit()); },
                    PwmChannel::Channel2 => { tim.ccer().modify(|_, w| w.cc2e().clear_bit()); },
                    PwmChannel::Channel3 => { tim.ccer().modify(|_, w| w.cc3e().clear_bit()); },
                    PwmChannel::Channel4 => { tim.ccer().modify(|_, w| w.cc4e().clear_bit()); },
                }
            },
            TimerNumber::TIM8 => {
                let tim = self.get_tim8();
                match channel {
                    PwmChannel::Channel1 => { tim.ccer().modify(|_, w| w.cc1e().clear_bit()); },
                    PwmChannel::Channel2 => { tim.ccer().modify(|_, w| w.cc2e().clear_bit()); },
                    PwmChannel::Channel3 => { tim.ccer().modify(|_, w| w.cc3e().clear_bit()); },
                    PwmChannel::Channel4 => { tim.ccer().modify(|_, w| w.cc4e().clear_bit()); },
                }
            },
            _ => {}, // 基本定时器不支持PWM通道
        }
    }
    
    /// 初始化编码器模式
    /// 
    /// # 参数
    /// * `mode` - 编码器模式
    /// * `prescaler` - 预分频器值（0-65535）
    /// 
    /// # 返回值
    /// * `Ok(())` - 初始化成功
    /// * `Err(TimerError)` - 初始化失败
    pub unsafe fn init_encoder(&self, mode: EncoderMode, prescaler: u16) -> Result<(), TimerError> {
        // 启用定时器时钟
        self.enable_clock();
        
        // 配置编码器模式
        match self.number {
            TimerNumber::TIM1 => {
                let tim = self.get_tim1();
                self.config_encoder_tim1(tim, mode, prescaler);
            },
            TimerNumber::TIM2 => {
                let tim = self.get_tim2();
                self.config_encoder_tim2(tim, mode, prescaler);
            },
            TimerNumber::TIM3 => {
                let tim = self.get_tim3();
                self.config_encoder_tim3(tim, mode, prescaler);
            },
            TimerNumber::TIM4 => {
                let tim = self.get_tim4();
                self.config_encoder_tim4(tim, mode, prescaler);
            },
            TimerNumber::TIM5 => {
                let tim = self.get_tim5();
                self.config_encoder_tim5(tim, mode, prescaler);
            },
            TimerNumber::TIM8 => {
                let tim = self.get_tim8();
                self.config_encoder_tim8(tim, mode, prescaler);
            },
            TimerNumber::TIM6 | TimerNumber::TIM7 => {
                // 基本定时器不支持编码器模式
                return Err(TimerError::UnsupportedFeature);
            },
        }
        
        Ok(())
    }
    
    /// 配置编码器模式（针对TIM1）
    unsafe fn config_encoder_tim1(
        &self, 
        tim: &mut tim1::RegisterBlock, 
        mode: EncoderMode, 
        prescaler: u16
    ) {
        // 禁用定时器
        tim.cr1().write(|w| w.cen().clear_bit());
        
        // 设置预分频器
        tim.psc().write(|w| w.psc().bits(prescaler));
        
        // 配置SMCR寄存器，启用编码器模式
        let smcr_value = match mode {
            EncoderMode::Mode1 => 0b010,  // SMS=010: 仅在TI1边沿计数
            EncoderMode::Mode2 => 0b100,  // SMS=100: 仅在TI2边沿计数
            EncoderMode::Mode3 => 0b110,  // SMS=110: 在TI1和TI2边沿计数
        };
        tim.smcr().write(|w| w.sms().bits(smcr_value));
        
        // 配置CCMR1寄存器，设置TI1和TI2为输入模式
        tim.ccmr1_input().write(|w| {
            w.cc1s().bits(0b01)  // CC1S=01: TI1作为输入，映射到IC1
                .cc2s().bits(0b01)  // CC2S=01: TI2作为输入，映射到IC2
        });
        
        // 配置CCER寄存器，禁用输入捕获通道的极性反转
        // 通用定时器TIM2没有CC1NP和CC2NP位
        tim.ccer().write(|w| {
            w.cc1p().clear_bit()  // CC1P=0: 非反转
                .cc1e().set_bit()    // 启用CC1通道
                .cc2p().clear_bit()  // CC2P=0: 非反转
                .cc2e().set_bit()    // 启用CC2通道
        });
        
        // 重置计数器
        tim.cnt().write(|w| w.cnt().bits(0));
        
        // 生成更新事件
        tim.egr().write(|w| w.ug().set_bit());
        
        // 清除更新中断标志
        tim.sr().write(|w| w.uif().clear_bit());
        
        // 启用定时器
        tim.cr1().write(|w| w.cen().set_bit());
    }
    
    /// 配置编码器模式（针对TIM2）
    unsafe fn config_encoder_tim2(
        &self, 
        tim: &mut tim2::RegisterBlock, 
        mode: EncoderMode, 
        prescaler: u16
    ) {
        // 禁用定时器
        tim.cr1().write(|w| w.cen().clear_bit());
        
        // 设置预分频器
        tim.psc().write(|w| w.psc().bits(prescaler));
        
        // 配置SMCR寄存器，启用编码器模式
        let smcr_value = match mode {
            EncoderMode::Mode1 => 0b010,  // SMS=010: 仅在TI1边沿计数
            EncoderMode::Mode2 => 0b100,  // SMS=100: 仅在TI2边沿计数
            EncoderMode::Mode3 => 0b110,  // SMS=110: 在TI1和TI2边沿计数
        };
        tim.smcr().write(|w| w.sms().bits(smcr_value));
        
        // 配置CCMR1寄存器，设置TI1和TI2为输入模式
        tim.ccmr1_input().write(|w| {
            w.cc1s().bits(0b01)  // CC1S=01: TI1作为输入，映射到IC1
                .cc2s().bits(0b01)  // CC2S=01: TI2作为输入，映射到IC2
        });
        
        // 配置CCER寄存器，禁用输入捕获通道的极性反转
        // 通用定时器TIM3没有CC1NP和CC2NP位
        tim.ccer().write(|w| {
            w.cc1p().clear_bit()  // CC1P=0: 非反转
                .cc1e().set_bit()    // 启用CC1通道
                .cc2p().clear_bit()  // CC2P=0: 非反转
                .cc2e().set_bit()    // 启用CC2通道
        });
        
        // 重置计数器
        tim.cnt().write(|w| w.cnt().bits(0));
        
        // 生成更新事件
        tim.egr().write(|w| w.ug().set_bit());
        
        // 清除更新中断标志
        tim.sr().write(|w| w.uif().clear_bit());
        
        // 启用定时器
        tim.cr1().write(|w| w.cen().set_bit());
    }
    
    /// 配置编码器模式（针对TIM3）
    unsafe fn config_encoder_tim3(
        &self, 
        tim: &mut tim3::RegisterBlock, 
        mode: EncoderMode, 
        prescaler: u16
    ) {
        // 禁用定时器
        tim.cr1().write(|w| w.cen().clear_bit());
        
        // 设置预分频器
        tim.psc().write(|w| w.psc().bits(prescaler));
        
        // 配置SMCR寄存器，启用编码器模式
        let smcr_value = match mode {
            EncoderMode::Mode1 => 0b010,  // SMS=010: 仅在TI1边沿计数
            EncoderMode::Mode2 => 0b100,  // SMS=100: 仅在TI2边沿计数
            EncoderMode::Mode3 => 0b110,  // SMS=110: 在TI1和TI2边沿计数
        };
        tim.smcr().write(|w| w.sms().bits(smcr_value));
        
        // 配置CCMR1寄存器，设置TI1和TI2为输入模式
        tim.ccmr1_input().write(|w| {
            w.cc1s().bits(0b01)  // CC1S=01: TI1作为输入，映射到IC1
                .cc2s().bits(0b01)  // CC2S=01: TI2作为输入，映射到IC2
        });
        
        // 配置CCER寄存器，禁用输入捕获通道的极性反转
        // 通用定时器TIM4没有CC1NP和CC2NP位
        tim.ccer().write(|w| {
            w.cc1p().clear_bit()  // CC1P=0: 非反转
                .cc1e().set_bit()    // 启用CC1通道
                .cc2p().clear_bit()  // CC2P=0: 非反转
                .cc2e().set_bit()    // 启用CC2通道
        });
        
        // 重置计数器
        tim.cnt().write(|w| w.cnt().bits(0));
        
        // 生成更新事件
        tim.egr().write(|w| w.ug().set_bit());
        
        // 清除更新中断标志
        tim.sr().write(|w| w.uif().clear_bit());
        
        // 启用定时器
        tim.cr1().write(|w| w.cen().set_bit());
    }
    
    /// 配置编码器模式（针对TIM4）
    unsafe fn config_encoder_tim4(
        &self, 
        tim: &mut tim4::RegisterBlock, 
        mode: EncoderMode, 
        prescaler: u16
    ) {
        // 禁用定时器
        tim.cr1().write(|w| w.cen().clear_bit());
        
        // 设置预分频器
        tim.psc().write(|w| w.psc().bits(prescaler));
        
        // 配置SMCR寄存器，启用编码器模式
        let smcr_value = match mode {
            EncoderMode::Mode1 => 0b010,  // SMS=010: 仅在TI1边沿计数
            EncoderMode::Mode2 => 0b100,  // SMS=100: 仅在TI2边沿计数
            EncoderMode::Mode3 => 0b110,  // SMS=110: 在TI1和TI2边沿计数
        };
        tim.smcr().write(|w| w.sms().bits(smcr_value));
        
        // 配置CCMR1寄存器，设置TI1和TI2为输入模式
        tim.ccmr1_input().write(|w| {
            w.cc1s().bits(0b01)  // CC1S=01: TI1作为输入，映射到IC1
                .cc2s().bits(0b01)  // CC2S=01: TI2作为输入，映射到IC2
        });
        
        // 配置CCER寄存器，禁用输入捕获通道的极性反转
        // 通用定时器TIM5没有CC1NP和CC2NP位
        tim.ccer().write(|w| {
            w.cc1p().clear_bit()  // CC1P=0: 非反转
                .cc1e().set_bit()    // 启用CC1通道
                .cc2p().clear_bit()  // CC2P=0: 非反转
                .cc2e().set_bit()    // 启用CC2通道
        });
        
        // 重置计数器
        tim.cnt().write(|w| w.cnt().bits(0));
        
        // 生成更新事件
        tim.egr().write(|w| w.ug().set_bit());
        
        // 清除更新中断标志
        tim.sr().write(|w| w.uif().clear_bit());
        
        // 启用定时器
        tim.cr1().write(|w| w.cen().set_bit());
    }
    
    /// 配置编码器模式（针对TIM5）
    unsafe fn config_encoder_tim5(
        &self, 
        tim: &mut tim5::RegisterBlock, 
        mode: EncoderMode, 
        prescaler: u16
    ) {
        // 禁用定时器
        tim.cr1().write(|w| w.cen().clear_bit());
        
        // 设置预分频器
        tim.psc().write(|w| w.psc().bits(prescaler));
        
        // 配置SMCR寄存器，启用编码器模式
        let smcr_value = match mode {
            EncoderMode::Mode1 => 0b010,  // SMS=010: 仅在TI1边沿计数
            EncoderMode::Mode2 => 0b100,  // SMS=100: 仅在TI2边沿计数
            EncoderMode::Mode3 => 0b110,  // SMS=110: 在TI1和TI2边沿计数
        };
        tim.smcr().write(|w| w.sms().bits(smcr_value));
        
        // 配置CCMR1寄存器，设置TI1和TI2为输入模式
        tim.ccmr1_input().write(|w| {
            w.cc1s().bits(0b01)  // CC1S=01: TI1作为输入，映射到IC1
                .cc2s().bits(0b01)  // CC2S=01: TI2作为输入，映射到IC2
        });
        
        // 配置CCER寄存器，禁用输入捕获通道的极性反转
        // 通用定时器TIM5没有CC1NP和CC2NP位
        tim.ccer().write(|w| {
            w.cc1p().clear_bit()  // CC1P=0: 非反转
                .cc1e().set_bit()    // 启用CC1通道
                .cc2p().clear_bit()  // CC2P=0: 非反转
                .cc2e().set_bit()    // 启用CC2通道
        });
        
        // 重置计数器
        tim.cnt().write(|w| w.cnt().bits(0));
        
        // 生成更新事件
        tim.egr().write(|w| w.ug().set_bit());
        
        // 清除更新中断标志
        tim.sr().write(|w| w.uif().clear_bit());
        
        // 启用定时器
        tim.cr1().write(|w| w.cen().set_bit());
    }
    
    /// 配置编码器模式（针对TIM8）
    unsafe fn config_encoder_tim8(
        &self, 
        tim: &mut tim8::RegisterBlock, 
        mode: EncoderMode, 
        prescaler: u16
    ) {
        // 禁用定时器
        tim.cr1().write(|w| w.cen().clear_bit());
        
        // 设置预分频器
        tim.psc().write(|w| w.psc().bits(prescaler));
        
        // 配置SMCR寄存器，启用编码器模式
        let smcr_value = match mode {
            EncoderMode::Mode1 => 0b010,  // SMS=010: 仅在TI1边沿计数
            EncoderMode::Mode2 => 0b100,  // SMS=100: 仅在TI2边沿计数
            EncoderMode::Mode3 => 0b110,  // SMS=110: 在TI1和TI2边沿计数
        };
        tim.smcr().write(|w| w.sms().bits(smcr_value));
        
        // 配置CCMR1寄存器，设置TI1和TI2为输入模式
        tim.ccmr1_input().write(|w| {
            w.cc1s().bits(0b01)  // CC1S=01: TI1作为输入，映射到IC1
                .cc2s().bits(0b01)  // CC2S=01: TI2作为输入，映射到IC2
        });
        
        // 配置CCER寄存器，禁用输入捕获通道的极性反转
        tim.ccer().write(|w| {
            w.cc1p().clear_bit()  // CC1P=0: 非反转
                .cc1np().clear_bit() // CC1NP=0: 非反转
                .cc1e().set_bit()    // 启用CC1通道
                .cc2p().clear_bit()  // CC2P=0: 非反转
                .cc2np().clear_bit() // CC2NP=0: 非反转
                .cc2e().set_bit()    // 启用CC2通道
        });
        
        // 重置计数器
        tim.cnt().write(|w| w.cnt().bits(0));
        
        // 生成更新事件
        tim.egr().write(|w| w.ug().set_bit());
        
        // 清除更新中断标志
        tim.sr().write(|w| w.uif().clear_bit());
        
        // 启用定时器
        tim.cr1().write(|w| w.cen().set_bit());
    }
    
    /// 获取编码器计数值
    pub unsafe fn get_encoder_count(&self) -> i16 {
        match self.number {
            TimerNumber::TIM1 => self.get_tim1().cnt().read().cnt().bits() as i16,
            TimerNumber::TIM2 => self.get_tim2().cnt().read().cnt().bits() as i16,
            TimerNumber::TIM3 => self.get_tim3().cnt().read().cnt().bits() as i16,
            TimerNumber::TIM4 => self.get_tim4().cnt().read().cnt().bits() as i16,
            TimerNumber::TIM5 => self.get_tim5().cnt().read().cnt().bits() as i16,
            TimerNumber::TIM8 => self.get_tim8().cnt().read().cnt().bits() as i16,
            _ => 0, // 基本定时器不支持编码器模式
        }
    }
    
    /// 重置编码器计数值
    pub unsafe fn reset_encoder_count(&self) {
        match self.number {
            TimerNumber::TIM1 => { self.get_tim1().cnt().write(|w| w.cnt().bits(0)); },
            TimerNumber::TIM2 => { self.get_tim2().cnt().write(|w| w.cnt().bits(0)); },
            TimerNumber::TIM3 => { self.get_tim3().cnt().write(|w| w.cnt().bits(0)); },
            TimerNumber::TIM4 => { self.get_tim4().cnt().write(|w| w.cnt().bits(0)); },
            TimerNumber::TIM5 => { self.get_tim5().cnt().write(|w| w.cnt().bits(0)); },
            TimerNumber::TIM8 => { self.get_tim8().cnt().write(|w| w.cnt().bits(0)); },
            _ => {}, // 基本定时器不支持编码器模式
        }
    }
    
    /// 获取编码器计数方向
    pub unsafe fn get_encoder_direction(&self) -> EncoderDirection {
        match self.number {
            TimerNumber::TIM1 => {
                if self.get_tim1().cr1().read().dir().bit_is_set() {
                    EncoderDirection::Down
                } else {
                    EncoderDirection::Up
                }
            },
            TimerNumber::TIM2 => {
                if self.get_tim2().cr1().read().dir().bit_is_set() {
                    EncoderDirection::Down
                } else {
                    EncoderDirection::Up
                }
            },
            TimerNumber::TIM3 => {
                if self.get_tim3().cr1().read().dir().bit_is_set() {
                    EncoderDirection::Down
                } else {
                    EncoderDirection::Up
                }
            },
            TimerNumber::TIM4 => {
                if self.get_tim4().cr1().read().dir().bit_is_set() {
                    EncoderDirection::Down
                } else {
                    EncoderDirection::Up
                }
            },
            TimerNumber::TIM5 => {
                if self.get_tim5().cr1().read().dir().bit_is_set() {
                    EncoderDirection::Down
                } else {
                    EncoderDirection::Up
                }
            },
            TimerNumber::TIM8 => {
                if self.get_tim8().cr1().read().dir().bit_is_set() {
                    EncoderDirection::Down
                } else {
                    EncoderDirection::Up
                }
            },
            _ => EncoderDirection::Up, // 基本定时器不支持编码器模式
        }
    }
    
    /// 初始化输入捕获通道
    /// 
    /// 该方法用于配置定时器的输入捕获功能，可用于测量外部信号的频率、脉宽等参数。
    /// 
    /// # 参数
    /// * `channel` - 输入捕获通道（支持Channel1-Channel4）
    /// * `polarity` - 触发极性（上升沿、下降沿或双边沿）
    /// * `prescaler` - 定时器预分频器值（0-65535）
    /// * `capture_prescaler` - 输入捕获预分频器（不分频、2分频、4分频或8分频）
    /// 
    /// # 返回值
    /// * `Ok(())` - 初始化成功
    /// * `Err(TimerError::UnsupportedFeature)` - 基本定时器（TIM6-TIM7）不支持输入捕获功能
    /// * `Err(TimerError)` - 其他初始化失败情况
    /// 
    /// # 示例
    /// ```rust
    /// // 初始化TIM3通道1为上升沿触发，不分频
    /// let timer = Timer::new(TimerNumber::TIM3);
    /// timer.init_input_capture(
    ///     PwmChannel::Channel1,
    ///     InputCapturePolarity::RisingEdge,
    ///     72_000 - 1,  // 1MHz计数频率
    ///     InputCapturePrescaler::Div1
    /// ).unwrap();
    /// ```
    pub unsafe fn init_input_capture(
        &self, 
        channel: PwmChannel, 
        polarity: InputCapturePolarity,
        prescaler: u16,
        capture_prescaler: InputCapturePrescaler
    ) -> Result<(), TimerError> {
        // 参数有效性验证
        match channel {
            // 所有支持输入捕获的定时器都支持4个通道
            PwmChannel::Channel1 | PwmChannel::Channel2 | 
            PwmChannel::Channel3 | PwmChannel::Channel4 => {},
        }
        
        // 启用定时器时钟
        self.enable_clock();
        
        // 配置输入捕获模式
        match self.number {
            TimerNumber::TIM1 => {
                let tim = self.get_tim1();
                self.config_input_capture_tim1(tim, channel, polarity, prescaler, capture_prescaler);
            },
            TimerNumber::TIM2 => {
                let tim = self.get_tim2();
                self.config_input_capture_tim2(tim, channel, polarity, prescaler, capture_prescaler);
            },
            TimerNumber::TIM3 => {
                let tim = self.get_tim3();
                self.config_input_capture_tim3(tim, channel, polarity, prescaler, capture_prescaler);
            },
            TimerNumber::TIM4 => {
                let tim = self.get_tim4();
                self.config_input_capture_tim4(tim, channel, polarity, prescaler, capture_prescaler);
            },
            TimerNumber::TIM5 => {
                let tim = self.get_tim5();
                self.config_input_capture_tim5(tim, channel, polarity, prescaler, capture_prescaler);
            },
            TimerNumber::TIM8 => {
                let tim = self.get_tim8();
                self.config_input_capture_tim8(tim, channel, polarity, prescaler, capture_prescaler);
            },
            TimerNumber::TIM6 | TimerNumber::TIM7 => {
                // 基本定时器不支持输入捕获功能
                return Err(TimerError::UnsupportedFeature);
            },
        }
        
        Ok(())
    }
    
    /// 配置输入捕获通道（针对TIM1）
    unsafe fn config_input_capture_tim1(
        &self, 
        tim: &mut tim1::RegisterBlock, 
        channel: PwmChannel, 
        polarity: InputCapturePolarity,
        prescaler: u16,
        capture_prescaler: InputCapturePrescaler
    ) {
        // 禁用定时器
        tim.cr1().write(|w| w.cen().clear_bit());
        
        // 设置预分频器
        tim.psc().write(|w| w.psc().bits(prescaler));
        
        // 设置自动重装载值为最大值
        tim.arr().write(|w| w.arr().bits(u16::MAX));
        
        // 配置输入捕获预分频器
        let icpsc_value = match capture_prescaler {
            InputCapturePrescaler::Div1 => 0b00,
            InputCapturePrescaler::Div2 => 0b01,
            InputCapturePrescaler::Div4 => 0b10,
            InputCapturePrescaler::Div8 => 0b11,
        };
        
        // 配置捕获极性
        let (ccp_value, ccnp_value) = match polarity {
            InputCapturePolarity::RisingEdge => (false, false),
            InputCapturePolarity::FallingEdge => (true, false),
            InputCapturePolarity::BothEdges => (true, true),
        };
        
        // 根据通道配置输入捕获
        match channel {
            PwmChannel::Channel1 => {
                // 配置CCMR1寄存器
                tim.ccmr1_input().write(|w| {
                    w.cc1s().bits(0b01)  // CC1S=01: TI1作为输入
                        .ic2pcs().bits(icpsc_value)  // 输入捕获预分频器
                        .ic1f().bits(0b0000)  // 输入滤波器关闭
                });
                
                // 配置CCER寄存器
                tim.ccer().modify(|_, w| {
                    let mut w = w;
                    if ccp_value { w = w.cc1p().set_bit(); }
                    if ccnp_value { w = w.cc1np().set_bit(); }
                    w.cc1e().set_bit()  // 启用捕获通道
                });
            },
            PwmChannel::Channel2 => {
                // 配置CCMR1寄存器
                tim.ccmr1_input().write(|w| {
                    w.cc2s().bits(0b01)  // CC2S=01: TI2作为输入
                        .ic2pcs().bits(icpsc_value)  // 输入捕获预分频器
                        .ic2f().bits(0b0000)  // 输入滤波器关闭
                });
                
                // 配置CCER寄存器
                tim.ccer().modify(|_, w| {
                    let mut w = w;
                    if ccp_value { w = w.cc2p().set_bit(); }
                    if ccnp_value { w = w.cc2np().set_bit(); }
                    w.cc2e().set_bit()  // 启用捕获通道
                });
            },
            PwmChannel::Channel3 => {
                // 配置CCMR2寄存器
                tim.ccmr2_input().write(|w| {
                    w.cc3s().bits(0b01)  // CC3S=01: TI3作为输入
                        .ic3psc().bits(icpsc_value)  // 输入捕获预分频器
                        .ic3f().bits(0b0000)  // 输入滤波器关闭
                });
                
                // 配置CCER寄存器
                tim.ccer().modify(|_, w| {
                    let mut w = w;
                    if ccp_value { w = w.cc3p().set_bit(); }
                    if ccnp_value { w = w.cc3np().set_bit(); }
                    w.cc3e().set_bit()  // 启用捕获通道
                });
            },
            PwmChannel::Channel4 => {
                // 配置CCMR2寄存器
                tim.ccmr2_input().write(|w| {
                    w.cc4s().bits(0b01)  // CC4S=01: TI4作为输入
                        .ic4psc().bits(icpsc_value)  // 输入捕获预分频器
                        .ic4f().bits(0b0000)  // 输入滤波器关闭
                });
                
                // 配置CCER寄存器
                tim.ccer().modify(|_, w| {
                    let ccp = if ccp_value { w.cc4p().set_bit() } else { w.cc4p().clear_bit() };
                    // 通道4没有CC4NP位
                    ccp
                        .cc4e().set_bit()  // 启用捕获通道
                });
            },
        }
        
        // 清除更新中断标志
        tim.sr().write(|w| w.uif().clear_bit());
        
        // 启用定时器
        tim.cr1().write(|w| w.cen().set_bit());
    }
    
    /// 配置输入捕获通道（针对TIM2）
    unsafe fn config_input_capture_tim2(
        &self, 
        tim: &mut tim2::RegisterBlock, 
        channel: PwmChannel, 
        polarity: InputCapturePolarity,
        prescaler: u16,
        capture_prescaler: InputCapturePrescaler
    ) {
        // 禁用定时器
        tim.cr1().write(|w| w.cen().clear_bit());
        
        // 设置预分频器
        tim.psc().write(|w| w.psc().bits(prescaler));
        
        // 设置自动重装载值为最大值
        tim.arr().write(|w| w.arr().bits(u16::MAX));
        
        // 配置输入捕获预分频器
        let icpsc_value = match capture_prescaler {
            InputCapturePrescaler::Div1 => 0b00,
            InputCapturePrescaler::Div2 => 0b01,
            InputCapturePrescaler::Div4 => 0b10,
            InputCapturePrescaler::Div8 => 0b11,
        };
        
        // 配置捕获极性
        let ccp_value = match polarity {
            InputCapturePolarity::RisingEdge => false,
            InputCapturePolarity::FallingEdge => true,
            InputCapturePolarity::BothEdges => true,
        };
        
        // 根据通道配置输入捕获
        match channel {
            PwmChannel::Channel1 => {
                // 配置CCMR1寄存器
                tim.ccmr1_input().write(|w| {
                    w.cc1s().bits(0b01)  // CC1S=01: TI1作为输入
                        .ic1psc().bits(icpsc_value)  // 输入捕获预分频器
                        .ic1f().bits(0b0000)  // 输入滤波器关闭
                });
                
                // 配置CCER寄存器
                tim.ccer().modify(|_, w| {
                    let ccp = if ccp_value { w.cc1p().set_bit() } else { w.cc1p().clear_bit() };
                    ccp
                        .cc1e().set_bit()  // 启用捕获通道
                });
            },
            PwmChannel::Channel2 => {
                // 配置CCMR1寄存器
                tim.ccmr1_input().write(|w| {
                    w.cc2s().bits(0b01)  // CC2S=01: TI2作为输入
                        .ic2psc().bits(icpsc_value)  // 输入捕获预分频器
                        .ic2f().bits(0b0000)  // 输入滤波器关闭
                });
                
                // 配置CCER寄存器
                tim.ccer().modify(|_, w| {
                    let ccp = if ccp_value { w.cc2p().set_bit() } else { w.cc2p().clear_bit() };
                    ccp
                        .cc2e().set_bit()  // 启用捕获通道
                });
            },
            PwmChannel::Channel3 => {
                // 配置CCMR2寄存器
                tim.ccmr2_input().write(|w| {
                    w.cc3s().bits(0b01)  // CC3S=01: TI3作为输入
                        .ic3psc().bits(icpsc_value)  // 输入捕获预分频器
                        .ic3f().bits(0b0000)  // 输入滤波器关闭
                });
                
                // 配置CCER寄存器
                tim.ccer().modify(|_, w| {
                    let ccp = if ccp_value { w.cc3p().set_bit() } else { w.cc3p().clear_bit() };
                    ccp
                        .cc3e().set_bit()  // 启用捕获通道
                });
            },
            PwmChannel::Channel4 => {
                // 配置CCMR2寄存器
                tim.ccmr2_input().write(|w| {
                    w.cc4s().bits(0b01)  // CC4S=01: TI4作为输入
                        .ic4psc().bits(icpsc_value)  // 输入捕获预分频器
                        .ic4f().bits(0b0000)  // 输入滤波器关闭
                });
                
                // 配置CCER寄存器
                tim.ccer().modify(|_, w| {
                    let ccp = if ccp_value { w.cc4p().set_bit() } else { w.cc4p().clear_bit() };
                    ccp
                        .cc4e().set_bit()  // 启用捕获通道
                });
            },
        }
        
        // 清除更新中断标志
        tim.sr().write(|w| w.uif().clear_bit());
        
        // 启用定时器
        tim.cr1().write(|w| w.cen().set_bit());
    }
    
    /// 配置输入捕获通道（针对TIM3）
    unsafe fn config_input_capture_tim3(
        &self, 
        tim: &mut tim3::RegisterBlock, 
        channel: PwmChannel, 
        polarity: InputCapturePolarity,
        prescaler: u16,
        capture_prescaler: InputCapturePrescaler
    ) {
        // 禁用定时器
        tim.cr1().write(|w| w.cen().clear_bit());
        
        // 设置预分频器
        tim.psc().write(|w| w.psc().bits(prescaler));
        
        // 设置自动重装载值为最大值
        tim.arr().write(|w| w.arr().bits(u16::MAX));
        
        // 配置输入捕获预分频器
        let icpsc_value = match capture_prescaler {
            InputCapturePrescaler::Div1 => 0b00,
            InputCapturePrescaler::Div2 => 0b01,
            InputCapturePrescaler::Div4 => 0b10,
            InputCapturePrescaler::Div8 => 0b11,
        };
        
        // 配置捕获极性
        let ccp_value = match polarity {
            InputCapturePolarity::RisingEdge => false,
            InputCapturePolarity::FallingEdge => true,
            InputCapturePolarity::BothEdges => true,
        };
        
        // 根据通道配置输入捕获
        match channel {
            PwmChannel::Channel1 => {
                // 配置CCMR1寄存器
                tim.ccmr1_input().write(|w| {
                    w.cc1s().bits(0b01)  // CC1S=01: TI1作为输入
                        .ic1psc().bits(icpsc_value)  // 输入捕获预分频器
                        .ic1f().bits(0b0000)  // 输入滤波器关闭
                });
                
                // 配置CCER寄存器
                tim.ccer().modify(|_, w| {
                    let ccp = if ccp_value { w.cc1p().set_bit() } else { w.cc1p().clear_bit() };
                    ccp
                        .cc1e().set_bit()  // 启用捕获通道
                });
            },
            PwmChannel::Channel2 => {
                // 配置CCMR1寄存器
                tim.ccmr1_input().write(|w| {
                    w.cc2s().bits(0b01)  // CC2S=01: TI2作为输入
                        .ic2psc().bits(icpsc_value)  // 输入捕获预分频器
                        .ic2f().bits(0b0000)  // 输入滤波器关闭
                });
                
                // 配置CCER寄存器
                tim.ccer().modify(|_, w| {
                    let ccp = if ccp_value { w.cc2p().set_bit() } else { w.cc2p().clear_bit() };
                    ccp
                        .cc2e().set_bit()  // 启用捕获通道
                });
            },
            PwmChannel::Channel3 => {
                // 配置CCMR2寄存器
                tim.ccmr2_input().write(|w| {
                    w.cc3s().bits(0b01)  // CC3S=01: TI3作为输入
                        .ic3psc().bits(icpsc_value)  // 输入捕获预分频器
                        .ic3f().bits(0b0000)  // 输入滤波器关闭
                });
                
                // 配置CCER寄存器
                tim.ccer().modify(|_, w| {
                    let ccp = if ccp_value { w.cc3p().set_bit() } else { w.cc3p().clear_bit() };
                    ccp
                        .cc3e().set_bit()  // 启用捕获通道
                });
            },
            PwmChannel::Channel4 => {
                // 配置CCMR2寄存器
                tim.ccmr2_input().write(|w| {
                    w.cc4s().bits(0b01)  // CC4S=01: TI4作为输入
                        .ic4psc().bits(icpsc_value)  // 输入捕获预分频器
                        .ic4f().bits(0b0000)  // 输入滤波器关闭
                });
                
                // 配置CCER寄存器
                tim.ccer().modify(|_, w| {
                    let ccp = if ccp_value { w.cc4p().set_bit() } else { w.cc4p().clear_bit() };
                    ccp
                        .cc4e().set_bit()  // 启用捕获通道
                });
            },
        }
        
        // 清除更新中断标志
        tim.sr().write(|w| w.uif().clear_bit());
        
        // 启用定时器
        tim.cr1().write(|w| w.cen().set_bit());
    }
    
    /// 配置输入捕获通道（针对TIM4）
    unsafe fn config_input_capture_tim4(
        &self, 
        tim: &mut tim4::RegisterBlock, 
        channel: PwmChannel, 
        polarity: InputCapturePolarity,
        prescaler: u16,
        capture_prescaler: InputCapturePrescaler
    ) {
        // 禁用定时器
        tim.cr1().write(|w| w.cen().clear_bit());
        
        // 设置预分频器
        tim.psc().write(|w| w.psc().bits(prescaler));
        
        // 设置自动重装载值为最大值
        tim.arr().write(|w| w.arr().bits(u16::MAX));
        
        // 配置输入捕获预分频器
        let icpsc_value = match capture_prescaler {
            InputCapturePrescaler::Div1 => 0b00,
            InputCapturePrescaler::Div2 => 0b01,
            InputCapturePrescaler::Div4 => 0b10,
            InputCapturePrescaler::Div8 => 0b11,
        };
        
        // 配置捕获极性
        let ccp_value = match polarity {
            InputCapturePolarity::RisingEdge => false,
            InputCapturePolarity::FallingEdge => true,
            InputCapturePolarity::BothEdges => true,
        };
        
        // 根据通道配置输入捕获
        match channel {
            PwmChannel::Channel1 => {
                // 配置CCMR1寄存器
                tim.ccmr1_input().write(|w| {
                    w.cc1s().bits(0b01)  // CC1S=01: TI1作为输入
                        .ic1psc().bits(icpsc_value)  // 输入捕获预分频器
                        .ic1f().bits(0b0000)  // 输入滤波器关闭
                });
                
                // 配置CCER寄存器
                tim.ccer().modify(|_, w| {
                    let ccp = if ccp_value { w.cc1p().set_bit() } else { w.cc1p().clear_bit() };
                    ccp
                        .cc1e().set_bit()  // 启用捕获通道
                });
            },
            PwmChannel::Channel2 => {
                // 配置CCMR1寄存器
                tim.ccmr1_input().write(|w| {
                    w.cc2s().bits(0b01)  // CC2S=01: TI2作为输入
                        .ic2psc().bits(icpsc_value)  // 输入捕获预分频器
                        .ic2f().bits(0b0000)  // 输入滤波器关闭
                });
                
                // 配置CCER寄存器
                tim.ccer().modify(|_, w| {
                    let ccp = if ccp_value { w.cc2p().set_bit() } else { w.cc2p().clear_bit() };
                    ccp
                        .cc2e().set_bit()  // 启用捕获通道
                });
            },
            PwmChannel::Channel3 => {
                // 配置CCMR2寄存器
                tim.ccmr2_input().write(|w| {
                    w.cc3s().bits(0b01)  // CC3S=01: TI3作为输入
                        .ic3psc().bits(icpsc_value)  // 输入捕获预分频器
                        .ic3f().bits(0b0000)  // 输入滤波器关闭
                });
                
                // 配置CCER寄存器
                tim.ccer().modify(|_, w| {
                    let ccp = if ccp_value { w.cc3p().set_bit() } else { w.cc3p().clear_bit() };
                    ccp
                        .cc3e().set_bit()  // 启用捕获通道
                });
            },
            PwmChannel::Channel4 => {
                // 配置CCMR2寄存器
                tim.ccmr2_input().write(|w| {
                    w.cc4s().bits(0b01)  // CC4S=01: TI4作为输入
                        .ic4psc().bits(icpsc_value)  // 输入捕获预分频器
                        .ic4f().bits(0b0000)  // 输入滤波器关闭
                });
                
                // 配置CCER寄存器
                tim.ccer().modify(|_, w| {
                    let ccp = if ccp_value { w.cc4p().set_bit() } else { w.cc4p().clear_bit() };
                    ccp
                        .cc4e().set_bit()  // 启用捕获通道
                });
            },
        }
        
        // 清除更新中断标志
        tim.sr().write(|w| w.uif().clear_bit());
        
        // 启用定时器
        tim.cr1().write(|w| w.cen().set_bit());
    }
    
    /// 配置输入捕获通道（针对TIM5）
    unsafe fn config_input_capture_tim5(
        &self, 
        tim: &mut tim5::RegisterBlock, 
        channel: PwmChannel, 
        polarity: InputCapturePolarity,
        prescaler: u16,
        capture_prescaler: InputCapturePrescaler
    ) {
        // 禁用定时器
        tim.cr1().write(|w| w.cen().clear_bit());
        
        // 设置预分频器
        tim.psc().write(|w| w.psc().bits(prescaler));
        
        // 设置自动重装载值为最大值
        tim.arr().write(|w| w.arr().bits(u16::MAX));
        
        // 配置输入捕获预分频器
        let icpsc_value = match capture_prescaler {
            InputCapturePrescaler::Div1 => 0b00,
            InputCapturePrescaler::Div2 => 0b01,
            InputCapturePrescaler::Div4 => 0b10,
            InputCapturePrescaler::Div8 => 0b11,
        };
        
        // 配置捕获极性
        let ccp_value = match polarity {
            InputCapturePolarity::RisingEdge => false,
            InputCapturePolarity::FallingEdge => true,
            InputCapturePolarity::BothEdges => true,
        };
        
        // 根据通道配置输入捕获
        match channel {
            PwmChannel::Channel1 => {
                // 配置CCMR1寄存器
                tim.ccmr1_input().write(|w| {
                    w.cc1s().bits(0b01)  // CC1S=01: TI1作为输入
                        .ic1psc().bits(icpsc_value)  // 输入捕获预分频器
                        .ic1f().bits(0b0000)  // 输入滤波器关闭
                });
                
                // 配置CCER寄存器
                tim.ccer().modify(|_, w| {
                    let ccp = if ccp_value { w.cc1p().set_bit() } else { w.cc1p().clear_bit() };
                    ccp
                        .cc1e().set_bit()  // 启用捕获通道
                });
            },
            PwmChannel::Channel2 => {
                // 配置CCMR1寄存器
                tim.ccmr1_input().write(|w| {
                    w.cc2s().bits(0b01)  // CC2S=01: TI2作为输入
                        .ic2psc().bits(icpsc_value)  // 输入捕获预分频器
                        .ic2f().bits(0b0000)  // 输入滤波器关闭
                });
                
                // 配置CCER寄存器
                tim.ccer().modify(|_, w| {
                    let ccp = if ccp_value { w.cc2p().set_bit() } else { w.cc2p().clear_bit() };
                    ccp
                        .cc2e().set_bit()  // 启用捕获通道
                });
            },
            PwmChannel::Channel3 => {
                // 配置CCMR2寄存器
                tim.ccmr2_input().write(|w| {
                    w.cc3s().bits(0b01)  // CC3S=01: TI3作为输入
                        .ic3psc().bits(icpsc_value)  // 输入捕获预分频器
                        .ic3f().bits(0b0000)  // 输入滤波器关闭
                });
                
                // 配置CCER寄存器
                tim.ccer().modify(|_, w| {
                    let ccp = if ccp_value { w.cc3p().set_bit() } else { w.cc3p().clear_bit() };
                    ccp
                        .cc3e().set_bit()  // 启用捕获通道
                });
            },
            PwmChannel::Channel4 => {
                // 配置CCMR2寄存器
                tim.ccmr2_input().write(|w| {
                    w.cc4s().bits(0b01)  // CC4S=01: TI4作为输入
                        .ic4psc().bits(icpsc_value)  // 输入捕获预分频器
                        .ic4f().bits(0b0000)  // 输入滤波器关闭
                });
                
                // 配置CCER寄存器
                tim.ccer().modify(|_, w| {
                    let ccp = if ccp_value { w.cc4p().set_bit() } else { w.cc4p().clear_bit() };
                    ccp
                        .cc4e().set_bit()  // 启用捕获通道
                });
            },
        }
        
        // 清除更新中断标志
        tim.sr().write(|w| w.uif().clear_bit());
        
        // 启用定时器
        tim.cr1().write(|w| w.cen().set_bit());
    }
    
    /// 配置输入捕获通道（针对TIM8）
    unsafe fn config_input_capture_tim8(
        &self, 
        tim: &mut tim8::RegisterBlock, 
        channel: PwmChannel, 
        polarity: InputCapturePolarity,
        prescaler: u16,
        capture_prescaler: InputCapturePrescaler
    ) {
        // 禁用定时器
        tim.cr1().write(|w| w.cen().clear_bit());
        
        // 设置预分频器
        tim.psc().write(|w| w.psc().bits(prescaler));
        
        // 设置自动重装载值为最大值
        tim.arr().write(|w| w.arr().bits(u16::MAX));
        
        // 配置输入捕获预分频器
        let icpsc_value = match capture_prescaler {
            InputCapturePrescaler::Div1 => 0b00,
            InputCapturePrescaler::Div2 => 0b01,
            InputCapturePrescaler::Div4 => 0b10,
            InputCapturePrescaler::Div8 => 0b11,
        };
        
        // 配置捕获极性
        let (ccp_value, ccnp_value) = match polarity {
            InputCapturePolarity::RisingEdge => (false, false),
            InputCapturePolarity::FallingEdge => (true, false),
            InputCapturePolarity::BothEdges => (true, true),
        };
        
        // 根据通道配置输入捕获
        match channel {
            PwmChannel::Channel1 => {
                // 配置CCMR1寄存器
                tim.ccmr1_input().write(|w| {
                    w.cc1s().bits(0b01)  // CC1S=01: TI1作为输入
                        .ic2pcs().bits(icpsc_value)  // 输入捕获预分频器
                        .ic1f().bits(0b0000)  // 输入滤波器关闭
                });
                
                // 配置CCER寄存器
                tim.ccer().modify(|_, w| {
                    let mut w = w;
                    if ccp_value { w = w.cc1p().set_bit(); }
                    if ccnp_value { w = w.cc1np().set_bit(); }
                    w.cc1e().set_bit()  // 启用捕获通道
                });
            },
            PwmChannel::Channel2 => {
                // 配置CCMR1寄存器
                tim.ccmr1_input().write(|w| {
                    w.cc2s().bits(0b01)  // CC2S=01: TI2作为输入
                        .ic2pcs().bits(icpsc_value)  // 输入捕获预分频器
                        .ic2f().bits(0b0000)  // 输入滤波器关闭
                });
                
                // 配置CCER寄存器
                tim.ccer().modify(|_, w| {
                    let mut w = w;
                    if ccp_value { w = w.cc2p().set_bit(); }
                    if ccnp_value { w = w.cc2np().set_bit(); }
                    w.cc2e().set_bit()  // 启用捕获通道
                });
            },
            PwmChannel::Channel3 => {
                // 配置CCMR2寄存器
                tim.ccmr2_input().write(|w| {
                    w.cc3s().bits(0b01)  // CC3S=01: TI3作为输入
                        .ic3psc().bits(icpsc_value)  // 输入捕获预分频器
                        .ic3f().bits(0b0000)  // 输入滤波器关闭
                });
                
                // 配置CCER寄存器
                tim.ccer().modify(|_, w| {
                    let mut w = w;
                    if ccp_value { w = w.cc3p().set_bit(); }
                    if ccnp_value { w = w.cc3np().set_bit(); }
                    w.cc3e().set_bit()  // 启用捕获通道
                });
            },
            PwmChannel::Channel4 => {
                // 配置CCMR2寄存器
                tim.ccmr2_input().write(|w| {
                    w.cc4s().bits(0b01)  // CC4S=01: TI4作为输入
                        .ic4psc().bits(icpsc_value)  // 输入捕获预分频器
                        .ic4f().bits(0b0000)  // 输入滤波器关闭
                });
                
                // 配置CCER寄存器
                tim.ccer().modify(|_, w| {
                    let ccp = if ccp_value { w.cc4p().set_bit() } else { w.cc4p().clear_bit() };
                    // 通道4没有CC4NP位
                    ccp
                        .cc4e().set_bit()  // 启用捕获通道
                });
            },
        }
        
        // 清除更新中断标志
        tim.sr().write(|w| w.uif().clear_bit());
        
        // 启用定时器
        tim.cr1().write(|w| w.cen().set_bit());
    }
    
    /// 检查输入捕获中断标志
    pub unsafe fn has_capture_interrupt(&self, channel: PwmChannel) -> bool {
        // 参数有效性验证
        match channel {
            PwmChannel::Channel1 | PwmChannel::Channel2 | 
            PwmChannel::Channel3 | PwmChannel::Channel4 => {},
        }
        
        match self.number {
            TimerNumber::TIM1 => {
                let tim = self.get_tim1();
                match channel {
                    PwmChannel::Channel1 => tim.sr().read().cc1if().bit_is_set(),
                    PwmChannel::Channel2 => tim.sr().read().cc2if().bit_is_set(),
                    PwmChannel::Channel3 => tim.sr().read().cc3if().bit_is_set(),
                    PwmChannel::Channel4 => tim.sr().read().cc4if().bit_is_set(),
                }
            },
            TimerNumber::TIM2 => {
                let tim = self.get_tim2();
                match channel {
                    PwmChannel::Channel1 => tim.sr().read().cc1if().bit_is_set(),
                    PwmChannel::Channel2 => tim.sr().read().cc2if().bit_is_set(),
                    PwmChannel::Channel3 => tim.sr().read().cc3if().bit_is_set(),
                    PwmChannel::Channel4 => tim.sr().read().cc4if().bit_is_set(),
                }
            },
            TimerNumber::TIM3 => {
                let tim = self.get_tim3();
                match channel {
                    PwmChannel::Channel1 => tim.sr().read().cc1if().bit_is_set(),
                    PwmChannel::Channel2 => tim.sr().read().cc2if().bit_is_set(),
                    PwmChannel::Channel3 => tim.sr().read().cc3if().bit_is_set(),
                    PwmChannel::Channel4 => tim.sr().read().cc4if().bit_is_set(),
                }
            },
            TimerNumber::TIM4 => {
                let tim = self.get_tim4();
                match channel {
                    PwmChannel::Channel1 => tim.sr().read().cc1if().bit_is_set(),
                    PwmChannel::Channel2 => tim.sr().read().cc2if().bit_is_set(),
                    PwmChannel::Channel3 => tim.sr().read().cc3if().bit_is_set(),
                    PwmChannel::Channel4 => tim.sr().read().cc4if().bit_is_set(),
                }
            },
            TimerNumber::TIM5 => {
                let tim = self.get_tim5();
                match channel {
                    PwmChannel::Channel1 => tim.sr().read().cc1if().bit_is_set(),
                    PwmChannel::Channel2 => tim.sr().read().cc2if().bit_is_set(),
                    PwmChannel::Channel3 => tim.sr().read().cc3if().bit_is_set(),
                    PwmChannel::Channel4 => tim.sr().read().cc4if().bit_is_set(),
                }
            },
            TimerNumber::TIM8 => {
                let tim = self.get_tim8();
                match channel {
                    PwmChannel::Channel1 => tim.sr().read().cc1if().bit_is_set(),
                    PwmChannel::Channel2 => tim.sr().read().cc2if().bit_is_set(),
                    PwmChannel::Channel3 => tim.sr().read().cc3if().bit_is_set(),
                    PwmChannel::Channel4 => tim.sr().read().cc4if().bit_is_set(),
                }
            },
            _ => false, // 基本定时器不支持输入捕获
        }
    }
    
    /// 清除输入捕获中断标志
    pub unsafe fn clear_capture_interrupt(&self, channel: PwmChannel) {
        match self.number {
            TimerNumber::TIM1 => {
                let tim = self.get_tim1();
                let _ = match channel {
                    PwmChannel::Channel1 => { tim.sr().write(|w| w.cc1if().clear_bit()); },
                    PwmChannel::Channel2 => { tim.sr().write(|w| w.cc2if().clear_bit()); },
                    PwmChannel::Channel3 => { tim.sr().write(|w| w.cc3if().clear_bit()); },
                    PwmChannel::Channel4 => { tim.sr().write(|w| w.cc4if().clear_bit()); },
                };
            },
            TimerNumber::TIM2 => {
                let tim = self.get_tim2();
                let _ = match channel {
                    PwmChannel::Channel1 => tim.sr().write(|w| w.cc1if().clear_bit()),  
                    PwmChannel::Channel2 => tim.sr().write(|w| w.cc2if().clear_bit()),  
                    PwmChannel::Channel3 => tim.sr().write(|w| w.cc3if().clear_bit()),  
                    PwmChannel::Channel4 => tim.sr().write(|w| w.cc4if().clear_bit()),  
                };
            },
            TimerNumber::TIM3 => {
                let tim = self.get_tim3();
                let _ = match channel {
                    PwmChannel::Channel1 => tim.sr().write(|w| w.cc1if().clear_bit()),
                    PwmChannel::Channel2 => tim.sr().write(|w| w.cc2if().clear_bit()),
                    PwmChannel::Channel3 => tim.sr().write(|w| w.cc3if().clear_bit()),
                    PwmChannel::Channel4 => tim.sr().write(|w| w.cc4if().clear_bit()),
                };
            },
            TimerNumber::TIM4 => {
                let tim = self.get_tim4();
                let _ = match channel {
                    PwmChannel::Channel1 => tim.sr().write(|w| w.cc1if().clear_bit()),
                    PwmChannel::Channel2 => tim.sr().write(|w| w.cc2if().clear_bit()),
                    PwmChannel::Channel3 => tim.sr().write(|w| w.cc3if().clear_bit()),
                    PwmChannel::Channel4 => tim.sr().write(|w| w.cc4if().clear_bit()),
                };
            },
            TimerNumber::TIM5 => {
                let tim = self.get_tim5();
                let _ = match channel {
                    PwmChannel::Channel1 => tim.sr().write(|w| w.cc1if().clear_bit()),
                    PwmChannel::Channel2 => tim.sr().write(|w| w.cc2if().clear_bit()),
                    PwmChannel::Channel3 => tim.sr().write(|w| w.cc3if().clear_bit()),
                    PwmChannel::Channel4 => tim.sr().write(|w| w.cc4if().clear_bit()),
                };
            },
            TimerNumber::TIM8 => {
                let tim = self.get_tim8();
                let _ = match channel {
                    PwmChannel::Channel1 => tim.sr().write(|w| w.cc1if().clear_bit()),
                    PwmChannel::Channel2 => tim.sr().write(|w| w.cc2if().clear_bit()),
                    PwmChannel::Channel3 => tim.sr().write(|w| w.cc3if().clear_bit()),
                    PwmChannel::Channel4 => tim.sr().write(|w| w.cc4if().clear_bit()),
                };
            },
            _ => {}, // 基本定时器不支持输入捕获
        }
    }
    
    /// 获取输入捕获值
    pub unsafe fn get_capture_value(&self, channel: PwmChannel) -> u16 {
        match self.number {
            TimerNumber::TIM1 => {
                let tim = self.get_tim1();
                match channel {
                    PwmChannel::Channel1 => tim.ccr1().read().ccr1().bits(),
                    PwmChannel::Channel2 => tim.ccr2().read().ccr2().bits(),
                    PwmChannel::Channel3 => tim.ccr3().read().ccr3().bits(),
                    PwmChannel::Channel4 => tim.ccr4().read().ccr4().bits(),
                }
            },
            TimerNumber::TIM2 => {
                let tim = self.get_tim2();
                match channel {
                    PwmChannel::Channel1 => tim.ccr1().read().ccr1().bits(),
                    PwmChannel::Channel2 => tim.ccr2().read().ccr2().bits(),
                    PwmChannel::Channel3 => tim.ccr3().read().ccr3().bits(),
                    PwmChannel::Channel4 => tim.ccr4().read().ccr4().bits(),
                }
            },
            TimerNumber::TIM3 => {
                let tim = self.get_tim3();
                match channel {
                    PwmChannel::Channel1 => tim.ccr1().read().ccr1().bits(),
                    PwmChannel::Channel2 => tim.ccr2().read().ccr2().bits(),
                    PwmChannel::Channel3 => tim.ccr3().read().ccr3().bits(),
                    PwmChannel::Channel4 => tim.ccr4().read().ccr4().bits(),
                }
            },
            TimerNumber::TIM4 => {
                let tim = self.get_tim4();
                match channel {
                    PwmChannel::Channel1 => tim.ccr1().read().ccr1().bits(),
                    PwmChannel::Channel2 => tim.ccr2().read().ccr2().bits(),
                    PwmChannel::Channel3 => tim.ccr3().read().ccr3().bits(),
                    PwmChannel::Channel4 => tim.ccr4().read().ccr4().bits(),
                }
            },
            TimerNumber::TIM5 => {
                let tim = self.get_tim5();
                match channel {
                    PwmChannel::Channel1 => tim.ccr1().read().ccr1().bits(),
                    PwmChannel::Channel2 => tim.ccr2().read().ccr2().bits(),
                    PwmChannel::Channel3 => tim.ccr3().read().ccr3().bits(),
                    PwmChannel::Channel4 => tim.ccr4().read().ccr4().bits(),
                }
            },
            TimerNumber::TIM8 => {
                let tim = self.get_tim8();
                match channel {
                    PwmChannel::Channel1 => tim.ccr1().read().ccr1().bits(),
                    PwmChannel::Channel2 => tim.ccr2().read().ccr2().bits(),
                    PwmChannel::Channel3 => tim.ccr3().read().ccr3().bits(),
                    PwmChannel::Channel4 => tim.ccr4().read().ccr4().bits(),
                }
            },
            _ => 0, // 基本定时器不支持输入捕获
        }
    }
    
    /// 启用输入捕获中断
    pub unsafe fn enable_capture_interrupt(&self, channel: PwmChannel) {
        match self.number {
            TimerNumber::TIM1 => {
                let tim = self.get_tim1();
                let _ = match channel {
                    PwmChannel::Channel1 => { tim.dier().modify(|_, w| w.cc1ie().set_bit()); },
                    PwmChannel::Channel2 => { tim.dier().modify(|_, w| w.cc2ie().set_bit()); },
                    PwmChannel::Channel3 => { tim.dier().modify(|_, w| w.cc3ie().set_bit()); },
                    PwmChannel::Channel4 => { tim.dier().modify(|_, w| w.cc4ie().set_bit()); },
                };
            },
            TimerNumber::TIM2 => {
                let tim = self.get_tim2();
                let _ = match channel {
                    PwmChannel::Channel1 => tim.dier().modify(|_, w| w.cc1ie().set_bit()),        
                    PwmChannel::Channel2 => tim.dier().modify(|_, w| w.cc2ie().set_bit()),        
                    PwmChannel::Channel3 => tim.dier().modify(|_, w| w.cc3ie().set_bit()),        
                    PwmChannel::Channel4 => tim.dier().modify(|_, w| w.cc4ie().set_bit()),        
                };
            },
            TimerNumber::TIM3 => {
                let tim = self.get_tim3();
                let _ = match channel {
                    PwmChannel::Channel1 => tim.dier().modify(|_, w| w.cc1ie().set_bit()),
                    PwmChannel::Channel2 => tim.dier().modify(|_, w| w.cc2ie().set_bit()),
                    PwmChannel::Channel3 => tim.dier().modify(|_, w| w.cc3ie().set_bit()),
                    PwmChannel::Channel4 => tim.dier().modify(|_, w| w.cc4ie().set_bit()),
                };
            },
            TimerNumber::TIM4 => {
                let tim = self.get_tim4();
                let _ = match channel {
                    PwmChannel::Channel1 => tim.dier().modify(|_, w| w.cc1ie().set_bit()),
                    PwmChannel::Channel2 => tim.dier().modify(|_, w| w.cc2ie().set_bit()),
                    PwmChannel::Channel3 => tim.dier().modify(|_, w| w.cc3ie().set_bit()),
                    PwmChannel::Channel4 => tim.dier().modify(|_, w| w.cc4ie().set_bit()),
                };
            },
            TimerNumber::TIM5 => {
                let tim = self.get_tim5();
                let _ = match channel {
                    PwmChannel::Channel1 => tim.dier().modify(|_, w| w.cc1ie().set_bit()),
                    PwmChannel::Channel2 => tim.dier().modify(|_, w| w.cc2ie().set_bit()),
                    PwmChannel::Channel3 => tim.dier().modify(|_, w| w.cc3ie().set_bit()),
                    PwmChannel::Channel4 => tim.dier().modify(|_, w| w.cc4ie().set_bit()),
                };
            },
            TimerNumber::TIM8 => {
                let tim = self.get_tim8();
                let _ = match channel {
                    PwmChannel::Channel1 => tim.dier().modify(|_, w| w.cc1ie().set_bit()),
                    PwmChannel::Channel2 => tim.dier().modify(|_, w| w.cc2ie().set_bit()),
                    PwmChannel::Channel3 => tim.dier().modify(|_, w| w.cc3ie().set_bit()),
                    PwmChannel::Channel4 => tim.dier().modify(|_, w| w.cc4ie().set_bit()),
                };
            },
            _ => {}, // 基本定时器不支持输入捕获
        }
    }
    
    /// 禁用输入捕获中断
    pub unsafe fn disable_capture_interrupt(&self, channel: PwmChannel) {
        match self.number {
            TimerNumber::TIM1 => {
                let tim = self.get_tim1();
                let _ = match channel {
                    PwmChannel::Channel1 => { tim.dier().modify(|_, w| w.cc1ie().clear_bit()); },
                    PwmChannel::Channel2 => { tim.dier().modify(|_, w| w.cc2ie().clear_bit()); },
                    PwmChannel::Channel3 => { tim.dier().modify(|_, w| w.cc3ie().clear_bit()); },
                    PwmChannel::Channel4 => { tim.dier().modify(|_, w| w.cc4ie().clear_bit()); },
                };
            },
            TimerNumber::TIM2 => {
                let tim = self.get_tim2();
                let _ = match channel {
                    PwmChannel::Channel1 => tim.dier().modify(|_, w| w.cc1ie().clear_bit()),      
                    PwmChannel::Channel2 => tim.dier().modify(|_, w| w.cc2ie().clear_bit()),      
                    PwmChannel::Channel3 => tim.dier().modify(|_, w| w.cc3ie().clear_bit()),      
                    PwmChannel::Channel4 => tim.dier().modify(|_, w| w.cc4ie().clear_bit()),      
                };
            },
            TimerNumber::TIM3 => {
                let tim = self.get_tim3();
                let _ = match channel {
                    PwmChannel::Channel1 => tim.dier().modify(|_, w| w.cc1ie().clear_bit()),
                    PwmChannel::Channel2 => tim.dier().modify(|_, w| w.cc2ie().clear_bit()),
                    PwmChannel::Channel3 => tim.dier().modify(|_, w| w.cc3ie().clear_bit()),
                    PwmChannel::Channel4 => tim.dier().modify(|_, w| w.cc4ie().clear_bit()),
                };
            },
            TimerNumber::TIM4 => {
                let tim = self.get_tim4();
                let _ = match channel {
                    PwmChannel::Channel1 => tim.dier().modify(|_, w| w.cc1ie().clear_bit()),
                    PwmChannel::Channel2 => tim.dier().modify(|_, w| w.cc2ie().clear_bit()),
                    PwmChannel::Channel3 => tim.dier().modify(|_, w| w.cc3ie().clear_bit()),
                    PwmChannel::Channel4 => tim.dier().modify(|_, w| w.cc4ie().clear_bit()),
                };
            },
            TimerNumber::TIM5 => {
                let tim = self.get_tim5();
                let _ = match channel {
                    PwmChannel::Channel1 => tim.dier().modify(|_, w| w.cc1ie().clear_bit()),
                    PwmChannel::Channel2 => tim.dier().modify(|_, w| w.cc2ie().clear_bit()),
                    PwmChannel::Channel3 => tim.dier().modify(|_, w| w.cc3ie().clear_bit()),
                    PwmChannel::Channel4 => tim.dier().modify(|_, w| w.cc4ie().clear_bit()),
                };
            },
            TimerNumber::TIM8 => {
                let tim = self.get_tim8();
                let _ = match channel {
                    PwmChannel::Channel1 => tim.dier().modify(|_, w| w.cc1ie().clear_bit()),
                    PwmChannel::Channel2 => tim.dier().modify(|_, w| w.cc2ie().clear_bit()),
                    PwmChannel::Channel3 => tim.dier().modify(|_, w| w.cc3ie().clear_bit()),
                    PwmChannel::Channel4 => tim.dier().modify(|_, w| w.cc4ie().clear_bit()),
                };
            },
            _ => {}, // 基本定时器不支持输入捕获
        }
    }
    

}

/// 预定义的定时器常量
pub const TIM1: Timer = Timer::new(TimerNumber::TIM1);
pub const TIM2: Timer = Timer::new(TimerNumber::TIM2);
pub const TIM3: Timer = Timer::new(TimerNumber::TIM3);
pub const TIM4: Timer = Timer::new(TimerNumber::TIM4);
