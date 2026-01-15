//! IWDG模块
//! 提供独立看门狗功能封装

#![allow(unused)]

// 导入内部生成的设备驱动库
use stm32f103::*;

// 键值
const IWDG_KEY_ENABLE: u16 = 0xCCCC;
const IWDG_KEY_FEED: u16 = 0xAAAA;
const IWDG_KEY_WRITE_ACCESS_ENABLE: u16 = 0x5555;

/// IWDG预分频系数枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IwdgPrescaler {
    Div4 = 0,
    Div8 = 1,
    Div16 = 2,
    Div32 = 3,
    Div64 = 4,
    Div128 = 5,
    Div256 = 6,
}

/// IWDG结构体
pub struct Iwdg;

impl Iwdg {
    /// 创建新的IWDG实例
    pub const fn new() -> Self {
        Self
    }
    
    /// 获取IWDG寄存器块
    unsafe fn iwdg(&self) -> &'static mut stm32f103::iwdg::RegisterBlock {
        &mut *(0x40003000 as *mut stm32f103::iwdg::RegisterBlock)
    }
    
    /// 初始化IWDG
    pub unsafe fn init(&self, prescaler: IwdgPrescaler, reload: u16) {
        let iwdg = self.iwdg();
        
        // 启用写入访问
        iwdg.kr().write(|w| w
            .key().bits(IWDG_KEY_WRITE_ACCESS_ENABLE)
        );
        
        // 设置预分频系数
        iwdg.pr().write(|w: &mut stm32f103::iwdg::pr::W| w
            .pr().bits(prescaler as u8)
        );
        
        // 设置重载值
        iwdg.rlr().write(|w: &mut stm32f103::iwdg::rlr::W| w
            .rl().bits(reload)
        );
        
        // 重载计数器
        self.feed();
        
        // 启用IWDG
        iwdg.kr().write(|w| w
            .key().bits(IWDG_KEY_ENABLE)
        );
    }
    
    /// 喂狗（重载计数器）
    pub unsafe fn feed(&self) {
        let iwdg = self.iwdg();
        iwdg.kr().write(|w| w
            .key().bits(IWDG_KEY_FEED)
        );
    }
    
    /// 检查预分频寄存器是否正在更新
    pub unsafe fn is_prescaler_busy(&self) -> bool {
        let iwdg = self.iwdg();
        iwdg.sr().read().pvu().bit_is_set()
    }
    
    /// 检查重载寄存器是否正在更新
    pub unsafe fn is_reload_busy(&self) -> bool {
        let iwdg = self.iwdg();
        iwdg.sr().read().rvu().bit_is_set()
    }
    
    /// 计算看门狗超时时间
    /// 
    /// # 参数
    /// * `prescaler` - 预分频系数
    /// * `reload` - 重载值
    /// 
    /// # 返回值
    /// 超时时间（毫秒）
    pub fn calculate_timeout(prescaler: IwdgPrescaler, reload: u16) -> u32 {
        // 独立看门狗时钟频率为40kHz
        let clk_freq = 40_000;
        
        // 计算预分频后的频率
        let prescaler_value = match prescaler {
            IwdgPrescaler::Div4 => 4,
            IwdgPrescaler::Div8 => 8,
            IwdgPrescaler::Div16 => 16,
            IwdgPrescaler::Div32 => 32,
            IwdgPrescaler::Div64 => 64,
            IwdgPrescaler::Div128 => 128,
            IwdgPrescaler::Div256 => 256,
        };
        
        let tick_freq = clk_freq / prescaler_value;
        let timeout_ms = (reload as u32 * 1000) / tick_freq;
        
        timeout_ms
    }
}

/// 预定义的IWDG实例
pub const IWDG: Iwdg = Iwdg::new();
