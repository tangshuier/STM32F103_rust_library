//! WWDG模块
//! 提供窗口看门狗功能封装

#![allow(unused)]

// 导入内部生成的设备驱动库
use stm32f103::*;

/// WWDG预分频系数枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WwdgPrescaler {
    Div1 = 0x00,    // 1分频
    Div2 = 0x01,    // 2分频
    Div4 = 0x02,    // 4分频
    Div8 = 0x03,    // 8分频
}

/// WWDG结构体
pub struct Wwdg;

impl Wwdg {
    /// 创建新的WWDG实例
    pub const fn new() -> Self {
        Self
    }
    
    /// 获取WWDG寄存器块
    unsafe fn wwdg() -> &'static mut stm32f103::wwdg::RegisterBlock {
        &mut *(0x40002C00 as *mut stm32f103::wwdg::RegisterBlock)
    }
    
    /// 初始化WWDG
    /// 
    /// # 参数
    /// * `prescaler` - 预分频系数
    /// * `window` - 窗口值 (0x40-0x7F)
    /// * `counter` - 计数器值 (0x40-0x7F)
    pub unsafe fn init(&self, prescaler: WwdgPrescaler, window: u8, counter: u8) {
        // 检查参数范围
        assert!((window & 0xC0) == 0, "Window value must be between 0x40 and 0x7F");
        assert!((counter & 0xC0) == 0, "Counter value must be between 0x40 and 0x7F");
        
        let wwdg = Wwdg::wwdg();
        
        // 配置预分频系数和窗口值
        wwdg.cfr().write(|w: &mut stm32f103::wwdg::cfr::W| unsafe {
            w
                .wdgtb().bits(prescaler as u8)
                .w().bits(window)
        });
        
        // 设置计数器值并启用WWDG
        wwdg.cr().write(|w: &mut stm32f103::wwdg::cr::W| unsafe {
            w
                .t().bits(counter)
                .wdga().set_bit()
        });
    }
    
    /// 设置窗口值
    /// 
    /// # 参数
    /// * `window` - 窗口值 (0x40-0x7F)
    pub unsafe fn set_window(&self, window: u8) {
        // 检查参数范围
        assert!((window & 0xC0) == 0, "Window value must be between 0x40 and 0x7F");
        
        let wwdg = Wwdg::wwdg();
        
        // 设置窗口值
        wwdg.cfr().modify(|_, w: &mut stm32f103::wwdg::cfr::W| unsafe {
            w.w().bits(window)
        });
    }
    
    /// 设置预分频系数
    /// 
    /// # 参数
    /// * `prescaler` - 预分频系数
    pub unsafe fn set_prescaler(&self, prescaler: WwdgPrescaler) {
        let wwdg = Wwdg::wwdg();
        
        // 设置预分频系数
        wwdg.cfr().modify(|_, w: &mut stm32f103::wwdg::cfr::W| unsafe {
            w.wdgtb().bits(prescaler as u8)
        });
    }
    
    /// 设置计数器值
    /// 
    /// # 参数
    /// * `counter` - 计数器值 (0x40-0x7F)
    pub unsafe fn set_counter(&self, counter: u8) {
        // 检查参数范围
        assert!((counter & 0xC0) == 0, "Counter value must be between 0x40 and 0x7F");
        
        let wwdg = Wwdg::wwdg();
        
        // 设置计数器值
        wwdg.cr().modify(|_, w: &mut stm32f103::wwdg::cr::W| unsafe {
            w.t().bits(counter)
        });
    }
    
    /// 获取计数器值
    pub unsafe fn get_counter(&self) -> u8 {
        let wwdg = Wwdg::wwdg();
        wwdg.cr().read().t().bits()
    }
    
    /// 喂狗
    /// 
    /// # 参数
    /// * `counter` - 计数器值 (0x40-0x7F)
    pub unsafe fn feed(&self, counter: u8) {
        self.set_counter(counter);
    }
    
    /// 启用早期唤醒中断
    pub unsafe fn enable_ewi(&self) {
        let wwdg = Wwdg::wwdg();
        wwdg.cfr().modify(|_, w: &mut stm32f103::wwdg::cfr::W| {
            w.ewi().set_bit()
        });
    }
    
    /// 禁用早期唤醒中断
    pub unsafe fn disable_ewi(&self) {
        let wwdg = Wwdg::wwdg();
        wwdg.cfr().modify(|_, w: &mut stm32f103::wwdg::cfr::W| {
            w.ewi().clear_bit()
        });
    }
    
    /// 清除早期唤醒中断标志
    pub unsafe fn clear_ewi_flag(&self) {
        let wwdg = Wwdg::wwdg();
        wwdg.sr().write(|w: &mut stm32f103::wwdg::sr::W| {
            w.ewi().clear_bit()
        });
    }
    
    /// 检查早期唤醒中断标志
    pub unsafe fn get_ewi_flag(&self) -> bool {
        let wwdg = Wwdg::wwdg();
        wwdg.sr().read().ewi().bit()
    }
    
    /// 计算超时时间
    /// 
    /// # 参数
    /// * `prescaler` - 预分频系数
    /// * `counter` - 计数器值 (0x40-0x7F)
    /// * `apb1_freq` - APB1时钟频率 (Hz)
    /// 
    /// # 返回值
    /// 超时时间 (ms)
    pub fn calculate_timeout(prescaler: WwdgPrescaler, counter: u8, apb1_freq: u32) -> u32 {
        // WWDG时钟频率 = APB1时钟频率 / 4096
        let wwdg_freq = apb1_freq / 4096;
        
        // 预分频系数
        let prescaler_value = match prescaler {
            WwdgPrescaler::Div1 => 1,
            WwdgPrescaler::Div2 => 2,
            WwdgPrescaler::Div4 => 4,
            WwdgPrescaler::Div8 => 8,
        };
        
        // 计算周期
        let period = (counter & 0x7F) as u32;
        
        // 超时时间 (ms) = (period * prescaler_value * 1000) / wwdg_freq
        (period * prescaler_value * 1000) / wwdg_freq
    }
}

/// 预定义的WWDG实例
pub const WWDG: Wwdg = Wwdg::new();