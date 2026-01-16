//! GPIO模块
//! 提供GPIO引脚的封装和操作

// 屏蔽未使用代码警告
#![allow(unused)]

// 使用内部生成的设备驱动库
use library::*;
use core::marker::PhantomData;
use core::fmt::Debug;

/// GPIO速度枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GpioSpeed {
    Speed10MHz,   // 10MHz
    Speed2MHz,    // 2MHz
    Speed50MHz,   // 50MHz
}

/// GPIO模式枚举（向后兼容）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GpioMode {
    FloatingInput,
    PullUpInput,
    PullDownInput,
    AnalogInput,
    PushPullOutput,
    OpenDrainOutput,
    AlternatePushPull,
    AlternateOpenDrain,
}

/// 推挽类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PushPullType {
    PushPull,
    OpenDrain,
}

/// 上拉下拉类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PullType {
    Floating,
    PullUp,
    PullDown,
    Analog,
}

/// 引脚状态类型
pub trait PinMode: Sized {}

/// 浮动输入
pub struct Floating;
impl PinMode for Floating {}

/// 上拉输入
pub struct PullUp;
impl PinMode for PullUp {}

/// 下拉输入
pub struct PullDown;
impl PinMode for PullDown {}

/// 模拟输入
pub struct Analog;
impl PinMode for Analog {}

/// 推挽输出
pub struct PushPull;
impl PinMode for PushPull {}

/// 开漏输出
pub struct OpenDrain;
impl PinMode for OpenDrain {}

/// 复用推挽输出
pub struct AlternatePushPull;
impl PinMode for AlternatePushPull {}

/// 复用开漏输出
pub struct AlternateOpenDrain;
impl PinMode for AlternateOpenDrain {}

/// GPIO引脚结构体（向后兼容）
pub struct GpioPin {
    port: GpioPort,
    pin: u8,
}

impl GpioPin {
    /// 转换为指定模式
    /// # Safety
    /// - 调用者必须确保相应GPIO端口时钟已启用
    /// - 调用者必须确保引脚未被其他代码或外设占用
    /// - 某些模式转换可能需要重新配置相关外设
    pub unsafe fn into_mode(&self, mode: GpioMode, speed: GpioSpeed) {
        // 这里可以实现简单的模式转换，实际上会被type state pattern替代
    }
}

/// GPIO端口枚举
#[derive(Debug, Clone, Copy)]
pub enum GpioPort {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

/// GPIO端口结构体（向后兼容）
#[derive(Debug, Clone, Copy)]
pub struct GpioPortStruct {
    pub port: GpioPort,
    pub pin: u8,
}

/// GPIO初始化配置结构体（类似标准库的GPIO_InitTypeDef）
pub struct GpioInitConfig {
    pub pin: u16,
    pub speed: GpioSpeed,
    pub mode: GpioMode,
}

/// GPIO端口结构体（用于批量操作）
#[derive(Debug, Clone, Copy)]
pub struct GpioPortBatch {
    pub port: GpioPort,
}

/// 为GpioPortStruct实现向后兼容的方法
impl GpioPortStruct {
    /// 转换为推挽输出
    /// # Safety
    /// - 调用者必须确保相应GPIO端口时钟已启用
    /// - 调用者必须确保引脚未被其他代码或外设占用
    pub unsafe fn into_push_pull_output(self) {
        // 这里实现简单的向后兼容，实际使用类型状态模式
        let port_ptr = match self.port {
            GpioPort::A => 0x4001_0800 as *mut u32,
            GpioPort::B => 0x4001_0C00 as *mut u32,
            GpioPort::C => 0x4001_1000 as *mut u32,
            GpioPort::D => 0x4001_1400 as *mut u32,
            GpioPort::E => 0x4001_1800 as *mut u32,
            GpioPort::F => 0x4001_1C00 as *mut u32,
            GpioPort::G => 0x4001_2000 as *mut u32,
        };
        
        // 使能时钟
        let rcc_ptr = 0x4002_1000 as *mut u32;
        let apb2enr = (rcc_ptr as usize + 0x18) as *mut u32; // APB2ENR寄存器
        let clock_bit = 1 << (2 + self.port as u32);
        *apb2enr |= clock_bit;
        
        // 配置为推挽输出
        let cr_offset = if self.pin < 8 { 0x00 } else { 0x04 };
        let pin_pos = self.pin % 8;
        let cr_ptr = (port_ptr as usize + cr_offset) as *mut u32;
        
        let pin_mask = 0x0F << (pin_pos * 4);
        let config = 0b0011; // CNF=00, MODE=11 (50MHz)
        
        let mut value = *cr_ptr;
        value = (value & !pin_mask) | (config << (pin_pos * 4));
        *cr_ptr = value;
    }
    
    /// 转换为复用推挽输出
    /// # Safety
    /// - 调用者必须确保相应GPIO端口时钟已启用
    /// - 调用者必须确保引脚未被其他代码或外设占用
    /// - 调用者必须确保已正确配置相关外设的复用功能
    pub unsafe fn into_alternate_push_pull(self) {
        let port_ptr = match self.port {
            GpioPort::A => 0x4001_0800 as *mut u32,
            GpioPort::B => 0x4001_0C00 as *mut u32,
            GpioPort::C => 0x4001_1000 as *mut u32,
            GpioPort::D => 0x4001_1400 as *mut u32,
            GpioPort::E => 0x4001_1800 as *mut u32,
            GpioPort::F => 0x4001_1C00 as *mut u32,
            GpioPort::G => 0x4001_2000 as *mut u32,
        };
        
        // 使能时钟
        let rcc_ptr = 0x4002_1000 as *mut u32;
        let apb2enr = (rcc_ptr as usize + 0x18) as *mut u32; // APB2ENR寄存器
        let clock_bit = 1 << (2 + self.port as u32);
        *apb2enr |= clock_bit;
        
        // 配置为复用推挽输出
        let cr_offset = if self.pin < 8 { 0x00 } else { 0x04 };
        let pin_pos = self.pin % 8;
        let cr_ptr = (port_ptr as usize + cr_offset) as *mut u32;
        
        let pin_mask = 0x0F << (pin_pos * 4);
        let config = 0b1011; // CNF=10, MODE=11 (50MHz)
        
        let mut value = *cr_ptr;
        value = (value & !pin_mask) | (config << (pin_pos * 4));
        *cr_ptr = value;
    }
    
    /// 转换为浮动输入
    /// # Safety
    /// - 调用者必须确保相应GPIO端口时钟已启用
    /// - 调用者必须确保引脚未被其他代码或外设占用
    pub unsafe fn into_floating_input(self) {
        let port_ptr = match self.port {
            GpioPort::A => 0x4001_0800 as *mut u32,
            GpioPort::B => 0x4001_0C00 as *mut u32,
            GpioPort::C => 0x4001_1000 as *mut u32,
            GpioPort::D => 0x4001_1400 as *mut u32,
            GpioPort::E => 0x4001_1800 as *mut u32,
            GpioPort::F => 0x4001_1C00 as *mut u32,
            GpioPort::G => 0x4001_2000 as *mut u32,
        };
        
        // 使能时钟
        let rcc_ptr = 0x4002_1000 as *mut u32;
        let apb2enr = (rcc_ptr as usize + 0x18) as *mut u32; // APB2ENR寄存器
        let clock_bit = 1 << (2 + self.port as u32);
        *apb2enr |= clock_bit;
        
        // 配置为浮动输入
        let cr_offset = if self.pin < 8 { 0x00 } else { 0x04 };
        let pin_pos = self.pin % 8;
        let cr_ptr = (port_ptr as usize + cr_offset) as *mut u32;
        
        let pin_mask = 0x0F << (pin_pos * 4);
        let config = 0b0100; // CNF=01, MODE=00
        
        let mut value = *cr_ptr;
        value = (value & !pin_mask) | (config << (pin_pos * 4));
        *cr_ptr = value;
    }
    
    /// 设置引脚为高电平
    /// # Safety
    /// - 调用者必须确保引脚已被配置为输出模式
    /// - 调用者必须确保引脚未被其他代码或外设占用
    pub unsafe fn set_high(self) {
        let port_ptr = match self.port {
            GpioPort::A => 0x4001_0800 as *mut u32,
            GpioPort::B => 0x4001_0C00 as *mut u32,
            GpioPort::C => 0x4001_1000 as *mut u32,
            GpioPort::D => 0x4001_1400 as *mut u32,
            GpioPort::E => 0x4001_1800 as *mut u32,
            GpioPort::F => 0x4001_1C00 as *mut u32,
            GpioPort::G => 0x4001_2000 as *mut u32,
        };
        
        let bsrr = (port_ptr as usize + 0x10) as *mut u32; // BSRR寄存器
        *bsrr = 1 << self.pin;
    }
    
    /// 设置引脚为低电平
    /// # Safety
    /// - 调用者必须确保引脚已被配置为输出模式
    /// - 调用者必须确保引脚未被其他代码或外设占用
    pub unsafe fn set_low(self) {
        let port_ptr = match self.port {
            GpioPort::A => 0x4001_0800 as *mut u32,
            GpioPort::B => 0x4001_0C00 as *mut u32,
            GpioPort::C => 0x4001_1000 as *mut u32,
            GpioPort::D => 0x4001_1400 as *mut u32,
            GpioPort::E => 0x4001_1800 as *mut u32,
            GpioPort::F => 0x4001_1C00 as *mut u32,
            GpioPort::G => 0x4001_2000 as *mut u32,
        };
        
        let brr = (port_ptr as usize + 0x14) as *mut u32; // BRR寄存器
        *brr = 1 << self.pin;
    }
}

/// 为GpioPortBatch实现批量操作方法
impl GpioPortBatch {
    /// 创建新的GpioPortBatch实例
    pub const fn new(port: GpioPort) -> Self {
        Self {
            port,
        }
    }
    
    /// 读取整个端口的输入数据
    /// # Safety
    /// - 调用者必须确保相应GPIO端口时钟已启用
    pub unsafe fn read_input_data(&self) -> u16 {
        let port_ptr = match self.port {
            GpioPort::A => 0x4001_0800 as *mut u32,
            GpioPort::B => 0x4001_0C00 as *mut u32,
            GpioPort::C => 0x4001_1000 as *mut u32,
            GpioPort::D => 0x4001_1400 as *mut u32,
            GpioPort::E => 0x4001_1800 as *mut u32,
            GpioPort::F => 0x4001_1C00 as *mut u32,
            GpioPort::G => 0x4001_2000 as *mut u32,
        };
        
        let idr = (port_ptr as usize + 0x08) as *const u32; // IDR寄存器
        (*idr & 0xFFFF) as u16
    }
    
    /// 读取整个端口的输出数据
    /// # Safety
    /// - 调用者必须确保相应GPIO端口时钟已启用
    pub unsafe fn read_output_data(&self) -> u16 {
        let port_ptr = match self.port {
            GpioPort::A => 0x4001_0800 as *mut u32,
            GpioPort::B => 0x4001_0C00 as *mut u32,
            GpioPort::C => 0x4001_1000 as *mut u32,
            GpioPort::D => 0x4001_1400 as *mut u32,
            GpioPort::E => 0x4001_1800 as *mut u32,
            GpioPort::F => 0x4001_1C00 as *mut u32,
            GpioPort::G => 0x4001_2000 as *mut u32,
        };
        
        let odr = (port_ptr as usize + 0x0C) as *const u32; // ODR寄存器
        (*odr & 0xFFFF) as u16
    }
    
    /// 写入整个端口的输出数据
    /// # Safety
    /// - 调用者必须确保相应GPIO端口时钟已启用
    /// - 调用者必须确保端口引脚已被配置为输出模式
    /// - 调用者必须确保写入操作不会影响其他关键功能
    pub unsafe fn write(&self, data: u16) {
        let port_ptr = match self.port {
            GpioPort::A => 0x4001_0800 as *mut u32,
            GpioPort::B => 0x4001_0C00 as *mut u32,
            GpioPort::C => 0x4001_1000 as *mut u32,
            GpioPort::D => 0x4001_1400 as *mut u32,
            GpioPort::E => 0x4001_1800 as *mut u32,
            GpioPort::F => 0x4001_1C00 as *mut u32,
            GpioPort::G => 0x4001_2000 as *mut u32,
        };
        
        let odr = (port_ptr as usize + 0x0C) as *mut u32; // ODR寄存器
        *odr = data as u32;
    }
    
    /// 批量设置引脚为高电平
    /// # Safety
    /// - 调用者必须确保相应GPIO端口时钟已启用
    /// - 调用者必须确保指定的引脚已被配置为输出模式
    /// - 调用者必须确保操作不会影响其他关键功能
    pub unsafe fn set_bits(&self, pins: u16) {
        let port_ptr = match self.port {
            GpioPort::A => 0x4001_0800 as *mut u32,
            GpioPort::B => 0x4001_0C00 as *mut u32,
            GpioPort::C => 0x4001_1000 as *mut u32,
            GpioPort::D => 0x4001_1400 as *mut u32,
            GpioPort::E => 0x4001_1800 as *mut u32,
            GpioPort::F => 0x4001_1C00 as *mut u32,
            GpioPort::G => 0x4001_2000 as *mut u32,
        };
        
        let bsrr = (port_ptr as usize + 0x10) as *mut u32; // BSRR寄存器
        *bsrr = pins as u32;
    }
    
    /// 批量设置引脚为低电平
    /// # Safety
    /// - 调用者必须确保相应GPIO端口时钟已启用
    /// - 调用者必须确保指定的引脚已被配置为输出模式
    /// - 调用者必须确保操作不会影响其他关键功能
    pub unsafe fn reset_bits(&self, pins: u16) {
        let port_ptr = match self.port {
            GpioPort::A => 0x4001_0800 as *mut u32,
            GpioPort::B => 0x4001_0C00 as *mut u32,
            GpioPort::C => 0x4001_1000 as *mut u32,
            GpioPort::D => 0x4001_1400 as *mut u32,
            GpioPort::E => 0x4001_1800 as *mut u32,
            GpioPort::F => 0x4001_1C00 as *mut u32,
            GpioPort::G => 0x4001_2000 as *mut u32,
        };
        
        let brr = (port_ptr as usize + 0x14) as *mut u32; // BRR寄存器
        *brr = pins as u32;
    }
    
    /// 锁定引脚配置，防止意外修改
    /// # Safety
    /// - 调用者必须确保相应GPIO端口时钟已启用
    /// - 锁定后无法修改引脚配置，直到下一次系统复位
    pub unsafe fn pin_lock_config(&self, pins: u16) {
        let port_ptr = match self.port {
            GpioPort::A => 0x4001_0800 as *mut u32,
            GpioPort::B => 0x4001_0C00 as *mut u32,
            GpioPort::C => 0x4001_1000 as *mut u32,
            GpioPort::D => 0x4001_1400 as *mut u32,
            GpioPort::E => 0x4001_1800 as *mut u32,
            GpioPort::F => 0x4001_1C00 as *mut u32,
            GpioPort::G => 0x4001_2000 as *mut u32,
        };
        
        let lckr = (port_ptr as usize + 0x18) as *mut u32; // LCKR寄存器
        
        // 锁定序列
        *lckr = 0x0001_0000 | pins as u32;
        *lckr = pins as u32; // 写入锁定引脚
        *lckr = 0x0001_0000 | pins as u32; // 再次写入
        let _ = *lckr; // 读取确认
        let _ = *lckr; // 再次读取确认
    }
}

/// GPIO引脚结构体
#[derive(Debug, Clone, Copy)]
pub struct Pin<P: GpioPortType, M: PinMode> {
    port: P,
    pin: u8,
    _mode: PhantomData<M>,
}

/// GPIO端口类型标记
pub trait GpioPortType: Debug {
    const PORT: GpioPort;
    type Periph: Deref<Target = gpioa::RegisterBlock> + 'static;
}

/// 为Gpioa实现GpioPortType
impl GpioPortType for Gpioa {
    const PORT: GpioPort = GpioPort::A;
    type Periph = Gpioa;
}

/// 为Gpiob实现GpioPortType
impl GpioPortType for Gpiob {
    const PORT: GpioPort = GpioPort::B;
    type Periph = Gpiob;
}

/// 为Gpioc实现GpioPortType
impl GpioPortType for Gpioc {
    const PORT: GpioPort = GpioPort::C;
    type Periph = Gpioc;
}

/// 为Gpiod实现GpioPortType
impl GpioPortType for Gpiod {
    const PORT: GpioPort = GpioPort::D;
    type Periph = Gpiod;
}

/// 为Gpioe实现GpioPortType
impl GpioPortType for Gpioe {
    const PORT: GpioPort = GpioPort::E;
    type Periph = Gpioe;
}

/// 为Gpiof实现GpioPortType
impl GpioPortType for Gpiof {
    const PORT: GpioPort = GpioPort::F;
    type Periph = Gpiof;
}

/// 为Gpiog实现GpioPortType
impl GpioPortType for Gpiog {
    const PORT: GpioPort = GpioPort::G;
    type Periph = Gpiog;
}

/// 从core::ops导入Deref
use core::ops::Deref;

/// 为Pin实现基础方法
impl<P: GpioPortType, M: PinMode> Pin<P, M> {
    /// 创建新的Pin实例
    pub const unsafe fn new(port: P, pin: u8) -> Self {
        Self {
            port,
            pin,
            _mode: PhantomData,
        }
    }
    
    /// 获取端口实例
    pub unsafe fn get_port(&self) -> &'static P::Periph {
        match P::PORT {
            GpioPort::A => &*(0x4001_0800 as *const P::Periph),
            GpioPort::B => &*(0x4001_0C00 as *const P::Periph),
            GpioPort::C => &*(0x4001_1000 as *const P::Periph),
            GpioPort::D => &*(0x4001_1400 as *const P::Periph),
            GpioPort::E => &*(0x4001_1800 as *const P::Periph),
            GpioPort::F => &*(0x4001_1C00 as *const P::Periph),
            GpioPort::G => &*(0x4001_2000 as *const P::Periph),
        }
    }
    
    /// 获取端口时钟使能位
    fn clock_en_bit(&self) -> u32 {
        match P::PORT {
            GpioPort::A => 1 << 2,
            GpioPort::B => 1 << 3,
            GpioPort::C => 1 << 4,
            GpioPort::D => 1 << 5,
            GpioPort::E => 1 << 6,
            GpioPort::F => 1 << 7,
            GpioPort::G => 1 << 8,
        }
    }
    
    /// 启用端口时钟
    unsafe fn enable_clock(&self) {
        let rcc = &mut *(0x4002_1000 as *mut rcc::RegisterBlock);
        let mut value = rcc.apb2enr().read().bits();
        value |= self.clock_en_bit();
        rcc.apb2enr().write(|w| unsafe { w.bits(value) });
    }
    
    /// 配置引脚为浮动输入
    unsafe fn configure_floating(&self) {
        self.enable_clock();
        
        let port = self.get_port();
        let pin_pos = self.pin % 8;
        
        let pin_mask = 0x0F << (pin_pos * 4);
        let config = 0b0100; // CNF=01, MODE=00
        
        // 设置配置寄存器
        if self.pin < 8 {
            let mut value = port.crl().read().bits();
            value = (value & !pin_mask) | (config << (pin_pos * 4));
            port.crl().write(|w| unsafe { w.bits(value) });
        } else {
            let mut value = port.crh().read().bits();
            value = (value & !pin_mask) | (config << (pin_pos * 4));
            port.crh().write(|w| unsafe { w.bits(value) });
        }
    }
    
    /// 配置引脚为上拉输入
    unsafe fn configure_pull_up(&self) {
        self.enable_clock();
        
        let port = self.get_port();
        let pin_pos = self.pin % 8;
        
        let pin_mask = 0x0F << (pin_pos * 4);
        let config = 0b1000; // CNF=10, MODE=00
        
        // 设置配置寄存器
        if self.pin < 8 {
            let mut value = port.crl().read().bits();
            value = (value & !pin_mask) | (config << (pin_pos * 4));
            port.crl().write(|w| unsafe { w.bits(value) });
        } else {
            let mut value = port.crh().read().bits();
            value = (value & !pin_mask) | (config << (pin_pos * 4));
            port.crh().write(|w| unsafe { w.bits(value) });
        }
        
        // 上拉
        let mut odr_value = port.odr().read().bits();
        odr_value |= (1 << self.pin);
        port.odr().write(|w| unsafe { w.bits(odr_value) });
    }
    
    /// 配置引脚为下拉输入
    unsafe fn configure_pull_down(&self) {
        self.enable_clock();
        
        let port = self.get_port();
        let pin_pos = self.pin % 8;
        
        let pin_mask = 0x0F << (pin_pos * 4);
        let config = 0b1000; // CNF=10, MODE=00
        
        // 设置配置寄存器
        if self.pin < 8 {
            let mut value = port.crl().read().bits();
            value = (value & !pin_mask) | (config << (pin_pos * 4));
            port.crl().write(|w| unsafe { w.bits(value) });
        } else {
            let mut value = port.crh().read().bits();
            value = (value & !pin_mask) | (config << (pin_pos * 4));
            port.crh().write(|w| unsafe { w.bits(value) });
        }
        
        // 下拉
        let mut odr_value = port.odr().read().bits();
        odr_value &= !(1 << self.pin);
        port.odr().write(|w| unsafe { w.bits(odr_value) });
    }
    
    /// 配置引脚为模拟输入
    unsafe fn configure_analog(&self) {
        self.enable_clock();
        
        let port = self.get_port();
        let pin_pos = self.pin % 8;
        
        let pin_mask = 0x0F << (pin_pos * 4);
        let config = 0b0000; // CNF=00, MODE=00
        
        // 设置配置寄存器
        if self.pin < 8 {
            let mut value = port.crl().read().bits();
            value = (value & !pin_mask) | (config << (pin_pos * 4));
            port.crl().write(|w| unsafe { w.bits(value) });
        } else {
            let mut value = port.crh().read().bits();
            value = (value & !pin_mask) | (config << (pin_pos * 4));
            port.crh().write(|w| unsafe { w.bits(value) });
        }
    }
    
    /// 配置引脚为推挽输出
    unsafe fn configure_push_pull_output(&self, speed: GpioSpeed) {
        self.enable_clock();
        
        let port = self.get_port();
        let pin_pos = self.pin % 8;
        
        let mode_bits = match speed {
            GpioSpeed::Speed10MHz => 0b01,
            GpioSpeed::Speed2MHz => 0b10,
            GpioSpeed::Speed50MHz => 0b11,
        };
        
        let pin_mask = 0x0F << (pin_pos * 4);
        let config = 0b0000 | mode_bits; // CNF=00, MODE=xx
        
        // 设置配置寄存器
        if self.pin < 8 {
            let mut value = port.crl().read().bits();
            value = (value & !pin_mask) | (config << (pin_pos * 4));
            port.crl().write(|w| unsafe { w.bits(value) });
        } else {
            let mut value = port.crh().read().bits();
            value = (value & !pin_mask) | (config << (pin_pos * 4));
            port.crh().write(|w| unsafe { w.bits(value) });
        }
    }
    
    /// 配置引脚为开漏输出
    unsafe fn configure_open_drain_output(&self, speed: GpioSpeed) {
        self.enable_clock();
        
        let port = self.get_port();
        let pin_pos = self.pin % 8;
        
        let mode_bits = match speed {
            GpioSpeed::Speed10MHz => 0b01,
            GpioSpeed::Speed2MHz => 0b10,
            GpioSpeed::Speed50MHz => 0b11,
        };
        
        let pin_mask = 0x0F << (pin_pos * 4);
        let config = 0b0100 | mode_bits; // CNF=01, MODE=xx
        
        // 设置配置寄存器
        if self.pin < 8 {
            let mut value = port.crl().read().bits();
            value = (value & !pin_mask) | (config << (pin_pos * 4));
            port.crl().write(|w| unsafe { w.bits(value) });
        } else {
            let mut value = port.crh().read().bits();
            value = (value & !pin_mask) | (config << (pin_pos * 4));
            port.crh().write(|w| unsafe { w.bits(value) });
        }
    }
}

/// 输入模式的通用方法
macro_rules! impl_input_methods {
    ($($mode:ty),*) => {
        $(impl<P: GpioPortType> Pin<P, $mode> {
            /// 读取引脚输入状态（高电平返回true）
            pub unsafe fn is_high(&self) -> bool {
                let port = self.get_port();
                (port.idr().read().bits() & (1 << self.pin)) != 0
            }
            
            /// 读取引脚输入状态（低电平返回true）
            pub unsafe fn is_low(&self) -> bool {
                !self.is_high()
            }
            
            /// 获取引脚输入值（0或1）
            pub unsafe fn get_input(&self) -> u8 {
                if self.is_high() {
                    1
                } else {
                    0
                }
            }
            
            /// 读取引脚输入状态（与is_high相同，更直观的命名）
            pub unsafe fn read_input(&self) -> bool {
                self.is_high()
            }
            
            /// 读取引脚输入状态（返回u8，0或1）
            pub unsafe fn read_input_raw(&self) -> u8 {
                self.get_input()
            }
        })*
    };
}

// 为所有输入模式实现通用方法
impl_input_methods!(Floating, PullUp, PullDown, Analog);

/// 浮动输入模式扩展
impl<P: GpioPortType> Pin<P, Floating> {
    /// 转换为上拉输入
    pub unsafe fn into_pull_up(self) -> Pin<P, PullUp> {
        self.configure_pull_up();
        Pin {
            port: self.port,
            pin: self.pin,
            _mode: PhantomData,
        }
    }
    
    /// 转换为下拉输入
    pub unsafe fn into_pull_down(self) -> Pin<P, PullDown> {
        self.configure_pull_down();
        Pin {
            port: self.port,
            pin: self.pin,
            _mode: PhantomData,
        }
    }
    
    /// 转换为模拟输入
    pub unsafe fn into_analog(self) -> Pin<P, Analog> {
        self.configure_analog();
        Pin {
            port: self.port,
            pin: self.pin,
            _mode: PhantomData,
        }
    }
    
    /// 转换为推挽输出
    pub unsafe fn into_push_pull_output(self, speed: GpioSpeed) -> Pin<P, PushPull> {
        self.configure_push_pull_output(speed);
        Pin {
            port: self.port,
            pin: self.pin,
            _mode: PhantomData,
        }
    }
    
    /// 转换为开漏输出
    pub unsafe fn into_open_drain_output(self, speed: GpioSpeed) -> Pin<P, OpenDrain> {
        self.configure_open_drain_output(speed);
        Pin {
            port: self.port,
            pin: self.pin,
            _mode: PhantomData,
        }
    }
    
    /// 转换为复用推挽输出
    pub unsafe fn into_alternate_push_pull(self, speed: GpioSpeed) -> Pin<P, AlternatePushPull> {
        // 复用推挽输出配置与推挽输出相同
        self.configure_push_pull_output(speed);
        Pin {
            port: self.port,
            pin: self.pin,
            _mode: PhantomData,
        }
    }
    
    /// 转换为复用开漏输出
    pub unsafe fn into_alternate_open_drain(self, speed: GpioSpeed) -> Pin<P, AlternateOpenDrain> {
        // 复用开漏输出配置与开漏输出相同
        self.configure_open_drain_output(speed);
        Pin {
            port: self.port,
            pin: self.pin,
            _mode: PhantomData,
        }
    }
}

/// 输出模式的通用方法
macro_rules! impl_output_methods {
    ($($mode:ty),*) => {
        $(impl<P: GpioPortType> Pin<P, $mode> {
            /// 设置引脚为高电平
            pub unsafe fn set_high(&mut self) {
                let port = self.get_port();
                port.bsrr().write(|w| unsafe { w.bits(1 << self.pin) });
            }
            
            /// 设置引脚为低电平
            pub unsafe fn set_low(&mut self) {
                let port = self.get_port();
                port.brr().write(|w| unsafe { w.bits(1 << self.pin) });
            }
            
            /// 切换引脚状态
            pub unsafe fn toggle(&mut self) {
                let port = self.get_port();
                let current = port.odr().read().bits();
                port.odr().write(|w| unsafe { w.bits(current ^ (1 << self.pin)) });
            }
            
            /// 获取引脚输出状态（高电平返回true）
            pub unsafe fn is_high(&self) -> bool {
                let port = self.get_port();
                (port.odr().read().bits() & (1 << self.pin)) != 0
            }
            
            /// 获取引脚输出状态（低电平返回true）
            pub unsafe fn is_low(&self) -> bool {
                !self.is_high()
            }
            
            /// 获取引脚输出状态（高电平返回true）
            pub unsafe fn get_output(&self) -> bool {
                self.is_high()
            }
            
            /// 读取引脚输出状态（与is_high相同，更直观的命名）
            pub unsafe fn read_output(&self) -> bool {
                self.is_high()
            }
            
            /// 读取引脚输出状态（返回u8，0或1）
            pub unsafe fn read_output_raw(&self) -> u8 {
                if self.is_high() {
                    1
                } else {
                    0
                }
            }
            
            /// 设置引脚的输出状态
            pub unsafe fn write_bit(&mut self, bit_val: bool) {
                if bit_val {
                    self.set_high();
                } else {
                    self.set_low();
                }
            }
            
            /// 直接写入引脚状态（1或0）
            pub unsafe fn write_raw(&mut self, value: u8) {
                if value != 0 {
                    self.set_high();
                } else {
                    self.set_low();
                }
            }
        })*
    };
}

// 为所有输出模式实现通用方法
impl_output_methods!(PushPull, OpenDrain, AlternatePushPull, AlternateOpenDrain);

/// 预定义的GPIO引脚常量
pub mod pins {
    use super::*;
    
    // 端口A引脚
    /// # Safety
    /// - 调用者必须确保GPIOA外设时钟已启用
    /// - 调用者必须确保引脚未被其他代码或外设占用
    /// - 此函数会修改寄存器状态，可能影响其他使用同一端口的代码
    pub unsafe fn pa0() -> Pin<Gpioa, Floating> {
        let pin = Pin::new(Gpioa::steal(), 0);
        pin.configure_floating();
        pin
    }
    
    /// # Safety
    /// - 调用者必须确保GPIOA外设时钟已启用
    /// - 调用者必须确保引脚未被其他代码或外设占用
    /// - 此函数会修改寄存器状态，可能影响其他使用同一端口的代码
    pub unsafe fn pa1() -> Pin<Gpioa, Floating> {
        let pin = Pin::new(Gpioa::steal(), 1);
        pin.configure_floating();
        pin
    }
    
    /// # Safety
    /// - 调用者必须确保GPIOA外设时钟已启用
    /// - 调用者必须确保引脚未被其他代码或外设占用
    /// - 此函数会修改寄存器状态，可能影响其他使用同一端口的代码
    pub unsafe fn pa2() -> Pin<Gpioa, Floating> {
        let pin = Pin::new(Gpioa::steal(), 2);
        pin.configure_floating();
        pin
    }
    
    /// # Safety
    /// - 调用者必须确保GPIOA外设时钟已启用
    /// - 调用者必须确保引脚未被其他代码或外设占用
    /// - 此函数会修改寄存器状态，可能影响其他使用同一端口的代码
    pub unsafe fn pa3() -> Pin<Gpioa, Floating> {
        let pin = Pin::new(Gpioa::steal(), 3);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pa4() -> Pin<Gpioa, Floating> {
        let pin = Pin::new(Gpioa::steal(), 4);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pa5() -> Pin<Gpioa, Floating> {
        let pin = Pin::new(Gpioa::steal(), 5);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pa6() -> Pin<Gpioa, Floating> {
        let pin = Pin::new(Gpioa::steal(), 6);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pa7() -> Pin<Gpioa, Floating> {
        let pin = Pin::new(Gpioa::steal(), 7);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pa8() -> Pin<Gpioa, Floating> {
        let pin = Pin::new(Gpioa::steal(), 8);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pa9() -> Pin<Gpioa, Floating> {
        let pin = Pin::new(Gpioa::steal(), 9);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pa10() -> Pin<Gpioa, Floating> {
        let pin = Pin::new(Gpioa::steal(), 10);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pa11() -> Pin<Gpioa, Floating> {
        let pin = Pin::new(Gpioa::steal(), 11);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pa12() -> Pin<Gpioa, Floating> {
        let pin = Pin::new(Gpioa::steal(), 12);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pa13() -> Pin<Gpioa, Floating> {
        let pin = Pin::new(Gpioa::steal(), 13);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pa14() -> Pin<Gpioa, Floating> {
        let pin = Pin::new(Gpioa::steal(), 14);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pa15() -> Pin<Gpioa, Floating> {
        let pin = Pin::new(Gpioa::steal(), 15);
        pin.configure_floating();
        pin
    }
    
    // 端口B引脚
    pub unsafe fn pb0() -> Pin<Gpiob, Floating> {
        let pin = Pin::new(Gpiob::steal(), 0);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pb1() -> Pin<Gpiob, Floating> {
        let pin = Pin::new(Gpiob::steal(), 1);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pb2() -> Pin<Gpiob, Floating> {
        let pin = Pin::new(Gpiob::steal(), 2);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pb3() -> Pin<Gpiob, Floating> {
        let pin = Pin::new(Gpiob::steal(), 3);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pb4() -> Pin<Gpiob, Floating> {
        let pin = Pin::new(Gpiob::steal(), 4);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pb5() -> Pin<Gpiob, Floating> {
        let pin = Pin::new(Gpiob::steal(), 5);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pb6() -> Pin<Gpiob, Floating> {
        let pin = Pin::new(Gpiob::steal(), 6);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pb7() -> Pin<Gpiob, Floating> {
        let pin = Pin::new(Gpiob::steal(), 7);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pb8() -> Pin<Gpiob, Floating> {
        let pin = Pin::new(Gpiob::steal(), 8);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pb9() -> Pin<Gpiob, Floating> {
        let pin = Pin::new(Gpiob::steal(), 9);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pb10() -> Pin<Gpiob, Floating> {
        let pin = Pin::new(Gpiob::steal(), 10);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pb11() -> Pin<Gpiob, Floating> {
        let pin = Pin::new(Gpiob::steal(), 11);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pb12() -> Pin<Gpiob, Floating> {
        let pin = Pin::new(Gpiob::steal(), 12);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pb13() -> Pin<Gpiob, Floating> {
        let pin = Pin::new(Gpiob::steal(), 13);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pb14() -> Pin<Gpiob, Floating> {
        let pin = Pin::new(Gpiob::steal(), 14);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pb15() -> Pin<Gpiob, Floating> {
        let pin = Pin::new(Gpiob::steal(), 15);
        pin.configure_floating();
        pin
    }
    
    // 端口C引脚
    pub unsafe fn pc0() -> Pin<Gpioc, Floating> {
        let pin = Pin::new(Gpioc::steal(), 0);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pc1() -> Pin<Gpioc, Floating> {
        let pin = Pin::new(Gpioc::steal(), 1);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pc2() -> Pin<Gpioc, Floating> {
        let pin = Pin::new(Gpioc::steal(), 2);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pc3() -> Pin<Gpioc, Floating> {
        let pin = Pin::new(Gpioc::steal(), 3);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pc4() -> Pin<Gpioc, Floating> {
        let pin = Pin::new(Gpioc::steal(), 4);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pc5() -> Pin<Gpioc, Floating> {
        let pin = Pin::new(Gpioc::steal(), 5);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pc6() -> Pin<Gpioc, Floating> {
        let pin = Pin::new(Gpioc::steal(), 6);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pc7() -> Pin<Gpioc, Floating> {
        let pin = Pin::new(Gpioc::steal(), 7);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pc8() -> Pin<Gpioc, Floating> {
        let pin = Pin::new(Gpioc::steal(), 8);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pc9() -> Pin<Gpioc, Floating> {
        let pin = Pin::new(Gpioc::steal(), 9);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pc10() -> Pin<Gpioc, Floating> {
        let pin = Pin::new(Gpioc::steal(), 10);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pc11() -> Pin<Gpioc, Floating> {
        let pin = Pin::new(Gpioc::steal(), 11);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pc12() -> Pin<Gpioc, Floating> {
        let pin = Pin::new(Gpioc::steal(), 12);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pc13() -> Pin<Gpioc, Floating> {
        let pin = Pin::new(Gpioc::steal(), 13);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pc14() -> Pin<Gpioc, Floating> {
        let pin = Pin::new(Gpioc::steal(), 14);
        pin.configure_floating();
        pin
    }
    
    pub unsafe fn pc15() -> Pin<Gpioc, Floating> {
        let pin = Pin::new(Gpioc::steal(), 15);
        pin.configure_floating();
        pin
    }
}

/// 向后兼容的GPIO引脚常量
// 端口A引脚
pub const PA0: GpioPortStruct = GpioPortStruct { port: GpioPort::A, pin: 0 };
pub const PA1: GpioPortStruct = GpioPortStruct { port: GpioPort::A, pin: 1 };
pub const PA2: GpioPortStruct = GpioPortStruct { port: GpioPort::A, pin: 2 };
pub const PA3: GpioPortStruct = GpioPortStruct { port: GpioPort::A, pin: 3 };
pub const PA4: GpioPortStruct = GpioPortStruct { port: GpioPort::A, pin: 4 };
pub const PA5: GpioPortStruct = GpioPortStruct { port: GpioPort::A, pin: 5 };
pub const PA6: GpioPortStruct = GpioPortStruct { port: GpioPort::A, pin: 6 };
pub const PA7: GpioPortStruct = GpioPortStruct { port: GpioPort::A, pin: 7 };
pub const PA8: GpioPortStruct = GpioPortStruct { port: GpioPort::A, pin: 8 };
pub const PA9: GpioPortStruct = GpioPortStruct { port: GpioPort::A, pin: 9 };
pub const PA10: GpioPortStruct = GpioPortStruct { port: GpioPort::A, pin: 10 };
pub const PA11: GpioPortStruct = GpioPortStruct { port: GpioPort::A, pin: 11 };
pub const PA12: GpioPortStruct = GpioPortStruct { port: GpioPort::A, pin: 12 };
pub const PA13: GpioPortStruct = GpioPortStruct { port: GpioPort::A, pin: 13 };
pub const PA14: GpioPortStruct = GpioPortStruct { port: GpioPort::A, pin: 14 };
pub const PA15: GpioPortStruct = GpioPortStruct { port: GpioPort::A, pin: 15 };

// 端口B引脚
pub const PB0: GpioPortStruct = GpioPortStruct { port: GpioPort::B, pin: 0 };
pub const PB1: GpioPortStruct = GpioPortStruct { port: GpioPort::B, pin: 1 };
pub const PB2: GpioPortStruct = GpioPortStruct { port: GpioPort::B, pin: 2 };
pub const PB3: GpioPortStruct = GpioPortStruct { port: GpioPort::B, pin: 3 };
pub const PB4: GpioPortStruct = GpioPortStruct { port: GpioPort::B, pin: 4 };
pub const PB5: GpioPortStruct = GpioPortStruct { port: GpioPort::B, pin: 5 };
pub const PB6: GpioPortStruct = GpioPortStruct { port: GpioPort::B, pin: 6 };
pub const PB7: GpioPortStruct = GpioPortStruct { port: GpioPort::B, pin: 7 };
pub const PB8: GpioPortStruct = GpioPortStruct { port: GpioPort::B, pin: 8 };
pub const PB9: GpioPortStruct = GpioPortStruct { port: GpioPort::B, pin: 9 };
pub const PB10: GpioPortStruct = GpioPortStruct { port: GpioPort::B, pin: 10 };
pub const PB11: GpioPortStruct = GpioPortStruct { port: GpioPort::B, pin: 11 };
pub const PB12: GpioPortStruct = GpioPortStruct { port: GpioPort::B, pin: 12 };
pub const PB13: GpioPortStruct = GpioPortStruct { port: GpioPort::B, pin: 13 };
pub const PB14: GpioPortStruct = GpioPortStruct { port: GpioPort::B, pin: 14 };
pub const PB15: GpioPortStruct = GpioPortStruct { port: GpioPort::B, pin: 15 };

// 端口C引脚
pub const PC0: GpioPortStruct = GpioPortStruct { port: GpioPort::C, pin: 0 };
pub const PC1: GpioPortStruct = GpioPortStruct { port: GpioPort::C, pin: 1 };
pub const PC2: GpioPortStruct = GpioPortStruct { port: GpioPort::C, pin: 2 };
pub const PC3: GpioPortStruct = GpioPortStruct { port: GpioPort::C, pin: 3 };
pub const PC4: GpioPortStruct = GpioPortStruct { port: GpioPort::C, pin: 4 };
pub const PC5: GpioPortStruct = GpioPortStruct { port: GpioPort::C, pin: 5 };
pub const PC6: GpioPortStruct = GpioPortStruct { port: GpioPort::C, pin: 6 };
pub const PC7: GpioPortStruct = GpioPortStruct { port: GpioPort::C, pin: 7 };
pub const PC8: GpioPortStruct = GpioPortStruct { port: GpioPort::C, pin: 8 };
pub const PC9: GpioPortStruct = GpioPortStruct { port: GpioPort::C, pin: 9 };
pub const PC10: GpioPortStruct = GpioPortStruct { port: GpioPort::C, pin: 10 };
pub const PC11: GpioPortStruct = GpioPortStruct { port: GpioPort::C, pin: 11 };
pub const PC12: GpioPortStruct = GpioPortStruct { port: GpioPort::C, pin: 12 };
pub const PC13: GpioPortStruct = GpioPortStruct { port: GpioPort::C, pin: 13 };
pub const PC14: GpioPortStruct = GpioPortStruct { port: GpioPort::C, pin: 14 };
pub const PC15: GpioPortStruct = GpioPortStruct { port: GpioPort::C, pin: 15 };

// 端口D引脚
pub const PD0: GpioPortStruct = GpioPortStruct { port: GpioPort::D, pin: 0 };
pub const PD1: GpioPortStruct = GpioPortStruct { port: GpioPort::D, pin: 1 };
pub const PD2: GpioPortStruct = GpioPortStruct { port: GpioPort::D, pin: 2 };
pub const PD3: GpioPortStruct = GpioPortStruct { port: GpioPort::D, pin: 3 };
pub const PD4: GpioPortStruct = GpioPortStruct { port: GpioPort::D, pin: 4 };
pub const PD5: GpioPortStruct = GpioPortStruct { port: GpioPort::D, pin: 5 };
pub const PD6: GpioPortStruct = GpioPortStruct { port: GpioPort::D, pin: 6 };
pub const PD7: GpioPortStruct = GpioPortStruct { port: GpioPort::D, pin: 7 };
pub const PD8: GpioPortStruct = GpioPortStruct { port: GpioPort::D, pin: 8 };
pub const PD9: GpioPortStruct = GpioPortStruct { port: GpioPort::D, pin: 9 };
pub const PD10: GpioPortStruct = GpioPortStruct { port: GpioPort::D, pin: 10 };
pub const PD11: GpioPortStruct = GpioPortStruct { port: GpioPort::D, pin: 11 };
pub const PD12: GpioPortStruct = GpioPortStruct { port: GpioPort::D, pin: 12 };
pub const PD13: GpioPortStruct = GpioPortStruct { port: GpioPort::D, pin: 13 };
pub const PD14: GpioPortStruct = GpioPortStruct { port: GpioPort::D, pin: 14 };
pub const PD15: GpioPortStruct = GpioPortStruct { port: GpioPort::D, pin: 15 };

// 端口E引脚
pub const PE0: GpioPortStruct = GpioPortStruct { port: GpioPort::E, pin: 0 };
pub const PE1: GpioPortStruct = GpioPortStruct { port: GpioPort::E, pin: 1 };
pub const PE2: GpioPortStruct = GpioPortStruct { port: GpioPort::E, pin: 2 };
pub const PE3: GpioPortStruct = GpioPortStruct { port: GpioPort::E, pin: 3 };
pub const PE4: GpioPortStruct = GpioPortStruct { port: GpioPort::E, pin: 4 };
pub const PE5: GpioPortStruct = GpioPortStruct { port: GpioPort::E, pin: 5 };
pub const PE6: GpioPortStruct = GpioPortStruct { port: GpioPort::E, pin: 6 };
pub const PE7: GpioPortStruct = GpioPortStruct { port: GpioPort::E, pin: 7 };
pub const PE8: GpioPortStruct = GpioPortStruct { port: GpioPort::E, pin: 8 };
pub const PE9: GpioPortStruct = GpioPortStruct { port: GpioPort::E, pin: 9 };
pub const PE10: GpioPortStruct = GpioPortStruct { port: GpioPort::E, pin: 10 };
pub const PE11: GpioPortStruct = GpioPortStruct { port: GpioPort::E, pin: 11 };
pub const PE12: GpioPortStruct = GpioPortStruct { port: GpioPort::E, pin: 12 };
pub const PE13: GpioPortStruct = GpioPortStruct { port: GpioPort::E, pin: 13 };
pub const PE14: GpioPortStruct = GpioPortStruct { port: GpioPort::E, pin: 14 };
pub const PE15: GpioPortStruct = GpioPortStruct { port: GpioPort::E, pin: 15 };

// 端口F引脚
pub const PF0: GpioPortStruct = GpioPortStruct { port: GpioPort::F, pin: 0 };
pub const PF1: GpioPortStruct = GpioPortStruct { port: GpioPort::F, pin: 1 };
pub const PF2: GpioPortStruct = GpioPortStruct { port: GpioPort::F, pin: 2 };
pub const PF3: GpioPortStruct = GpioPortStruct { port: GpioPort::F, pin: 3 };
pub const PF4: GpioPortStruct = GpioPortStruct { port: GpioPort::F, pin: 4 };
pub const PF5: GpioPortStruct = GpioPortStruct { port: GpioPort::F, pin: 5 };
pub const PF6: GpioPortStruct = GpioPortStruct { port: GpioPort::F, pin: 6 };
pub const PF7: GpioPortStruct = GpioPortStruct { port: GpioPort::F, pin: 7 };
pub const PF8: GpioPortStruct = GpioPortStruct { port: GpioPort::F, pin: 8 };
pub const PF9: GpioPortStruct = GpioPortStruct { port: GpioPort::F, pin: 9 };
pub const PF10: GpioPortStruct = GpioPortStruct { port: GpioPort::F, pin: 10 };
pub const PF11: GpioPortStruct = GpioPortStruct { port: GpioPort::F, pin: 11 };
pub const PF12: GpioPortStruct = GpioPortStruct { port: GpioPort::F, pin: 12 };
pub const PF13: GpioPortStruct = GpioPortStruct { port: GpioPort::F, pin: 13 };
pub const PF14: GpioPortStruct = GpioPortStruct { port: GpioPort::F, pin: 14 };
pub const PF15: GpioPortStruct = GpioPortStruct { port: GpioPort::F, pin: 15 };

// 端口G引脚
pub const PG0: GpioPortStruct = GpioPortStruct { port: GpioPort::G, pin: 0 };
pub const PG1: GpioPortStruct = GpioPortStruct { port: GpioPort::G, pin: 1 };
pub const PG2: GpioPortStruct = GpioPortStruct { port: GpioPort::G, pin: 2 };
pub const PG3: GpioPortStruct = GpioPortStruct { port: GpioPort::G, pin: 3 };
pub const PG4: GpioPortStruct = GpioPortStruct { port: GpioPort::G, pin: 4 };
pub const PG5: GpioPortStruct = GpioPortStruct { port: GpioPort::G, pin: 5 };
pub const PG6: GpioPortStruct = GpioPortStruct { port: GpioPort::G, pin: 6 };
pub const PG7: GpioPortStruct = GpioPortStruct { port: GpioPort::G, pin: 7 };
pub const PG8: GpioPortStruct = GpioPortStruct { port: GpioPort::G, pin: 8 };
pub const PG9: GpioPortStruct = GpioPortStruct { port: GpioPort::G, pin: 9 };
pub const PG10: GpioPortStruct = GpioPortStruct { port: GpioPort::G, pin: 10 };
pub const PG11: GpioPortStruct = GpioPortStruct { port: GpioPort::G, pin: 11 };
pub const PG12: GpioPortStruct = GpioPortStruct { port: GpioPort::G, pin: 12 };
pub const PG13: GpioPortStruct = GpioPortStruct { port: GpioPort::G, pin: 13 };
pub const PG14: GpioPortStruct = GpioPortStruct { port: GpioPort::G, pin: 14 };
pub const PG15: GpioPortStruct = GpioPortStruct { port: GpioPort::G, pin: 15 };

/// GPIO重映射配置函数
/// # Safety
/// - 调用者必须确保AFIO外设时钟已启用
/// - 调用者必须确保重映射配置不会与其他外设冲突
/// - 某些重映射可能需要同时配置相关GPIO引脚为复用功能
pub unsafe fn gpio_pin_remap_config(remap: GpioRemap, enable: bool) {
    let afio = &mut *(0x40010000 as *mut library::afio::RegisterBlock);
    
    match remap {
        GpioRemap::RemapSPI1 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00000001;
                } else {
                    value &= !0x00000001;
                }
                w.bits(value)
            });
        },
        GpioRemap::RemapI2C1 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00000002;
                } else {
                    value &= !0x00000002;
                }
                w.bits(value)
            });
        },
        GpioRemap::RemapUSART1 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00000004;
                } else {
                    value &= !0x00000004;
                }
                w.bits(value)
            });
        },
        GpioRemap::RemapUSART2 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00000008;
                } else {
                    value &= !0x00000008;
                }
                w.bits(value)
            });
        },
        GpioRemap::PartialRemapUSART3 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00140010;
                } else {
                    value &= !0x00140030;
                }
                w.bits(value)
            });
        },
        GpioRemap::FullRemapUSART3 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00140030;
                } else {
                    value &= !0x00140030;
                }
                w.bits(value)
            });
        },
        GpioRemap::PartialRemapTIM1 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00160040;
                } else {
                    value &= !0x001600C0;
                }
                w.bits(value)
            });
        },
        GpioRemap::FullRemapTIM1 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x001600C0;
                } else {
                    value &= !0x001600C0;
                }
                w.bits(value)
            });
        },
        GpioRemap::PartialRemap1TIM2 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00180100;
                } else {
                    value &= !0x00180300;
                }
                w.bits(value)
            });
        },
        GpioRemap::PartialRemap2TIM2 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00180200;
                } else {
                    value &= !0x00180300;
                }
                w.bits(value)
            });
        },
        GpioRemap::FullRemapTIM2 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00180300;
                } else {
                    value &= !0x00180300;
                }
                w.bits(value)
            });
        },
        GpioRemap::PartialRemapTIM3 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x001A0800;
                } else {
                    value &= !0x001A0C00;
                }
                w.bits(value)
            });
        },
        GpioRemap::FullRemapTIM3 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x001A0C00;
                } else {
                    value &= !0x001A0C00;
                }
                w.bits(value)
            });
        },
        GpioRemap::RemapTIM4 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00001000;
                } else {
                    value &= !0x00001000;
                }
                w.bits(value)
            });
        },
        GpioRemap::Remap1CAN1 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x001D4000;
                } else {
                    value &= !0x001D6000;
                }
                w.bits(value)
            });
        },
        GpioRemap::Remap2CAN1 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x001D6000;
                } else {
                    value &= !0x001D6000;
                }
                w.bits(value)
            });
        },
        GpioRemap::RemapPD01 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00008000;
                } else {
                    value &= !0x00008000;
                }
                w.bits(value)
            });
        },
        GpioRemap::RemapSWJNoJTRST => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00300100;
                } else {
                    value &= !0x00300700;
                }
                w.bits(value)
            });
        },
        GpioRemap::RemapSWJJTAGDisable => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00300200;
                } else {
                    value &= !0x00300700;
                }
                w.bits(value)
            });
        },
        GpioRemap::RemapSWJDisable => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00300400;
                } else {
                    value &= !0x00300700;
                }
                w.bits(value)
            });
        },
        GpioRemap::RemapTim5Ch4Lsi => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00010000;
                } else {
                    value &= !0x00010000;
                }
                w.bits(value)
            });
        },
        GpioRemap::RemapAdc1EtrgInj => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00000100;
                } else {
                    value &= !0x00000100;
                }
                w.bits(value)
            });
        },
        GpioRemap::RemapAdc1EtrgReg => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00000200;
                } else {
                    value &= !0x00000200;
                }
                w.bits(value)
            });
        },
        GpioRemap::RemapAdc2EtrgInj => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00000400;
                } else {
                    value &= !0x00000400;
                }
                w.bits(value)
            });
        },
        GpioRemap::RemapAdc2EtrgReg => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00000800;
                } else {
                    value &= !0x00000800;
                }
                w.bits(value)
            });
        },
        GpioRemap::RemapEth => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00800000;
                } else {
                    value &= !0x00800000;
                }
                w.bits(value)
            });
        },
        GpioRemap::RemapCan2 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00200000;
                } else {
                    value &= !0x00200000;
                }
                w.bits(value)
            });
        },
        GpioRemap::RemapSpi3 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x03000000;
                } else {
                    value &= !0x03000000;
                }
                w.bits(value)
            });
        },
        GpioRemap::RemapTim2Itr1PtpSof => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x40000000;
                } else {
                    value &= !0x40000000;
                }
                w.bits(value)
            });
        },
        GpioRemap::RemapPtpPps => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x80000000;
                } else {
                    value &= !0x80000000;
                }
                w.bits(value)
            });
        },
        // 以下为F4系列特有的重映射，在F1系列中可能不适用，仅作占位符
        GpioRemap::RemapTim15 |
        GpioRemap::RemapTim16 |
        GpioRemap::RemapTim17 |
        GpioRemap::RemapCec |
        GpioRemap::RemapTim1Dma |
        GpioRemap::RemapTim9 |
        GpioRemap::RemapTim10 |
        GpioRemap::RemapTim11 |
        GpioRemap::RemapTim13 |
        GpioRemap::RemapTim14 |
        GpioRemap::RemapFsmcNadv |
        GpioRemap::RemapTim67DacDma |
        GpioRemap::RemapTim12 |
        GpioRemap::RemapMisc => {
            // 在F1系列中这些重映射不适用，故不做处理
        },
    }
}

/// 扩展GpioRemap枚举，添加更多重映射选项
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GpioRemap {
    // SPI1重映射
    RemapSPI1,
    // I2C1重映射
    RemapI2C1,
    // USART1重映射
    RemapUSART1,
    // USART2重映射
    RemapUSART2,
    // USART3部分重映射
    PartialRemapUSART3,
    // USART3完全重映射
    FullRemapUSART3,
    // TIM1部分重映射
    PartialRemapTIM1,
    // TIM1完全重映射
    FullRemapTIM1,
    // TIM2部分重映射1
    PartialRemap1TIM2,
    // TIM2部分重映射2
    PartialRemap2TIM2,
    // TIM2完全重映射
    FullRemapTIM2,
    // TIM3部分重映射
    PartialRemapTIM3,
    // TIM3完全重映射
    FullRemapTIM3,
    // TIM4重映射
    RemapTIM4,
    // CAN1重映射1
    Remap1CAN1,
    // CAN1重映射2
    Remap2CAN1,
    // PD01重映射
    RemapPD01,
    // SWJ无JTRST重映射
    RemapSWJNoJTRST,
    // SWJ禁用JTAG重映射
    RemapSWJJTAGDisable,
    // SWJ完全禁用重映射
    RemapSWJDisable,
    // TIM5CH4_LSI重映射
    RemapTim5Ch4Lsi,
    // ADC1外部触发注入转换重映射
    RemapAdc1EtrgInj,
    // ADC1外部触发规则转换重映射
    RemapAdc1EtrgReg,
    // ADC2外部触发注入转换重映射
    RemapAdc2EtrgInj,
    // ADC2外部触发规则转换重映射
    RemapAdc2EtrgReg,
    // 以太网重映射
    RemapEth,
    // CAN2重映射
    RemapCan2,
    // SPI3重映射
    RemapSpi3,
    // TIM2ITR1_PTP_SOF重映射
    RemapTim2Itr1PtpSof,
    // PTP_PPS重映射
    RemapPtpPps,
    // TIM15重映射
    RemapTim15,
    // TIM16重映射
    RemapTim16,
    // TIM17重映射
    RemapTim17,
    // CEC重映射
    RemapCec,
    // TIM1 DMA重映射
    RemapTim1Dma,
    // TIM9重映射
    RemapTim9,
    // TIM10重映射
    RemapTim10,
    // TIM11重映射
    RemapTim11,
    // TIM13重映射
    RemapTim13,
    // TIM14重映射
    RemapTim14,
    // FSMC_NADV重映射
    RemapFsmcNadv,
    // TIM67_DAC_DMA重映射
    RemapTim67DacDma,
    // TIM12重映射
    RemapTim12,
    // MISC重映射
    RemapMisc,
}

/// 复位AFIO寄存器到默认状态
/// # Safety
/// - 调用者必须确保RCC外设时钟已启用
/// - 此函数会重置所有AFIO配置，包括重映射和EXTI配置
/// - 调用后需要重新配置所有必要的AFIO功能
pub unsafe fn gpio_afio_deinit() {
    let rcc = &mut *(0x40021000 as *mut library::rcc::RegisterBlock);
    
    // 使能AFIO复位
    rcc.apb2rstr().write(|w| unsafe { w.bits(1 << 0) });
    rcc.apb2rstr().write(|w| unsafe { w.bits(0) });
}

/// 统一的GPIO初始化函数（类似标准库的GPIO_Init）
/// 使用GpioInitConfig结构体配置引脚
/// # Safety
/// - 调用者必须确保相应GPIO端口时钟已启用
/// - 调用者必须确保引脚未被其他代码或外设占用
pub unsafe fn gpio_init(port: GpioPort, config: GpioInitConfig) {
    // 使能GPIO时钟
    let rcc = &mut *(0x4002_1000 as *mut rcc::RegisterBlock);
    let clock_bit = 1 << (2 + port as u32);
    rcc.apb2enr().write(|w| unsafe { w.bits(rcc.apb2enr().read().bits() | clock_bit) });
    
    // 获取GPIO端口寄存器指针
    let gpio_ptr = match port {
        GpioPort::A => 0x4001_0800 as *mut u32,
        GpioPort::B => 0x4001_0C00 as *mut u32,
        GpioPort::C => 0x4001_1000 as *mut u32,
        GpioPort::D => 0x4001_1400 as *mut u32,
        GpioPort::E => 0x4001_1800 as *mut u32,
        GpioPort::F => 0x4001_1C00 as *mut u32,
        GpioPort::G => 0x4001_2000 as *mut u32,
    };
    
    // 配置每个引脚
    for pin in 0..16 {
        if config.pin & (1 << pin) != 0 {
            // 计算配置值
            let (cnf, mode) = match config.mode {
                GpioMode::FloatingInput => (0b01, 0b00),
                GpioMode::PullUpInput => (0b10, 0b00),
                GpioMode::PullDownInput => (0b10, 0b00),
                GpioMode::AnalogInput => (0b00, 0b00),
                GpioMode::PushPullOutput => (0b00, match config.speed {
                    GpioSpeed::Speed10MHz => 0b01,
                    GpioSpeed::Speed2MHz => 0b10,
                    GpioSpeed::Speed50MHz => 0b11,
                }),
                GpioMode::OpenDrainOutput => (0b01, match config.speed {
                    GpioSpeed::Speed10MHz => 0b01,
                    GpioSpeed::Speed2MHz => 0b10,
                    GpioSpeed::Speed50MHz => 0b11,
                }),
                GpioMode::AlternatePushPull => (0b10, match config.speed {
                    GpioSpeed::Speed10MHz => 0b01,
                    GpioSpeed::Speed2MHz => 0b10,
                    GpioSpeed::Speed50MHz => 0b11,
                }),
                GpioMode::AlternateOpenDrain => (0b11, match config.speed {
                    GpioSpeed::Speed10MHz => 0b01,
                    GpioSpeed::Speed2MHz => 0b10,
                    GpioSpeed::Speed50MHz => 0b11,
                }),
            };
            
            let config_val = (cnf << 2) | mode;
            
            // 设置配置寄存器
            let cr_offset = if pin < 8 { 0x00 } else { 0x04 };
            let pin_pos = pin % 8;
            let cr_ptr = (gpio_ptr as usize + cr_offset) as *mut u32;
            
            let pin_mask = 0x0F << (pin_pos * 4);
            let mut value = *cr_ptr;
            value = (value & !pin_mask) | (config_val << (pin_pos * 4));
            *cr_ptr = value;
            
            // 配置上拉下拉
            if matches!(config.mode, GpioMode::PullUpInput) {
                let odr_ptr = (gpio_ptr as usize + 0x0C) as *mut u32;
                *odr_ptr |= 1 << pin;
            } else if matches!(config.mode, GpioMode::PullDownInput) {
                let odr_ptr = (gpio_ptr as usize + 0x0C) as *mut u32;
                *odr_ptr &= !(1 << pin);
            }
        }
    }
}

/// 配置外部中断线
/// 该函数用于将指定GPIO端口的引脚映射到对应的外部中断线上
/// 注意：每个外部中断线(0-15)可以连接到不同端口的相同引脚号
/// # Safety
/// - 调用者必须确保AFIO外设时钟已启用
/// - 调用者必须确保pin_source在0-15范围内
/// - 配置后需要在EXTI控制器中设置对应的中断触发条件
pub unsafe fn gpio_exti_line_config(port_source: GpioPort, pin_source: u8) {
    let afio = &mut *(0x40010000 as *mut library::afio::RegisterBlock);
    
    // 确保pin_source在0-15范围内
    assert!(pin_source < 16, "Pin source must be between 0 and 15");
    
    // 将端口源转换为数字(0-6对应A-G)
    let port_num = match port_source {
        GpioPort::A => 0x00,
        GpioPort::B => 0x01,
        GpioPort::C => 0x02,
        GpioPort::D => 0x03,
        GpioPort::E => 0x04,
        GpioPort::F => 0x05,
        GpioPort::G => 0x06,
    };
    
    // 根据引脚号选择对应的EXTICR寄存器
    match pin_source / 4 {
        0 => {
            // EXTICR1 - 引脚0-3
            let pos = (pin_source % 4) * 4;
            let mask = 0x0F << pos;
            
            afio.exticr1().modify(|r, w| {
                let mut bits = r.bits();
                bits &= !mask;
                bits |= port_num << pos;
                unsafe { w.bits(bits) }
            });
        },
        1 => {
            // EXTICR2 - 引脚4-7
            let pos = (pin_source % 4) * 4;
            let mask = 0x0F << pos;
            
            afio.exticr2().modify(|r, w| {
                let mut bits = r.bits();
                bits &= !mask;
                bits |= port_num << pos;
                unsafe { w.bits(bits) }
            });
        },
        2 => {
            // EXTICR3 - 引脚8-11
            let pos = (pin_source % 4) * 4;
            let mask = 0x0F << pos;
            
            afio.exticr3().modify(|r, w| {
                let mut bits = r.bits();
                bits &= !mask;
                bits |= port_num << pos;
                unsafe { w.bits(bits) }
            });
        },
        3 => {
            // EXTICR4 - 引脚12-15
            let pos = (pin_source % 4) * 4;
            let mask = 0x0F << pos;
            
            afio.exticr4().modify(|r, w| {
                let mut bits = r.bits();
                bits &= !mask;
                bits |= port_num << pos;
                unsafe { w.bits(bits) }
            });
        },
        _ => unreachable!(),
    }
}
