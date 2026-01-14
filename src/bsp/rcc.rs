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
pub struct RccDriver;

impl RccDriver {
    /// 创建新的RCC实例
    pub const fn new() -> Self {
        Self
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
    
    /// 禁用HSE（外部高速时钟）
    pub unsafe fn disable_hse(&self) {
        let rcc = self.get_rcc();
        let mut value = rcc.cr().read().bits();
        value &= !(1 << 16);
        rcc.cr().write(|w: &mut stm32f103::rcc::cr::W| unsafe { w.bits(value) });
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
    pub unsafe fn enable_ahb_peripheral(&self, peripheral: u32) {
        let rcc = self.get_rcc();
        let mut value = rcc.ahbenr().read().bits();
        value |= peripheral;
        rcc.ahbenr().write(|w: &mut stm32f103::rcc::ahbenr::W| unsafe { w.bits(value) });
    }
    
    /// 禁用AHB外设时钟
    pub unsafe fn disable_ahb_peripheral(&self, peripheral: u32) {
        let rcc = self.get_rcc();
        let mut value = rcc.ahbenr().read().bits();
        value &= !peripheral;
        rcc.ahbenr().write(|w: &mut stm32f103::rcc::ahbenr::W| unsafe { w.bits(value) });
    }
    
    /// 启用APB1外设时钟
    pub unsafe fn enable_apb1_peripheral(&self, peripheral: u32) {
        let rcc = self.get_rcc();
        let mut value = rcc.apb1enr().read().bits();
        value |= peripheral;
        rcc.apb1enr().write(|w: &mut stm32f103::rcc::apb1enr::W| unsafe { w.bits(value) });
    }
    
    /// 禁用APB1外设时钟
    pub unsafe fn disable_apb1_peripheral(&self, peripheral: u32) {
        let rcc = self.get_rcc();
        let mut value = rcc.apb1enr().read().bits();
        value &= !peripheral;
        rcc.apb1enr().write(|w: &mut stm32f103::rcc::apb1enr::W| unsafe { w.bits(value) });
    }
    
    /// 启用APB2外设时钟
    pub unsafe fn enable_apb2_peripheral(&self, peripheral: u32) {
        let rcc = self.get_rcc();
        let mut value = rcc.apb2enr().read().bits();
        value |= peripheral;
        rcc.apb2enr().write(|w: &mut stm32f103::rcc::apb2enr::W| unsafe { w.bits(value) });
    }
    
    /// 禁用APB2外设时钟
    pub unsafe fn disable_apb2_peripheral(&self, peripheral: u32) {
        let rcc = self.get_rcc();
        let mut value = rcc.apb2enr().read().bits();
        value &= !peripheral;
        rcc.apb2enr().write(|w: &mut stm32f103::rcc::apb2enr::W| unsafe { w.bits(value) });
    }
    
    /// 复位APB1外设
    pub unsafe fn reset_apb1_peripheral(&self, peripheral: u32) {
        let rcc = self.get_rcc();
        rcc.apb1rstr().write(|w: &mut stm32f103::rcc::apb1rstr::W| unsafe { w.bits(peripheral) });
        rcc.apb1rstr().write(|w: &mut stm32f103::rcc::apb1rstr::W| unsafe { w.bits(0) });
    }
    
    /// 复位APB2外设
    pub unsafe fn reset_apb2_peripheral(&self, peripheral: u32) {
        let rcc = self.get_rcc();
        rcc.apb2rstr().write(|w: &mut stm32f103::rcc::apb2rstr::W| unsafe { w.bits(peripheral) });
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
                // 这里假设外部晶振为8MHz
                8_000_000
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
                        (8_000_000 / 2) * pll_mul
                    } else {
                        // HSE作为PLL输入
                        8_000_000 * pll_mul
                    }
                }
            }
            _ => {
                // 未知时钟源，返回默认值
                8_000_000
            }
        }
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
pub const RCC_DRIVER: RccDriver = RccDriver::new();
