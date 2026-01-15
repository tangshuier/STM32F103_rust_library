//! ADC模块
//! 提供ADC转换功能封装

// 屏蔽未使用代码警告
#![allow(unused)]

// 导入内部生成的设备驱动库
use stm32f103::*;

/// ADC模式枚举
#[derive(Debug, Clone, Copy)]
pub enum AdcMode {
    Independent = 0x00000000,            // 独立模式
    RegInjecSimult = 0x00010000,          // 规则和注入通道同时转换
    RegSimultAlterTrig = 0x00020000,      // 规则通道同时转换，交替触发
    InjecSimultFastInterl = 0x00030000,   // 注入通道同时转换，快速交叉
    InjecSimultSlowInterl = 0x00040000,   // 注入通道同时转换，慢速交叉
    InjecSimult = 0x00050000,             // 注入通道同时转换
    RegSimult = 0x00060000,               // 规则通道同时转换
    FastInterl = 0x00070000,              // 快速交叉模式
    SlowInterl = 0x00080000,              // 慢速交叉模式
    AlterTrig = 0x00090000,               // 交替触发模式
}

/// ADC外部触发源枚举
#[derive(Debug, Clone, Copy)]
pub enum AdcExternalTrig {
    None = 0x000E0000,                   // 无外部触发
    T1CC1 = 0x00000000,                  // 定时器1捕获比较1
    T1CC2 = 0x00020000,                  // 定时器1捕获比较2
    T1CC3 = 0x00040000,                  // 定时器1捕获比较3
    T2CC2 = 0x00060000,                  // 定时器2捕获比较2
    T3TRGO = 0x00080000,                 // 定时器3触发输出
    T4CC4 = 0x000A0000,                  // 定时器4捕获比较4
    ExtIT11TIM8TRGO = 0x000C0000,        // 外部中断11或定时器8触发输出
}

/// ADC数据对齐方式枚举
#[derive(Debug, Clone, Copy)]
pub enum AdcDataAlign {
    Right = 0x00000000,                  // 右对齐
    Left = 0x00000800,                   // 左对齐
}

/// ADC中断枚举
#[derive(Debug, Clone, Copy)]
pub enum AdcInterrupt {
    EOC,    // 转换结束中断
    AWD,    // 模拟看门狗中断
    JEOC,   // 注入通道转换结束中断
}

/// ADC标志枚举
#[derive(Debug, Clone, Copy)]
pub enum AdcFlag {
    AWD,    // 模拟看门狗标志
    EOC,    // 转换结束标志
    JEOC,   // 注入通道转换结束标志
    JSTRT,  // 注入通道开始转换标志
    STRT,   // 规则通道开始转换标志
}

/// ADC通道枚举
#[derive(Debug, Clone, Copy)]
pub enum AdcChannel {
    Channel0 = 0,
    Channel1 = 1,
    Channel2 = 2,
    Channel3 = 3,
    Channel4 = 4,
    Channel5 = 5,
    Channel6 = 6,
    Channel7 = 7,
    Channel8 = 8,
    Channel9 = 9,
    Channel10 = 10,
    Channel11 = 11,
    Channel12 = 12,
    Channel13 = 13,
    Channel14 = 14,
    Channel15 = 15,
    Channel16 = 16,  // 温度传感器
    Channel17 = 17,  // 内部参考电压
}

/// ADC采样时间枚举
#[derive(Debug, Clone, Copy)]
pub enum AdcSampleTime {
    Cycles1_5 = 0,    // 1.5个ADC时钟周期
    Cycles7_5 = 1,    // 7.5个ADC时钟周期
    Cycles13_5 = 2,   // 13.5个ADC时钟周期
    Cycles28_5 = 3,   // 28.5个ADC时钟周期
    Cycles41_5 = 4,   // 41.5个ADC时钟周期
    Cycles55_5 = 5,   // 55.5个ADC时钟周期
    Cycles71_5 = 6,   // 71.5个ADC时钟周期
    Cycles239_5 = 7,  // 239.5个ADC时钟周期
}

/// ADC枚举
#[derive(Debug, Clone, Copy)]
pub enum AdcNumber {
    ADC1,
    ADC2,
}

/// ADC配置结构体
#[derive(Debug, Clone, Copy)]
pub struct AdcConfig {
    pub mode: AdcMode,                    // ADC模式
    pub scan_conv_mode: bool,             // 扫描模式
    pub continuous_conv_mode: bool,       // 连续转换模式
    pub external_trig_conv: AdcExternalTrig, // 外部触发源
    pub data_align: AdcDataAlign,         // 数据对齐方式
    pub nbr_of_channel: u8,               // 通道数量(1-16)
}

impl Default for AdcConfig {
    fn default() -> Self {
        AdcConfig {
            mode: AdcMode::Independent,
            scan_conv_mode: false,
            continuous_conv_mode: false,
            external_trig_conv: AdcExternalTrig::None,
            data_align: AdcDataAlign::Right,
            nbr_of_channel: 1,
        }
    }
}

/// ADC结构体
pub struct Adc {
    number: AdcNumber,
}

impl AdcNumber {
    /// 获取ADC1寄存器
    fn get_adc1(&self) -> Option<&'static mut stm32f103::adc1::RegisterBlock> {
        match self {
            AdcNumber::ADC1 => unsafe { Some(&mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock)) },
            AdcNumber::ADC2 => None,
        }
    }
    
    /// 获取ADC2寄存器
    fn get_adc2(&self) -> Option<&'static mut stm32f103::adc2::RegisterBlock> {
        match self {
            AdcNumber::ADC1 => None,
            AdcNumber::ADC2 => unsafe { Some(&mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock)) },
        }
    }
    
    /// 获取ADC时钟使能位
    const fn clock_en_bit(&self) -> u32 {
        match self {
            AdcNumber::ADC1 => 1 << 9,  // APB2
            AdcNumber::ADC2 => 1 << 10,  // APB2
        }
    }
}

impl Adc {
    /// 创建新的ADC实例
    pub const fn new(number: AdcNumber) -> Self {
        Self {
            number,
        }
    }
    
    /// 获取ADC1寄存器
    fn get_adc1(&self) -> Option<&'static mut stm32f103::adc1::RegisterBlock> {
        match self.number {
            AdcNumber::ADC1 => unsafe { Some(&mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock)) },
            AdcNumber::ADC2 => None,
        }
    }
    
    /// 获取ADC2寄存器
    fn get_adc2(&self) -> Option<&'static mut stm32f103::adc2::RegisterBlock> {
        match self.number {
            AdcNumber::ADC1 => None,
            AdcNumber::ADC2 => unsafe { Some(&mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock)) },
        }
    }
    
    /// 初始化ADC
    pub fn init(&self, config: &AdcConfig) {
        let rcc = unsafe { &mut *(0x40021000 as *mut stm32f103::rcc::RegisterBlock) };
        
        // 1. 启用ADC时钟
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    rcc.apb2enr().modify(|_, w: &mut stm32f103::rcc::apb2enr::W| {
                        w.adc1en().set_bit()
                    });
                    
                    // 3. 配置ADC1控制寄存器
                    let adc = unsafe { &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock) };
                    
                    // CR1: 配置ADC模式和扫描模式
                    let mut cr1_bits = config.mode as u32;
                    if config.scan_conv_mode {
                        cr1_bits |= 0x00000100; // 设置扫描模式位
                    }
                    adc.cr1().write(|w| w.bits(cr1_bits));
                    
                    // CR2: 配置连续转换、外部触发、数据对齐和启动ADC
                    let mut cr2_bits = (config.external_trig_conv as u32) | (config.data_align as u32);
                    if config.continuous_conv_mode {
                        cr2_bits |= 0x00000001; // 设置连续转换位
                    }
                    cr2_bits |= 0x00000001; // 启用ADC
                    adc.cr2().write(|w| w.bits(cr2_bits));
                    
                    // 4. 配置通道数量
                    adc.sqr1().modify(|_, w| w.l().bits(config.nbr_of_channel - 1));
                    
                    // 5. 校准ADC
                    self.calibrate();
                },
                AdcNumber::ADC2 => {
                    rcc.apb2enr().modify(|_, w: &mut stm32f103::rcc::apb2enr::W| {
                        w.adc2en().set_bit()
                    });
                    
                    // 3. 配置ADC2控制寄存器
                    let adc = unsafe { &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock) };
                    
                    // CR1: 配置ADC模式和扫描模式
                    let mut cr1_bits = config.mode as u32;
                    if config.scan_conv_mode {
                        cr1_bits |= 0x00000100; // 设置扫描模式位
                    }
                    adc.cr1().write(|w| w.bits(cr1_bits));
                    
                    // CR2: 配置连续转换、外部触发、数据对齐和启动ADC
                    let mut cr2_bits = (config.external_trig_conv as u32) | (config.data_align as u32);
                    if config.continuous_conv_mode {
                        cr2_bits |= 0x00000001; // 设置连续转换位
                    }
                    cr2_bits |= 0x00000001; // 启用ADC
                    adc.cr2().write(|w| w.bits(cr2_bits));
                    
                    // 4. 配置通道数量
                    adc.sqr1().modify(|_, w| w.l().bits(config.nbr_of_channel - 1));
                    
                    // 5. 校准ADC
                    self.calibrate();
                },
            }
        }
    }
    
    /// 重置校准
    pub fn reset_calibration(&self) {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    adc.cr2().modify(|_, w| w.rstcal().set_bit());
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    adc.cr2().modify(|_, w| w.rstcal().set_bit());
                },
            }
        }
    }
    
    /// 获取重置校准状态
    pub fn get_reset_calibration_status(&self) -> bool {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    adc.cr2().read().rstcal().bit()
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    adc.cr2().read().rstcal().bit()
                },
            }
        }
    }
    
    /// 开始校准
    pub fn start_calibration(&self) {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    adc.cr2().modify(|_, w| w.cal().set_bit());
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    adc.cr2().modify(|_, w| w.cal().set_bit());
                },
            }
        }
    }
    
    /// 获取校准状态
    pub fn get_calibration_status(&self) -> bool {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    adc.cr2().read().cal().bit()
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    adc.cr2().read().cal().bit()
                },
            }
        }
    }
    
    /// 校准ADC
    pub fn calibrate(&self) {
        // 重置校准
        self.reset_calibration();
        // 等待重置校准完成
        while self.get_reset_calibration_status() {
            core::hint::spin_loop();
        }
        
        // 开始校准
        self.start_calibration();
        // 等待校准完成
        while self.get_calibration_status() {
            core::hint::spin_loop();
        }
    }
    
    /// 设置通道采样时间
    pub fn set_sample_time(&self, channel: AdcChannel, time: AdcSampleTime) {
        let channel = channel as u8;
        let time = time as u8;
        
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    if channel < 10 {
                        // 使用SMPR2寄存器（通道0-9）
                        let shift = channel * 3;
                        adc.smpr2().modify(|r, w| {
                            let mut value = r.bits();
                            value &= !(0x07 << shift);
                            value |= (time as u32) << shift;
                            w.bits(value)
                        });
                    } else {
                        // 使用SMPR1寄存器（通道10-17）
                        let shift = (channel - 10) * 3;
                        adc.smpr1().modify(|r, w| {
                            let mut value = r.bits();
                            value &= !(0x07 << shift);
                            value |= (time as u32) << shift;
                            w.bits(value)
                        });
                    }
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    if channel < 10 {
                        // 使用SMPR2寄存器（通道0-9）
                        let shift = channel * 3;
                        adc.smpr2().modify(|r, w| {
                            let mut value = r.bits();
                            value &= !(0x07 << shift);
                            value |= (time as u32) << shift;
                            w.bits(value)
                        });
                    } else {
                        // 使用SMPR1寄存器（通道10-17）
                        let shift = (channel - 10) * 3;
                        adc.smpr1().modify(|r, w| {
                            let mut value = r.bits();
                            value &= !(0x07 << shift);
                            value |= (time as u32) << shift;
                            w.bits(value)
                        });
                    }
                },
            }
        }
    }
    
    /// 配置规则通道
    pub fn regular_channel_config(&self, channel: AdcChannel, rank: u8, sample_time: AdcSampleTime) {
        let rank = rank as u8;
        
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    
                    // 设置采样时间
                    self.set_sample_time(channel, sample_time);
                    
                    // 配置通道序列
                    if rank <= 6 {
                        // 使用SQR3寄存器（通道1-6）
                        let shift = (rank - 1) * 5;
                        adc.sqr3().modify(|r, w| {
                            let mut value = r.bits();
                            value &= !(0x1F << shift);
                            value |= (channel as u32) << shift;
                            w.bits(value)
                        });
                    } else if rank <= 12 {
                        // 使用SQR2寄存器（通道7-12）
                        let shift = (rank - 7) * 5;
                        adc.sqr2().modify(|r, w| {
                            let mut value = r.bits();
                            value &= !(0x1F << shift);
                            value |= (channel as u32) << shift;
                            w.bits(value)
                        });
                    } else if rank <= 16 {
                        // 使用SQR1寄存器（通道13-16）
                        let shift = (rank - 13) * 5;
                        adc.sqr1().modify(|r, w| {
                            let mut value = r.bits();
                            value &= !(0x1F << shift);
                            value |= (channel as u32) << shift;
                            w.bits(value)
                        });
                    }
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    
                    // 设置采样时间
                    self.set_sample_time(channel, sample_time);
                    
                    // 配置通道序列
                    if rank <= 6 {
                        // 使用SQR3寄存器（通道1-6）
                        let shift = (rank - 1) * 5;
                        adc.sqr3().modify(|r, w| {
                            let mut value = r.bits();
                            value &= !(0x1F << shift);
                            value |= (channel as u32) << shift;
                            w.bits(value)
                        });
                    } else if rank <= 12 {
                        // 使用SQR2寄存器（通道7-12）
                        let shift = (rank - 7) * 5;
                        adc.sqr2().modify(|r, w| {
                            let mut value = r.bits();
                            value &= !(0x1F << shift);
                            value |= (channel as u32) << shift;
                            w.bits(value)
                        });
                    } else if rank <= 16 {
                        // 使用SQR1寄存器（通道13-16）
                        let shift = (rank - 13) * 5;
                        adc.sqr1().modify(|r, w| {
                            let mut value = r.bits();
                            value &= !(0x1F << shift);
                            value |= (channel as u32) << shift;
                            w.bits(value)
                        });
                    }
                },
            }
        }
    }
    
    /// 软件启动转换命令
    pub fn software_start_conv_cmd(&self, enable: bool) {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    if enable {
                        adc.cr2().modify(|_, w| w.swstart().set_bit());
                    }
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    if enable {
                        adc.cr2().modify(|_, w| w.swstart().set_bit());
                    }
                },
            }
        }
    }
    
    /// 获取软件启动转换状态
    pub fn get_software_start_conv_status(&self) -> bool {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    adc.cr2().read().swstart().bit()
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    adc.cr2().read().swstart().bit()
                },
            }
        }
    }
    
    /// 单次转换（阻塞式）
    pub fn read_single_channel(&self, channel: AdcChannel) -> u16 {
        // 配置通道
        self.regular_channel_config(channel, 1, AdcSampleTime::Cycles13_5);
        
        // 启动转换
        self.software_start_conv_cmd(true);
        
        // 等待转换完成
        while !self.is_conversion_complete() {
            core::hint::spin_loop();
        }
        
        // 读取结果
        self.read_result()
    }
    
    /// 开始连续转换
    pub fn start_continuous(&self, channel: AdcChannel) {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    // 配置规则序列
                    self.regular_channel_config(channel, 1, AdcSampleTime::Cycles13_5);
                    
                    // 启用连续转换
                    adc.cr2().modify(|_, w| w.cont().set_bit());
                    
                    // 启动转换
                    self.software_start_conv_cmd(true);
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    // 配置规则序列
                    self.regular_channel_config(channel, 1, AdcSampleTime::Cycles13_5);
                    
                    // 启用连续转换
                    adc.cr2().modify(|_, w| w.cont().set_bit());
                    
                    // 启动转换
                    self.software_start_conv_cmd(true);
                },
            }
        }
    }
    
    /// 停止连续转换
    pub fn stop_continuous(&self) {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    // 禁用连续转换
                    adc.cr2().modify(|_, w| w.cont().clear_bit());
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    // 禁用连续转换
                    adc.cr2().modify(|_, w| w.cont().clear_bit());
                },
            }
        }
    }
    
    /// 外部触发转换命令
    pub fn external_trig_conv_cmd(&self, enable: bool) {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    if enable {
                        adc.cr2().modify(|_, w| w.exttrig().set_bit());
                    } else {
                        adc.cr2().modify(|_, w| w.exttrig().clear_bit());
                    }
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    if enable {
                        adc.cr2().modify(|_, w| w.exttrig().set_bit());
                    } else {
                        adc.cr2().modify(|_, w| w.exttrig().clear_bit());
                    }
                },
            }
        }
    }
    
    /// DMA使能命令
    pub fn dma_cmd(&self, enable: bool) {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    if enable {
                        adc.cr2().modify(|_, w| w.dma().set_bit());
                    } else {
                        adc.cr2().modify(|_, w| w.dma().clear_bit());
                    }
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    if enable {
                        adc.cr2().modify(|_, w| w.dma().set_bit());
                    } else {
                        adc.cr2().modify(|_, w| w.dma().clear_bit());
                    }
                },
            }
        }
    }
    
    /// 中断使能命令
    pub fn it_config(&self, it: AdcInterrupt, enable: bool) {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    match it {
                        AdcInterrupt::EOC => {
                            if enable {
                                adc.cr1().modify(|_, w| w.eocie().set_bit());
                            } else {
                                adc.cr1().modify(|_, w| w.eocie().clear_bit());
                            }
                        },
                        AdcInterrupt::AWD => {
                            if enable {
                                adc.cr1().modify(|_, w| w.awdie().set_bit());
                            } else {
                                adc.cr1().modify(|_, w| w.awdie().clear_bit());
                            }
                        },
                        AdcInterrupt::JEOC => {
                            if enable {
                                adc.cr1().modify(|_, w| w.jeocie().set_bit());
                            } else {
                                adc.cr1().modify(|_, w| w.jeocie().clear_bit());
                            }
                        },
                    }
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    match it {
                        AdcInterrupt::EOC => {
                            if enable {
                                adc.cr1().modify(|_, w| w.eocie().set_bit());
                            } else {
                                adc.cr1().modify(|_, w| w.eocie().clear_bit());
                            }
                        },
                        AdcInterrupt::AWD => {
                            if enable {
                                adc.cr1().modify(|_, w| w.awdie().set_bit());
                            } else {
                                adc.cr1().modify(|_, w| w.awdie().clear_bit());
                            }
                        },
                        AdcInterrupt::JEOC => {
                            if enable {
                                adc.cr1().modify(|_, w| w.jeocie().set_bit());
                            } else {
                                adc.cr1().modify(|_, w| w.jeocie().clear_bit());
                            }
                        },
                    }
                },
            }
        }
    }
    
    /// 检查转换是否完成
    pub fn is_conversion_complete(&self) -> bool {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    adc.sr().read().eoc().bit()
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    adc.sr().read().eoc().bit()
                },
            }
        }
    }
    
    /// 读取转换结果（非阻塞式）
    pub fn read_result(&self) -> u16 {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    (adc.dr().read().bits() & 0x0000FFFF) as u16
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    (adc.dr().read().bits() & 0x0000FFFF) as u16
                },
            }
        }
    }
    
    /// 获取标志状态
    pub fn get_flag_status(&self, flag: AdcFlag) -> bool {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    match flag {
                        AdcFlag::AWD => adc.sr().read().awd().bit(),
                        AdcFlag::EOC => adc.sr().read().eoc().bit(),
                        AdcFlag::JEOC => adc.sr().read().jeoc().bit(),
                        AdcFlag::JSTRT => adc.sr().read().jstrt().bit(),
                        AdcFlag::STRT => adc.sr().read().strt().bit(),
                    }
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    match flag {
                        AdcFlag::AWD => adc.sr().read().awd().bit(),
                        AdcFlag::EOC => adc.sr().read().eoc().bit(),
                        AdcFlag::JEOC => adc.sr().read().jeoc().bit(),
                        AdcFlag::JSTRT => adc.sr().read().jstrt().bit(),
                        AdcFlag::STRT => adc.sr().read().strt().bit(),
                    }
                },
            }
        }
    }
    
    /// 清除标志
    pub fn clear_flag(&self, flag: AdcFlag) {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    match flag {
                        AdcFlag::AWD => { adc.sr().write(|w| w.awd().clear_bit()); },
                        AdcFlag::EOC => { adc.sr().write(|w| w.eoc().clear_bit()); },
                        AdcFlag::JEOC => { adc.sr().write(|w| w.jeoc().clear_bit()); },
                        AdcFlag::JSTRT => { adc.sr().write(|w| w.jstrt().clear_bit()); },
                        AdcFlag::STRT => { adc.sr().write(|w| w.strt().clear_bit()); },
                    }
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    match flag {
                        AdcFlag::AWD => { adc.sr().write(|w| w.awd().clear_bit()); },
                        AdcFlag::EOC => { adc.sr().write(|w| w.eoc().clear_bit()); },
                        AdcFlag::JEOC => { adc.sr().write(|w| w.jeoc().clear_bit()); },
                        AdcFlag::JSTRT => { adc.sr().write(|w| w.jstrt().clear_bit()); },
                        AdcFlag::STRT => { adc.sr().write(|w| w.strt().clear_bit()); },
                    }
                },
            }
        }
    }
    
    /// 获取中断状态
    pub fn get_it_status(&self, it: AdcInterrupt) -> bool {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    match it {
                        AdcInterrupt::EOC => adc.sr().read().eoc().bit() && adc.cr1().read().eocie().bit(),
                        AdcInterrupt::AWD => adc.sr().read().awd().bit() && adc.cr1().read().awdie().bit(),
                        AdcInterrupt::JEOC => adc.sr().read().jeoc().bit() && adc.cr1().read().jeocie().bit(),
                    }
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    match it {
                        AdcInterrupt::EOC => adc.sr().read().eoc().bit() && adc.cr1().read().eocie().bit(),
                        AdcInterrupt::AWD => adc.sr().read().awd().bit() && adc.cr1().read().awdie().bit(),
                        AdcInterrupt::JEOC => adc.sr().read().jeoc().bit() && adc.cr1().read().jeocie().bit(),
                    }
                },
            }
        }
    }
    
    /// 清除中断挂起位
    pub fn clear_it_pending_bit(&self, it: AdcInterrupt) {
        // 清除对应的标志位即可清除中断
        match it {
            AdcInterrupt::EOC => self.clear_flag(AdcFlag::EOC),
            AdcInterrupt::AWD => self.clear_flag(AdcFlag::AWD),
            AdcInterrupt::JEOC => self.clear_flag(AdcFlag::JEOC),
        }
    }
    
    /// 重置ADC到默认值
    pub fn deinit(&self) {
        let rcc = unsafe { &mut *(0x40021000 as *mut stm32f103::rcc::RegisterBlock) };
        
        // 通过RCC重置ADC
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    // 启用ADC1重置
                    rcc.apb2rstr().modify(|_, w| w.adc1rst().set_bit());
                    // 禁用ADC1重置
                    rcc.apb2rstr().modify(|_, w| w.adc1rst().clear_bit());
                },
                AdcNumber::ADC2 => {
                    // 启用ADC2重置
                    rcc.apb2rstr().modify(|_, w| w.adc2rst().set_bit());
                    // 禁用ADC2重置
                    rcc.apb2rstr().modify(|_, w| w.adc2rst().clear_bit());
                },
            }
        }
    }
    
    /// 启用/禁用ADC
    pub fn cmd(&self, enable: bool) {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    if enable {
                        adc.cr2().modify(|_, w| w.adon().set_bit());
                    } else {
                        adc.cr2().modify(|_, w| w.adon().clear_bit());
                    }
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    if enable {
                        adc.cr2().modify(|_, w| w.adon().set_bit());
                    } else {
                        adc.cr2().modify(|_, w| w.adon().clear_bit());
                    }
                },
            }
        }
    }
    
    /// 配置温度传感器和内部参考电压
    pub fn temp_sensor_vrefint_cmd(&self, enable: bool) {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    if enable {
                        adc.cr2().modify(|_, w| w.tsvrefe().set_bit());
                    } else {
                        adc.cr2().modify(|_, w| w.tsvrefe().clear_bit());
                    }
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    if enable {
                        adc.cr2().modify(|_, w| w.tsvrefe().set_bit());
                    } else {
                        adc.cr2().modify(|_, w| w.tsvrefe().clear_bit());
                    }
                },
            }
        }
    }
    
    /// 配置注入通道
    pub fn injected_channel_config(&self, channel: AdcChannel, rank: u8, sample_time: AdcSampleTime) {
        let rank = rank as u8;
        let channel_u8 = channel as u8;
        
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    
                    // 设置采样时间
                    self.set_sample_time(channel, sample_time);
                    
                    // 配置注入通道序列
                    match rank {
                        1 => {
                            adc.jsqr().modify(|r, w| {
                                let mut value = r.bits();
                                value &= !(0x1F << 15);
                                value |= (channel_u8 as u32) << 15;
                                w.bits(value)
                            });
                        },
                        2 => {
                            adc.jsqr().modify(|r, w| {
                                let mut value = r.bits();
                                value &= !(0x1F << 10);
                                value |= (channel_u8 as u32) << 10;
                                w.bits(value)
                            });
                        },
                        3 => {
                            adc.jsqr().modify(|r, w| {
                                let mut value = r.bits();
                                value &= !(0x1F << 5);
                                value |= (channel_u8 as u32) << 5;
                                w.bits(value)
                            });
                        },
                        4 => {
                            adc.jsqr().modify(|r, w| {
                                let mut value = r.bits();
                                value &= !(0x1F << 0);
                                value |= (channel_u8 as u32) << 0;
                                w.bits(value)
                            });
                        },
                        _ => {},
                    }
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    
                    // 设置采样时间
                    self.set_sample_time(channel, sample_time);
                    
                    // 配置注入通道序列
                    match rank {
                        1 => {
                            adc.jsqr().modify(|r, w| {
                                let mut value = r.bits();
                                value &= !(0x1F << 15);
                                value |= (channel_u8 as u32) << 15;
                                w.bits(value)
                            });
                        },
                        2 => {
                            adc.jsqr().modify(|r, w| {
                                let mut value = r.bits();
                                value &= !(0x1F << 10);
                                value |= (channel_u8 as u32) << 10;
                                w.bits(value)
                            });
                        },
                        3 => {
                            adc.jsqr().modify(|r, w| {
                                let mut value = r.bits();
                                value &= !(0x1F << 5);
                                value |= (channel_u8 as u32) << 5;
                                w.bits(value)
                            });
                        },
                        4 => {
                            adc.jsqr().modify(|r, w| {
                                let mut value = r.bits();
                                value &= !(0x1F << 0);
                                value |= (channel_u8 as u32) << 0;
                                w.bits(value)
                            });
                        },
                        _ => {},
                    }
                },
            }
        }
    }
    
    /// 配置注入通道序列长度
    pub fn injected_sequencer_length_config(&self, length: u8) {
        let length = length as u8;
        
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    adc.jsqr().modify(|r, w| {
                        let mut value = r.bits();
                        value &= !(0x03 << 20);
                        value |= ((length - 1) as u32) << 20;
                        w.bits(value)
                    });
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    adc.jsqr().modify(|r, w| {
                        let mut value = r.bits();
                        value &= !(0x03 << 20);
                        value |= ((length - 1) as u32) << 20;
                        w.bits(value)
                    });
                },
            }
        }
    }
    
    /// 设置注入通道偏移
    pub fn set_injected_offset(&self, injected_channel: u8, offset: u16) {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    match injected_channel {
                        1 => {
                            adc.jofr1().write(|w| w.joffset1().bits(offset));
                        },
                        2 => {
                            adc.jofr2().write(|w| w.joffset2().bits(offset));
                        },
                        3 => {
                            adc.jofr3().write(|w| w.joffset3().bits(offset));
                        },
                        4 => {
                            adc.jofr4().write(|w| w.joffset4().bits(offset));
                        },
                        _ => {},
                    }
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    match injected_channel {
                        1 => {
                            adc.jofr1().write(|w| w.joffset1().bits(offset));
                        },
                        2 => {
                            adc.jofr2().write(|w| w.joffset2().bits(offset));
                        },
                        3 => {
                            adc.jofr3().write(|w| w.joffset3().bits(offset));
                        },
                        4 => {
                            adc.jofr4().write(|w| w.joffset4().bits(offset));
                        },
                        _ => {},
                    }
                },
            }
        }
    }
    
    /// 外部触发注入转换配置
    pub fn external_trig_injected_conv_config(&self, trig: u32) {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    adc.cr2().modify(|r, w| {
                        let mut value = r.bits();
                        value &= !(0x00007000); // 清除JEXTSEL位
                        value |= trig & 0x00007000;
                        w.bits(value)
                    });
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    adc.cr2().modify(|r, w| {
                        let mut value = r.bits();
                        value &= !(0x00007000); // 清除JEXTSEL位
                        value |= trig & 0x00007000;
                        w.bits(value)
                    });
                },
            }
        }
    }
    
    /// 外部触发注入转换命令
    pub fn external_trig_injected_conv_cmd(&self, enable: bool) {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    if enable {
                        adc.cr2().modify(|_, w| w.jexttrig().set_bit());
                    } else {
                        adc.cr2().modify(|_, w| w.jexttrig().clear_bit());
                    }
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    if enable {
                        adc.cr2().modify(|_, w| w.jexttrig().set_bit());
                    } else {
                        adc.cr2().modify(|_, w| w.jexttrig().clear_bit());
                    }
                },
            }
        }
    }
    
    /// 软件启动注入转换命令
    pub fn software_start_injected_conv_cmd(&self, enable: bool) {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    if enable {
                        adc.cr2().modify(|_, w| w.jswstart().set_bit());
                    }
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    if enable {
                        adc.cr2().modify(|_, w| w.jswstart().set_bit());
                    }
                },
            }
        }
    }
    
    /// 获取注入通道转换结果
    pub fn get_injected_conversion_value(&self, injected_channel: u8) -> u16 {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    match injected_channel {
                        1 => adc.jdr1().read().bits() as u16,
                        2 => adc.jdr2().read().bits() as u16,
                        3 => adc.jdr3().read().bits() as u16,
                        4 => adc.jdr4().read().bits() as u16,
                        _ => 0,
                    }
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    match injected_channel {
                        1 => adc.jdr1().read().bits() as u16,
                        2 => adc.jdr2().read().bits() as u16,
                        3 => adc.jdr3().read().bits() as u16,
                        4 => adc.jdr4().read().bits() as u16,
                        _ => 0,
                    }
                },
            }
        }
    }
    
    /// 自动注入转换命令
    pub fn auto_injected_conv_cmd(&self, enable: bool) {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    if enable {
                        adc.cr1().modify(|_, w| w.jauto().set_bit());
                    } else {
                        adc.cr1().modify(|_, w| w.jauto().clear_bit());
                    }
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    if enable {
                        adc.cr1().modify(|_, w| w.jauto().set_bit());
                    } else {
                        adc.cr1().modify(|_, w| w.jauto().clear_bit());
                    }
                },
            }
        }
    }
    
    /// 注入通道不连续模式命令
    pub fn injected_disc_mode_cmd(&self, enable: bool) {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    if enable {
                        adc.cr1().modify(|_, w| w.jdiscen().set_bit());
                    } else {
                        adc.cr1().modify(|_, w| w.jdiscen().clear_bit());
                    }
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    if enable {
                        adc.cr1().modify(|_, w| w.jdiscen().set_bit());
                    } else {
                        adc.cr1().modify(|_, w| w.jdiscen().clear_bit());
                    }
                },
            }
        }
    }
    
    /// 规则通道不连续模式通道计数配置
    pub fn disc_mode_channel_count_config(&self, number: u8) {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    adc.cr1().modify(|r, w| {
                        let mut value = r.bits();
                        value &= !(0x00001F00); // 清除DISCNUM位
                        value |= ((number - 1) as u32) << 8;
                        w.bits(value)
                    });
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    adc.cr1().modify(|r, w| {
                        let mut value = r.bits();
                        value &= !(0x00001F00); // 清除DISCNUM位
                        value |= ((number - 1) as u32) << 8;
                        w.bits(value)
                    });
                },
            }
        }
    }
    
    /// 规则通道不连续模式命令
    pub fn disc_mode_cmd(&self, enable: bool) {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    if enable {
                        adc.cr1().modify(|_, w| w.discen().set_bit());
                    } else {
                        adc.cr1().modify(|_, w| w.discen().clear_bit());
                    }
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    if enable {
                        adc.cr1().modify(|_, w| w.discen().set_bit());
                    } else {
                        adc.cr1().modify(|_, w| w.discen().clear_bit());
                    }
                },
            }
        }
    }
    
    /// 模拟看门狗配置
    pub fn analog_watchdog_cmd(&self, mode: u32) {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    adc.cr1().modify(|r, w| {
                        let mut value = r.bits();
                        value &= !(0x00C00200); // 清除AWDEN和JAWDEN位
                        value |= mode & 0x00C00200;
                        w.bits(value)
                    });
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    adc.cr1().modify(|r, w| {
                        let mut value = r.bits();
                        value &= !(0x00C00200); // 清除AWDEN和JAWDEN位
                        value |= mode & 0x00C00200;
                        w.bits(value)
                    });
                },
            }
        }
    }
    
    /// 模拟看门狗阈值配置
    pub fn analog_watchdog_thresholds_config(&self, high_threshold: u16, low_threshold: u16) {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    adc.htr().write(|w| w.bits(high_threshold as u32));
                    adc.ltr().write(|w| w.bits(low_threshold as u32));
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    adc.htr().write(|w| w.bits(high_threshold as u32));
                    adc.ltr().write(|w| w.bits(low_threshold as u32));
                },
            }
        }
    }
    
    /// 模拟看门狗单通道配置
    pub fn analog_watchdog_single_channel_config(&self, channel: AdcChannel) {
        let channel = channel as u8;
        
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    adc.cr1().modify(|r, w| {
                        let mut value = r.bits();
                        value &= !(0x0000001F); // 清除AWDCH位
                        value |= (channel as u32) & 0x0000001F;
                        w.bits(value)
                    });
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    adc.cr1().modify(|r, w| {
                        let mut value = r.bits();
                        value &= !(0x0000001F); // 清除AWDCH位
                        value |= (channel as u32) & 0x0000001F;
                        w.bits(value)
                    });
                },
            }
        }
    }
}

/// 预定义的ADC常量
pub const ADC1: Adc = Adc::new(AdcNumber::ADC1);
pub const ADC2: Adc = Adc::new(AdcNumber::ADC2);