//! RCC模块
//! 提供复位和时钟控制功能封装

#![allow(unused)]

// 使用生成的设备驱动库
use stm32f103::*;

/// RCC时钟源枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RccClockSource {
    HSI,    // 内部高速时钟
    HSE,    // 外部高速时钟
    PLL,    // 锁相环时钟
}

/// RCC PLL输入源枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RccPllSource {
    HsiDiv2,   // HSI除以2
    Hse,        // HSE
    HseDiv2,   // HSE除以2
}

/// RCC PLL倍频系数枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RccPllMul {
    Mul2 = 0x00,    // 2倍
    Mul3 = 0x01,    // 3倍
    Mul4 = 0x02,    // 4倍
    Mul5 = 0x03,    // 5倍
    Mul6 = 0x04,    // 6倍
    Mul7 = 0x05,    // 7倍
    Mul8 = 0x06,    // 8倍
    Mul9 = 0x07,    // 9倍
    Mul10 = 0x08,   // 10倍
    Mul11 = 0x09,   // 11倍
    Mul12 = 0x0A,   // 12倍
    Mul13 = 0x0B,   // 13倍
    Mul14 = 0x0C,   // 14倍
    Mul15 = 0x0D,   // 15倍
    Mul16 = 0x0E,   // 16倍
}

/// RCC AHB预分频系数枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RccAhbPrescaler {
    Div1 = 0x00,    // 1分频
    Div2 = 0x08,    // 2分频
    Div4 = 0x09,    // 4分频
    Div8 = 0x0A,    // 8分频
    Div16 = 0x0B,   // 16分频
    Div64 = 0x0C,   // 64分频
    Div128 = 0x0D,  // 128分频
    Div256 = 0x0E,  // 256分频
    Div512 = 0x0F,  // 512分频
}

/// RCC APB1预分频系数枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RccApb1Prescaler {
    Div1 = 0x00,    // 1分频
    Div2 = 0x04,    // 2分频
    Div4 = 0x05,    // 4分频
    Div8 = 0x06,    // 8分频
    Div16 = 0x07,   // 16分频
}

/// RCC APB2预分频系数枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RccApb2Prescaler {
    Div1 = 0x00,    // 1分频
    Div2 = 0x04,    // 2分频
    Div4 = 0x05,    // 4分频
    Div8 = 0x06,    // 8分频
    Div16 = 0x07,   // 16分频
}

/// RCC结构体
pub struct RccDriver {
    /// HSE频率，单位Hz
    hse_frequency: u32,
}

impl Default for RccDriver {
    fn default() -> Self {
        Self {
            hse_frequency: 8_000_000, // 默认HSE频率为8MHz
        }
    }
}

/// 系统时钟配置结构体
pub struct SystemClockConfig {
    pub clock_source: RccClockSource,
    pub hse_enabled: bool,
    pub hse_frequency: u32, // HSE频率，单位Hz
    pub hse_bypass: bool,   // HSE旁路模式
    pub pll_source: Option<RccPllSource>,
    pub pll_mul: Option<RccPllMul>,
    pub ahb_prescaler: RccAhbPrescaler,
    pub apb1_prescaler: RccApb1Prescaler,
    pub apb2_prescaler: RccApb2Prescaler,
}

/// 时钟频率结构体
pub struct RccClocks {
    pub sysclk_frequency: u32,  // 系统时钟频率，单位Hz
    pub hclk_frequency: u32,    // AHB时钟频率，单位Hz
    pub pclk1_frequency: u32,   // APB1时钟频率，单位Hz
    pub pclk2_frequency: u32,   // APB2时钟频率，单位Hz
    pub adcclk_frequency: u32,  // ADC时钟频率，单位Hz
}

/// RTC时钟源枚举
pub enum RtcClockSource {
    LSE,            // 外部低速时钟
    LSI,            // 内部低速时钟
    HseDiv128,      // HSE除以128
}

impl RccDriver {
    /// 创建新的RCC实例
    pub const fn new() -> Self {
        Self {
            hse_frequency: 8_000_000, // 默认HSE频率为8MHz
        }
    }
    
    /// 创建带有自定义HSE频率的RCC实例
    pub const fn new_with_hse_freq(hse_freq: u32) -> Self {
        Self {
            hse_frequency: hse_freq,
        }
    }
    
    /// 设置HSE频率
    pub fn set_hse_frequency(&mut self, freq: u32) {
        self.hse_frequency = freq;
    }
    
    /// 获取HSE频率
    pub fn get_hse_frequency(&self) -> u32 {
        self.hse_frequency
    }
    
    /// 获取RCC寄存器块
    unsafe fn get_rcc(&self) -> &'static mut Rcc {
        &mut *(0x40021000 as *mut Rcc)
    }
    
    /// 复位RCC配置
    pub unsafe fn reset(&self) {
        let rcc = self.get_rcc();
        
        // 启用内部高速时钟
        let mut cr_value = rcc.cr().read().bits();
        cr_value |= (1 << 0);
        rcc.cr().write(|w: &mut stm32f103::rcc::cr::W| unsafe { w.bits(cr_value) });
        
        // 重置CFGR寄存器
        rcc.cfgr().write(|w: &mut stm32f103::rcc::cfgr::W| unsafe { w.bits(0x00000000) });
        
        // 重置CR寄存器（保留HSION位）
        cr_value = rcc.cr().read().bits();
        cr_value &= 0xFEF6FFFF;
        rcc.cr().write(|w: &mut stm32f103::rcc::cr::W| unsafe { w.bits(cr_value) });
        
        // 重置BORCR寄存器
        cr_value = rcc.cr().read().bits();
        cr_value &= 0xFFFBFFFF;
        rcc.cr().write(|w: &mut stm32f103::rcc::cr::W| unsafe { w.bits(cr_value) });
        
        // 重置CFGR寄存器的PLL相关位
        let mut cfgr_value = rcc.cfgr().read().bits();
        cfgr_value &= 0xFF80FFFF;
        rcc.cfgr().write(|w: &mut stm32f103::rcc::cfgr::W| unsafe { w.bits(cfgr_value) });
        
        // 禁用所有中断
        rcc.cir().write(|w: &mut stm32f103::rcc::cir::W| unsafe { w.bits(0x009F0000) });
    }
    
    /// 启用HSI（内部高速时钟）
    pub unsafe fn enable_hsi(&self) {
        let rcc = self.get_rcc();
        
        let mut value = rcc.cr().read().bits();
        value |= (1 << 0);
        rcc.cr().write(|w: &mut stm32f103::rcc::cr::W| unsafe { w.bits(value) });
        
        // 等待HSI就绪
        while (rcc.cr().read().bits() & (1 << 1)) == 0 {
            core::hint::spin_loop();
        }
    }
    
    /// 禁用HSI（内部高速时钟）
    pub unsafe fn disable_hsi(&self) {
        let rcc = self.get_rcc();
        let mut value = rcc.cr().read().bits();
        value &= !(1 << 0);
        rcc.cr().write(|w: &mut stm32f103::rcc::cr::W| unsafe { w.bits(value) });
    }
    
    /// 启用HSE（外部高速时钟）
    pub unsafe fn enable_hse(&self) {
        let rcc = self.get_rcc();
        
        let mut value = rcc.cr().read().bits();
        value |= (1 << 16);
        rcc.cr().write(|w: &mut stm32f103::rcc::cr::W| unsafe { w.bits(value) });
        
        // 等待HSE就绪
        while (rcc.cr().read().bits() & (1 << 17)) == 0 {
            core::hint::spin_loop();
        }
    }
    
    /// 启用HSE（外部高速时钟），支持旁路模式
    pub unsafe fn enable_hse_with_bypass(&self, bypass: bool) {
        let rcc = self.get_rcc();
        
        let mut value = rcc.cr().read().bits();
        // 清除HSEON和HSEBYP位
        value &= !(0x00050000);
        
        // 设置HSEON位
        value |= (1 << 16);
        
        // 如果需要旁路模式，设置HSEBYP位
        if bypass {
            value |= (1 << 18);
        }
        
        rcc.cr().write(|w: &mut stm32f103::rcc::cr::W| unsafe { w.bits(value) });
        
        // 等待HSE就绪
        while (rcc.cr().read().bits() & (1 << 17)) == 0 {
            core::hint::spin_loop();
        }
    }
    
    /// 禁用HSE（外部高速时钟）
    pub unsafe fn disable_hse(&self) {
        let rcc = self.get_rcc();
        let mut value = rcc.cr().read().bits();
        value &= !(1 << 16);
        rcc.cr().write(|w: &mut stm32f103::rcc::cr::W| unsafe { w.bits(value) });
    }
    
    /// 检查HSE是否处于旁路模式
    pub unsafe fn is_hse_bypassed(&self) -> bool {
        let rcc = self.get_rcc();
        (rcc.cr().read().bits() & (1 << 18)) != 0
    }
    
    /// 启用PLL（锁相环）
    pub unsafe fn enable_pll(&self) {
        let rcc = self.get_rcc();
        
        let mut value = rcc.cr().read().bits();
        value |= (1 << 24);
        rcc.cr().write(|w: &mut stm32f103::rcc::cr::W| unsafe { w.bits(value) });
        
        // 等待PLL就绪
        while (rcc.cr().read().bits() & (1 << 25)) == 0 {
            core::hint::spin_loop();
        }
    }
    
    /// 禁用PLL（锁相环）
    pub unsafe fn disable_pll(&self) {
        let rcc = self.get_rcc();
        let mut value = rcc.cr().read().bits();
        value &= !(1 << 24);
        rcc.cr().write(|w: &mut stm32f103::rcc::cr::W| unsafe { w.bits(value) });
    }
    
    /// 配置PLL
    pub unsafe fn configure_pll(&self, source: RccPllSource, mul: RccPllMul) {
        // 禁用PLL
        self.disable_pll();
        
        let rcc = self.get_rcc();
        
        // 配置PLL源和倍频系数
        let mut value = rcc.cfgr().read().bits();
        // 清除PLL相关位
        value &= !((1 << 16) | (1 << 17) | 0x0F << 18);
        
        // 设置PLL源
        match source {
            RccPllSource::HsiDiv2 => {
                // HSI/2作为PLL输入，不需要设置PLLSRC位
            }
            RccPllSource::Hse => {
                // HSE作为PLL输入，设置PLLSRC位，清除PLLXTPRE位
                value |= (1 << 16);
            }
            RccPllSource::HseDiv2 => {
                // HSE/2作为PLL输入，设置PLLSRC和PLLXTPRE位
                value |= (1 << 16) | (1 << 17);
            }
        }
        
        // 设置PLL倍频系数
        value |= (mul as u32) << 18;
        
        rcc.cfgr().write(|w: &mut stm32f103::rcc::cfgr::W| unsafe { w.bits(value) });
    }
    
    /// 设置系统时钟源
    pub unsafe fn set_system_clock_source(&self, source: RccClockSource) {
        let rcc = self.get_rcc();
        
        // 设置系统时钟源
        let mut value = rcc.cfgr().read().bits();
        // 清除SW位
        value &= !0x03;
        
        // 设置SW位
        match source {
            RccClockSource::HSI => {
                // HSI作为系统时钟，SW=00
            }
            RccClockSource::HSE => {
                // HSE作为系统时钟，SW=01
                value |= 0x01;
            }
            RccClockSource::PLL => {
                // PLL作为系统时钟，SW=10
                value |= 0x02;
            }
        }
        
        rcc.cfgr().write(|w: &mut stm32f103::rcc::cfgr::W| unsafe { w.bits(value) });
        
        // 等待系统时钟切换完成
        while {
            let cfgr = rcc.cfgr().read().bits();
            let sws = (cfgr >> 2) & 0x03;
            match source {
                RccClockSource::HSI => sws != 0x00,
                RccClockSource::HSE => sws != 0x01,
                RccClockSource::PLL => sws != 0x02,
            }
        } {
            core::hint::spin_loop();
        }
    }
    
    /// 配置完整的系统时钟树
    pub unsafe fn configure_system_clock(&mut self, config: SystemClockConfig) {
        // 更新HSE频率
        if config.hse_enabled {
            self.hse_frequency = config.hse_frequency;
        }
        
        // 1. 启用必要的时钟源
        match config.clock_source {
            RccClockSource::HSI => {
                self.enable_hsi();
            }
            RccClockSource::HSE => {
                if config.hse_enabled {
                    self.enable_hse_with_bypass(config.hse_bypass);
                }
            }
            RccClockSource::PLL => {
                // 确保PLL源已启用
                if let Some(pll_source) = config.pll_source {
                    match pll_source {
                        RccPllSource::HsiDiv2 => {
                            self.enable_hsi();
                        }
                        RccPllSource::Hse | RccPllSource::HseDiv2 => {
                            if config.hse_enabled {
                                self.enable_hse_with_bypass(config.hse_bypass);
                            }
                        }
                    }
                }
            }
        }
        
        // 2. 配置PLL（如果需要）
        if let (Some(pll_source), Some(pll_mul)) = (config.pll_source, config.pll_mul) {
            self.configure_pll(pll_source, pll_mul);
            self.enable_pll();
        }
        
        // 3. 配置预分频器
        self.set_ahb_prescaler(config.ahb_prescaler);
        self.set_apb1_prescaler(config.apb1_prescaler);
        self.set_apb2_prescaler(config.apb2_prescaler);
        
        // 4. 设置系统时钟源
        self.set_system_clock_source(config.clock_source);
    }
    
    /// 配置AHB预分频系数
    pub unsafe fn set_ahb_prescaler(&self, prescaler: RccAhbPrescaler) {
        let rcc = self.get_rcc();
        
        let mut value = rcc.cfgr().read().bits();
        // 清除HPRE位
        value &= !0xF0;
        
        // 设置HPRE位
        value |= (prescaler as u32) << 4;
        
        rcc.cfgr().write(|w: &mut stm32f103::rcc::cfgr::W| unsafe { w.bits(value) });
    }
    
    /// 配置APB1预分频系数
    pub unsafe fn set_apb1_prescaler(&self, prescaler: RccApb1Prescaler) {
        let rcc = self.get_rcc();
        
        let mut value = rcc.cfgr().read().bits();
        // 清除PPRE1位
        value &= !0x700;
        
        // 设置PPRE1位
        value |= (prescaler as u32) << 8;
        
        rcc.cfgr().write(|w: &mut stm32f103::rcc::cfgr::W| unsafe { w.bits(value) });
    }
    
    /// 配置APB2预分频系数
    pub unsafe fn set_apb2_prescaler(&self, prescaler: RccApb2Prescaler) {
        let rcc = self.get_rcc();
        
        let mut value = rcc.cfgr().read().bits();
        // 清除PPRE2位
        value &= !0x3800;
        
        // 设置PPRE2位
        value |= (prescaler as u32) << 11;
        
        rcc.cfgr().write(|w: &mut stm32f103::rcc::cfgr::W| unsafe { w.bits(value) });
    }
    
    /// 启用AHB外设时钟
    pub unsafe fn enable_ahb_peripheral(&self, peripheral: AhbPeripheral) {
        let rcc = self.get_rcc();
        let mut value = rcc.ahbenr().read().bits();
        value |= peripheral as u32;
        rcc.ahbenr().write(|w: &mut stm32f103::rcc::ahbenr::W| unsafe { w.bits(value) });
    }
    
    /// 禁用AHB外设时钟
    pub unsafe fn disable_ahb_peripheral(&self, peripheral: AhbPeripheral) {
        let rcc = self.get_rcc();
        let mut value = rcc.ahbenr().read().bits();
        value &= !(peripheral as u32);
        rcc.ahbenr().write(|w: &mut stm32f103::rcc::ahbenr::W| unsafe { w.bits(value) });
    }
    
    /// 启用APB1外设时钟
    pub unsafe fn enable_apb1_peripheral(&self, peripheral: Apb1Peripheral) {
        let rcc = self.get_rcc();
        let mut value = rcc.apb1enr().read().bits();
        value |= peripheral as u32;
        rcc.apb1enr().write(|w: &mut stm32f103::rcc::apb1enr::W| unsafe { w.bits(value) });
    }
    
    /// 禁用APB1外设时钟
    pub unsafe fn disable_apb1_peripheral(&self, peripheral: Apb1Peripheral) {
        let rcc = self.get_rcc();
        let mut value = rcc.apb1enr().read().bits();
        value &= !(peripheral as u32);
        rcc.apb1enr().write(|w: &mut stm32f103::rcc::apb1enr::W| unsafe { w.bits(value) });
    }
    
    /// 启用APB2外设时钟
    pub unsafe fn enable_apb2_peripheral(&self, peripheral: Apb2Peripheral) {
        let rcc = self.get_rcc();
        let mut value = rcc.apb2enr().read().bits();
        value |= peripheral as u32;
        rcc.apb2enr().write(|w: &mut stm32f103::rcc::apb2enr::W| unsafe { w.bits(value) });
    }
    
    /// 禁用APB2外设时钟
    pub unsafe fn disable_apb2_peripheral(&self, peripheral: Apb2Peripheral) {
        let rcc = self.get_rcc();
        let mut value = rcc.apb2enr().read().bits();
        value &= !(peripheral as u32);
        rcc.apb2enr().write(|w: &mut stm32f103::rcc::apb2enr::W| unsafe { w.bits(value) });
    }
    
    /// 复位APB1外设
    pub unsafe fn reset_apb1_peripheral(&self, peripheral: Apb1Peripheral) {
        let rcc = self.get_rcc();
        rcc.apb1rstr().write(|w: &mut stm32f103::rcc::apb1rstr::W| unsafe { w.bits(peripheral as u32) });
        rcc.apb1rstr().write(|w: &mut stm32f103::rcc::apb1rstr::W| unsafe { w.bits(0) });
    }
    
    /// 复位APB2外设
    pub unsafe fn reset_apb2_peripheral(&self, peripheral: Apb2Peripheral) {
        let rcc = self.get_rcc();
        rcc.apb2rstr().write(|w: &mut stm32f103::rcc::apb2rstr::W| unsafe { w.bits(peripheral as u32) });
        rcc.apb2rstr().write(|w: &mut stm32f103::rcc::apb2rstr::W| unsafe { w.bits(0) });
    }
    
    /// 获取系统时钟频率
    pub unsafe fn get_system_clock_frequency(&self) -> u32 {
        let rcc = self.get_rcc();
        let cfgr = rcc.cfgr().read().bits();
        let clock_source = (cfgr >> 2) & 0x03;
        
        match clock_source {
            0x00 => {
                // HSI作为系统时钟，频率为8MHz
                8_000_000
            }
            0x01 => {
                // HSE作为系统时钟，频率取决于外部晶振
                self.hse_frequency
            }
            0x02 => {
                // PLL作为系统时钟
                let pll_source = (cfgr >> 16) & 0x01;
                let pll_hse_div2 = (cfgr >> 17) & 0x01;
                let pll_mul = ((cfgr >> 18) & 0x0F) as u32 + 2;
                
                if pll_source == 0 {
                    // HSI/2作为PLL输入
                    (8_000_000 / 2) * pll_mul
                } else {
                    if pll_hse_div2 == 1 {
                        // HSE/2作为PLL输入
                        (self.hse_frequency / 2) * pll_mul
                    } else {
                        // HSE作为PLL输入
                        self.hse_frequency * pll_mul
                    }
                }
            }
            _ => {
                // 未知时钟源，返回默认值
                8_000_000
            }
        }
    }
    
    /// 获取所有时钟频率
    pub unsafe fn get_clocks_freq(&self) -> RccClocks {
        let rcc = self.get_rcc();
        let cfgr = rcc.cfgr().read().bits();
        
        // 获取系统时钟频率
        let sysclk_frequency = self.get_system_clock_frequency();
        
        // 获取AHB预分频系数
        let ahb_prescaler = (cfgr >> 4) & 0x0F;
        let hclk_frequency = match ahb_prescaler {
            0x00 => sysclk_frequency,
            0x08 => sysclk_frequency / 2,
            0x09 => sysclk_frequency / 4,
            0x0A => sysclk_frequency / 8,
            0x0B => sysclk_frequency / 16,
            0x0C => sysclk_frequency / 64,
            0x0D => sysclk_frequency / 128,
            0x0E => sysclk_frequency / 256,
            0x0F => sysclk_frequency / 512,
            _ => sysclk_frequency,
        };
        
        // 获取APB1预分频系数
        let apb1_prescaler = (cfgr >> 8) & 0x07;
        let pclk1_frequency = match apb1_prescaler {
            0x00 => hclk_frequency,
            0x04 => hclk_frequency / 2,
            0x05 => hclk_frequency / 4,
            0x06 => hclk_frequency / 8,
            0x07 => hclk_frequency / 16,
            _ => hclk_frequency,
        };
        
        // 获取APB2预分频系数
        let apb2_prescaler = (cfgr >> 11) & 0x07;
        let pclk2_frequency = match apb2_prescaler {
            0x00 => hclk_frequency,
            0x04 => hclk_frequency / 2,
            0x05 => hclk_frequency / 4,
            0x06 => hclk_frequency / 8,
            0x07 => hclk_frequency / 16,
            _ => hclk_frequency,
        };
        
        // 获取ADC预分频系数
        let adc_prescaler = (cfgr >> 14) & 0x03;
        let adcclk_frequency = match adc_prescaler {
            0x00 => pclk2_frequency / 2,
            0x01 => pclk2_frequency / 4,
            0x02 => pclk2_frequency / 6,
            0x03 => pclk2_frequency / 8,
            _ => pclk2_frequency / 2,
        };
        
        RccClocks {
            sysclk_frequency,
            hclk_frequency,
            pclk1_frequency,
            pclk2_frequency,
            adcclk_frequency,
        }
    }
    
    /// 检查HSI是否就绪
    pub unsafe fn is_hsi_ready(&self) -> bool {
        let rcc = self.get_rcc();
        (rcc.cr().read().bits() & (1 << 1)) != 0
    }
    
    /// 检查HSE是否就绪
    pub unsafe fn is_hse_ready(&self) -> bool {
        let rcc = self.get_rcc();
        (rcc.cr().read().bits() & (1 << 17)) != 0
    }
    
    /// 检查PLL是否就绪
    pub unsafe fn is_pll_ready(&self) -> bool {
        let rcc = self.get_rcc();
        (rcc.cr().read().bits() & (1 << 25)) != 0
    }
    
    /// 获取当前系统时钟源
    pub unsafe fn get_current_system_clock_source(&self) -> RccClockSource {
        let rcc = self.get_rcc();
        let cfgr = rcc.cfgr().read().bits();
        let sws = (cfgr >> 2) & 0x03;
        
        match sws {
            0x00 => RccClockSource::HSI,
            0x01 => RccClockSource::HSE,
            0x02 => RccClockSource::PLL,
            _ => RccClockSource::HSI, // 默认返回HSI
        }
    }
    
    /// 等待HSI就绪
    pub unsafe fn wait_for_hsi_ready(&self) {
        while !self.is_hsi_ready() {
            core::hint::spin_loop();
        }
    }
    
    /// 等待HSE就绪
    pub unsafe fn wait_for_hse_ready(&self) {
        while !self.is_hse_ready() {
            core::hint::spin_loop();
        }
    }
    
    /// 等待PLL就绪
    pub unsafe fn wait_for_pll_ready(&self) {
        while !self.is_pll_ready() {
            core::hint::spin_loop();
        }
    }
    
    /// 配置USB时钟
    /// freq: USB时钟频率，单位MHz，通常为48MHz
    pub unsafe fn configure_usb_clock(&self, freq: u32) {
        let rcc = self.get_rcc();
        
        // 配置USB预分频器
        let mut value = rcc.cfgr().read().bits();
        // 清除USBPRE位
        value &= !0x10000;
        
        // USB时钟频率 = PLL时钟 / USBPRE
        // 如果PLL时钟为72MHz，则USBPRE=1，USB时钟=72/1.5=48MHz
        // 如果PLL时钟为96MHz，则USBPRE=0，USB时钟=96/2=48MHz
        if freq == 48_000_000 {
            // 检查PLL时钟频率
            let pll_clock = self.get_system_clock_frequency();
            if pll_clock == 72_000_000 {
                // USBPRE=1，USB时钟=72/1.5=48MHz
                value |= 0x10000;
            } else if pll_clock == 96_000_000 {
                // USBPRE=0，USB时钟=96/2=48MHz
                value &= !0x10000;
            }
        }
        
        rcc.cfgr().write(|w: &mut stm32f103::rcc::cfgr::W| unsafe { w.bits(value) });
    }
    
    /// 配置ADC时钟
    /// prescaler: ADC预分频系数，可选值：2, 4, 6, 8
    pub unsafe fn configure_adc_clock(&self, prescaler: u32) {
        let rcc = self.get_rcc();
        
        // 配置ADC预分频器
        let mut value = rcc.cfgr().read().bits();
        // 清除ADCPRE位
        value &= !0xC00000;
        
        // 设置ADCPRE位
        match prescaler {
            2 => {
                // ADC时钟 = PCLK2 / 2
                value |= 0x000000;
            }
            4 => {
                // ADC时钟 = PCLK2 / 4
                value |= 0x400000;
            }
            6 => {
                // ADC时钟 = PCLK2 / 6
                value |= 0x800000;
            }
            8 => {
                // ADC时钟 = PCLK2 / 8
                value |= 0xC00000;
            }
            _ => {
                // 默认使用2分频
                value |= 0x000000;
            }
        }
        
        rcc.cfgr().write(|w: &mut stm32f103::rcc::cfgr::W| unsafe { w.bits(value) });
    }
    
    /// 配置MCO（微控制器时钟输出）
    /// source: MCO时钟源
    /// prescaler: MCO预分频系数，可选值：1, 2, 4, 8
    pub unsafe fn configure_mco(&self, source: RccClockSource, prescaler: u32) {
        let rcc = self.get_rcc();
        
        // 配置MCO
        let mut value = rcc.cfgr().read().bits();
        // 清除MCO位
        value &= !0x7F000000;
        
        // 设置MCO源
        match source {
            RccClockSource::HSI => {
                // HSI作为MCO源
                value |= 0x00000000;
            }
            RccClockSource::HSE => {
                // HSE作为MCO源
                value |= 0x40000000;
            }
            RccClockSource::PLL => {
                // PLL作为MCO源
                value |= 0x80000000;
            }
        }
        
        // 设置MCO预分频系数
        match prescaler {
            1 => {
                // MCO不分频
                value &= !0x30000000;
            }
            2 => {
                // MCO 2分频
                value |= 0x10000000;
            }
            4 => {
                // MCO 4分频
                value |= 0x20000000;
            }
            8 => {
                // MCO 8分频
                value |= 0x30000000;
            }
            _ => {
                // 默认不分频
                value &= !0x30000000;
            }
        }
        
        rcc.cfgr().write(|w: &mut stm32f103::rcc::cfgr::W| unsafe { w.bits(value) });
    }
    
    /// 启用HSI就绪中断
    pub unsafe fn enable_hsi_ready_interrupt(&self) {
        let rcc = self.get_rcc();
        let mut value = rcc.cir().read().bits();
        value |= 0x00000001; // 置位HSIRDYIE位
        rcc.cir().write(|w: &mut stm32f103::rcc::cir::W| unsafe { w.bits(value) });
    }
    
    /// 禁用HSI就绪中断
    pub unsafe fn disable_hsi_ready_interrupt(&self) {
        let rcc = self.get_rcc();
        let mut value = rcc.cir().read().bits();
        value &= !0x00000001; // 清除HSIRDYIE位
        rcc.cir().write(|w: &mut stm32f103::rcc::cir::W| unsafe { w.bits(value) });
    }
    
    /// 启用HSE就绪中断
    pub unsafe fn enable_hse_ready_interrupt(&self) {
        let rcc = self.get_rcc();
        let mut value = rcc.cir().read().bits();
        value |= 0x00000008; // 置位HSERDYIE位
        rcc.cir().write(|w: &mut stm32f103::rcc::cir::W| unsafe { w.bits(value) });
    }
    
    /// 禁用HSE就绪中断
    pub unsafe fn disable_hse_ready_interrupt(&self) {
        let rcc = self.get_rcc();
        let mut value = rcc.cir().read().bits();
        value &= !0x00000008; // 清除HSERDYIE位
        rcc.cir().write(|w: &mut stm32f103::rcc::cir::W| unsafe { w.bits(value) });
    }
    
    /// 启用PLL就绪中断
    pub unsafe fn enable_pll_ready_interrupt(&self) {
        let rcc = self.get_rcc();
        let mut value = rcc.cir().read().bits();
        value |= 0x00000020; // 置位PLLRDYIE位
        rcc.cir().write(|w: &mut stm32f103::rcc::cir::W| unsafe { w.bits(value) });
    }
    
    /// 禁用PLL就绪中断
    pub unsafe fn disable_pll_ready_interrupt(&self) {
        let rcc = self.get_rcc();
        let mut value = rcc.cir().read().bits();
        value &= !0x00000020; // 清除PLLRDYIE位
        rcc.cir().write(|w: &mut stm32f103::rcc::cir::W| unsafe { w.bits(value) });
    }
    
    /// 启用时钟安全系统（CSS）
    /// 当HSE时钟失效时，会自动切换到HSI时钟并产生中断
    pub unsafe fn enable_clock_security_system(&self) {
        let rcc = self.get_rcc();
        let mut value = rcc.cr().read().bits();
        value |= 0x00000080; // 置位CSSON位
        rcc.cr().write(|w: &mut stm32f103::rcc::cr::W| unsafe { w.bits(value) });
    }
    
    /// 禁用时钟安全系统（CSS）
    pub unsafe fn disable_clock_security_system(&self) {
        let rcc = self.get_rcc();
        let mut value = rcc.cr().read().bits();
        value &= !0x00000080; // 清除CSSON位
        rcc.cr().write(|w: &mut stm32f103::rcc::cr::W| unsafe { w.bits(value) });
    }
    
    /// 清除所有时钟中断标志
    pub unsafe fn clear_all_interrupt_flags(&self) {
        let rcc = self.get_rcc();
        // 写入1清除所有中断标志
        rcc.cir().write(|w: &mut stm32f103::rcc::cir::W| unsafe { w.bits(0x009F0000) });
    }
    
    /// 调整HSI校准值
    /// calibration_value: 校准值，范围0-31
    pub unsafe fn adjust_hsi_calibration(&self, calibration_value: u8) {
        let rcc = self.get_rcc();
        
        // 检查校准值范围
        if calibration_value > 31 {
            return;
        }
        
        let mut value = rcc.cr().read().bits();
        // 清除HSITRIM位
        value &= !0x000000F8;
        // 设置新的校准值
        value |= (calibration_value as u32) << 3;
        rcc.cr().write(|w: &mut stm32f103::rcc::cr::W| unsafe { w.bits(value) });
    }
    
    /// 复位备份域
    pub unsafe fn reset_backup_domain(&self) {
        // 启用PWR和BKP时钟
        self.enable_apb1_peripheral(Apb1Peripheral::PWR);
        self.enable_apb1_peripheral(Apb1Peripheral::BKP);
        
        // 解锁备份域访问
        let pwr = &mut *(0x40007000 as *mut stm32f103::Pwr);
        pwr.cr().write(|w: &mut stm32f103::pwr::cr::W| unsafe { w.bits(0x10) });
        
        // 注意：当前stm32f103库可能不支持bdcr寄存器访问
        // 这里暂时省略备份域复位操作
        
        // 锁定备份域访问
        pwr.cr().write(|w: &mut stm32f103::pwr::cr::W| unsafe { w.bits(0x00) });
    }
    
    /// 启用LSI（内部低速时钟）
    pub unsafe fn enable_lsi(&self) {
        let rcc = self.get_rcc();
        
        let mut value = rcc.csr().read().bits();
        value |= 0x00000001;
        rcc.csr().write(|w: &mut stm32f103::rcc::csr::W| unsafe { w.bits(value) });
        
        // 等待LSI就绪
        while (rcc.csr().read().bits() & 0x00000002) == 0 {
            core::hint::spin_loop();
        }
    }
    
    /// 禁用LSI（内部低速时钟）
    pub unsafe fn disable_lsi(&self) {
        let rcc = self.get_rcc();
        
        let mut value = rcc.csr().read().bits();
        value &= !0x00000001;
        rcc.csr().write(|w: &mut stm32f103::rcc::csr::W| unsafe { w.bits(value) });
    }
    
    /// 检查LSI是否就绪
    pub unsafe fn is_lsi_ready(&self) -> bool {
        let rcc = self.get_rcc();
        (rcc.csr().read().bits() & 0x00000002) != 0
    }
    
    /// 启用LSE（外部低速时钟）
    /// 注意：当前stm32f103库可能不支持bdcr寄存器访问
    pub unsafe fn enable_lse(&self) {
        // 启用PWR和BKP时钟
        self.enable_apb1_peripheral(Apb1Peripheral::PWR);
        self.enable_apb1_peripheral(Apb1Peripheral::BKP);
        
        // 解锁备份域访问
        let pwr = &mut *(0x40007000 as *mut stm32f103::Pwr);
        pwr.cr().write(|w: &mut stm32f103::pwr::cr::W| unsafe { w.bits(0x10) });
        
        // 注意：当前stm32f103库可能不支持bdcr寄存器访问
        // 这里暂时省略LSE启用操作
        
        // 锁定备份域访问
        pwr.cr().write(|w: &mut stm32f103::pwr::cr::W| unsafe { w.bits(0x00) });
    }
    
    /// 禁用LSE（外部低速时钟）
    /// 注意：当前stm32f103库可能不支持bdcr寄存器访问
    pub unsafe fn disable_lse(&self) {
        // 启用PWR和BKP时钟
        self.enable_apb1_peripheral(Apb1Peripheral::PWR);
        self.enable_apb1_peripheral(Apb1Peripheral::BKP);
        
        // 解锁备份域访问
        let pwr = &mut *(0x40007000 as *mut stm32f103::Pwr);
        pwr.cr().write(|w: &mut stm32f103::pwr::cr::W| unsafe { w.bits(0x10) });
        
        // 注意：当前stm32f103库可能不支持bdcr寄存器访问
        // 这里暂时省略LSE禁用操作
        
        // 锁定备份域访问
        pwr.cr().write(|w: &mut stm32f103::pwr::cr::W| unsafe { w.bits(0x00) });
    }
    
    /// 检查LSE是否就绪
    /// 注意：当前stm32f103库可能不支持bdcr寄存器访问
    pub unsafe fn is_lse_ready(&self) -> bool {
        // 注意：当前stm32f103库可能不支持bdcr寄存器访问
        // 这里暂时返回false
        false
    }
}

/// AHB外设枚举
pub enum AhbPeripheral {
    DMA1 = 1 << 0,
    DMA2 = 1 << 1,
    SRAM = 1 << 2,
    FLITF = 1 << 4,
    CRC = 1 << 6,
    FSMC = 1 << 8,
    SDIO = 1 << 10,
}

/// APB1外设枚举
pub enum Apb1Peripheral {
    TIM2 = 1 << 0,
    TIM3 = 1 << 1,
    TIM4 = 1 << 2,
    TIM5 = 1 << 3,
    TIM6 = 1 << 4,
    TIM7 = 1 << 5,
    TIM12 = 1 << 6,
    TIM13 = 1 << 7,
    TIM14 = 1 << 8,
    WWDG = 1 << 11,
    SPI2 = 1 << 14,
    SPI3 = 1 << 15,
    USART2 = 1 << 17,
    USART3 = 1 << 18,
    UART4 = 1 << 19,
    UART5 = 1 << 20,
    I2C1 = 1 << 21,
    I2C2 = 1 << 22,
    USB = 1 << 23,
    CAN = 1 << 25,
    BKP = 1 << 27,
    PWR = 1 << 28,
    DAC = 1 << 29,
}

/// APB2外设枚举
pub enum Apb2Peripheral {
    AFIO = 1 << 0,
    GPIOA = 1 << 2,
    GPIOB = 1 << 3,
    GPIOC = 1 << 4,
    GPIOD = 1 << 5,
    GPIOE = 1 << 6,
    GPIOF = 1 << 7,
    GPIOG = 1 << 8,
    ADC1 = 1 << 9,
    ADC2 = 1 << 10,
    TIM1 = 1 << 11,
    SPI1 = 1 << 12,
    TIM8 = 1 << 13,
    USART1 = 1 << 14,
    ADC3 = 1 << 15,
    TIM9 = 1 << 19,
    TIM10 = 1 << 20,
    TIM11 = 1 << 21,
}

/// 预定义的RCC实例
pub const RCC_DRIVER: RccDriver = RccDriver {
    hse_frequency: 8_000_000,
};
