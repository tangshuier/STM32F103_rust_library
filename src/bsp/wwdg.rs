//! WWDG模块
//! 提供窗口看门狗功能封装

#![allow(unused)]

// 导入内部生成的设备驱动库
use library::*;

/// WWDG错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WwdgError {
    /// 初始化失败
    InitializationFailed,
    /// 参数无效
    InvalidParameter,
    /// 窗口值超出范围（0x40-0x7F）
    WindowOutOfRange,
    /// 计数器值超出范围（0x40-0x7F）
    CounterOutOfRange,
    /// WWDG已启用
    AlreadyEnabled,
    /// WWDG未启用
    NotEnabled,
    /// 无效的预分频系数
    InvalidPrescaler,
    /// 未知错误
    UnknownError,
}

/// WWDG状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WwdgStatus {
    /// WWDG准备就绪
    Ready,
    /// WWDG正在初始化
    Initializing,
    /// WWDG已启用
    Enabled,
    /// WWDG已禁用
    Disabled,
    /// WWDG出现错误
    Error,
    /// 早期唤醒中断已触发
    EarlyWakeup,
    /// 计数器值较低
    CounterLow,
    /// 窗口违规
    WindowViolation,
}

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
    unsafe fn wwdg() -> &'static mut library::wwdg::RegisterBlock {
        &mut *(0x40002C00 as *mut library::wwdg::RegisterBlock)
    }
    
    /// 初始化WWDG
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// - 调用者必须确保提供的参数有效
    /// 
    /// # 参数
    /// * `prescaler` - 预分频系数
    /// * `window` - 窗口值 (0x40-0x7F)
    /// * `counter` - 计数器值 (0x40-0x7F)
    /// 
    /// # 返回值
    /// - Ok(())：初始化成功
    /// - Err(WwdgError)：初始化失败或参数无效
    pub unsafe fn init(&self, prescaler: WwdgPrescaler, window: u8, counter: u8) -> Result<(), WwdgError> {
        // 检查参数范围
        if (window & 0xC0) != 0 {
            return Err(WwdgError::WindowOutOfRange);
        }
        if (counter & 0xC0) != 0 {
            return Err(WwdgError::CounterOutOfRange);
        }
        
        let wwdg = Wwdg::wwdg();
        
        // 检查WWDG是否已启用
        if wwdg.cr().read().wdga().bit_is_set() {
            return Err(WwdgError::AlreadyEnabled);
        }
        
        // 配置预分频系数和窗口值
        wwdg.cfr().write(|w: &mut library::wwdg::cfr::W| unsafe {
            w
                .wdgtb().bits(prescaler as u8)
                .w().bits(window)
        });
        
        // 设置计数器值并启用WWDG
        wwdg.cr().write(|w: &mut library::wwdg::cr::W| unsafe {
            w
                .t().bits(counter)
                .wdga().set_bit()
        });
        
        Ok(())
    }
    
    /// 设置窗口值
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// * `window` - 窗口值 (0x40-0x7F)
    /// 
    /// # 返回值
    /// - Ok(())：设置成功
    /// - Err(WwdgError)：设置失败或参数无效
    pub unsafe fn set_window(&self, window: u8) -> Result<(), WwdgError> {
        // 检查参数范围
        if (window & 0xC0) != 0 {
            return Err(WwdgError::WindowOutOfRange);
        }
        
        let wwdg = Wwdg::wwdg();
        
        // 设置窗口值
        wwdg.cfr().modify(|_, w: &mut library::wwdg::cfr::W| unsafe {
            w.w().bits(window)
        });
        
        Ok(())
    }
    
    /// 设置预分频系数
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// * `prescaler` - 预分频系数
    /// 
    /// # 返回值
    /// - Ok(())：设置成功
    /// - Err(WwdgError)：设置失败或参数无效
    pub unsafe fn set_prescaler(&self, prescaler: WwdgPrescaler) -> Result<(), WwdgError> {
        let wwdg = Wwdg::wwdg();
        
        // 设置预分频系数
        wwdg.cfr().modify(|_, w: &mut library::wwdg::cfr::W| unsafe {
            w.wdgtb().bits(prescaler as u8)
        });
        
        Ok(())
    }
    
    /// 设置计数器值
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// - 调用者必须确保WWDG已经启用
    /// 
    /// # 参数
    /// * `counter` - 计数器值 (0x40-0x7F)
    /// 
    /// # 返回值
    /// - Ok(())：设置成功
    /// - Err(WwdgError)：设置失败或参数无效
    pub unsafe fn set_counter(&self, counter: u8) -> Result<(), WwdgError> {
        // 检查参数范围
        if (counter & 0xC0) != 0 {
            return Err(WwdgError::CounterOutOfRange);
        }
        
        let wwdg = Wwdg::wwdg();
        
        // 检查WWDG是否已启用
        if !wwdg.cr().read().wdga().bit_is_set() {
            return Err(WwdgError::NotEnabled);
        }
        
        // 设置计数器值
        wwdg.cr().modify(|_, w: &mut library::wwdg::cr::W| unsafe {
            w.t().bits(counter)
        });
        
        Ok(())
    }
    
    /// 获取计数器值
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - 当前计数器值 (0x00-0x7F)
    pub unsafe fn get_counter(&self) -> u8 {
        let wwdg = Wwdg::wwdg();
        wwdg.cr().read().t().bits()
    }
    
    /// 喂狗
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// - 调用者必须确保WWDG已经启用
    /// 
    /// # 参数
    /// * `counter` - 计数器值 (0x40-0x7F)
    /// 
    /// # 返回值
    /// - Ok(())：喂狗成功
    /// - Err(WwdgError)：喂狗失败或参数无效
    pub unsafe fn feed(&self, counter: u8) -> Result<(), WwdgError> {
        self.set_counter(counter)
    }
    
    /// 启用早期唤醒中断
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(())：启用成功
    /// - Err(WwdgError)：启用失败
    pub unsafe fn enable_ewi(&self) -> Result<(), WwdgError> {
        let wwdg = Wwdg::wwdg();
        wwdg.cfr().modify(|_, w: &mut library::wwdg::cfr::W| {
            w.ewi().set_bit()
        });
        Ok(())
    }
    
    /// 禁用早期唤醒中断
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(())：禁用成功
    /// - Err(WwdgError)：禁用失败
    pub unsafe fn disable_ewi(&self) -> Result<(), WwdgError> {
        let wwdg = Wwdg::wwdg();
        wwdg.cfr().modify(|_, w: &mut library::wwdg::cfr::W| {
            w.ewi().clear_bit()
        });
        Ok(())
    }
    
    /// 清除早期唤醒中断标志
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(())：清除成功
    /// - Err(WwdgError)：清除失败
    pub unsafe fn clear_ewi_flag(&self) -> Result<(), WwdgError> {
        let wwdg = Wwdg::wwdg();
        wwdg.sr().write(|w: &mut library::wwdg::sr::W| {
            w.ewi().clear_bit()
        });
        Ok(())
    }
    
    /// 检查早期唤醒中断标志
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - bool：早期唤醒中断标志状态
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
    
    /// 获取WWDG状态
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(WwdgStatus)：WWDG当前状态
    /// - Err(WwdgError)：获取状态失败
    pub unsafe fn get_status(&self) -> Result<WwdgStatus, WwdgError> {
        let wwdg = Wwdg::wwdg();
        let cr = wwdg.cr().read();
        let sr = wwdg.sr().read();
        
        // 检查WWDG是否已启用
        if !cr.wdga().bit_is_set() {
            return Ok(WwdgStatus::Disabled);
        }
        
        // 检查早期唤醒中断标志
        if sr.ewi().bit_is_set() {
            return Ok(WwdgStatus::EarlyWakeup);
        }
        
        // 检查计数器值
        let counter = cr.t().bits();
        if counter < 0x40 {
            return Ok(WwdgStatus::CounterLow);
        }
        
        Ok(WwdgStatus::Enabled)
    }
}

/// 预定义的WWDG实例
pub const WWDG: Wwdg = Wwdg::new();

/// 测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试WWDG初始化
    #[test]
    fn test_wwdg_init() {
        let wwdg = Wwdg::new();
        
        unsafe {
            // 初始化WWDG
            let init_result = wwdg.init(WwdgPrescaler::Div8, 0x50, 0x7F);
            assert!(init_result.is_ok(), "WWDG初始化应该成功");
            
            // 获取状态
            let status = wwdg.get_status();
            assert!(status.is_ok(), "获取WWDG状态应该成功");
            assert_eq!(status.unwrap(), WwdgStatus::Enabled, "WWDG状态应该是Enabled");
        }
    }
    
    /// 测试WWDG参数验证
    #[test]
    fn test_wwdg_param_validation() {
        let wwdg = Wwdg::new();
        
        unsafe {
            // 测试无效窗口值
            let init_result = wwdg.init(WwdgPrescaler::Div8, 0x80, 0x7F);
            assert!(init_result.is_err(), "无效窗口值应该返回错误");
            assert_eq!(init_result.unwrap_err(), WwdgError::WindowOutOfRange, "错误类型应该是WindowOutOfRange");
            
            // 测试无效计数器值
            let init_result = wwdg.init(WwdgPrescaler::Div8, 0x50, 0x80);
            assert!(init_result.is_err(), "无效计数器值应该返回错误");
            assert_eq!(init_result.unwrap_err(), WwdgError::CounterOutOfRange, "错误类型应该是CounterOutOfRange");
        }
    }
    
    /// 测试WWDG喂狗
    #[test]
    fn test_wwdg_feed() {
        let wwdg = Wwdg::new();
        
        unsafe {
            // 初始化WWDG
            let init_result = wwdg.init(WwdgPrescaler::Div8, 0x50, 0x7F);
            assert!(init_result.is_ok(), "WWDG初始化应该成功");
            
            // 喂狗
            let feed_result = wwdg.feed(0x7F);
            assert!(feed_result.is_ok(), "喂狗应该成功");
        }
    }
    
    /// 测试WWDG状态检查
    #[test]
    fn test_wwdg_status() {
        let wwdg = Wwdg::new();
        
        unsafe {
            // 检查初始状态
            let status = wwdg.get_status();
            assert!(status.is_ok(), "获取WWDG状态应该成功");
            assert_eq!(status.unwrap(), WwdgStatus::Disabled, "初始状态应该是Disabled");
        }
    }
    
    /// 测试WWDG超时计算
    #[test]
    fn test_wwdg_timeout_calculation() {
        // 测试超时计算
        let timeout = Wwdg::calculate_timeout(WwdgPrescaler::Div8, 0x7F, 36_000_000);
        assert!(timeout > 0, "超时时间应该大于0");
    }
}
