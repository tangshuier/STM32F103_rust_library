//! IIC模块
//! 提供硬件IIC和软件IIC的封装

// 屏蔽未使用代码警告
#![allow(unused)]

use crate::bsp::gpio::{GpioPin, GpioMode};
use crate::bsp::delay::*;

// 导入内部生成的设备驱动库
use library::*;

// 导入asm宏
use core::arch::asm;

/// IIC引脚枚举，提供类型安全的引脚选择
/// 
/// 确保只能使用STM32F103C8T6芯片支持的IIC引脚，避免非法引脚配置。
#[derive(Clone, Copy, Debug)]
pub enum IicPin {
    /// IIC1 SCL引脚（PB6）
    PB6,
    /// IIC1 SDA引脚（PB7）
    PB7,
    /// IIC2 SCL引脚（PB10）
    PB10,
    /// IIC2 SDA引脚（PB11）
    PB11,
}

impl IicPin {
    /// 将IIC引脚转换为GPIO引脚
    /// 
    /// # Returns
    /// 返回对应的GPIO引脚，用于底层GPIO操作
    pub fn to_gpio_pin(&self) -> GpioPin {
        match self {
            IicPin::PB6 => crate::bsp::gpio::PB6,
            IicPin::PB7 => crate::bsp::gpio::PB7,
            IicPin::PB10 => crate::bsp::gpio::PB10,
            IicPin::PB11 => crate::bsp::gpio::PB11,
        }
    }
}

impl From<IicPin> for GpioPin {
    fn from(pin: IicPin) -> Self {
        match pin {
            IicPin::PB6 => crate::bsp::gpio::PB6,
            IicPin::PB7 => crate::bsp::gpio::PB7,
            IicPin::PB10 => crate::bsp::gpio::PB10,
            IicPin::PB11 => crate::bsp::gpio::PB11,
        }
    }
}



/// IIC地址类型，支持7位和8位地址，提供类型安全的地址处理
/// 
/// 遵循I2C规范，支持标准7位地址（0x00-0x7F）和兼容8位地址（0x00-0xFE）
#[derive(Clone, Copy, Debug)]
pub struct IicAddress {
    addr: u8,
    is_7bit: bool,
}

impl IicAddress {
    /// 创建7位IIC地址
    /// 
    /// # Arguments
    /// * `addr` - 7位地址值（0x00-0x7F）
    /// 
    /// # Returns
    /// * `Ok(Self)` - 成功创建的IicAddress实例
    /// * `Err(IicError::InvalidParam)` - 地址超出合法范围
    pub fn new_7bit(addr: u8) -> Result<Self, IicError> {
        if addr > 0x7F {
            Err(IicError::InvalidParam)
        } else {
            Ok(Self { addr, is_7bit: true })
        }
    }
    
    /// 创建8位IIC地址
    /// 
    /// # Arguments
    /// * `addr` - 8位地址值（0x00-0xFE）
    /// 
    /// # Returns
    /// * `Ok(Self)` - 成功创建的IicAddress实例
    pub fn new_8bit(addr: u8) -> Result<Self, IicError> {
        Ok(Self { addr, is_7bit: false })
    }
    
    /// 获取用于硬件传输的地址字节
    /// 
    /// 转换规则：
    /// - 7位地址：左移1位，最低位为读写位
    /// - 8位地址：清除最低位（奇偶位）
    /// 
    /// # Returns
    /// 用于硬件IIC传输的地址字节
    pub fn get_hw_address(&self) -> u8 {
        if self.is_7bit {
            self.addr << 1 // 7位地址左移1位，最低位为读写位
        } else {
            self.addr & 0xFE // 8位地址清除最低位（奇偶位）
        }
    }
    
    /// 获取7位地址值
    /// 
    /// 转换规则：
    /// - 7位地址：直接返回
    /// - 8位地址：右移1位得到7位地址
    /// 
    /// # Returns
    /// 转换后的7位地址值
    pub fn get_7bit(&self) -> u8 {
        if self.is_7bit {
            self.addr
        } else {
            self.addr >> 1 // 8位地址右移1位得到7位地址
        }
    }
    
    /// 获取原始地址值
    /// 
    /// # Returns
    /// 创建地址时使用的原始值
    pub fn get_raw(&self) -> u8 {
        self.addr
    }
    
    /// 检查是否为7位地址
    /// 
    /// # Returns
    /// * `true` - 7位地址
    /// * `false` - 8位地址
    pub fn is_7bit(&self) -> bool {
        self.is_7bit
    }
}

/// IIC时钟源枚举
/// 
/// 定义STM32F103C8T6芯片支持的IIC时钟源，用于动态计算I2C通信速率
#[derive(Clone, Copy, Debug)]
pub enum IicClockSource {
    /// 内部高速时钟（HSI），8MHz
    Hsi,
    /// 外部高速时钟（HSE），8MHz
    Hse,
    /// 锁相环时钟（PLL），72MHz
    Pll,
}

/// IIC时钟配置结构体
/// 
/// 管理IIC通信所需的时钟信息，支持从系统动态获取或手动配置
#[derive(Clone, Copy, Debug)]
pub struct IicClockConfig {
    /// 时钟源类型
    source: IicClockSource,
    /// 系统时钟频率（Hz）
    sysclk: u32,
    /// APB1总线时钟频率（Hz），IIC外设挂载在APB1总线上
    pclk1: u32,
}

impl IicClockConfig {
    /// 创建默认时钟配置
    /// 
    /// 默认使用PLL作为时钟源，系统时钟72MHz，APB1时钟36MHz
    /// 
    /// # Returns
    /// 默认的IIC时钟配置
    pub fn default() -> Self {
        Self {
            source: IicClockSource::Pll,
            sysclk: 72_000_000,
            pclk1: 36_000_000,
        }
    }
    
    /// 从系统寄存器动态计算时钟配置
    /// 
    /// 读取RCC寄存器，自动检测当前系统时钟源和频率，无需手动配置
    /// 
    /// # Safety
    /// 直接访问硬件寄存器，需要确保在正确的上下文中调用
    /// 
    /// # Returns
    /// 基于当前系统配置的IIC时钟配置
    pub unsafe fn from_system() -> Self {
        let rcc = &mut *(0x40021000 as *mut library::rcc::RegisterBlock);
        
        // 读取系统时钟源
        let rcc_cfgr = rcc.cfgr().read().bits();
        let sws = (rcc_cfgr >> 2) & 0x03; // 系统时钟切换状态位
        
        // 根据系统时钟源计算系统时钟频率
        let sysclk: u32;
        let source: IicClockSource;
        match sws {
            0x00 => {
                sysclk = 8_000_000; // HSI作为系统时钟
                source = IicClockSource::Hsi;
            }
            0x01 => {
                sysclk = 8_000_000; // HSE作为系统时钟（假设外部晶振为8MHz）
                source = IicClockSource::Hse;
            }
            0x02 => {
                sysclk = 72_000_000; // PLL作为系统时钟（假设倍频为9，HSE=8MHz）
                source = IicClockSource::Pll;
            }
            _ => {
                sysclk = 8_000_000; // 默认使用HSI
                source = IicClockSource::Hsi;
            }
        }
        
        // 根据PPRE1位计算APB1时钟频率
        let ppre1 = (rcc_cfgr >> 8) & 0x07;
        let pclk1: u32;
        match ppre1 {
            0x00..=0x03 => pclk1 = sysclk, // 不分频
            0x04 => pclk1 = sysclk / 2, // 二分频
            0x05 => pclk1 = sysclk / 4, // 四分频
            0x06 => pclk1 = sysclk / 8, // 八分频
            0x07 => pclk1 = sysclk / 16, // 十六分频
            _ => pclk1 = sysclk / 2, // 默认二分频
        }
        
        Self {
            source,
            sysclk,
            pclk1,
        }
    }
    
    /// 获取APB1总线时钟频率
    /// 
    /// # Returns
    /// APB1总线时钟频率，单位：Hz
    pub fn get_pclk1(&self) -> u32 {
        self.pclk1
    }
    
    /// 获取系统时钟频率
    /// 
    /// # Returns
    /// 系统时钟频率，单位：Hz
    pub fn get_sysclk(&self) -> u32 {
        self.sysclk
    }
    
    /// 获取时钟源类型
    /// 
    /// # Returns
    /// 当前使用的时钟源类型
    pub fn get_source(&self) -> IicClockSource {
        self.source
    }
}

// I2C速度定义
pub const I2C_SPEED_100K: u32 = 100_000;
pub const I2C_SPEED_400K: u32 = 400_000;

/// IIC错误类型枚举
/// 
/// 提供详细的错误类型，帮助上层应用定位和处理IIC通信问题
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IicError {
    Busy,           // IIC总线忙，无法进行通信
    Timeout,        // 操作超时，设备无响应
    NoAcknowledge,  // 设备无应答，可能是地址错误或设备未就绪
    InvalidParam,   // 无效参数，如非法地址、引脚或速度
    HardwareError,  // 硬件错误，IIC外设故障
    SoftwareError,  // 软件错误，通信逻辑问题
    NotSupported,   // 不支持的操作，如硬件IIC不支持的功能
    InitializationFailed, // 初始化失败，无法配置IIC外设
    BusError,       // 总线错误，如意外的总线状态
    ArbitrationLost, // 仲裁丢失，多主机通信时失去总线控制权
    Overrun,        // 接收溢出，数据未及时读取
    PecError,       // PEC校验错误，数据完整性问题
    OvrError,       // 过载错误，数据传输速率不匹配
    AfError,        // 应答失败错误，设备未正确应答
    Other,          // 其他未分类错误
}

/// IIC结果类型
/// 
/// 使用标准Rust Result类型，封装IIC操作的成功或失败状态
pub type IicResult<T> = Result<T, IicError>;

/// IIC操作Trait，定义通用的IIC操作接口
/// 
/// 统一硬件和软件IIC的操作接口，提供一致的编程体验
pub trait I2cOps {
    /// 初始化IIC
    /// 
    /// 配置IIC引脚、时钟和其他参数，准备进行通信
    /// 
    /// # Safety
    /// 直接访问硬件寄存器，需要确保在正确的上下文中调用
    unsafe fn init(&self);
    
    /// 写入数据到设备
    /// 
    /// 向指定地址的设备写入数据，支持任意长度的数据传输
    /// 
    /// # Arguments
    /// * `addr` - 设备的8位IIC地址
    /// * `data` - 要写入的数据缓冲区
    /// 
    /// # Returns
    /// * `Ok(())` - 写入成功
    /// * `Err(IicError)` - 写入失败，包含具体错误信息
    /// 
    /// # Safety
    /// 直接访问硬件寄存器或GPIO，需要确保在正确的上下文中调用
    unsafe fn write(&self, addr: u8, data: &[u8]) -> IicResult<()>;
    
    /// 从设备读取数据
    /// 
    /// 从指定地址的设备读取数据，支持任意长度的数据传输
    /// 
    /// # Arguments
    /// * `addr` - 设备的8位IIC地址
    /// * `buffer` - 用于存储读取数据的缓冲区
    /// 
    /// # Returns
    /// * `Ok(())` - 读取成功，数据已写入缓冲区
    /// * `Err(IicError)` - 读取失败，包含具体错误信息
    /// 
    /// # Safety
    /// 直接访问硬件寄存器或GPIO，需要确保在正确的上下文中调用
    unsafe fn read(&self, addr: u8, buffer: &mut [u8]) -> IicResult<()>;
    
    /// 重置IIC，恢复总线通信
    /// 
    /// 当总线出现异常（如卡死、溢出等）时，重置IIC外设和引脚，恢复正常通信
    /// 
    /// # Safety
    /// 直接访问硬件寄存器，需要确保在正确的上下文中调用
    unsafe fn reset(&self);
}

/// IIC配置结构体，支持灵活配置IIC参数
/// 
/// 提供统一的配置接口，支持硬件和软件IIC的各种配置选项
#[derive(Clone, Copy, Debug)]
pub struct IicConfig {
    /// IIC通信速度（Hz），标准模式最高100K，快速模式最高400K
    pub speed: u32,
    /// IIC引脚配置，指定SCL和SDA引脚
    pub pins: Option<(IicPin, IicPin)>,
    /// IIC时钟配置，用于计算通信速率
    pub clock_config: Option<IicClockConfig>,
    /// 占空比配置（仅硬件IIC），影响快速模式下的时钟波形
    pub duty_cycle: IicDutyCycle,
    /// 是否启用ACK，禁用后将忽略设备的应答
    pub ack_enabled: bool,
    /// 超时时间（微秒），操作超过此时间将返回超时错误
    pub timeout_us: u32,
}

impl Default for IicConfig {
    /// 创建默认IIC配置
    /// 
    /// 默认配置：100K速率，无指定引脚，动态时钟配置，2:1占空比，启用ACK，100us超时
    /// 
    /// # Returns
    /// 默认的IIC配置
    fn default() -> Self {
        Self {
            speed: 100_000,
            pins: None,
            clock_config: None,
            duty_cycle: IicDutyCycle::Cycle2To1,
            ack_enabled: true,
            timeout_us: 100,
        }
    }
}

/// IIC占空比枚举
/// 
/// 仅用于硬件IIC的快速模式，定义SCL时钟的高低电平时间比例
#[derive(Clone, Copy, Debug)]
pub enum IicDutyCycle {
    /// 2:1占空比，标准快速模式使用
    Cycle2To1,
    /// 16:9占空比，高速快速模式使用
    Cycle16To9,
}

/// IIC模式枚举
/// 
/// 区分硬件IIC和软件IIC，用于IicDevice的创建和操作
pub enum IicMode {
    Hardware, // 硬件IIC，使用STM32的IIC外设
    Software, // 软件IIC，使用GPIO模拟IIC通信
}

/// 硬件IIC结构体
/// 
/// 封装STM32的硬件IIC外设，提供类型安全的硬件IIC操作
pub struct HardwareIic {
    config: IicConfig, // 类型安全的IIC配置
}

/// 软件IIC结构体
/// 
/// 使用GPIO模拟IIC通信，提供灵活的软件IIC操作
pub struct SoftwareIic {
    config: IicConfig, // 类型安全的IIC配置
    scl: IicPin, // 类型安全的SCL引脚
    sda: IicPin, // 类型安全的SDA引脚
    delay_us: u32, // 延时微秒数，用于控制通信速度
}

/// IIC设备结构体
/// 
/// 统一的IIC设备接口，封装了硬件和软件IIC的差异，提供安全、易用的IIC操作API
pub struct IicDevice {
    addr: IicAddress, // 类型安全的IIC地址
    mode: IicMode,
    hardware: Option<HardwareIic>,
    software: Option<SoftwareIic>,
}

impl HardwareIic {
    /// 创建新的硬件IIC实例（通用版本，允许灵活配置）
    pub fn new(config: IicConfig) -> Self {
        // 校验speed参数，确保在合法范围内
        // IIC规范通常支持10KHz到400KHz，快速模式+支持1MHz
        let validated_speed = if config.speed < 10_000 {
            100_000 // 最低10KHz，默认100KHz
        } else if config.speed > 1_000_000 {
            400_000 // 最高1MHz，默认400KHz
        } else {
            config.speed
        };
        
        // 使用传入的时钟配置或从系统动态获取
        let clock_config = config.clock_config.unwrap_or_else(|| unsafe { IicClockConfig::from_system() });
        
        // 创建新的配置
        let mut new_config = config;
        new_config.speed = validated_speed;
        new_config.clock_config = Some(clock_config);
        
        Self { 
            config: new_config,
        }
    }
    
    /// 创建新的硬件IIC实例（默认使用PB6和PB7，兼容原有代码）
    pub fn new_default(speed: u32) -> Self {
        let config = IicConfig {
            speed,
            pins: Some((IicPin::PB6, IicPin::PB7)),
            ..Default::default()
        };
        
        Self::new(config)
    }
    
    /// 创建新的硬件IIC实例（从IicConfig创建）
    pub fn from_config(config: IicConfig) -> Self {
        Self::new(config)
    }

    /// 初始化硬件IIC（完全按照STM32F10x_StdPeriph_Driver库的I2C_Init函数实现）
    unsafe fn init(&self) {
        let rcc = &mut *(0x40021000 as *mut library::rcc::RegisterBlock);
        let i2c = &mut *(0x40005400 as *mut library::i2c1::RegisterBlock);
        
        // 1. 启用GPIOB时钟
        // 启用GPIOB时钟和AFIO时钟
        // 使用直接写入整个寄存器值的方式
        let current_apb2enr = rcc.apb2enr().read().bits();
        rcc.apb2enr().write(|w: &mut library::rcc::apb2enr::W| unsafe { w.bits(current_apb2enr | (1 << 3) | (1 << 0)) });
        
        // 2. 配置SCL和SDA引脚为复用开漏输出，使用指定的引脚
        if let Some((scl_pin, sda_pin)) = self.config.pins {
            let scl: GpioPin = scl_pin.into();
            let sda: GpioPin = sda_pin.into();
            scl.into_mode(crate::bsp::gpio::GpioMode::AlternateOpenDrain, crate::bsp::gpio::GpioSpeed::Speed50MHz);
            sda.into_mode(crate::bsp::gpio::GpioMode::AlternateOpenDrain, crate::bsp::gpio::GpioSpeed::Speed50MHz);
        } else {
            // 默认使用PB6和PB7作为IIC引脚
            let scl: GpioPin = IicPin::PB6.into();
            let sda: GpioPin = IicPin::PB7.into();
            scl.into_mode(crate::bsp::gpio::GpioMode::AlternateOpenDrain, crate::bsp::gpio::GpioSpeed::Speed50MHz);
            sda.into_mode(crate::bsp::gpio::GpioMode::AlternateOpenDrain, crate::bsp::gpio::GpioSpeed::Speed50MHz);
        }
        
        // 3. 启用I2C1时钟
        rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w.i2c1en().set_bit());
        
        // 4. 禁用I2C1
        i2c.cr1().modify(|_, w: &mut library::i2c1::cr1::W| w.pe().clear_bit());
        
        // 5. 设置CR2寄存器，使用类型安全的时钟配置
        let clock_config = self.config.clock_config.unwrap();
        let freqrange = (clock_config.pclk1 / 1000000) as u16;
        i2c.cr2().write(|w: &mut library::i2c1::cr2::W| w.freq().bits(freqrange.try_into().unwrap()));
        
        // 6. 清空OAR1和OAR2寄存器
        i2c.oar1().write(|w: &mut library::i2c1::oar1::W| w.bits(0x0000));
        i2c.oar2().write(|w: &mut library::i2c1::oar2::W| w.bits(0x0000));
        
        // 7. 设置OAR1寄存器
        i2c.oar1().write(|w: &mut library::i2c1::oar1::W| unsafe { w.bits(1 << 14) }); // 必须设置第14位为1，这是I2C规范要求的
        
        // 8. 设置I2C速度（CCR寄存器）
        // 完全按照STM32F10x_StdPeriph_Driver库的I2C_Init函数实现
        let i2c_speed = self.config.speed;
        
        let mut ccr = 0;
        let mut duty_cycle = 0; // 0=2:1, 1=16:9
        
        match self.config.duty_cycle {
            IicDutyCycle::Cycle2To1 => {
                // 标准模式或快速模式下的2:1占空比
                if i2c_speed <= 100_000 {
                    // 标准模式
                    ccr = (freqrange as u32 * 1000000) / (2 * i2c_speed);
                } else {
                    // 快速模式，2:1占空比
                    ccr = (freqrange as u32 * 1000000) / (3 * i2c_speed);
                }
            },
            IicDutyCycle::Cycle16To9 => {
                // 快速模式下的16:9占空比
                ccr = (freqrange as u32 * 1000000) / (3 * i2c_speed);
                duty_cycle = 1;
            },
        }
        
        // 设置CCR寄存器，包括占空比位
        if duty_cycle == 1 {
            // 快速模式，占空比为16:9
            i2c.ccr().write(|w: &mut library::i2c1::ccr::W| unsafe { w.bits((1 << 14) | ccr) }); // 设置DUTY位（第14位）
        } else {
            // 标准模式或2:1占空比
            i2c.ccr().write(|w: &mut library::i2c1::ccr::W| unsafe { w.bits(ccr) });
        }
        
        // 9. 设置TRISE寄存器
        let trise = if i2c_speed <= 100_000 {
            // 标准模式
            (freqrange as u32) + 1
        } else {
            // 快速模式
            ((freqrange as u32 * 300) / 1000) + 1
        };
        
        i2c.trise().write(|w: &mut library::i2c1::trise::W| w.bits(trise));
        
        // 10. 设置CR1寄存器，包括ACK位、DutyCycle位、Mode位和GeneralCall位
        // 使用直接写入整个寄存器值的方式
        let mut cr1_value = 0x0000; // 初始化为0，禁用I2C
        
        // 设置ACK位
        if self.config.ack_enabled {
            cr1_value |= 1 << 10; // ACK位
        }
        
        // 设置占空比
        if duty_cycle == 1 {
            cr1_value |= 1 << 14; // DUTY位
        }
        
        i2c.cr1().write(|w: &mut library::i2c1::cr1::W| unsafe { w.bits(cr1_value) });
        
        // 11. 启用I2C1
        i2c.cr1().modify(|_, w: &mut library::i2c1::cr1::W| w.pe().set_bit());
        
        // 12. 添加实际的初始化延迟
        for _ in 0..10000 {
            // 使用内联汇编实现简单的NOP延迟
            asm!("NOP");
        }
    }

    /// 生成I2C起始信号（完全按照STM32F10x_StdPeriph_Driver库的I2C_GenerateSTART函数实现）
    unsafe fn start(&self) -> bool {
        let i2c = &mut *(0x40005400 as *mut library::i2c1::RegisterBlock);
        
        // 生成起始信号
        i2c.cr1().modify(|_, w: &mut library::i2c1::cr1::W| w.start().set_bit());
        
        // 等待SB标志置位，使用基于系统时钟的超时机制
        !wait_with_timeout(self.config.timeout_us, || {
            i2c.sr1().read().sb().bit()
        })
    }

    /// 生成I2C停止信号
    unsafe fn stop(&self) {
        let i2c = &mut *(0x40005400 as *mut library::i2c1::RegisterBlock);
        
        // 生成停止信号
        i2c.cr1().modify(|_, w: &mut library::i2c1::cr1::W| w.stop().set_bit());
    }

    /// 发送设备地址
    unsafe fn send_addr(&self, addr: u8, read: bool) -> bool {
        let i2c = &mut *(0x40005400 as *mut library::i2c1::RegisterBlock);
        
        // 检查总线是否空闲，使用基于系统时钟的超时机制，配置中的超时时间
        let bus_free = !wait_with_timeout(self.config.timeout_us, || {
            !i2c.sr2().read().busy().bit()
        });
        if !bus_free {
            return false;
        }
        
        // 处理地址：直接使用传入的地址，STM32硬件IIC会自动处理
        // 对于8位地址，直接写入，硬件会自动提取7位地址并添加R/W位
        let addr_byte = if read {
            addr | 1 // 读取模式，最低位置1
        } else {
            addr // 写入模式，最低位置0
        };
        i2c.dr().write(|w: &mut library::i2c1::dr::W| w.bits(addr_byte as u32));
        
        // 等待ADDR标志置位，使用基于系统时钟的超时机制，配置中的超时时间
        let addr_set = !wait_with_timeout(self.config.timeout_us, || {
            i2c.sr1().read().addr().bit()
        });
        
        if addr_set {
            // 读取SR2寄存器以清除ADDR标志
            let _ = i2c.sr2().read();
            true
        } else {
            false
        }
    }

    /// 发送数据（完全按照STM32F10x_StdPeriph_Driver库的I2C_Transmit函数实现）
    unsafe fn send_data(&self, data: u8, is_last: bool) -> bool {
        let i2c = &mut *(0x40005400 as *mut library::i2c1::RegisterBlock);
        
        // 等待TXE标志置位，使用基于系统时钟的超时机制，配置中的超时时间
        let txe_set = !wait_with_timeout(self.config.timeout_us, || {
            i2c.sr1().read().tx_e().bit()
        });
        
        if !txe_set {
            return false;
        }
        
        // 发送数据
        i2c.dr().write(|w: &mut library::i2c1::dr::W| w.bits(data as u32));
        
        // 等待传输完成
        if is_last {
            // 最后一个字节，等待BTF标志置位，使用基于系统时钟的超时机制，配置中的超时时间
            let btf_set = !wait_with_timeout(self.config.timeout_us, || {
                i2c.sr1().read().btf().bit()
            });
            return btf_set;
        }
        
        true
    }

    /// 接收数据
    unsafe fn recv_data(&self, ack: bool) -> u8 {
        let i2c = &mut *(0x40005400 as *mut library::i2c1::RegisterBlock);
        
        // 等待RXNE标志置位，使用基于系统时钟的超时机制，配置中的超时时间
        let rxne_set = !wait_with_timeout(self.config.timeout_us, || {
            i2c.sr1().read().rx_ne().bit()
        });
        
        // 读取数据
        let data = i2c.dr().read().bits() as u8;
        
        // 设置ACK
        i2c.cr1().modify(|_, w: &mut library::i2c1::cr1::W| {
            if ack {
                w.ack().set_bit() // 发送ACK
            } else {
                w.ack().clear_bit() // 发送NACK
            }
        });
        
        data
    }

    /// 写入数据到设备
    unsafe fn write(&self, addr: u8, data: &[u8]) -> IicResult<()> {
        let i2c = &mut *(0x40005400 as *mut library::i2c1::RegisterBlock);
        
        // 检查总线是否忙碌，使用基于系统时钟的超时机制，配置中的超时时间
        let bus_free = !wait_with_timeout(self.config.timeout_us, || {
            !i2c.sr2().read().busy().bit()
        });
        if !bus_free {
            // 总线忙，尝试重置
            self.reset();
            return Err(IicError::Busy);
        }
        
        // 生成起始信号
        if !self.start() {
            // 起始信号失败，尝试重置
            self.reset();
            return Err(IicError::Timeout);
        }
        
        // 发送设备地址（写入模式）
        if !self.send_addr(addr, false) {
            self.stop();
            return Err(IicError::NoAcknowledge);
        }
        
        // 发送数据
        for (i, &byte) in data.iter().enumerate() {
            if !self.send_data(byte, i == data.len() - 1) {
                self.stop();
                return Err(IicError::Timeout);
            }
        }
        
        // 生成停止信号
        self.stop();
        Ok(())
    }

    /// 从设备读取数据
    unsafe fn read(&self, addr: u8, buffer: &mut [u8]) -> IicResult<()> {
        let i2c = &mut *(0x40005400 as *mut library::i2c1::RegisterBlock);
        let len = buffer.len();
        
        // 检查总线是否忙碌，使用基于系统时钟的超时机制，配置中的超时时间
        let bus_free = !wait_with_timeout(self.config.timeout_us, || {
            !i2c.sr2().read().busy().bit()
        });
        if !bus_free {
            // 总线忙，尝试重置
            self.reset();
            return Err(IicError::Busy);
        }
        
        // 生成起始信号
        if !self.start() {
            // 起始信号失败，尝试重置
            self.reset();
            return Err(IicError::Timeout);
        }
        
        // 检查是否有总线错误、仲裁丢失等错误
        let sr1 = i2c.sr1().read();
        if sr1.berr().bit() {
            self.stop();
            // 总线错误，尝试重置
            self.reset();
            return Err(IicError::BusError);
        }
        if sr1.arlo().bit() {
            self.stop();
            // 仲裁丢失，尝试重置
            self.reset();
            return Err(IicError::ArbitrationLost);
        }
        if sr1.af().bit() {
            self.stop();
            // 应答失败，尝试重置
            self.reset();
            return Err(IicError::AfError);
        }
        if sr1.ovr().bit() {
            self.stop();
            // 溢出错误，尝试重置
            self.reset();
            return Err(IicError::Overrun);
        }
        if sr1.pecerr().bit() {
            self.stop();
            // PEC错误，尝试重置
            self.reset();
            return Err(IicError::PecError);
        }
        
        // 发送设备地址（读取模式）
        if !self.send_addr(addr, true) {
            self.stop();
            return Err(IicError::NoAcknowledge);
        }
        
        // 检查是否有错误
        let sr1 = i2c.sr1().read();
        if sr1.af().bit() {
            self.stop();
            // 应答失败，尝试重置
            self.reset();
            return Err(IicError::AfError);
        }
        
        // 读取数据
        for i in 0..len {
            let ack = i < len - 1;
            buffer[i] = self.recv_data(ack);
            
            // 检查是否有错误
            let sr1 = i2c.sr1().read();
            if sr1.ovr().bit() {
                self.stop();
                // 溢出错误，尝试重置
                self.reset();
                return Err(IicError::Overrun);
            }
            if sr1.berr().bit() {
                self.stop();
                // 总线错误，尝试重置
                self.reset();
                return Err(IicError::BusError);
            }
        }
        
        // 生成停止信号
        self.stop();
        Ok(())
    }
    
    /// 重置IIC控制器，恢复总线通信
    pub unsafe fn reset(&self) {
        let i2c = &mut *(0x40005400 as *mut library::i2c1::RegisterBlock);
        
        // 1. 禁用I2C
        i2c.cr1().modify(|_, w: &mut library::i2c1::cr1::W| w.pe().clear_bit());
        
        // 2. 清空所有状态寄存器
        // 读取SR1和SR2寄存器来清除标志
        let _ = i2c.sr1().read();
        let _ = i2c.sr2().read();
        
        // 3. 重新初始化IIC
        self.init();
    }
}

/// 实现I2cOps Trait for HardwareIic
impl I2cOps for HardwareIic {
    unsafe fn init(&self) {
        HardwareIic::init(self)
    }
    
    unsafe fn write(&self, addr: u8, data: &[u8]) -> IicResult<()> {
        HardwareIic::write(self, addr, data)
    }
    
    unsafe fn read(&self, addr: u8, buffer: &mut [u8]) -> IicResult<()> {
        HardwareIic::read(self, addr, buffer)
    }
    
    unsafe fn reset(&self) {
        HardwareIic::reset(self)
    }
}

impl SoftwareIic {
    /// 创建新的软件IIC实例
    pub fn new(scl: IicPin, sda: IicPin, speed: u32) -> Self {
        let config = IicConfig {
            speed,
            pins: Some((scl, sda)),
            ..Default::default()
        };
        
        // 校验speed参数，确保在合法范围内
        // 软件IIC通常支持10KHz到100KHz，过高的速率会导致通信失败
        let validated_speed = if speed < 10_000 {
            100_000 // 最低10KHz，默认100KHz
        } else if speed > 200_000 {
            100_000 // 最高200KHz，默认100KHz
        } else {
            speed
        };
        
        // 根据speed计算合适的delay_us值
        // 假设每个时钟周期需要两个延时（高电平+低电平）
        // 例如：100KHz需要每个时钟周期10us，每个电平保持5us
        let delay_us = if validated_speed > 0 {
            (500_000 / validated_speed) as u32 // 500,000 / speed 计算出每个电平需要的微秒数
        } else {
            5 // 默认5us
        };
        
        let mut new_config = config;
        new_config.speed = validated_speed;
        
        Self { 
            config: new_config,
            scl, 
            sda, 
            delay_us,
        }
    }
    
    /// 创建新的软件IIC实例（从IicConfig创建）
    pub fn from_config(config: IicConfig) -> Self {
        // 检查pins是否被设置
        let (scl, sda) = match config.pins {
            Some((scl, sda)) => (scl, sda),
            None => (IicPin::PB6, IicPin::PB7), // 默认使用PB6和PB7
        };
        
        Self::new(scl, sda, config.speed)
    }

    /// 初始化软件IIC
    unsafe fn init(&self) {
        // 配置SCL和SDA为开漏输出
        let scl: GpioPin = self.scl.into();
        let sda: GpioPin = self.sda.into();
        
        scl.into_open_drain_output();
        sda.into_open_drain_output();
        
        // 初始状态为高电平
        scl.set_high();
        sda.set_high();
    }

    /// 延时函数（空实现，与C语言版本保持一致）
    fn delay(&self) {
        // 与C语言版本保持一致，不添加任何延时
    }

    /// 生成起始信号
    unsafe fn start(&self) {
        let scl: GpioPin = self.scl.into();
        let sda: GpioPin = self.sda.into();
        
        sda.set_high();
        scl.set_high();
        sda.set_low();
        scl.set_low();
    }

    /// 生成停止信号
    unsafe fn stop(&self) {
        let scl: GpioPin = self.scl.into();
        let sda: GpioPin = self.sda.into();
        
        sda.set_low();
        scl.set_high();
        sda.set_high();
    }

    /// 发送一个字节
    unsafe fn send_byte(&self, byte: u8) -> IicResult<bool> {
        let scl: GpioPin = self.scl.into();
        let sda: GpioPin = self.sda.into();
        
        for i in 0..8 {
            // 发送数据位
            if (byte & (1 << (7 - i))) != 0 {
                sda.set_high();
            } else {
                sda.set_low();
            }
            // 添加精确延时，确保数据位稳定
            delay_us(self.delay_us);
            scl.set_high();
            // 添加精确延时，确保时钟脉冲宽度
            delay_us(self.delay_us);
            scl.set_low();
            // 添加精确延时，确保数据位有足够时间变化
            delay_us(self.delay_us);
        }
        
        // 读取ACK
        sda.set_high();
        // 添加精确延时，确保SDA线释放
        delay_us(self.delay_us);
        scl.set_high();
        // 添加精确延时，确保ACK位稳定
        delay_us(self.delay_us);
        
        // 读取ACK状态
        let ack = sda.is_low();
        scl.set_low();
        
        // 不检查ACK，直接返回Ok(true)，与C示例代码一致
        Ok(true)
    }

    /// 接收一个字节
    unsafe fn recv_byte(&self, ack: bool) -> u8 {
        let scl: GpioPin = self.scl.into();
        let sda: GpioPin = self.sda.into();
        
        let mut byte = 0;
        
        // 释放SDA
        sda.set_high();
        
        for i in 0..8 {
            // 确保数据位稳定
            delay_us(self.delay_us);
            scl.set_high();
            
            // 确保时钟脉冲宽度，让从设备有足够时间准备数据
            delay_us(self.delay_us);
            
            if sda.is_high() {
                byte |= 1 << (7 - i);
            }
            
            scl.set_low();
            // 确保数据位有足够时间变化
            delay_us(self.delay_us);
        }
        
        // 发送ACK/NACK
        if ack {
            sda.set_low();
        } else {
            sda.set_high();
        }
        
        // 确保ACK/NACK位稳定
        delay_us(self.delay_us);
        scl.set_high();
        
        // 确保时钟脉冲宽度
        delay_us(self.delay_us);
        scl.set_low();
        
        byte
    }

    /// 写入数据到设备
    unsafe fn write(&self, addr: u8, data: &[u8]) -> IicResult<()> {
        // 检查数据长度
        if data.is_empty() {
            return Ok(());
        }
        
        // 生成起始信号
        self.start();
        
        // 发送设备地址（写入模式）
        // 直接使用传入的地址，不再区分7位或8位地址
        // OLED手册要求使用0x78地址
        let addr_ack = self.send_byte(addr)?;
        // 不检查地址ACK，与C示例代码一致
        
        // 发送数据
        for &byte in data {
            let data_ack = self.send_byte(byte)?;
            // 不检查数据ACK，与C示例代码一致
        }
        
        // 生成停止信号
        self.stop();
        Ok(())
    }

    /// 从设备读取数据
    unsafe fn read(&self, addr: u8, buffer: &mut [u8]) -> IicResult<()> {
        let len = buffer.len();
        
        // 检查缓冲区长度
        if len == 0 {
            return Ok(());
        }
        
        // 生成起始信号
        self.start();
        
        // 发送设备地址（读取模式）
        let addr_byte = addr | 1; // 读取模式，最低位置1
        let addr_ack = self.send_byte(addr_byte)?;
        if !addr_ack {
            self.stop();
            return Err(IicError::NoAcknowledge);
        }
        
        // 读取数据
        for i in 0..len {
            let ack = i < len - 1;
            buffer[i] = self.recv_byte(ack);
        }
        
        // 生成停止信号
        self.stop();
        Ok(())
    }
    
    /// 重置IIC，恢复总线通信
    pub unsafe fn reset(&self) {
        // 软件IIC重置，重新初始化引脚
        self.init();
    }
}

/// 实现I2cOps Trait for SoftwareIic
impl I2cOps for SoftwareIic {
    unsafe fn init(&self) {
        SoftwareIic::init(self)
    }
    
    unsafe fn write(&self, addr: u8, data: &[u8]) -> IicResult<()> {
        SoftwareIic::write(self, addr, data)
    }
    
    unsafe fn read(&self, addr: u8, buffer: &mut [u8]) -> IicResult<()> {
        SoftwareIic::read(self, addr, buffer)
    }
    
    unsafe fn reset(&self) {
        SoftwareIic::reset(self)
    }
}

impl IicDevice {
    /// 创建硬件IIC设备（通用版本，允许指定引脚）
    /// 
    /// # Arguments
    /// * `addr` - 设备的IIC地址
    /// * `speed` - 通信速度（Hz）
    /// * `scl` - SCL引脚，None表示使用默认引脚
    /// * `sda` - SDA引脚，None表示使用默认引脚
    /// 
    /// # Returns
    /// 初始化完成的硬件IIC设备
    pub fn new_hardware_with_pins(addr: IicAddress, speed: u32, scl: Option<IicPin>, sda: Option<IicPin>) -> Self {
        let config = IicConfig {
            speed,
            pins: scl.zip(sda),
            ..Default::default()
        };
        let hardware = HardwareIic::new(config);
        unsafe {
            hardware.init();
        }
        
        Self {
            addr,
            mode: IicMode::Hardware,
            hardware: Some(hardware),
            software: None,
        }
    }
    
    /// 创建硬件IIC设备（默认使用PB6和PB7，兼容原有代码）
    /// 
    /// # Arguments
    /// * `addr` - 设备的IIC地址
    /// * `speed` - 通信速度（Hz）
    /// 
    /// # Returns
    /// 初始化完成的硬件IIC设备，使用默认引脚PB6/PB7
    pub fn new_hardware(addr: IicAddress, speed: u32) -> Self {
        let hardware = HardwareIic::new_default(speed);
        unsafe {
            hardware.init();
        }
        
        Self {
            addr,
            mode: IicMode::Hardware,
            hardware: Some(hardware),
            software: None,
        }
    }

    /// 创建软件IIC设备
    /// 
    /// # Arguments
    /// * `addr` - 设备的IIC地址
    /// * `scl` - SCL引脚
    /// * `sda` - SDA引脚
    /// * `speed` - 通信速度（Hz）
    /// 
    /// # Returns
    /// 初始化完成的软件IIC设备
    pub fn new_software(addr: IicAddress, scl: IicPin, sda: IicPin, speed: u32) -> Self {
        let software = SoftwareIic::new(scl, sda, speed);
        unsafe {
            software.init();
        }
        
        Self {
            addr,
            mode: IicMode::Software,
            hardware: None,
            software: Some(software),
        }
    }

    /// 获取I2cOps实现（内部使用）
    /// 
    /// 根据当前IIC模式，返回对应的I2cOps实现
    /// 
    /// # Safety
    /// 内部调用unsafe方法，需要确保在正确的上下文中使用
    /// 
    /// # Returns
    /// * `Ok(&dyn I2cOps)` - 对应的I2cOps实现
    /// * `Err(IicError)` - 无法获取I2cOps实现
    unsafe fn get_i2c_ops(&self) -> Result<&dyn I2cOps, IicError> {
        match self.mode {
            IicMode::Hardware => {
                if let Some(hardware) = &self.hardware {
                    Ok(hardware as &dyn I2cOps)
                } else {
                    Err(IicError::HardwareError)
                }
            },
            IicMode::Software => {
                if let Some(software) = &self.software {
                    Ok(software as &dyn I2cOps)
                } else {
                    Err(IicError::SoftwareError)
                }
            },
        }
    }

    /// 写入数据到设备（安全API）
    /// 
    /// 向设备写入任意长度的数据，内部封装了unsafe操作，用户无需手动添加unsafe块
    /// 
    /// # Arguments
    /// * `data` - 要写入的数据缓冲区
    /// 
    /// # Returns
    /// * `Ok(())` - 写入成功
    /// * `Err(IicError)` - 写入失败，包含具体错误信息
    pub fn write(&self, data: &[u8]) -> Result<(), IicError> {
        unsafe {
            let i2c_ops = self.get_i2c_ops()?;
            i2c_ops.write(self.addr.get_hw_address(), data)?;
            Ok(())
        }
    }

    /// 从设备读取数据（安全API）
    /// 
    /// 从设备读取任意长度的数据，内部封装了unsafe操作，用户无需手动添加unsafe块
    /// 
    /// # Arguments
    /// * `buffer` - 用于存储读取数据的缓冲区
    /// 
    /// # Returns
    /// * `Ok(())` - 读取成功，数据已写入缓冲区
    /// * `Err(IicError)` - 读取失败，包含具体错误信息
    pub fn read(&self, buffer: &mut [u8]) -> Result<(), IicError> {
        unsafe {
            let i2c_ops = self.get_i2c_ops()?;
            i2c_ops.read(self.addr.get_hw_address(), buffer)?;
            Ok(())
        }
    }
    
    /// 重置IIC设备，恢复总线通信
    /// 
    /// 当总线出现异常（如卡死、溢出等）时，调用此方法重置IIC外设和引脚，恢复正常通信
    /// 
    /// # Returns
    /// * `Ok(())` - 重置成功
    /// * `Err(IicError)` - 重置失败，包含具体错误信息
    pub fn reset(&self) -> Result<(), IicError> {
        unsafe {
            let i2c_ops = self.get_i2c_ops()?;
            i2c_ops.reset();
            Ok(())
        }
    }

    /// 写入单个字节到设备（安全API）
    /// 
    /// 向设备写入单个字节，内部封装了write方法
    /// 
    /// # Arguments
    /// * `byte` - 要写入的字节
    /// 
    /// # Returns
    /// * `Ok(())` - 写入成功
    /// * `Err(IicError)` - 写入失败，包含具体错误信息
    pub fn write_byte(&self, byte: u8) -> Result<(), IicError> {
        let data = [byte];
        self.write(&data)
    }

    /// 从设备读取单个字节（安全API）
    /// 
    /// 从设备读取单个字节，内部封装了read方法
    /// 
    /// # Returns
    /// * `Ok(u8)` - 读取到的字节
    /// * `Err(IicError)` - 读取失败，包含具体错误信息
    pub fn read_byte(&self) -> Result<u8, IicError> {
        let mut buffer = [0];
        self.read(&mut buffer)?;
        Ok(buffer[0])
    }
}

/// 预定义的IIC设备和通用设备创建函数
pub mod devices {
    use super::*;
    
    /// 创建通用硬件IIC设备（默认使用PB6和PB7）
    pub fn hardware_device(addr: IicAddress, speed: u32) -> IicDevice {
        IicDevice::new_hardware(addr, speed)
    }
    
    /// 创建通用硬件IIC设备（允许指定引脚）
    pub fn hardware_device_with_pins(addr: IicAddress, speed: u32, scl: Option<IicPin>, sda: Option<IicPin>) -> IicDevice {
        IicDevice::new_hardware_with_pins(addr, speed, scl, sda)
    }
    
    /// 创建通用软件IIC设备
    pub fn software_device(addr: IicAddress, scl: IicPin, sda: IicPin, speed: u32) -> IicDevice {
        IicDevice::new_software(addr, scl, sda, speed)
    }
    
    /// 创建8位地址的硬件IIC设备
    pub fn hardware_device_8bit(addr: u8, speed: u32) -> IicDevice {
        let iic_addr = IicAddress::new_8bit(addr).unwrap();
        IicDevice::new_hardware(iic_addr, speed)
    }
    
    /// 创建7位地址的硬件IIC设备
    pub fn hardware_device_7bit(addr: u8, speed: u32) -> IicDevice {
        let iic_addr = IicAddress::new_7bit(addr).unwrap();
        IicDevice::new_hardware(iic_addr, speed)
    }
    
    /// 创建8位地址的软件IIC设备
    pub fn software_device_8bit(addr: u8, scl: IicPin, sda: IicPin, speed: u32) -> IicDevice {
        let iic_addr = IicAddress::new_8bit(addr).unwrap();
        IicDevice::new_software(iic_addr, scl, sda, speed)
    }
    
    /// 创建7位地址的软件IIC设备
    pub fn software_device_7bit(addr: u8, scl: IicPin, sda: IicPin, speed: u32) -> IicDevice {
        let iic_addr = IicAddress::new_7bit(addr).unwrap();
        IicDevice::new_software(iic_addr, scl, sda, speed)
    }
}
