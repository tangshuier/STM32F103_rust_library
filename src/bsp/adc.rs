//! ADC模块
//! 提供ADC转换功能封装

// 屏蔽未使用代码警告
#![allow(unused)]

// 导入内部生成的设备驱动库
use stm32f103::*;

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
    Cycles13 = 0,    // 13.5个ADC时钟周期
    Cycles28 = 1,    // 28.5个ADC时钟周期
    Cycles41 = 2,    // 41.5个ADC时钟周期
    Cycles55 = 3,    // 55.5个ADC时钟周期
    Cycles71 = 4,    // 71.5个ADC时钟周期
    Cycles239 = 5,   // 239.5个ADC时钟周期
}

/// ADC枚举
#[derive(Debug, Clone, Copy)]
pub enum AdcNumber {
    ADC1,
    ADC2,
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
    pub fn init(&self) {
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
                    // CR1: 禁用扫描模式，禁用中断
                    adc.cr1().write(|w: &mut stm32f103::adc1::cr1::W| w.bits(0x00000000));
                    // CR2: 单次转换模式，右对齐，外部触发禁用，禁用连续转换
                    adc.cr2().write(|w: &mut stm32f103::adc1::cr2::W| w.adon().set_bit());  // 启用ADC
                    // 4. 校准ADC
                    self.calibrate_adc1();
                },
                AdcNumber::ADC2 => {
                    rcc.apb2enr().modify(|_, w: &mut stm32f103::rcc::apb2enr::W| {
                        w.adc2en().set_bit()
                    });
                    
                    // 3. 配置ADC2控制寄存器
                    let adc = unsafe { &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock) };
                    // CR1: 禁用扫描模式，禁用中断
                    adc.cr1().write(|w: &mut stm32f103::adc2::cr1::W| w.bits(0x00000000));
                    // CR2: 单次转换模式，右对齐，外部触发禁用，禁用连续转换
                    adc.cr2().write(|w: &mut stm32f103::adc2::cr2::W| w.adon().set_bit());  // 启用ADC
                    // 4. 校准ADC
                    self.calibrate_adc2();
                },
            }
        }
    }
    
    /// 校准ADC1
    fn calibrate_adc1(&self) {
        let adc = unsafe { &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock) };
        
        unsafe {
            // 关闭ADC
            adc.cr2().modify(|_, w: &mut stm32f103::adc1::cr2::W| w.adon().clear_bit());
            
            // 启动校准
            adc.cr2().modify(|_, w: &mut stm32f103::adc1::cr2::W| w.cal().set_bit());
            
            // 等待校准完成
            while adc.cr2().read().cal().bit() {
                core::hint::spin_loop();
            }
            
            // 开启ADC
            adc.cr2().modify(|_, w: &mut stm32f103::adc1::cr2::W| w.adon().set_bit());
        }
    }
    
    /// 校准ADC2
    fn calibrate_adc2(&self) {
        let adc = unsafe { &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock) };
        
        unsafe {
            // 关闭ADC
            adc.cr2().modify(|_, w: &mut stm32f103::adc2::cr2::W| w.adon().clear_bit());
            
            // 启动校准
            adc.cr2().modify(|_, w: &mut stm32f103::adc2::cr2::W| w.cal().set_bit());
            
            // 等待校准完成
            while adc.cr2().read().cal().bit() {
                core::hint::spin_loop();
            }
            
            // 开启ADC
            adc.cr2().modify(|_, w: &mut stm32f103::adc2::cr2::W| w.adon().set_bit());
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
                        adc.smpr2().modify(|r: &stm32f103::adc1::smpr2::R, w: &mut stm32f103::adc1::smpr2::W| {
                            let mut value = r.bits();
                            value &= !(0x07 << shift);
                            value |= (time as u32) << shift;
                            w.bits(value)
                        });
                    } else {
                        // 使用SMPR1寄存器（通道10-17）
                        let shift = (channel - 10) * 3;
                        adc.smpr1().modify(|r: &stm32f103::adc1::smpr1::R, w: &mut stm32f103::adc1::smpr1::W| {
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
                        adc.smpr2().modify(|r: &stm32f103::adc2::smpr2::R, w: &mut stm32f103::adc2::smpr2::W| {
                            let mut value = r.bits();
                            value &= !(0x07 << shift);
                            value |= (time as u32) << shift;
                            w.bits(value)
                        });
                    } else {
                        // 使用SMPR1寄存器（通道10-17）
                        let shift = (channel - 10) * 3;
                        adc.smpr1().modify(|r: &stm32f103::adc2::smpr1::R, w: &mut stm32f103::adc2::smpr1::W| {
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
    
    /// 单次转换（阻塞式）
    pub fn read_single_channel(&self, channel: AdcChannel) -> u16 {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    // 配置规则序列
                    adc.sqr1().write(|w: &mut stm32f103::adc1::sqr1::W| w.l().bits(0));  // 1个转换
                    adc.sqr3().write(|w: &mut stm32f103::adc1::sqr3::W| w.sq1().bits(channel as u8));  // 第一个转换通道
                    
                    // 启动转换
                    adc.cr2().modify(|_, w: &mut stm32f103::adc1::cr2::W| w.swstart().set_bit());
                    
                    // 等待转换完成
                    while !adc.sr().read().eoc().bit() {
                        core::hint::spin_loop();
                    }
                    
                    // 读取结果
                    (adc.dr().read().bits() & 0x0000FFFF) as u16
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    // 配置规则序列
                    adc.sqr1().write(|w: &mut stm32f103::adc2::sqr1::W| w.l().bits(0));  // 1个转换
                    adc.sqr3().write(|w: &mut stm32f103::adc2::sqr3::W| w.sq1().bits(channel as u8));  // 第一个转换通道
                    
                    // 启动转换
                    adc.cr2().modify(|_, w: &mut stm32f103::adc2::cr2::W| w.swstart().set_bit());
                    
                    // 等待转换完成
                    while !adc.sr().read().eoc().bit() {
                        core::hint::spin_loop();
                    }
                    
                    // 读取结果
                    (adc.dr().read().bits() & 0x0000FFFF) as u16
                },
            }
        }
    }
    
    /// 开始连续转换
    pub fn start_continuous(&self, channel: AdcChannel) {
        unsafe {
            match self.number {
                AdcNumber::ADC1 => {
                    let adc = &mut *(0x40012400 as *mut stm32f103::adc1::RegisterBlock);
                    // 配置规则序列
                    adc.sqr1().write(|w: &mut stm32f103::adc1::sqr1::W| w.l().bits(0));  // 1个转换
                    adc.sqr3().write(|w: &mut stm32f103::adc1::sqr3::W| w.sq1().bits(channel as u8));  // 第一个转换通道
                    
                    // 启用连续转换
                    adc.cr2().modify(|_, w: &mut stm32f103::adc1::cr2::W| w.cont().set_bit());
                    
                    // 启动转换
                    adc.cr2().modify(|_, w: &mut stm32f103::adc1::cr2::W| w.swstart().set_bit());
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    // 配置规则序列
                    adc.sqr1().write(|w: &mut stm32f103::adc2::sqr1::W| w.l().bits(0));  // 1个转换
                    adc.sqr3().write(|w: &mut stm32f103::adc2::sqr3::W| w.sq1().bits(channel as u8));  // 第一个转换通道
                    
                    // 启用连续转换
                    adc.cr2().modify(|_, w: &mut stm32f103::adc2::cr2::W| w.cont().set_bit());
                    
                    // 启动转换
                    adc.cr2().modify(|_, w: &mut stm32f103::adc2::cr2::W| w.swstart().set_bit());
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
                    adc.cr2().modify(|_, w: &mut stm32f103::adc1::cr2::W| w.cont().clear_bit());
                },
                AdcNumber::ADC2 => {
                    let adc = &mut *(0x40012800 as *mut stm32f103::adc2::RegisterBlock);
                    // 禁用连续转换
                    adc.cr2().modify(|_, w: &mut stm32f103::adc2::cr2::W| w.cont().clear_bit());
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
}

/// 预定义的ADC常量
pub const ADC1: Adc = Adc::new(AdcNumber::ADC1);
pub const ADC2: Adc = Adc::new(AdcNumber::ADC2);