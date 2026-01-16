//! IWDG（独立看门狗）模块
//! 提供独立看门狗的封装和操作，实现系统的故障检测和自动重启

// 屏蔽未使用代码警告
#![allow(unused)]

// 使用内部生成的设备驱动库
use library::*;

/// IWDG预分频系数枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IwdgPrescaler {
    /// 预分频系数：4
    Div4 = 0,
    /// 预分频系数：8
    Div8 = 1,
    /// 预分频系数：16
    Div16 = 2,
    /// 预分频系数：32
    Div32 = 3,
    /// 预分频系数：64
    Div64 = 4,
    /// 预分频系数：128
    Div128 = 5,
    /// 预分频系数：256
    Div256 = 6,
}

/// IWDG状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IwdgStatus {
    /// IWDG准备就绪
    Ready,
    /// 预分频寄存器正在更新
    PrescalerBusy,
    /// 重载寄存器正在更新
    ReloadBusy,
    /// 预分频和重载寄存器都在更新
    Busy,
}

/// IWDG错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IwdgError {
    /// 无效的重载值
    InvalidReloadValue,
    /// 预分频更新失败
    PrescalerUpdateFailed,
    /// 重载更新失败
    ReloadUpdateFailed,
}

/// IWDG结构体
#[derive(Debug, Clone, Copy)]
pub struct Iwdg;

impl Iwdg {
    /// 创建新的IWDG实例
    pub const fn new() -> Self {
        Self
    }
    
    /// 获取IWDG寄存器块的不可变引用
    pub unsafe fn iwdg_reg(&self) -> &'static iwdg::RegisterBlock {
        &*(0x40003000 as *const iwdg::RegisterBlock)
    }
    
    /// 获取IWDG寄存器块的可变引用
    pub unsafe fn iwdg_reg_mut(&self) -> &'static mut iwdg::RegisterBlock {
        &mut *(0x40003000 as *mut iwdg::RegisterBlock)
    }
    
    /// 初始化IWDG
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// - 调用此函数后，IWDG将开始运行，需要定期喂狗
    /// 
    /// # 参数
    /// - `prescaler`：预分频系数
    /// - `reload`：重载值，范围：0x0000 - 0x0FFF
    pub unsafe fn init(&self, prescaler: IwdgPrescaler, reload: u16) -> Result<(), IwdgError> {
        if reload > 0x0FFF {
            return Err(IwdgError::InvalidReloadValue);
        }
        
        let iwdg = self.iwdg_reg_mut();
        
        // 启用写入访问
        iwdg.kr().write(|w| w
            .key().bits(0x5555) // 写入访问使能键值
        );
        
        // 设置预分频系数
        iwdg.pr().write(|w| w
            .pr().bits(prescaler as u8)
        );
        
        // 设置重载值
        iwdg.rlr().write(|w| w
            .rl().bits(reload)
        );
        
        // 重载计数器
        self.feed();
        
        // 启用IWDG
        iwdg.kr().write(|w| w
            .key().bits(0xCCCC) // IWDG使能键值
        );
        
        Ok(())
    }
    
    /// 喂狗（重载计数器）
    /// 
    /// # 安全
    /// - 调用者必须确保IWDG已经初始化
    pub unsafe fn feed(&self) {
        let iwdg = self.iwdg_reg_mut();
        iwdg.kr().write(|w| w
            .key().bits(0xAAAA) // 喂狗键值
        );
    }
    
    /// 获取IWDG状态
    pub fn get_status(&self) -> IwdgStatus {
        let iwdg = unsafe { self.iwdg_reg() };
        let sr = iwdg.sr().read();
        
        match (sr.pvu().bit_is_set(), sr.rvu().bit_is_set()) {
            (false, false) => IwdgStatus::Ready,
            (true, false) => IwdgStatus::PrescalerBusy,
            (false, true) => IwdgStatus::ReloadBusy,
            (true, true) => IwdgStatus::Busy,
        }
    }
    
    /// 检查预分频寄存器是否正在更新
    pub fn is_prescaler_busy(&self) -> bool {
        let iwdg = unsafe { self.iwdg_reg() };
        iwdg.sr().read().pvu().bit_is_set()
    }
    
    /// 检查重载寄存器是否正在更新
    pub fn is_reload_busy(&self) -> bool {
        let iwdg = unsafe { self.iwdg_reg() };
        iwdg.sr().read().rvu().bit_is_set()
    }
    
    /// 等待IWDG准备就绪
    pub fn wait_ready(&self) {
        while !matches!(self.get_status(), IwdgStatus::Ready) {
            // 空循环等待
        }
    }
    
    /// 计算看门狗超时时间
    /// 
    /// # 参数
    /// - `prescaler`：预分频系数
    /// - `reload`：重载值
    /// 
    /// # 返回值
    /// 超时时间（毫秒）
    pub fn calculate_timeout(prescaler: IwdgPrescaler, reload: u16) -> u32 {
        // 独立看门狗时钟频率为40kHz（内部RC振荡器）
        const IWDG_CLK_FREQ: u32 = 40_000;
        
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
        
        let tick_freq = IWDG_CLK_FREQ / prescaler_value;
        let timeout_ms = (reload as u32 * 1000) / tick_freq;
        
        timeout_ms
    }
    
    /// 设置预分频系数
    /// 
    /// # 安全
    /// - 调用者必须确保IWDG已经初始化
    pub unsafe fn set_prescaler(&self, prescaler: IwdgPrescaler) -> Result<(), IwdgError> {
        let iwdg = self.iwdg_reg_mut();
        
        // 启用写入访问
        iwdg.kr().write(|w| w
            .key().bits(0x5555)
        );
        
        // 设置预分频系数
        iwdg.pr().write(|w| w
            .pr().bits(prescaler as u8)
        );
        
        Ok(())
    }
    
    /// 设置重载值
    /// 
    /// # 安全
    /// - 调用者必须确保IWDG已经初始化
    pub unsafe fn set_reload(&self, reload: u16) -> Result<(), IwdgError> {
        if reload > 0x0FFF {
            return Err(IwdgError::InvalidReloadValue);
        }
        
        let iwdg = self.iwdg_reg_mut();
        
        // 启用写入访问
        iwdg.kr().write(|w| w
            .key().bits(0x5555)
        );
        
        // 设置重载值
        iwdg.rlr().write(|w| w
            .rl().bits(reload)
        );
        
        Ok(())
    }
    
    /// 获取当前预分频系数
    pub fn get_prescaler(&self) -> IwdgPrescaler {
        let iwdg = unsafe { self.iwdg_reg() };
        let pr = iwdg.pr().read().pr().bits();
        
        match pr {
            0 => IwdgPrescaler::Div4,
            1 => IwdgPrescaler::Div8,
            2 => IwdgPrescaler::Div16,
            3 => IwdgPrescaler::Div32,
            4 => IwdgPrescaler::Div64,
            5 => IwdgPrescaler::Div128,
            6 => IwdgPrescaler::Div256,
            _ => IwdgPrescaler::Div4, // 默认值
        }
    }
    
    /// 获取当前重载值
    pub fn get_reload(&self) -> u16 {
        let iwdg = unsafe { self.iwdg_reg() };
        iwdg.rlr().read().rl().bits()
    }
    
    /// 获取当前计数器值
    pub fn get_counter(&self) -> u16 {
        let iwdg = unsafe { self.iwdg_reg() };
        iwdg.cnt().read().cnt().bits()
    }
}

/// 预定义的IWDG实例
pub const IWDG: Iwdg = Iwdg::new();

/// 测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试IWDG初始化
    #[test]
    fn test_iwdg_init() {
        let iwdg = Iwdg::new();
        
        // 初始化IWDG
        unsafe {
            let result = iwdg.init(IwdgPrescaler::Div32, 0x0FFF);
            assert!(result.is_ok(), "IWDG初始化失败");
        }
        
        // 检查状态
        assert_eq!(iwdg.get_status(), IwdgStatus::Ready, "IWDG状态错误");
    }
    
    /// 测试IWDG喂狗
    #[test]
    fn test_iwdg_feed() {
        let iwdg = Iwdg::new();
        
        // 初始化IWDG
        unsafe {
            let result = iwdg.init(IwdgPrescaler::Div32, 0x0FFF);
            assert!(result.is_ok(), "IWDG初始化失败");
        }
        
        // 喂狗
        unsafe {
            iwdg.feed();
        }
        
        // 检查状态
        assert_eq!(iwdg.get_status(), IwdgStatus::Ready, "喂狗后状态错误");
    }
    
    /// 测试IWDG状态获取
    #[test]
    fn test_iwdg_status() {
        let iwdg = Iwdg::new();
        
        // 初始化IWDG
        unsafe {
            let result = iwdg.init(IwdgPrescaler::Div32, 0x0FFF);
            assert!(result.is_ok(), "IWDG初始化失败");
        }
        
        // 检查状态
        let status = iwdg.get_status();
        assert!(matches!(status, IwdgStatus::Ready), "IWDG状态错误");
        
        // 检查单独的状态标志
        assert!(!iwdg.is_prescaler_busy(), "预分频器不应该忙");
        assert!(!iwdg.is_reload_busy(), "重载寄存器不应该忙");
    }
    
    /// 测试IWDG超时计算
    #[test]
    fn test_iwdg_timeout_calculation() {
        // 测试不同预分频系数下的超时时间
        let timeout1 = Iwdg::calculate_timeout(IwdgPrescaler::Div4, 0x0FFF);
        let timeout2 = Iwdg::calculate_timeout(IwdgPrescaler::Div32, 0x0FFF);
        let timeout3 = Iwdg::calculate_timeout(IwdgPrescaler::Div256, 0x0FFF);
        
        // 预分频系数越大，超时时间越长
        assert!(timeout1 < timeout2, "超时时间计算错误：Div4应该小于Div32");
        assert!(timeout2 < timeout3, "超时时间计算错误：Div32应该小于Div256");
        
        // 检查具体数值
        assert_eq!(timeout1, 1023, "Div4超时时间计算错误");
        assert_eq!(timeout2, 8184, "Div32超时时间计算错误");
        assert_eq!(timeout3, 65472, "Div256超时时间计算错误");
    }
    
    /// 测试IWDG参数获取
    #[test]
    fn test_iwdg_get_parameters() {
        let iwdg = Iwdg::new();
        
        // 初始化IWDG
        unsafe {
            let result = iwdg.init(IwdgPrescaler::Div32, 0x0ABC);
            assert!(result.is_ok(), "IWDG初始化失败");
        }
        
        // 检查参数获取
        assert_eq!(iwdg.get_prescaler(), IwdgPrescaler::Div32, "预分频系数获取错误");
        assert_eq!(iwdg.get_reload(), 0x0ABC, "重载值获取错误");
    }
    
    /// 测试IWDG无效参数
    #[test]
    fn test_iwdg_invalid_parameters() {
        let iwdg = Iwdg::new();
        
        // 测试无效的重载值
        unsafe {
            let result = iwdg.init(IwdgPrescaler::Div32, 0x1000); // 超过最大值0x0FFF
            assert!(result.is_err(), "IWDG应该拒绝无效的重载值");
            assert_eq!(result.unwrap_err(), IwdgError::InvalidReloadValue, "错误类型不匹配");
        }
    }
}
