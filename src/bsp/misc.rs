#![allow(unused)]

//! MISC（杂项）模块
//! 提供NVIC、SCB和SysTick等系统控制功能的封装和操作

// 使用内部生成的设备驱动库
use library::*;

/// MISC错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiscError {
    /// 初始化失败
    InitializationFailed,
    /// 参数无效
    InvalidParameter,
    /// 无效的优先级组
    InvalidPriorityGroup,
    /// 无效的中断
    InvalidInterrupt,
    /// 无效的优先级
    InvalidPriority,
    /// 无效的向量表
    InvalidVectorTable,
    /// 无效的偏移量
    InvalidOffset,
    /// 操作失败
    OperationFailed,
    /// 未知错误
    UnknownError,
}

/// MISC状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiscStatus {
    /// MISC准备就绪
    Ready,
    /// MISC正在初始化
    Initializing,
    /// MISC出现错误
    Error,
    /// 中断已启用
    InterruptsEnabled,
    /// 低功耗模式
    LowPowerMode,
    /// 正常模式
    NormalMode,
}

const SCB_BASE: u32 = 0xE000ED00;
const NVIC_BASE: u32 = 0xE000E100;
const SYSTICK_BASE: u32 = 0xE000E010;

const SCB_AIRCR: *mut u32 = (SCB_BASE + 0x0C) as *mut u32;
const SCB_VTOR: *mut u32 = (SCB_BASE + 0x08) as *mut u32;
const SCB_SCR: *mut u32 = (SCB_BASE + 0x04) as *mut u32;

const NVIC_ISER: *mut u32 = NVIC_BASE as *mut u32;
const NVIC_ICER: *mut u32 = (NVIC_BASE + 0x080) as *mut u32;
const NVIC_IP: *mut u32 = (NVIC_BASE + 0x300) as *mut u32;

const SYSTICK_CTRL: *mut u32 = SYSTICK_BASE as *mut u32;

const AIRCR_VECTKEY_MASK: u32 = 0x05FA0000;

/// NVIC优先级组
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NvicPriorityGroup {
    Group0 = 0x700, /// 0位抢占优先级，4位子优先级
    Group1 = 0x600, /// 1位抢占优先级，3位子优先级
    Group2 = 0x500, /// 2位抢占优先级，2位子优先级
    Group3 = 0x400, /// 3位抢占优先级，1位子优先级
    Group4 = 0x300, /// 4位抢占优先级，0位子优先级
}

/// NVIC初始化结构体
#[derive(Debug, Clone, Copy)]
pub struct NvicInitStruct {
    pub irq_channel: u8,          /// 中断通道
    pub preemption_priority: u8,  /// 抢占优先级
    pub sub_priority: u8,         /// 子优先级
    pub enable: bool,             /// 中断使能
}

/// 低功耗模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum LowPowerMode {
    SleepOnExit = 0x02, /// 退出时进入睡眠模式
    SleepDeep = 0x04,   /// 深度睡眠模式
    SevOnPend = 0x10,    /// 悬而未决的请求生成事件
}

/// SysTick时钟源
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SysTickClkSource {
    HclkDiv8 = 0xFFFFFFFB, /// HCLK/8
    Hclk = 0x00000004,     /// HCLK
}

/// 向量表基址
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NvicVectTab {
    Ram = 0x20000000,  /// RAM
    Flash = 0x08000000, /// Flash
}

/// MISC结构体
#[derive(Debug, Clone, Copy)]
pub struct Misc;

impl Misc {
    /// 创建新的MISC实例
    pub const fn new() -> Self {
        Self
    }

    /// 配置NVIC优先级分组
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// - 调用者必须确保提供有效的优先级组
    /// 
    /// # 参数
    /// - `priority_group`：NVIC优先级分组
    /// 
    /// # 返回值
    /// - Ok(())：优先级分组配置成功
    /// - Err(MiscError)：优先级分组配置失败
    pub unsafe fn nvic_priority_group_config(&self, priority_group: NvicPriorityGroup) -> Result<(), MiscError> {
        // 检查优先级组是否有效
        match priority_group {
            NvicPriorityGroup::Group0 | NvicPriorityGroup::Group1 | NvicPriorityGroup::Group2 | 
            NvicPriorityGroup::Group3 | NvicPriorityGroup::Group4 => {
                *SCB_AIRCR = AIRCR_VECTKEY_MASK | (priority_group as u32);
                Ok(())
            },
            _ => Err(MiscError::InvalidPriorityGroup),
        }
    }

    /// 初始化NVIC
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// - 调用者必须确保提供有效的初始化结构体
    /// - 调用者必须确保已经配置了优先级分组
    /// 
    /// # 参数
    /// - `init_struct`：NVIC初始化结构体
    /// 
    /// # 返回值
    /// - Ok(())：NVIC初始化成功
    /// - Err(MiscError)：NVIC初始化失败
    pub unsafe fn nvic_init(&self, init_struct: NvicInitStruct) -> Result<(), MiscError> {
        // 检查中断通道是否有效（STM32F103有60个中断，0-59）
        if init_struct.irq_channel > 59 {
            return Err(MiscError::InvalidInterrupt);
        }
        
        // 检查优先级是否有效
        let priority_group = ((*SCB_AIRCR) & 0x700) >> 8;
        let pre_bits = 4 - priority_group;
        let sub_bits = priority_group;
        
        let max_pre_priority = (1 << pre_bits) - 1;
        let max_sub_priority = (1 << sub_bits) - 1;
        
        if init_struct.preemption_priority as u32 > max_pre_priority || 
           init_struct.sub_priority as u32 > max_sub_priority {
            return Err(MiscError::InvalidPriority);
        }
        
        if init_struct.enable {
            // 计算优先级值
            let mut priority = (init_struct.preemption_priority as u32) << sub_bits;
            priority |= (init_struct.sub_priority as u32) & ((1 << sub_bits) - 1);
            priority <<= 4;
            
            // 配置中断优先级
            let ip_index = init_struct.irq_channel as usize;
            let ip_register = NVIC_IP.add(ip_index / 4);
            let shift = (ip_index % 4) * 8 + 4;
            *ip_register &= !(0xFF << shift);
            *ip_register |= priority << shift;
            
            // 启用中断
            let iser_index = init_struct.irq_channel as usize / 32;
            let iser_bit = init_struct.irq_channel % 32;
            let iser_register = NVIC_ISER.add(iser_index);
            *iser_register |= 1 << iser_bit;
        } else {
            // 禁用中断
            let icer_index = init_struct.irq_channel as usize / 32;
            let icer_bit = init_struct.irq_channel % 32;
            let icer_register = NVIC_ICER.add(icer_index);
            *icer_register |= 1 << icer_bit;
        }
        
        Ok(())
    }

    /// 设置向量表
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// - 调用者必须确保提供有效的向量表基址和偏移量
    /// 
    /// # 参数
    /// - `vect_tab`：向量表基址
    /// - `offset`：向量表偏移量（必须对齐到128字节）
    /// 
    /// # 返回值
    /// - Ok(())：向量表设置成功
    /// - Err(MiscError)：向量表设置失败
    pub unsafe fn nvic_set_vector_table(&self, vect_tab: NvicVectTab, offset: u32) -> Result<(), MiscError> {
        // 检查偏移量是否对齐到128字节
        if (offset & 0x7F) != 0 {
            return Err(MiscError::InvalidOffset);
        }
        
        // 检查向量表基址是否有效
        match vect_tab {
            NvicVectTab::Ram | NvicVectTab::Flash => {
                *SCB_VTOR = (vect_tab as u32) | (offset & 0x1FFFFF80);
                Ok(())
            },
            _ => Err(MiscError::InvalidVectorTable),
        }
    }

    /// 配置系统低功耗模式
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// - 调用者必须确保提供有效的低功耗模式
    /// 
    /// # 参数
    /// - `low_power_mode`：低功耗模式
    /// - `new_state`：是否启用该模式
    /// 
    /// # 返回值
    /// - Ok(())：低功耗模式配置成功
    /// - Err(MiscError)：低功耗模式配置失败
    pub unsafe fn nvic_system_lp_config(&self, low_power_mode: LowPowerMode, new_state: bool) -> Result<(), MiscError> {
        // 检查低功耗模式是否有效
        match low_power_mode {
            LowPowerMode::SleepOnExit | LowPowerMode::SleepDeep | LowPowerMode::SevOnPend => {
                if new_state {
                    *SCB_SCR |= low_power_mode as u32;
                } else {
                    *SCB_SCR &= !(low_power_mode as u32);
                }
                Ok(())
            },
            _ => Err(MiscError::InvalidParameter),
        }
    }

    /// 配置SysTick时钟源
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// - 调用者必须确保提供有效的时钟源
    /// 
    /// # 参数
    /// - `clk_source`：SysTick时钟源
    /// 
    /// # 返回值
    /// - Ok(())：时钟源配置成功
    /// - Err(MiscError)：时钟源配置失败
    pub unsafe fn systick_clk_source_config(&self, clk_source: SysTickClkSource) -> Result<(), MiscError> {
        // 检查时钟源是否有效
        match clk_source {
            SysTickClkSource::Hclk | SysTickClkSource::HclkDiv8 => {
                if clk_source == SysTickClkSource::Hclk {
                    *SYSTICK_CTRL |= SysTickClkSource::Hclk as u32;
                } else {
                    *SYSTICK_CTRL &= SysTickClkSource::HclkDiv8 as u32;
                }
                Ok(())
            },
            _ => Err(MiscError::InvalidParameter),
        }
    }
    
    /// 获取NVIC优先级分组
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(NvicPriorityGroup)：当前NVIC优先级分组
    /// - Err(MiscError)：获取优先级分组失败
    pub unsafe fn get_nvic_priority_group(&self) -> Result<NvicPriorityGroup, MiscError> {
        let aircr_value = *SCB_AIRCR;
        let priority_group_bits = (aircr_value & 0x700) >> 8;
        
        match priority_group_bits {
            7 => Ok(NvicPriorityGroup::Group0),
            6 => Ok(NvicPriorityGroup::Group1),
            5 => Ok(NvicPriorityGroup::Group2),
            4 => Ok(NvicPriorityGroup::Group3),
            3 => Ok(NvicPriorityGroup::Group4),
            _ => Err(MiscError::InvalidPriorityGroup),
        }
    }
    
    /// 获取MISC状态
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(MiscStatus)：MISC当前状态
    /// - Err(MiscError)：获取状态失败
    pub unsafe fn get_status(&self) -> Result<MiscStatus, MiscError> {
        let scr_value = *SCB_SCR;
        
        // 检查是否处于低功耗模式
        if (scr_value & (LowPowerMode::SleepDeep as u32)) != 0 {
            return Ok(MiscStatus::LowPowerMode);
        }
        
        Ok(MiscStatus::NormalMode)
    }
    
    /// 初始化MISC
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(())：MISC初始化成功
    /// - Err(MiscError)：MISC初始化失败
    pub unsafe fn init(&self) -> Result<(), MiscError> {
        // 初始化优先级分组为默认值Group2（2位抢占优先级，2位子优先级）
        self.nvic_priority_group_config(NvicPriorityGroup::Group2)
    }
}

/// 预定义的MISC实例
pub const MISC: Misc = Misc::new();

/// 测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试MISC初始化和状态获取
    #[test]
    fn test_misc_init_status() {
        let misc = Misc::new();
        
        // 初始化MISC
        unsafe {
            let init_result = misc.init();
            assert!(init_result.is_ok(), "MISC初始化应该成功");
            
            let status = misc.get_status();
            assert!(status.is_ok(), "获取MISC状态应该成功");
            assert_eq!(status.unwrap(), MiscStatus::NormalMode, "MISC初始状态应该是NormalMode");
        }
    }
    
    /// 测试NVIC优先级分组配置
    #[test]
    fn test_nvic_priority_group_config() {
        let misc = Misc::new();
        
        unsafe {
            // 测试设置不同优先级分组
            let result = misc.nvic_priority_group_config(NvicPriorityGroup::Group1);
            assert!(result.is_ok(), "配置NVIC优先级分组Group1应该成功");
            
            let group = misc.get_nvic_priority_group();
            assert!(group.is_ok(), "获取NVIC优先级分组应该成功");
            assert_eq!(group.unwrap(), NvicPriorityGroup::Group1, "NVIC优先级分组应该是Group1");
            
            let result = misc.nvic_priority_group_config(NvicPriorityGroup::Group3);
            assert!(result.is_ok(), "配置NVIC优先级分组Group3应该成功");
            
            let group = misc.get_nvic_priority_group();
            assert!(group.is_ok(), "获取NVIC优先级分组应该成功");
            assert_eq!(group.unwrap(), NvicPriorityGroup::Group3, "NVIC优先级分组应该是Group3");
        }
    }
    
    /// 测试NVIC初始化
    #[test]
    fn test_nvic_init() {
        let misc = Misc::new();
        
        unsafe {
            // 初始化优先级分组
            let init_result = misc.nvic_priority_group_config(NvicPriorityGroup::Group2);
            assert!(init_result.is_ok(), "配置NVIC优先级分组应该成功");
            
            // 测试有效的NVIC初始化
            let nvic_init = NvicInitStruct {
                irq_channel: 10, // TIM1_UP_IRQn
                preemption_priority: 1,
                sub_priority: 2,
                enable: true,
            };
            
            let result = misc.nvic_init(nvic_init);
            assert!(result.is_ok(), "初始化NVIC应该成功");
            
            // 测试无效的中断通道
            let invalid_nvic_init = NvicInitStruct {
                irq_channel: 100, // 无效的中断通道
                preemption_priority: 1,
                sub_priority: 2,
                enable: true,
            };
            
            let result = misc.nvic_init(invalid_nvic_init);
            assert!(result.is_err(), "无效的中断通道应该返回错误");
            assert_eq!(result.unwrap_err(), MiscError::InvalidInterrupt, "错误类型应该是InvalidInterrupt");
        }
    }
    
    /// 测试系统低功耗配置
    #[test]
    fn test_nvic_system_lp_config() {
        let misc = Misc::new();
        
        unsafe {
            // 测试启用深度睡眠模式
            let result = misc.nvic_system_lp_config(LowPowerMode::SleepDeep, true);
            assert!(result.is_ok(), "启用深度睡眠模式应该成功");
            
            // 测试禁用深度睡眠模式
            let result = misc.nvic_system_lp_config(LowPowerMode::SleepDeep, false);
            assert!(result.is_ok(), "禁用深度睡眠模式应该成功");
            
            // 测试睡眠退出模式
            let result = misc.nvic_system_lp_config(LowPowerMode::SleepOnExit, true);
            assert!(result.is_ok(), "启用睡眠退出模式应该成功");
            
            let result = misc.nvic_system_lp_config(LowPowerMode::SleepOnExit, false);
            assert!(result.is_ok(), "禁用睡眠退出模式应该成功");
        }
    }
    
    /// 测试SysTick时钟源配置
    #[test]
    fn test_systick_clk_source_config() {
        let misc = Misc::new();
        
        unsafe {
            // 测试配置为HCLK/8
            let result = misc.systick_clk_source_config(SysTickClkSource::HclkDiv8);
            assert!(result.is_ok(), "配置SysTick时钟源为HCLK/8应该成功");
            
            // 测试配置为HCLK
            let result = misc.systick_clk_source_config(SysTickClkSource::Hclk);
            assert!(result.is_ok(), "配置SysTick时钟源为HCLK应该成功");
        }
    }
}
