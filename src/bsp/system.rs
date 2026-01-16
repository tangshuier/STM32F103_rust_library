//! 系统初始化模块
//! 提供STM32F103C8T6的系统时钟初始化和管理功能

#![allow(unused)]

use core::fmt;
use core::arch::asm;
use cortex_m::peripheral;
use heapless::String;
use library::*;
use library::rcc::RegisterBlock as RccRegisterBlock;
use library::flash::RegisterBlock as FlashRegisterBlock;

// 引用延时模块
use super::delay;

// 定义常量
const HSE_STARTUP_TIMEOUT: u32 = 0x05000;

/// 系统时钟频率结构体
pub struct SystemClocks {
    pub sysclk: u32,
    pub hclk: u32,
    pub pclk1: u32,
    pub pclk2: u32,
    pub adcclk: u32,
}

/// 系统初始化结果
#[derive(Debug, PartialEq)]
pub enum InitResult {
    Success,
    HseTimeout,
    HsiTimeout,
    PllTimeout,
    InvalidConfig,
    ClockConfigError,
}

/// 日志级别
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
    Debug,
}

/// 日志记录函数类型
pub type LogHandler = fn(LogLevel, &str);

/// 全局日志处理函数
static mut LOG_HANDLER: Option<LogHandler> = None;

/// 设置日志处理函数
/// 
/// # 参数
/// - `handler`：日志处理函数
pub fn set_log_handler(handler: LogHandler) {
    unsafe {
        LOG_HANDLER = Some(handler);
    }
}

/// 记录日志
/// 
/// # 参数
/// - `level`：日志级别
/// - `message`：日志消息
pub fn log(level: LogLevel, message: &str) {
    unsafe {
        if let Some(handler) = LOG_HANDLER {
            handler(level, message);
        }
    }
}

/// 记录错误日志
/// 
/// # 参数
/// - `message`：错误消息
pub fn log_error(message: &str) {
    log(LogLevel::Error, message);
}

/// 记录警告日志
/// 
/// # 参数
/// - `message`：警告消息
pub fn log_warning(message: &str) {
    log(LogLevel::Warning, message);
}

/// 记录信息日志
/// 
/// # 参数
/// - `message`：信息消息
pub fn log_info(message: &str) {
    log(LogLevel::Info, message);
}

/// 记录调试日志
/// 
/// # 参数
/// - `message`：调试消息
pub fn log_debug(message: &str) {
    log(LogLevel::Debug, message);
}

/// 外设时钟使能/禁用
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PeripheralClock {
    // AHB外设
    DMA1,
    DMA2,
    SRAM,
    FLITF,
    CRC,
    // APB2外设
    AFIO,
    GPIOA,
    GPIOB,
    GPIOC,
    GPIOD,
    GPIOE,
    GPIOF,
    GPIOG,
    ADC1,
    ADC2,
    TIM1,
    SPI1,
    USART1,
    // APB1外设
    TIM2,
    TIM3,
    TIM4,
    TIM5,
    TIM6,
    TIM7,
    WWDG,
    SPI2,
    SPI3,
    USART2,
    USART3,
    UART4,
    UART5,
    I2C1,
    I2C2,
    USB,
    CAN1,
    BKP,
    PWR,
    DAC,
}

/// 系统时钟配置结构体
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct ClockConfig {
    /// 系统时钟频率（Hz）
    pub sysclk: u32,
    /// HSE频率（Hz），如果使用HSE作为时钟源
    pub hse_freq: Option<u32>,
    /// 是否使用PLL
    pub use_pll: bool,
    /// PLL倍频系数
    pub pll_mul: u8,
    /// AHB预分频系数
    pub hpre: u32,
    /// APB1预分频系数
    pub ppre1: u32,
    /// APB2预分频系数
    pub ppre2: u32,
    /// ADC预分频系数
    pub adcpre: u32,
}

impl Default for ClockConfig {
    /// 默认时钟配置：72MHz系统时钟，使用HSE和PLL
    fn default() -> Self {
        Self {
            sysclk: 72_000_000,
            hse_freq: Some(8_000_000),
            use_pll: true,
            pll_mul: 9,
            hpre: 1,
            ppre1: 2,
            ppre2: 1,
            adcpre: 6,
        }
    }
}

impl ClockConfig {
    /// 创建8MHz时钟配置（使用HSI，不使用PLL）
    pub fn hsi_8mhz() -> Self {
        Self {
            sysclk: 8_000_000,
            hse_freq: None,
            use_pll: false,
            pll_mul: 1,
            hpre: 1,
            ppre1: 1,
            ppre2: 1,
            adcpre: 2,
        }
    }
    
    /// 创建36MHz时钟配置（使用HSE和PLL）
    pub fn hse_36mhz() -> Self {
        Self {
            sysclk: 36_000_000,
            hse_freq: Some(8_000_000),
            use_pll: true,
            pll_mul: 9,
            hpre: 2,
            ppre1: 1,
            ppre2: 1,
            adcpre: 6,
        }
    }
    
    /// 创建48MHz时钟配置（使用HSE和PLL）
    pub fn hse_48mhz() -> Self {
        Self {
            sysclk: 48_000_000,
            hse_freq: Some(8_000_000),
            use_pll: true,
            pll_mul: 6,
            hpre: 1,
            ppre1: 1,
            ppre2: 1,
            adcpre: 4,
        }
    }
    
    /// 创建72MHz时钟配置（使用HSE和PLL）
    pub fn hse_72mhz() -> Self {
        Self::default()
    }
}

/// 系统初始化函数（使用默认配置）
/// 初始化STM32F103C8T6的系统时钟到72MHz
/// 
/// 配置：
/// - 使用HSE（外部高速时钟，8MHz）作为系统时钟源
/// - 通过PLL倍频到72MHz
/// - SYSCLK = 72MHz
/// - HCLK = 72MHz
/// - PCLK1 = 36MHz
/// - PCLK2 = 72MHz
/// 
/// # 返回值
/// - `InitResult::Success`：初始化成功
/// - `InitResult::HseTimeout`：HSE启动超时
/// - `InitResult::HsiTimeout`：HSI启动超时
/// - `InitResult::PllTimeout`：PLL启动超时
pub fn init() -> InitResult {
    init_with_config(&ClockConfig::default())
}

/// 系统初始化函数（使用自定义配置）
/// 初始化STM32F103C8T6的系统时钟，根据提供的配置
/// 
/// # 参数
/// - `config`：时钟配置结构体
/// 
/// # 返回值
/// - `InitResult::Success`：初始化成功
/// - `InitResult::HseTimeout`：HSE启动超时
/// - `InitResult::HsiTimeout`：HSI启动超时
/// - `InitResult::PllTimeout`：PLL启动超时
/// - `InitResult::InvalidConfig`：无效的配置
/// - `InitResult::ClockConfigError`：时钟配置错误
pub fn init_with_config(config: &ClockConfig) -> InitResult {
    // 构建日志消息
    let mut msg = heapless::String::<128>::new();
    msg.push_str("开始系统初始化，目标时钟频率: ").unwrap();
    push_u32_to_string(&mut msg, config.sysclk).unwrap();
    msg.push_str(" Hz").unwrap();
    log_info(msg.as_str());
    
    let rcc = unsafe { &*library::Rcc::ptr() };
    let flash = unsafe { &*library::Flash::ptr() };
    let scb = unsafe { &mut *(peripheral::SCB::PTR as *mut peripheral::SCB) };
    
    // 1. 验证配置
    if config.sysclk > 72_000_000 {
        log_error("无效的系统时钟频率配置，超过最大允许值72MHz");
        return InitResult::InvalidConfig;
    }
    
    // 2. 重置RCC时钟配置到默认状态
    log_debug("重置RCC时钟配置到默认状态");
    if !reset_rcc_config(rcc) {
        log_error("HSI启动失败，无法重置RCC配置");
        return InitResult::HsiTimeout;
    }
    
    // 3. 配置Flash
    log_debug("配置Flash参数");
    // 启用预取缓冲区
    flash.acr().modify(|_, w: &mut library::flash::acr::W| w.prftbe().set_bit());
    
    // 根据系统时钟频率设置Flash延迟
    let latency = match config.sysclk {
        0..=24_000_000 => {
            log_debug("设置Flash延迟为0个等待周期");
            0x00 // 0个等待周期
        },
        24_000_001..=48_000_000 => {
            log_debug("设置Flash延迟为1个等待周期");
            0x01 // 1个等待周期
        },
        48_000_001..=72_000_000 => {
            log_debug("设置Flash延迟为2个等待周期");
            0x02 // 2个等待周期
        },
        _ => {
            log_warning("系统时钟频率过高，默认设置Flash延迟为2个等待周期");
            0x02 // 默认2个等待周期
        },
    };
    flash.acr().modify(|_, w: &mut library::flash::acr::W| unsafe { w.latency().bits(latency) });
    
    // 4. 处理时钟源配置
    if let Some(hse_freq) = config.hse_freq {
        // 构建日志消息
        let mut msg = heapless::String::<128>::new();
        msg.push_str("使用HSE作为时钟源，频率: ").unwrap();
        push_u32_to_string(&mut msg, hse_freq).unwrap();
        msg.push_str(" Hz").unwrap();
        log_info(msg.as_str());
        
        // 启用HSE（外部晶振）
        rcc.cr().modify(|_, w: &mut library::rcc::cr::W| w.hseon().set_bit());
        
        // 等待HSE就绪或超时
        log_debug("等待HSE就绪");
        if !wait_for_flag(|| rcc.cr().read().hserdy().bit_is_set(), HSE_STARTUP_TIMEOUT) {
            log_error("HSE启动超时，无法启用外部晶振");
            return InitResult::HseTimeout;
        }
        log_info("HSE已就绪");
        
        if config.use_pll {
            // 构建日志消息
            let mut msg = heapless::String::<128>::new();
            msg.push_str("使用PLL，倍频系数: ").unwrap();
            push_u32_to_string(&mut msg, config.pll_mul as u32).unwrap();
            log_info(msg.as_str());
            
            // 5. 配置PLL
            // HSE作为PLL输入，设置倍频系数
            let pll_mul_bits = (config.pll_mul - 2) as u8; // PLL倍频系数从2开始
            rcc.cfgr().modify(|_, w: &mut library::rcc::cfgr::W| {
                w.pllsrc().set_bit();
                unsafe { w.pllmul().bits(pll_mul_bits) }
            });
            
            // 6. 启用PLL
            rcc.cr().modify(|_, w: &mut library::rcc::cr::W| w.pllon().set_bit());
            
            // 等待PLL就绪
            log_debug("等待PLL就绪");
            if !wait_for_flag(|| rcc.cr().read().pllrdy().bit_is_set(), HSE_STARTUP_TIMEOUT) {
                log_error("PLL启动超时，无法锁定到目标频率");
                return InitResult::PllTimeout;
            }
            log_info("PLL已就绪");
            
            // 7. 选择PLL作为系统时钟源
            log_debug("切换系统时钟源到PLL");
            rcc.cfgr().modify(|_, w: &mut library::rcc::cfgr::W| unsafe { w.sw().bits(0x02) });
            
            // 等待系统时钟切换到PLL
            while rcc.cfgr().read().sws().bits() != 0x02 {};
            log_info("系统时钟已成功切换到PLL");
        } else {
            // 选择HSE作为系统时钟源
            log_debug("切换系统时钟源到HSE");
            rcc.cfgr().modify(|_, w: &mut library::rcc::cfgr::W| unsafe { w.sw().bits(0x01) });
            
            // 等待系统时钟切换到HSE
            while rcc.cfgr().read().sws().bits() != 0x01 {};
            log_info("系统时钟已成功切换到HSE");
        }
    } else {
        // 使用HSI作为系统时钟源
        log_info("使用HSI作为时钟源（8MHz）");
        rcc.cfgr().modify(|_, w: &mut library::rcc::cfgr::W| unsafe { w.sw().bits(0x00) });
        
        // 等待系统时钟切换到HSI
        while rcc.cfgr().read().sws().bits() != 0x00 {};
        log_info("系统时钟已成功切换到HSI");
    }
    
    // 8. 配置总线预分频
    log_debug("配置总线预分频器");
    
    // 配置AHB预分频
    let hpre_bits = match config.hpre {
        1 => 0x00,
        2 => 0x08,
        4 => 0x09,
        8 => 0x0A,
        16 => 0x0B,
        64 => 0x0C,
        128 => 0x0D,
        256 => 0x0E,
        512 => 0x0F,
        _ => 0x00,
    };
    rcc.cfgr().modify(|_, w: &mut library::rcc::cfgr::W| unsafe { w.hpre().bits(hpre_bits) });
    
    // 配置APB2预分频
    let ppre2_bits = match config.ppre2 {
        1 => 0x00,
        2 => 0x04,
        4 => 0x05,
        8 => 0x06,
        16 => 0x07,
        _ => 0x00,
    };
    rcc.cfgr().modify(|_, w: &mut library::rcc::cfgr::W| unsafe { w.ppre2().bits(ppre2_bits) });
    
    // 配置APB1预分频
    let ppre1_bits = match config.ppre1 {
        1 => 0x00,
        2 => 0x04,
        4 => 0x05,
        8 => 0x06,
        16 => 0x07,
        _ => 0x00,
    };
    rcc.cfgr().modify(|_, w: &mut library::rcc::cfgr::W| unsafe { w.ppre1().bits(ppre1_bits) });
    
    // 配置ADC预分频
    let adcpre_bits = match config.adcpre {
        2 => 0x00,
        4 => 0x01,
        6 => 0x02,
        8 => 0x03,
        _ => 0x00,
    };
    rcc.cfgr().modify(|_, w: &mut library::rcc::cfgr::W| unsafe { w.adcpre().bits(adcpre_bits) });
    
    // 9. 设置向量表偏移
    log_debug("设置向量表偏移");
    unsafe {
        scb.vtor.write(0x08000000);
    }
    
    // 10. 初始化延时模块
    log_debug("初始化延时模块");
    unsafe {
        delay::init_systick(config.sysclk);
    }
    
    // 构建日志消息
    let mut msg = heapless::String::<128>::new();
    msg.push_str("系统初始化完成，当前系统时钟频率: ").unwrap();
    push_u32_to_string(&mut msg, config.sysclk).unwrap();
    msg.push_str(" Hz").unwrap();
    log_info(msg.as_str());
    
    InitResult::Success
}

/// 等待标志位设置的辅助函数
/// 
/// # 参数
/// - `check_flag`：检查标志位的闭包
/// - `timeout`：超时计数
/// 
/// # 返回值
/// - `true`：标志位设置成功
/// - `false`：超时
fn wait_for_flag<F>(check_flag: F, timeout: u32) -> bool
where
    F: Fn() -> bool,
{
    let mut remaining = timeout;
    while !check_flag() && remaining > 0 {
        remaining -= 1;
    }
    remaining > 0
}

/// 重置RCC时钟配置到默认状态
/// 
/// # 参数
/// - `rcc`：RCC寄存器块引用
/// 
/// # 返回值
/// - `true`：重置成功
/// - `false`：HSI启动失败
fn reset_rcc_config(rcc: &library::rcc::RegisterBlock) -> bool {
    // 设置HSION位（启用内部高速时钟作为备用）
    rcc.cr().modify(|_, w: &mut library::rcc::cr::W| w.hsion().set_bit());
    
    // 等待HSI就绪
    if !wait_for_flag(|| rcc.cr().read().hsirdy().bit_is_set(), 1000) {
        return false;
    }
    
    // 重置SW, HPRE, PPRE1, PPRE2, ADCPRE和MCO位
    unsafe {
        rcc.cfgr().modify(|_, w: &mut library::rcc::cfgr::W| w.bits(0xF8FF0000));
    }
    
    // 重置HSEON, CSSON和PLLON位
    unsafe {
        rcc.cr().modify(|_, w: &mut library::rcc::cr::W| w.bits(0xFEF6FFFF));
    }
    
    // 重置HSEBYP位
    unsafe {
        rcc.cr().modify(|_, w: &mut library::rcc::cr::W| w.bits(0xFFFBFFFF));
    }
    
    // 重置PLLSRC, PLLXTPRE, PLLMUL和USBPRE/OTGFSPRE位
    unsafe {
        rcc.cfgr().modify(|_, w: &mut library::rcc::cfgr::W| w.bits(0xFF80FFFF));
    }
    
    // 禁用所有中断和清除挂起位
    unsafe {
        rcc.cir().write(|w: &mut library::rcc::cir::W| w.bits(0x009F0000));
    }
    
    true
}

/// 获取系统时钟频率
pub fn get_system_clocks() -> SystemClocks {
    let rcc = unsafe { &*library::Rcc::ptr() };
    
    // 计算系统时钟频率
    let sysclk = match rcc.cfgr().read().sws().bits() {
        0x00 => 8_000_000, // HSI
        0x01 => 8_000_000, // HSE
        0x02 => 72_000_000, // PLL
        _ => 8_000_000,
    };
    
    // 计算HCLK频率
    let hclk_div = match rcc.cfgr().read().hpre().bits() {
        0x0 => 1,
        0x8 => 2,
        0x9 => 4,
        0xA => 8,
        0xB => 16,
        0xC => 64,
        0xD => 128,
        0xE => 256,
        0xF => 512,
        _ => 1,
    };
    let hclk = sysclk / hclk_div;
    
    // 计算PCLK1频率
    let pclk1_div = match rcc.cfgr().read().ppre1().bits() {
        0x0 => 1,
        0x4 => 2,
        0x5 => 4,
        0x6 => 8,
        0x7 => 16,
        _ => 1,
    };
    let pclk1 = hclk / pclk1_div;
    
    // 计算PCLK2频率
    let pclk2_div = match rcc.cfgr().read().ppre2().bits() {
        0x0 => 1,
        0x4 => 2,
        0x5 => 4,
        0x6 => 8,
        0x7 => 16,
        _ => 1,
    };
    let pclk2 = hclk / pclk2_div;
    
    // 计算ADCCLK频率
    let adcclk_div = match rcc.cfgr().read().adcpre().bits() {
        0x0 => 2,
        0x1 => 4,
        0x2 => 6,
        0x3 => 8,
        _ => 2,
    };
    let adcclk = pclk2 / adcclk_div;
    
    SystemClocks {
        sysclk,
        hclk,
        pclk1,
        pclk2,
        adcclk,
    }
}

/// 使能或禁用外设时钟
pub fn set_peripheral_clock(periph: PeripheralClock, enable: bool) {
    let rcc = unsafe { &*library::Rcc::ptr() };
    
    match periph {
        // AHB外设
        PeripheralClock::DMA1 => if enable {
            rcc.ahbenr().modify(|_, w: &mut library::rcc::ahbenr::W| w.dma1en().set_bit());
        } else {
            rcc.ahbenr().modify(|_, w: &mut library::rcc::ahbenr::W| w.dma1en().clear_bit());
        },
        PeripheralClock::DMA2 => if enable {
            rcc.ahbenr().modify(|_, w: &mut library::rcc::ahbenr::W| w.dma2en().set_bit());
        } else {
            rcc.ahbenr().modify(|_, w: &mut library::rcc::ahbenr::W| w.dma2en().clear_bit());
        },
        PeripheralClock::SRAM => if enable {
            rcc.ahbenr().modify(|_, w: &mut library::rcc::ahbenr::W| w.sramen().set_bit());
        } else {
            rcc.ahbenr().modify(|_, w: &mut library::rcc::ahbenr::W| w.sramen().clear_bit());
        },
        PeripheralClock::FLITF => if enable {
            rcc.ahbenr().modify(|_, w: &mut library::rcc::ahbenr::W| w.flitfen().set_bit());
        } else {
            rcc.ahbenr().modify(|_, w: &mut library::rcc::ahbenr::W| w.flitfen().clear_bit());
        },
        PeripheralClock::CRC => if enable {
            rcc.ahbenr().modify(|_, w: &mut library::rcc::ahbenr::W| w.crcen().set_bit());
        } else {
            rcc.ahbenr().modify(|_, w: &mut library::rcc::ahbenr::W| w.crcen().clear_bit());
        },
        
        // APB2外设
        PeripheralClock::AFIO => if enable {
            rcc.apb2enr().modify(|_, w: &mut library::rcc::apb2enr::W| w.afioen().set_bit());
        } else {
            rcc.apb2enr().modify(|_, w: &mut library::rcc::apb2enr::W| w.afioen().clear_bit());
        },
        PeripheralClock::GPIOA => if enable {
            rcc.apb2enr().modify(|_, w: &mut library::rcc::apb2enr::W| w.iopaen().set_bit());
        } else {
            rcc.apb2enr().modify(|_, w: &mut library::rcc::apb2enr::W| w.iopaen().clear_bit());
        },
        PeripheralClock::GPIOB => if enable {
            rcc.apb2enr().modify(|_, w: &mut library::rcc::apb2enr::W| w.iopben().set_bit());
        } else {
            rcc.apb2enr().modify(|_, w: &mut library::rcc::apb2enr::W| w.iopben().clear_bit());
        },
        PeripheralClock::GPIOC => if enable {
            rcc.apb2enr().modify(|_, w: &mut library::rcc::apb2enr::W| w.iopcen().set_bit());
        } else {
            rcc.apb2enr().modify(|_, w: &mut library::rcc::apb2enr::W| w.iopcen().clear_bit());
        },
        PeripheralClock::GPIOD => if enable {
            rcc.apb2enr().modify(|_, w: &mut library::rcc::apb2enr::W| w.iopden().set_bit());
        } else {
            rcc.apb2enr().modify(|_, w: &mut library::rcc::apb2enr::W| w.iopden().clear_bit());
        },
        PeripheralClock::GPIOE => if enable {
            rcc.apb2enr().modify(|_, w: &mut library::rcc::apb2enr::W| w.iopeen().set_bit());
        } else {
            rcc.apb2enr().modify(|_, w: &mut library::rcc::apb2enr::W| w.iopeen().clear_bit());
        },
        PeripheralClock::GPIOF => if enable {
            rcc.apb2enr().modify(|_, w: &mut library::rcc::apb2enr::W| w.iopfen().set_bit());
        } else {
            rcc.apb2enr().modify(|_, w: &mut library::rcc::apb2enr::W| w.iopfen().clear_bit());
        },
        PeripheralClock::GPIOG => if enable {
            rcc.apb2enr().modify(|_, w: &mut library::rcc::apb2enr::W| w.iopgen().set_bit());
        } else {
            rcc.apb2enr().modify(|_, w: &mut library::rcc::apb2enr::W| w.iopgen().clear_bit());
        },
        PeripheralClock::ADC1 => if enable {
            rcc.apb2enr().modify(|_, w: &mut library::rcc::apb2enr::W| w.adc1en().set_bit());
        } else {
            rcc.apb2enr().modify(|_, w: &mut library::rcc::apb2enr::W| w.adc1en().clear_bit());
        },
        PeripheralClock::ADC2 => if enable {
            rcc.apb2enr().modify(|_, w: &mut library::rcc::apb2enr::W| w.adc2en().set_bit());
        } else {
            rcc.apb2enr().modify(|_, w: &mut library::rcc::apb2enr::W| w.adc2en().clear_bit());
        },
        PeripheralClock::TIM1 => if enable {
            rcc.apb2enr().modify(|_, w: &mut library::rcc::apb2enr::W| w.tim1en().set_bit());
        } else {
            rcc.apb2enr().modify(|_, w: &mut library::rcc::apb2enr::W| w.tim1en().clear_bit());
        },
        PeripheralClock::SPI1 => if enable {
            rcc.apb2enr().modify(|_, w: &mut library::rcc::apb2enr::W| w.spi1en().set_bit());
        } else {
            rcc.apb2enr().modify(|_, w: &mut library::rcc::apb2enr::W| w.spi1en().clear_bit());
        },
        PeripheralClock::USART1 => if enable {
            rcc.apb2enr().modify(|_, w: &mut library::rcc::apb2enr::W| w.usart1en().set_bit());
        } else {
            rcc.apb2enr().modify(|_, w: &mut library::rcc::apb2enr::W| w.usart1en().clear_bit());
        },
        
        // APB1外设
        PeripheralClock::TIM2 => if enable {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.tim2en().set_bit());
        } else {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.tim2en().clear_bit());
        },
        PeripheralClock::TIM3 => if enable {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.tim3en().set_bit());
        } else {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.tim3en().clear_bit());
        },
        PeripheralClock::TIM4 => if enable {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.tim4en().set_bit());
        } else {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.tim4en().clear_bit());
        },
        PeripheralClock::TIM5 => if enable {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.tim5en().set_bit());
        } else {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.tim5en().clear_bit());
        },
        PeripheralClock::TIM6 => if enable {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.tim6en().set_bit());
        } else {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.tim6en().clear_bit());
        },
        PeripheralClock::TIM7 => if enable {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.tim7en().set_bit());
        } else {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.tim7en().clear_bit());
        },
        PeripheralClock::WWDG => if enable {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.wwdgen().set_bit());
        } else {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.wwdgen().clear_bit());
        },
        PeripheralClock::SPI2 => if enable {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.spi2en().set_bit());
        } else {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.spi2en().clear_bit());
        },
        PeripheralClock::SPI3 => if enable {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.spi3en().set_bit());
        } else {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.spi3en().clear_bit());
        },
        PeripheralClock::USART2 => if enable {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.usart2en().set_bit());
        } else {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.usart2en().clear_bit());
        },
        PeripheralClock::USART3 => if enable {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.usart3en().set_bit());
        } else {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.usart3en().clear_bit());
        },
        PeripheralClock::UART4 => if enable {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.uart4en().set_bit());
        } else {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.uart4en().clear_bit());
        },
        PeripheralClock::UART5 => if enable {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.uart5en().set_bit());
        } else {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.uart5en().clear_bit());
        },
        PeripheralClock::I2C1 => if enable {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.i2c1en().set_bit());
        } else {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.i2c1en().clear_bit());
        },
        PeripheralClock::I2C2 => if enable {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.i2c2en().set_bit());
        } else {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.i2c2en().clear_bit());
        },
        PeripheralClock::USB => if enable {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.usben().set_bit());
        } else {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.usben().clear_bit());
        },
        PeripheralClock::CAN1 => if enable {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.canen().set_bit());
        } else {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.canen().clear_bit());
        },
        PeripheralClock::BKP => if enable {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.bkpen().set_bit());
        } else {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.bkpen().clear_bit());
        },
        PeripheralClock::PWR => if enable {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.pwren().set_bit());
        } else {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.pwren().clear_bit());
        },
        PeripheralClock::DAC => if enable {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.dacen().set_bit());
        } else {
            rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.dacen().clear_bit());
        },
    }
}

/// 配置USB时钟
/// 
/// 确保USB时钟为48MHz，这是USB功能正常工作的必要条件
pub fn configure_usb_clock() {
    let rcc = unsafe { &*library::Rcc::ptr() };
    
    // 配置USB时钟为48MHz
    // USB时钟 = PLLCLK / 1.5 = 48MHz（当PLLCLK为72MHz时）
    // 注意：实际的寄存器定义可能没有usbpre方法
    // 这里暂时注释掉，需要根据实际的寄存器定义调整
    // rcc.cfgr().modify(|_, w| {
    //     // 设置USB预分频
    //     unsafe {
    //         // 0: PLLCLK divided by 1.5
    //         // 1: PLLCLK divided by 1
    //         w.bits(w.bits() & !0x10000000 | 0x00000000)
    //     }
    // });
    
    // 使能USB时钟
    set_peripheral_clock(PeripheralClock::USB, true);
}

/// 获取系统复位原因
pub fn get_reset_reason() -> heapless::String<64> {
    let rcc = unsafe { &*library::Rcc::ptr() };
    let csr = rcc.csr().read();
    
    let mut reasons = heapless::Vec::<heapless::String<16>, 6>::new();
    
    if csr.pinrstf().bit_is_set() {
        reasons.push(heapless::String::from("引脚复位")).unwrap();
    }
    if csr.porrstf().bit_is_set() {
        reasons.push(heapless::String::from("掉电复位")).unwrap();
    }
    if csr.sftrstf().bit_is_set() {
        reasons.push(heapless::String::from("软件复位")).unwrap();
    }
    if csr.iwdgrstf().bit_is_set() {
        reasons.push(heapless::String::from("独立看门狗复位")).unwrap();
    }
    if csr.wwdgrstf().bit_is_set() {
        reasons.push(heapless::String::from("窗口看门狗复位")).unwrap();
    }
    if csr.lpwrrstf().bit_is_set() {
        reasons.push(heapless::String::from("低功耗复位")).unwrap();
    }
    
    // 清除复位标志
    rcc.csr().write(|w: &mut library::rcc::csr::W| w.rmvf().set_bit());
    
    if reasons.is_empty() {
        heapless::String::from("未知复位原因")
    } else {
        let mut result = heapless::String::new();
        for (i, reason) in reasons.iter().enumerate() {
            if i > 0 {
                result.push_str("、").unwrap();
            }
            result.push_str(reason.as_str()).unwrap();
        }
        result
    }
}

/// 软件复位系统
pub fn software_reset() {
    let scb = unsafe { &mut *(peripheral::SCB::PTR as *mut peripheral::SCB) };
    unsafe {
        scb.aircr.write(0x05FA_0004);
    }
}

/// 低功耗模式枚举
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum LowPowerMode {
    /// 睡眠模式 - CPU停止，外设继续运行
    Sleep,
    /// 停止模式 - 所有时钟停止，RAM内容保留
    Stop,
    /// 待机模式 - 最低功耗，RAM内容丢失
    Standby,
}

/// 进入低功耗模式
/// 
/// # 参数
/// - `mode`：要进入的低功耗模式
pub fn enter_low_power_mode(mode: LowPowerMode) {
    let scb = unsafe { &mut *(peripheral::SCB::PTR as *mut peripheral::SCB) };
    let pwr = unsafe { &*library::Pwr::ptr() };
    let rcc = unsafe { &*library::Rcc::ptr() };
    
    // 首先使能PWR时钟
    set_peripheral_clock(PeripheralClock::PWR, true);
    
    match mode {
        LowPowerMode::Sleep => {
            // 进入睡眠模式（执行WFI指令）
            unsafe {
                cortex_m::asm::wfi();
            }
        },
        
        LowPowerMode::Stop => {
            // 配置进入停止模式
            // 1. 清除WUF标志
            pwr.cr().modify(|_, w| w.csbf().set_bit());
            
            // 2. 设置PDDS位为0，选择停止模式
            pwr.cr().modify(|_, w| w.pdds().clear_bit());
            
            // 3. 选择电压调节器模式（低功耗）
            pwr.cr().modify(|_, w| w.lpds().set_bit());
            
            // 4. 进入停止模式
            unsafe {
                // 确保所有中断被禁用
                asm!("cpsid i");
                // 执行WFI指令
                asm!("wfi");
                // 重新启用中断
                asm!("cpsie i");
            }
        },
        
        LowPowerMode::Standby => {
            // 配置进入待机模式
            // 1. 清除WUF标志
            pwr.cr().modify(|_, w| w.csbf().set_bit());
            
            // 2. 设置PDDS位为1，选择待机模式
            pwr.cr().modify(|_, w| w.pdds().set_bit());
            
            // 3. 禁用唤醒引脚
            // 注意：实际的寄存器定义可能没有ewup方法
            // 这里暂时注释掉，需要根据实际的寄存器定义调整
            // pwr.cr().modify(|_, w| w.ewup().clear_bit());
            
            // 4. 进入待机模式
            unsafe {
                // 确保所有中断被禁用
                asm!("cpsid i");
                // 执行WFI指令
                asm!("wfi");
            }
        },
    }
}

/// 配置唤醒源
/// 
/// # 参数
/// - `enable_wakeup_pin`：是否启用唤醒引脚
pub fn configure_wakeup_source(enable_wakeup_pin: bool) {
    let pwr = unsafe { &*library::Pwr::ptr() };
    
    // 使能PWR时钟
    set_peripheral_clock(PeripheralClock::PWR, true);
    
    if enable_wakeup_pin {
        // 启用唤醒引脚
        // 注意：实际的寄存器定义可能没有ewup方法
        // 这里暂时注释掉，需要根据实际的寄存器定义调整
        // pwr.cr().modify(|_, w| w.ewup().set_bit());
    } else {
        // 禁用唤醒引脚
        // 注意：实际的寄存器定义可能没有ewup方法
        // 这里暂时注释掉，需要根据实际的寄存器定义调整
        // pwr.cr().modify(|_, w| w.ewup().clear_bit());
    }
}

/// 延时函数（微秒）
/// 
/// 包装delay模块的延时功能，提供安全的延时接口
/// 
/// # 参数
/// - `us`：延时时间，单位：微秒
pub fn delay_us(us: u32) {
    unsafe {
        delay::delay_us(us);
    }
}

/// 延时函数（毫秒）
/// 
/// 包装delay模块的延时功能，提供安全的延时接口
/// 
/// # 参数
/// - `ms`：延时时间，单位：毫秒
pub fn delay_ms(ms: u32) {
    unsafe {
        delay::delay_ms(ms);
    }
}

/// 带超时的等待函数
/// 
/// 包装delay模块的超时等待功能，提供安全的接口
/// 
/// # 参数
/// - `timeout_us`：超时时间，单位：微秒
/// - `condition`：检查条件的闭包
/// 
/// # 返回值
/// - `true`：超时
/// - `false`：条件满足
pub fn wait_with_timeout<F>(timeout_us: u32, condition: F) -> bool
where
    F: Fn() -> bool,
{
    unsafe {
        delay::wait_with_timeout(timeout_us, condition)
    }
}

/// 时钟安全系统(CSS)配置函数
/// 
/// 启用或禁用时钟安全系统，用于监控HSE振荡器的状态
/// 当CSS启用时，如果HSE发生故障，系统会自动切换到HSI并触发中断
/// 
/// # 参数
/// - `enable`：是否启用CSS
pub fn configure_clock_security_system(enable: bool) {
    let rcc = unsafe { &*library::Rcc::ptr() };
    
    if enable {
        // 启用时钟安全系统
        rcc.cr().modify(|_, w| w.csson().set_bit());
    } else {
        // 禁用时钟安全系统
        rcc.cr().modify(|_, w| w.csson().clear_bit());
    }
}

/// 检查时钟安全系统状态
/// 
/// # 返回值
/// - `true`：CSS已启用
/// - `false`：CSS已禁用
pub fn is_clock_security_enabled() -> bool {
    let rcc = unsafe { &*library::Rcc::ptr() };
    rcc.cr().read().csson().bit_is_set()
}

/// 检查HSE故障状态
/// 
/// # 返回值
/// - `true`：HSE发生故障
/// - `false`：HSE正常
pub fn has_hse_failed() -> bool {
    let rcc = unsafe { &*library::Rcc::ptr() };
    rcc.cir().read().cssf().bit_is_set()
}

/// 清除HSE故障标志
pub fn clear_hse_fault_flag() {
    let rcc = unsafe { &*library::Rcc::ptr() };
    rcc.cir().write(|w| w.cssc().set_bit());
}

/// 处理HSE故障
/// 
/// 当HSE发生故障时的处理函数，会清除故障标志并记录故障信息
pub fn handle_hse_fault() {
    // 清除故障标志
    clear_hse_fault_flag();
    
    // 这里可以添加额外的故障处理逻辑，如：
    // 1. 记录故障事件
    // 2. 通知应用层
    // 3. 执行相应的恢复操作
    
    // 注意：当HSE故障时，系统会自动切换到HSI，无需手动切换
}

/// 将u32数字转换为字符串并添加到heapless::String中
/// 
/// # 参数
/// - `s`：目标字符串
/// - `value`：要转换的数字
/// 
/// # 返回值
/// - `Result<(), ()>`：转换结果
fn push_u32_to_string<const N: usize>(s: &mut heapless::String<N>, value: u32) -> Result<(), ()> {
    if value == 0 {
        s.push('0')?;
        return Ok(());
    }
    
    let mut digits = [0u8; 10];
    let mut num = value;
    let mut count = 0;
    
    while num > 0 {
        digits[count] = (num % 10) as u8 + b'0';
        num /= 10;
        count += 1;
    }
    
    for i in (0..count).rev() {
        s.push(digits[i] as char)?;
    }
    
    Ok(())
}

/// 将u16数字转换为字符串并添加到heapless::String中
/// 
/// # 参数
/// - `s`：目标字符串
/// - `value`：要转换的数字
/// 
/// # 返回值
/// - `Result<(), ()>`：转换结果
fn push_u16_to_string<const N: usize>(s: &mut heapless::String<N>, value: u16) -> Result<(), ()> {
    push_u32_to_string(s, value as u32)
}

/// 系统状态监控结构体
pub struct SystemStatus {
    /// 系统运行时间（毫秒）
    pub uptime_ms: u32,
    /// 内部电压参考值（mV）
    pub vrefint_mv: Option<u16>,
    /// 系统时钟频率（Hz）
    pub sysclk: u32,
    /// 系统复位原因
    pub reset_reason: heapless::String<64>,
    /// HSE状态
    pub hse_ready: bool,
    /// PLL状态
    pub pll_ready: bool,
}

impl Default for SystemStatus {
    fn default() -> Self {
        Self {
            uptime_ms: 0,
            vrefint_mv: None,
            sysclk: 0,
            reset_reason: heapless::String::from("未知"),
            hse_ready: false,
            pll_ready: false,
        }
    }
}

/// 获取系统运行时间（毫秒）
/// 
/// 使用SysTick计数器计算系统运行时间
/// 
/// # 返回值
/// 系统运行时间，单位：毫秒
pub fn get_uptime_ms() -> u32 {
    // 注意：这里需要实现一个计数器，在SysTick中断中递增
    // 由于我们当前没有使用SysTick中断，这里返回一个模拟值
    // 实际应用中，应该在SysTick中断处理函数中实现真正的计数器
    
    // 读取当前SysTick值并计算
    unsafe {
        let current_value = core::ptr::read_volatile(0xE000E018 as *const u32);
        let reload_value = core::ptr::read_volatile(0xE000E014 as *const u32);
        
        // 假设系统启动后一直运行，这里返回一个基于当前值的计算
        // 实际应用中应该使用一个全局计数器
        (reload_value - current_value) / ((reload_value + 1) / 1000)
    }
}

/// 读取内部电压参考值
/// 
/// 读取VREFINT通道的ADC值并转换为电压值
/// 
/// # 返回值
/// 内部电压参考值，单位：mV
pub fn read_vrefint() -> Option<u16> {
    // 注意：STM32F103C8T6的ADC需要配置才能读取VREFINT
    // 这里提供一个框架实现，实际使用时需要根据ADC配置进行调整
    
    // 使能ADC1时钟
    set_peripheral_clock(PeripheralClock::ADC1, true);
    
    // 这里应该添加ADC配置和读取代码
    // 由于ADC配置较为复杂，这里返回一个默认值
    // 实际应用中，应该实现完整的ADC配置和读取逻辑
    
    // 禁用ADC1时钟
    set_peripheral_clock(PeripheralClock::ADC1, false);
    
    Some(1200) // 默认返回1.2V（1200mV）
}

/// 获取系统状态信息
/// 
/// 收集并返回系统的各种状态信息
/// 
/// # 返回值
/// 系统状态结构体
pub fn get_system_status() -> SystemStatus {
    let rcc = unsafe { &*library::Rcc::ptr() };
    let clocks = get_system_clocks();
    
    SystemStatus {
        uptime_ms: get_uptime_ms(),
        vrefint_mv: read_vrefint(),
        sysclk: clocks.sysclk,
        reset_reason: get_reset_reason(),
        hse_ready: rcc.cr().read().hserdy().bit_is_set(),
        pll_ready: rcc.cr().read().pllrdy().bit_is_set(),
    }
}

/// 打印系统状态信息
/// 
/// 将系统状态信息格式化为字符串并返回
/// 
/// # 返回值
/// 系统状态信息字符串
pub fn get_system_status_string() -> heapless::String<256> {
    let status = get_system_status();
    let mut result = heapless::String::new();
    
    // 构建状态信息字符串
    result.push_str("=== 系统状态信息 ===\n").unwrap();
    
    // 运行时间
    result.push_str("运行时间: ").unwrap();
    push_u32_to_string(&mut result, status.uptime_ms).unwrap();
    result.push_str(" ms\n").unwrap();
    
    // 系统时钟
    result.push_str("系统时钟: ").unwrap();
    push_u32_to_string(&mut result, status.sysclk).unwrap();
    result.push_str(" Hz\n").unwrap();
    
    // HSE状态
    result.push_str("HSE状态: ").unwrap();
    if status.hse_ready {
        result.push_str("就绪\n").unwrap();
    } else {
        result.push_str("未就绪\n").unwrap();
    }
    
    // PLL状态
    result.push_str("PLL状态: ").unwrap();
    if status.pll_ready {
        result.push_str("就绪\n").unwrap();
    } else {
        result.push_str("未就绪\n").unwrap();
    }
    
    // 内部参考电压
    if let Some(vref) = status.vrefint_mv {
        result.push_str("内部参考电压: ").unwrap();
        push_u16_to_string(&mut result, vref).unwrap();
        result.push_str(" mV\n").unwrap();
    } else {
        result.push_str("内部参考电压: 无法读取\n").unwrap();
    }
    
    // 复位原因
    result.push_str("复位原因: ").unwrap();
    result.push_str(status.reset_reason.as_str()).unwrap();
    result.push_str("\n").unwrap();
    
    result.push_str("===================\n").unwrap();
    
    result
}
