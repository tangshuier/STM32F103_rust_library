//! PWR（电源控制）模块
//! 提供电源控制的封装和操作，用于管理系统电源状态和备份域

// 屏蔽未使用代码警告
#![allow(unused)]

// 使用内部生成的设备驱动库
use library::*;

/// PWR错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PwrError {
    /// 初始化失败
    InitializationFailed,
    /// 参数无效
    InvalidParameter,
    /// 备份域访问失败
    BackupDomainAccessFailed,
    /// PVD配置失败
    PvdConfigurationFailed,
    /// 模式转换失败
    ModeTransitionFailed,
    /// 操作失败
    OperationFailed,
    /// 未知错误
    UnknownError,
}

/// PWR状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PwrStatus {
    /// PWR准备就绪
    Ready,
    /// PWR正在初始化
    Initializing,
    /// PWR出现错误
    Error,
    /// 正常模式
    NormalMode,
    /// 睡眠模式
    SleepMode,
    /// 停止模式
    StopMode,
    /// 待机模式
    StandbyMode,
    /// 备份域已启用
    BackupDomainEnabled,
    /// 备份域已禁用
    BackupDomainDisabled,
    /// PVD已启用
    PvdEnabled,
    /// PVD已禁用
    PvdDisabled,
}

/// PVD阈值级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PvdLevel {
    /// 2.0V
    Level0 = 0,
    /// 2.1V
    Level1 = 1,
    /// 2.3V
    Level2 = 2,
    /// 2.5V
    Level3 = 3,
    /// 2.7V
    Level4 = 4,
    /// 2.8V
    Level5 = 5,
    /// 2.9V
    Level6 = 6,
    /// 3.0V
    Level7 = 7,
}

/// PWR结构体
#[derive(Debug, Clone, Copy)]
pub struct Pwr;

impl Pwr {
    /// 创建新的PWR实例
    pub const fn new() -> Self {
        Self
    }
    
    /// 获取PWR寄存器块的不可变引用
    pub unsafe fn pwr_reg(&self) -> &'static pwr::RegisterBlock {
        &*(0x40007000 as *const pwr::RegisterBlock)
    }
    
    /// 获取PWR寄存器块的可变引用
    pub unsafe fn pwr_reg_mut(&self) -> &'static mut pwr::RegisterBlock {
        &mut *(0x40007000 as *mut pwr::RegisterBlock)
    }
    
    /// 获取RCC寄存器块的可变引用
    pub unsafe fn rcc_reg_mut(&self) -> &'static mut rcc::RegisterBlock {
        &mut *(0x40021000 as *mut rcc::RegisterBlock)
    }
    
    /// 初始化PWR
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(())：PWR初始化成功
    /// - Err(PwrError)：PWR初始化失败
    pub unsafe fn init(&self) -> Result<(), PwrError> {
        let rcc = self.rcc_reg_mut();
        
        // 启用PWR时钟
        rcc.apb1enr().modify(|_, w| w
            .pwren().set_bit()
        );
        
        Ok(())
    }
    
    /// 使能对备份域的访问
    /// 
    /// # 安全
    /// - 调用者必须确保PWR已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(())：备份域访问使能成功
    /// - Err(PwrError)：备份域访问使能失败
    pub unsafe fn enable_backup_domain_access(&self) -> Result<(), PwrError> {
        let pwr = self.pwr_reg_mut();
        pwr.cr().modify(|_, w| w
            .dbp().set_bit()
        );
        
        Ok(())
    }
    
    /// 禁用对备份域的访问
    /// 
    /// # 安全
    /// - 调用者必须确保PWR已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(())：备份域访问禁用成功
    /// - Err(PwrError)：备份域访问禁用失败
    pub unsafe fn disable_backup_domain_access(&self) -> Result<(), PwrError> {
        let pwr = self.pwr_reg_mut();
        pwr.cr().modify(|_, w| w
            .dbp().clear_bit()
        );
        
        Ok(())
    }
    
    /// 检查备份域访问是否已启用
    /// 
    /// # 安全
    /// - 调用者必须确保PWR已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(bool)：备份域访问是否已启用
    /// - Err(PwrError)：检查失败
    pub unsafe fn is_backup_domain_access_enabled(&self) -> Result<bool, PwrError> {
        let pwr = self.pwr_reg();
        Ok(pwr.cr().read().dbp().bit_is_set())
    }
    
    /// 启用PVD（可编程电压监测器）
    /// 
    /// # 安全
    /// - 调用者必须确保PWR已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(())：PVD启用成功
    /// - Err(PwrError)：PVD启用失败
    pub unsafe fn enable_pvd(&self) -> Result<(), PwrError> {
        let pwr = self.pwr_reg_mut();
        pwr.cr().modify(|_, w| w
            .pvde().set_bit()
        );
        
        Ok(())
    }
    
    /// 禁用PVD（可编程电压监测器）
    /// 
    /// # 安全
    /// - 调用者必须确保PWR已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(())：PVD禁用成功
    /// - Err(PwrError)：PVD禁用失败
    pub unsafe fn disable_pvd(&self) -> Result<(), PwrError> {
        let pwr = self.pwr_reg_mut();
        pwr.cr().modify(|_, w| w
            .pvde().clear_bit()
        );
        
        Ok(())
    }
    
    /// 检查PVD是否已启用
    /// 
    /// # 安全
    /// - 调用者必须确保PWR已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(bool)：PVD是否已启用
    /// - Err(PwrError)：检查失败
    pub unsafe fn is_pvd_enabled(&self) -> Result<bool, PwrError> {
        let pwr = self.pwr_reg();
        Ok(pwr.cr().read().pvde().bit_is_set())
    }
    
    /// 设置PVD阈值
    /// 
    /// # 安全
    /// - 调用者必须确保PWR已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `level`：PVD阈值级别
    /// 
    /// # 返回值
    /// - Ok(())：PVD阈值设置成功
    /// - Err(PwrError)：PVD阈值设置失败
    pub unsafe fn set_pvd_level(&self, level: PvdLevel) -> Result<(), PwrError> {
        let pwr = self.pwr_reg_mut();
        pwr.cr().modify(|_, w| w
            .pls().bits(level as u8)
        );
        
        Ok(())
    }
    
    /// 获取PVD阈值级别
    /// 
    /// # 安全
    /// - 调用者必须确保PWR已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(PvdLevel)：当前PVD阈值级别
    /// - Err(PwrError)：获取失败
    pub unsafe fn get_pvd_level(&self) -> Result<PvdLevel, PwrError> {
        let pwr = self.pwr_reg();
        let level_bits = pwr.cr().read().pls().bits();
        
        match level_bits {
            0 => Ok(PvdLevel::Level0),
            1 => Ok(PvdLevel::Level1),
            2 => Ok(PvdLevel::Level2),
            3 => Ok(PvdLevel::Level3),
            4 => Ok(PvdLevel::Level4),
            5 => Ok(PvdLevel::Level5),
            6 => Ok(PvdLevel::Level6),
            7 => Ok(PvdLevel::Level7),
            _ => Err(PwrError::InvalidParameter),
        }
    }
    
    /// 进入睡眠模式
    /// 
    /// # 安全
    /// - 调用者必须确保PWR已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// - 调用者必须确保所有必要的中断都已配置
    /// 
    /// # 参数
    /// - `wait_for_interrupt`：是否等待中断（WFI指令），否则使用WFE指令
    /// 
    /// # 返回值
    /// - Ok(())：进入睡眠模式成功
    /// - Err(PwrError)：进入睡眠模式失败
    pub unsafe fn enter_sleep_mode(&self, wait_for_interrupt: bool) -> Result<(), PwrError> {
        if wait_for_interrupt {
            // WFI指令：等待中断
            core::arch::asm!("wfi");
        } else {
            // WFE指令：等待事件
            core::arch::asm!("wfe");
        }
        
        Ok(())
    }
    
    /// 进入停止模式
    /// 
    /// # 安全
    /// - 调用者必须确保PWR已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// - 调用者必须确保系统时钟已经正确配置
    /// 
    /// # 参数
    /// - `regulator_low_power`：是否使用低功耗调节器
    /// 
    /// # 返回值
    /// - Ok(())：进入停止模式成功
    /// - Err(PwrError)：进入停止模式失败
    pub unsafe fn enter_stop_mode(&self, regulator_low_power: bool) -> Result<(), PwrError> {
        let pwr = self.pwr_reg_mut();
        
        // 设置LPDS位
        if regulator_low_power {
            pwr.cr().modify(|_, w| w
                .lpds().set_bit()
            );
        } else {
            pwr.cr().modify(|_, w| w
                .lpds().clear_bit()
            );
        }
        
        // 设置PDDS位为0（停止模式）
        pwr.cr().modify(|_, w| w
            .pdds().clear_bit()
        );
        
        // 设置CWUF位
        pwr.cr().modify(|_, w| w
            .cwuf().set_bit()
        );
        
        // WFI指令
        core::arch::asm!("wfi");
        
        Ok(())
    }
    
    /// 进入待机模式
    /// 
    /// # 安全
    /// - 调用者必须确保PWR已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// - 调用者必须确保已经保存了所有必要的数据
    /// - 调用者必须确保已经配置了唤醒源
    /// 
    /// # 返回值
    /// - Ok(())：进入待机模式成功
    /// - Err(PwrError)：进入待机模式失败
    pub unsafe fn enter_standby_mode(&self) -> Result<(), PwrError> {
        let pwr = self.pwr_reg_mut();
        
        // 设置PDDS位为1（待机模式）
        pwr.cr().modify(|_, w| w
            .pdds().set_bit()
        );
        
        // 设置CWUF位
        pwr.cr().modify(|_, w| w
            .cwuf().set_bit()
        );
        
        // WFI指令
        core::arch::asm!("wfi");
        
        Ok(())
    }
    
    /// 清除Wake-Up标志
    /// 
    /// # 安全
    /// - 调用者必须确保PWR已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(())：清除Wake-Up标志成功
    /// - Err(PwrError)：清除Wake-Up标志失败
    pub unsafe fn clear_wakeup_flag(&self) -> Result<(), PwrError> {
        let pwr = self.pwr_reg_mut();
        pwr.cr().modify(|_, w| w
            .cwuf().set_bit()
        );
        
        Ok(())
    }
    
    /// 清除待机标志
    /// 
    /// # 安全
    /// - 调用者必须确保PWR已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(())：清除待机标志成功
    /// - Err(PwrError)：清除待机标志失败
    pub unsafe fn clear_standby_flag(&self) -> Result<(), PwrError> {
        let pwr = self.pwr_reg_mut();
        pwr.cr().modify(|_, w| w
            .csbf().set_bit()
        );
        
        Ok(())
    }
    
    /// 检查Wake-Up标志
    /// 
    /// # 安全
    /// - 调用者必须确保PWR已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(bool)：Wake-Up标志是否设置
    /// - Err(PwrError)：检查失败
    pub unsafe fn get_wakeup_flag(&self) -> Result<bool, PwrError> {
        let pwr = self.pwr_reg();
        Ok(pwr.csr().read().wuf().bit_is_set())
    }
    
    /// 检查待机标志
    /// 
    /// # 安全
    /// - 调用者必须确保PWR已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(bool)：待机标志是否设置
    /// - Err(PwrError)：检查失败
    pub unsafe fn get_standby_flag(&self) -> Result<bool, PwrError> {
        let pwr = self.pwr_reg();
        Ok(pwr.csr().read().sbf().bit_is_set())
    }
    
    /// 检查PVD输出
    /// 
    /// # 安全
    /// - 调用者必须确保PWR已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(bool)：PVD输出是否为高
    /// - Err(PwrError)：检查失败
    pub unsafe fn get_pvd_output(&self) -> Result<bool, PwrError> {
        let pwr = self.pwr_reg();
        Ok(pwr.csr().read().pvdo().bit_is_set())
    }
    
    /// 获取PWR状态
    /// 
    /// # 安全
    /// - 调用者必须确保PWR已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(PwrStatus)：PWR当前状态
    /// - Err(PwrError)：获取状态失败
    pub unsafe fn get_status(&self) -> Result<PwrStatus, PwrError> {
        let pwr = self.pwr_reg();
        let csr = pwr.csr().read();
        let cr = pwr.cr().read();
        
        // 检查备份域访问状态
        let backup_domain_enabled = cr.dbp().bit_is_set();
        let pvd_enabled = cr.pvde().bit_is_set();
        
        if backup_domain_enabled {
            return Ok(PwrStatus::BackupDomainEnabled);
        } else if pvd_enabled {
            return Ok(PwrStatus::PvdEnabled);
        } else {
            return Ok(PwrStatus::NormalMode);
        }
    }
}

/// 预定义的PWR实例
pub const PWR: Pwr = Pwr::new();

/// 测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试PWR初始化和状态获取
    #[test]
    fn test_pwr_init_status() {
        let pwr = Pwr::new();
        
        // 初始化PWR
        unsafe {
            let init_result = pwr.init();
            assert!(init_result.is_ok(), "PWR初始化应该成功");
            
            let status = pwr.get_status();
            assert!(status.is_ok(), "获取PWR状态应该成功");
            assert_eq!(status.unwrap(), PwrStatus::NormalMode, "PWR初始状态应该是NormalMode");
        }
    }
    
    /// 测试备份域访问控制
    #[test]
    fn test_pwr_backup_domain_access() {
        let pwr = Pwr::new();
        
        unsafe {
            let init_result = pwr.init();
            assert!(init_result.is_ok(), "PWR初始化应该成功");
            
            // 启用备份域访问
            let enable_result = pwr.enable_backup_domain_access();
            assert!(enable_result.is_ok(), "启用备份域访问应该成功");
            
            // 检查备份域访问是否已启用
            let is_enabled = pwr.is_backup_domain_access_enabled();
            assert!(is_enabled.is_ok(), "检查备份域访问状态应该成功");
            assert!(is_enabled.unwrap(), "备份域访问应该已启用");
            
            // 禁用备份域访问
            let disable_result = pwr.disable_backup_domain_access();
            assert!(disable_result.is_ok(), "禁用备份域访问应该成功");
            
            // 检查备份域访问是否已禁用
            let is_enabled = pwr.is_backup_domain_access_enabled();
            assert!(is_enabled.is_ok(), "检查备份域访问状态应该成功");
            assert!(!is_enabled.unwrap(), "备份域访问应该已禁用");
        }
    }
    
    /// 测试PVD配置
    #[test]
    fn test_pwr_pvd_config() {
        let pwr = Pwr::new();
        
        unsafe {
            let init_result = pwr.init();
            assert!(init_result.is_ok(), "PWR初始化应该成功");
            
            // 设置PVD级别
            let set_level_result = pwr.set_pvd_level(PvdLevel::Level4);
            assert!(set_level_result.is_ok(), "设置PVD级别应该成功");
            
            // 获取PVD级别
            let get_level_result = pwr.get_pvd_level();
            assert!(get_level_result.is_ok(), "获取PVD级别应该成功");
            assert_eq!(get_level_result.unwrap(), PvdLevel::Level4, "PVD级别应该是Level4");
            
            // 启用PVD
            let enable_result = pwr.enable_pvd();
            assert!(enable_result.is_ok(), "启用PVD应该成功");
            
            // 检查PVD是否已启用
            let is_enabled = pwr.is_pvd_enabled();
            assert!(is_enabled.is_ok(), "检查PVD状态应该成功");
            assert!(is_enabled.unwrap(), "PVD应该已启用");
            
            // 禁用PVD
            let disable_result = pwr.disable_pvd();
            assert!(disable_result.is_ok(), "禁用PVD应该成功");
            
            // 检查PVD是否已禁用
            let is_enabled = pwr.is_pvd_enabled();
            assert!(is_enabled.is_ok(), "检查PVD状态应该成功");
            assert!(!is_enabled.unwrap(), "PVD应该已禁用");
        }
    }
    
    /// 测试标志管理
    #[test]
    fn test_pwr_flags() {
        let pwr = Pwr::new();
        
        unsafe {
            let init_result = pwr.init();
            assert!(init_result.is_ok(), "PWR初始化应该成功");
            
            // 清除唤醒标志
            let clear_wakeup_result = pwr.clear_wakeup_flag();
            assert!(clear_wakeup_result.is_ok(), "清除唤醒标志应该成功");
            
            // 清除待机标志
            let clear_standby_result = pwr.clear_standby_flag();
            assert!(clear_standby_result.is_ok(), "清除待机标志应该成功");
            
            // 检查唤醒标志
            let wakeup_flag = pwr.get_wakeup_flag();
            assert!(wakeup_flag.is_ok(), "获取唤醒标志应该成功");
            
            // 检查待机标志
            let standby_flag = pwr.get_standby_flag();
            assert!(standby_flag.is_ok(), "获取待机标志应该成功");
        }
    }
}
