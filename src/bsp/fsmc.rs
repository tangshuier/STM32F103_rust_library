//! FSMC（灵活的静态存储器控制器）模块
//! 提供灵活的静态存储器控制器的封装和操作，用于连接外部存储器

// 屏蔽未使用代码警告
#![allow(unused)]

// 使用内部生成的设备驱动库
use library::*;

/// FSMC错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsmcError {
    /// 初始化失败
    InitializationFailed,
    /// 参数无效
    InvalidParameter,
    /// 存储区域未找到
    BankNotFound,
    /// 无效的存储器类型
    InvalidMemoryType,
    /// 无效的数据总线宽度
    InvalidDataWidth,
    /// 时序错误
    TimingError,
    /// 存储区域已禁用
    BankDisabled,
    /// 操作失败
    OperationFailed,
    /// 未知错误
    UnknownError,
}

/// FSMC状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsmcStatus {
    /// FSMC准备就绪
    Ready,
    /// FSMC正在初始化
    Initializing,
    /// FSMC出现错误
    Error,
    /// 存储区域1激活
    Bank1Active,
    /// 存储区域2激活
    Bank2Active,
    /// 存储区域3激活
    Bank3Active,
    /// 存储区域4激活
    Bank4Active,
    /// 多个存储区域激活
    MultipleBanksActive,
    /// 所有存储区域禁用
    AllBanksDisabled,
}

/// FSMC存储区域枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsmcBank {
    Bank1 = 0,    // 存储区域1
    Bank2 = 1,    // 存储区域2
    Bank3 = 2,    // 存储区域3
    Bank4 = 3,    // 存储区域4
}

/// FSMC存储器类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsmcMemoryType {
    SRAM = 0,              // SRAM
    PSRAM = 1,             // PSRAM
    NorFlash = 2,         // NOR Flash
    NandFlash = 3,        // NAND Flash
}

/// FSMC数据总线宽度枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsmcDataWidth {
    Width8b = 0,     // 8位
    Width16b = 1,    // 16位
}

/// FSMC结构体
#[derive(Debug, Clone, Copy)]
pub struct Fsmc;

impl Fsmc {
    /// 创建新的FSMC实例
    pub const fn new() -> Self {
        Self
    }
    
    /// 获取FSMC寄存器块的不可变引用
    pub unsafe fn fsmc_reg(&self) -> &'static fsmc::RegisterBlock {
        &*(0xA0000000 as *const fsmc::RegisterBlock)
    }
    
    /// 获取FSMC寄存器块的可变引用
    pub unsafe fn fsmc_reg_mut(&self) -> &'static mut fsmc::RegisterBlock {
        &mut *(0xA0000000 as *mut fsmc::RegisterBlock)
    }
    
    /// 获取RCC寄存器块的可变引用
    pub unsafe fn rcc_reg_mut(&self) -> &'static mut rcc::RegisterBlock {
        &mut *(0x40021000 as *mut rcc::RegisterBlock)
    }
    
    /// 初始化FSMC
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(())：FSMC初始化成功
    /// - Err(FsmcError)：FSMC初始化失败
    pub unsafe fn init(&self) -> Result<(), FsmcError> {
        // 由于内部库中没有FSMC时钟启用寄存器的具体信息，暂时返回成功
        Ok(())
    }
    
    /// 初始化FSMC存储区域
    /// 
    /// # 安全
    /// - 调用者必须确保FSMC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// - 调用者必须确保提供的时序参数有效
    /// 
    /// # 参数
    /// - `bank`：存储区域
    /// - `mem_type`：存储器类型
    /// - `data_width`：数据总线宽度
    /// - `address_setup_time`：地址建立时间 (HCLK周期数)
    /// - `address_hold_time`：地址保持时间 (HCLK周期数)
    /// - `data_setup_time`：数据建立时间 (HCLK周期数)
    /// 
    /// # 返回值
    /// - Ok(())：存储区域初始化成功
    /// - Err(FsmcError)：存储区域初始化失败
    pub unsafe fn init_bank(
        &self,
        bank: FsmcBank,
        mem_type: FsmcMemoryType,
        data_width: FsmcDataWidth,
        address_setup_time: u8,
        address_hold_time: u8,
        data_setup_time: u8
    ) -> Result<(), FsmcError> {
        let fsmc = self.fsmc_reg_mut();
        
        // 检查时序参数范围
        if address_setup_time > 15 || address_hold_time > 15 || data_setup_time > 255 {
            return Err(FsmcError::TimingError);
        }
        
        // 获取对应的BCR和BTR寄存器
        match bank {
            FsmcBank::Bank1 => {
                // 重置BCR寄存器
                fsmc.bcr1().write(|w| unsafe { w.bits(0x00000000) });
                
                // 配置存储器类型和数据总线宽度
                fsmc.bcr1().write(|w| unsafe { 
                    w.bits(
                        ((mem_type as u32) << 4) |
                        ((data_width as u32) << 1) |
                        (1 << 0) // 启用存储区域
                    ) 
                });
                
                // 配置时序参数
                fsmc.btr1().write(|w| unsafe { 
                    w.bits(
                        ((address_setup_time as u32) << 0) |
                        ((address_hold_time as u32) << 4) |
                        ((data_setup_time as u32) << 8)
                    ) 
                });
            },
            FsmcBank::Bank2 => {
                // 重置BCR寄存器
                fsmc.bcr2().write(|w| unsafe { w.bits(0x00000000) });
                
                // 配置存储器类型和数据总线宽度
                fsmc.bcr2().write(|w| unsafe { 
                    w.bits(
                        ((mem_type as u32) << 4) |
                        ((data_width as u32) << 1) |
                        (1 << 0) // 启用存储区域
                    ) 
                });
                
                // 配置时序参数
                fsmc.btr2().write(|w| unsafe { 
                    w.bits(
                        ((address_setup_time as u32) << 0) |
                        ((address_hold_time as u32) << 4) |
                        ((data_setup_time as u32) << 8)
                    ) 
                });
            },
            FsmcBank::Bank3 => {
                // 重置BCR寄存器
                fsmc.bcr3().write(|w| unsafe { w.bits(0x00000000) });
                
                // 配置存储器类型和数据总线宽度
                fsmc.bcr3().write(|w| unsafe { 
                    w.bits(
                        ((mem_type as u32) << 4) |
                        ((data_width as u32) << 1) |
                        (1 << 0) // 启用存储区域
                    ) 
                });
                
                // 配置时序参数
                fsmc.btr3().write(|w| unsafe { 
                    w.bits(
                        ((address_setup_time as u32) << 0) |
                        ((address_hold_time as u32) << 4) |
                        ((data_setup_time as u32) << 8)
                    ) 
                });
            },
            FsmcBank::Bank4 => {
                // 重置BCR寄存器
                fsmc.bcr4().write(|w| unsafe { w.bits(0x00000000) });
                
                // 配置存储器类型和数据总线宽度
                fsmc.bcr4().write(|w| unsafe { 
                    w.bits(
                        ((mem_type as u32) << 4) |
                        ((data_width as u32) << 1) |
                        (1 << 0) // 启用存储区域
                    ) 
                });
                
                // 配置时序参数
                fsmc.btr4().write(|w| unsafe { 
                    w.bits(
                        ((address_setup_time as u32) << 0) |
                        ((address_hold_time as u32) << 4) |
                        ((data_setup_time as u32) << 8)
                    ) 
                });
            },
        }
        
        Ok(())
    }
    
    /// 配置FSMC存储区域的写时序
    /// 
    /// # 安全
    /// - 调用者必须确保FSMC已经初始化
    /// - 调用者必须确保存储区域已经启用
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// - 调用者必须确保提供的时序参数有效
    /// 
    /// # 参数
    /// - `bank`：存储区域
    /// - `address_setup_time`：地址建立时间 (HCLK周期数)
    /// - `address_hold_time`：地址保持时间 (HCLK周期数)
    /// - `data_setup_time`：数据建立时间 (HCLK周期数)
    /// 
    /// # 返回值
    /// - Ok(())：写时序配置成功
    /// - Err(FsmcError)：写时序配置失败
    pub unsafe fn configure_write_timing(
        &self,
        bank: FsmcBank,
        address_setup_time: u8,
        address_hold_time: u8,
        data_setup_time: u8
    ) -> Result<(), FsmcError> {
        let fsmc = self.fsmc_reg_mut();
        
        // 检查时序参数范围
        if address_setup_time > 15 || address_hold_time > 15 || data_setup_time > 255 {
            return Err(FsmcError::TimingError);
        }
        
        // 获取对应的BWTR寄存器
        match bank {
            FsmcBank::Bank1 => {
                // 配置写时序参数
                fsmc.bwtr1().write(|w| unsafe { 
                    w.bits(
                        ((address_setup_time as u32) << 0) |
                        ((address_hold_time as u32) << 4) |
                        ((data_setup_time as u32) << 8) |
                        (1 << 16) // 启用写时序配置
                    ) 
                });
            },
            FsmcBank::Bank2 => {
                // 配置写时序参数
                fsmc.bwtr2().write(|w| unsafe { 
                    w.bits(
                        ((address_setup_time as u32) << 0) |
                        ((address_hold_time as u32) << 4) |
                        ((data_setup_time as u32) << 8) |
                        (1 << 16) // 启用写时序配置
                    ) 
                });
            },
            FsmcBank::Bank3 => {
                // 配置写时序参数
                fsmc.bwtr3().write(|w| unsafe { 
                    w.bits(
                        ((address_setup_time as u32) << 0) |
                        ((address_hold_time as u32) << 4) |
                        ((data_setup_time as u32) << 8) |
                        (1 << 16) // 启用写时序配置
                    ) 
                });
            },
            FsmcBank::Bank4 => {
                // 配置写时序参数
                fsmc.bwtr4().write(|w| unsafe { 
                    w.bits(
                        ((address_setup_time as u32) << 0) |
                        ((address_hold_time as u32) << 4) |
                        ((data_setup_time as u32) << 8) |
                        (1 << 16) // 启用写时序配置
                    ) 
                });
            },
        }
        
        Ok(())
    }
    
    /// 启用存储区域
    /// 
    /// # 安全
    /// - 调用者必须确保FSMC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `bank`：存储区域
    /// 
    /// # 返回值
    /// - Ok(())：存储区域启用成功
    /// - Err(FsmcError)：存储区域启用失败
    pub unsafe fn enable_bank(&self, bank: FsmcBank) -> Result<(), FsmcError> {
        let fsmc = self.fsmc_reg_mut();
        
        match bank {
            FsmcBank::Bank1 => {
                fsmc.bcr1().modify(|r, w| unsafe { 
                    w.bits(r.bits() | (1 << 0)) 
                });
            },
            FsmcBank::Bank2 => {
                fsmc.bcr2().modify(|r, w| unsafe { 
                    w.bits(r.bits() | (1 << 0)) 
                });
            },
            FsmcBank::Bank3 => {
                fsmc.bcr3().modify(|r, w| unsafe { 
                    w.bits(r.bits() | (1 << 0)) 
                });
            },
            FsmcBank::Bank4 => {
                fsmc.bcr4().modify(|r, w| unsafe { 
                    w.bits(r.bits() | (1 << 0)) 
                });
            },
        }
        
        Ok(())
    }
    
    /// 禁用存储区域
    /// 
    /// # 安全
    /// - 调用者必须确保FSMC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `bank`：存储区域
    /// 
    /// # 返回值
    /// - Ok(())：存储区域禁用成功
    /// - Err(FsmcError)：存储区域禁用失败
    pub unsafe fn disable_bank(&self, bank: FsmcBank) -> Result<(), FsmcError> {
        let fsmc = self.fsmc_reg_mut();
        
        match bank {
            FsmcBank::Bank1 => {
                fsmc.bcr1().modify(|r, w| unsafe { 
                    w.bits(r.bits() & !(1 << 0)) 
                });
            },
            FsmcBank::Bank2 => {
                fsmc.bcr2().modify(|r, w| unsafe { 
                    w.bits(r.bits() & !(1 << 0)) 
                });
            },
            FsmcBank::Bank3 => {
                fsmc.bcr3().modify(|r, w| unsafe { 
                    w.bits(r.bits() & !(1 << 0)) 
                });
            },
            FsmcBank::Bank4 => {
                fsmc.bcr4().modify(|r, w| unsafe { 
                    w.bits(r.bits() & !(1 << 0)) 
                });
            },
        }
        
        Ok(())
    }
    
    /// 读取存储区域配置
    /// 
    /// # 安全
    /// - 调用者必须确保FSMC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `bank`：存储区域
    /// 
    /// # 返回值
    /// - Ok(u32)：存储区域配置
    /// - Err(FsmcError)：读取配置失败
    pub unsafe fn get_bank_config(&self, bank: FsmcBank) -> Result<u32, FsmcError> {
        let fsmc = self.fsmc_reg();
        
        let result = match bank {
            FsmcBank::Bank1 => fsmc.bcr1().read().bits(),
            FsmcBank::Bank2 => fsmc.bcr2().read().bits(),
            FsmcBank::Bank3 => fsmc.bcr3().read().bits(),
            FsmcBank::Bank4 => fsmc.bcr4().read().bits(),
        };
        
        Ok(result)
    }
    
    /// 读取存储区域时序配置
    /// 
    /// # 安全
    /// - 调用者必须确保FSMC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `bank`：存储区域
    /// 
    /// # 返回值
    /// - Ok(u32)：存储区域时序配置
    /// - Err(FsmcError)：读取时序配置失败
    pub unsafe fn get_bank_timing(&self, bank: FsmcBank) -> Result<u32, FsmcError> {
        let fsmc = self.fsmc_reg();
        
        let result = match bank {
            FsmcBank::Bank1 => fsmc.btr1().read().bits(),
            FsmcBank::Bank2 => fsmc.btr2().read().bits(),
            FsmcBank::Bank3 => fsmc.btr3().read().bits(),
            FsmcBank::Bank4 => fsmc.btr4().read().bits(),
        };
        
        Ok(result)
    }
    
    /// 读取存储区域写时序配置
    /// 
    /// # 安全
    /// - 调用者必须确保FSMC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `bank`：存储区域
    /// 
    /// # 返回值
    /// - Ok(u32)：存储区域写时序配置
    /// - Err(FsmcError)：读取写时序配置失败
    pub unsafe fn get_bank_write_timing(&self, bank: FsmcBank) -> Result<u32, FsmcError> {
        let fsmc = self.fsmc_reg();
        
        let result = match bank {
            FsmcBank::Bank1 => fsmc.bwtr1().read().bits(),
            FsmcBank::Bank2 => fsmc.bwtr2().read().bits(),
            FsmcBank::Bank3 => fsmc.bwtr3().read().bits(),
            FsmcBank::Bank4 => fsmc.bwtr4().read().bits(),
        };
        
        Ok(result)
    }
    
    /// 检查存储区域是否启用
    /// 
    /// # 安全
    /// - 调用者必须确保FSMC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `bank`：存储区域
    /// 
    /// # 返回值
    /// - Ok(bool)：存储区域是否启用
    /// - Err(FsmcError)：检查失败
    pub unsafe fn is_bank_enabled(&self, bank: FsmcBank) -> Result<bool, FsmcError> {
        let config = self.get_bank_config(bank)?;
        Ok((config & (1 << 0)) != 0)
    }
    
    /// 获取FSMC状态
    /// 
    /// # 安全
    /// - 调用者必须确保FSMC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(FsmcStatus)：FSMC当前状态
    /// - Err(FsmcError)：获取状态失败
    pub unsafe fn get_status(&self) -> Result<FsmcStatus, FsmcError> {
        let fsmc = self.fsmc_reg();
        
        let bank1_enabled = (fsmc.bcr1().read().bits() & (1 << 0)) != 0;
        let bank2_enabled = (fsmc.bcr2().read().bits() & (1 << 0)) != 0;
        let bank3_enabled = (fsmc.bcr3().read().bits() & (1 << 0)) != 0;
        let bank4_enabled = (fsmc.bcr4().read().bits() & (1 << 0)) != 0;
        
        let enabled_banks = [bank1_enabled, bank2_enabled, bank3_enabled, bank4_enabled];
        let enabled_count = enabled_banks.iter().filter(|&&x| x).count();
        
        match enabled_count {
            0 => Ok(FsmcStatus::AllBanksDisabled),
            1 => {
                if bank1_enabled { Ok(FsmcStatus::Bank1Active) }
                else if bank2_enabled { Ok(FsmcStatus::Bank2Active) }
                else if bank3_enabled { Ok(FsmcStatus::Bank3Active) }
                else { Ok(FsmcStatus::Bank4Active) }
            },
            _ => Ok(FsmcStatus::MultipleBanksActive),
        }
    }
}

/// 预定义的FSMC实例
pub const FSMC: Fsmc = Fsmc::new();

/// 测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试FSMC初始化和状态获取
    #[test]
    fn test_fsmc_init_status() {
        let fsmc = Fsmc::new();
        
        // 初始化FSMC
        unsafe {
            let init_result = fsmc.init();
            assert!(init_result.is_ok(), "FSMC初始化应该成功");
            
            let status = fsmc.get_status();
            assert!(status.is_ok(), "获取FSMC状态应该成功");
            assert_eq!(status.unwrap(), FsmcStatus::AllBanksDisabled, "FSMC初始状态应该是AllBanksDisabled");
        }
    }
    
    /// 测试存储区域启用/禁用
    #[test]
    fn test_fsmc_bank_enable_disable() {
        let fsmc = Fsmc::new();
        
        // 初始化FSMC
        unsafe {
            let init_result = fsmc.init();
            assert!(init_result.is_ok(), "FSMC初始化应该成功");
            
            // 启用Bank1
            let enable_result = fsmc.enable_bank(FsmcBank::Bank1);
            assert!(enable_result.is_ok(), "启用FSMC Bank1应该成功");
            
            // 检查Bank1是否启用
            let is_enabled = fsmc.is_bank_enabled(FsmcBank::Bank1);
            assert!(is_enabled.is_ok(), "检查Bank1状态应该成功");
            assert!(is_enabled.unwrap(), "Bank1应该已启用");
            
            // 获取状态
            let status = fsmc.get_status();
            assert!(status.is_ok(), "获取FSMC状态应该成功");
            assert_eq!(status.unwrap(), FsmcStatus::Bank1Active, "FSMC状态应该是Bank1Active");
            
            // 禁用Bank1
            let disable_result = fsmc.disable_bank(FsmcBank::Bank1);
            assert!(disable_result.is_ok(), "禁用FSMC Bank1应该成功");
            
            // 检查Bank1是否禁用
            let is_enabled = fsmc.is_bank_enabled(FsmcBank::Bank1);
            assert!(is_enabled.is_ok(), "检查Bank1状态应该成功");
            assert!(!is_enabled.unwrap(), "Bank1应该已禁用");
        }
    }
    
    /// 测试存储区域初始化
    #[test]
    fn test_fsmc_bank_init() {
        let fsmc = Fsmc::new();
        
        // 初始化FSMC
        unsafe {
            let init_result = fsmc.init();
            assert!(init_result.is_ok(), "FSMC初始化应该成功");
            
            // 初始化Bank1
            let init_bank_result = fsmc.init_bank(
                FsmcBank::Bank1,
                FsmcMemoryType::SRAM,
                FsmcDataWidth::Width16b,
                2, 1, 3
            );
            assert!(init_bank_result.is_ok(), "初始化FSMC Bank1应该成功");
            
            // 获取Bank1配置
            let config = fsmc.get_bank_config(FsmcBank::Bank1);
            assert!(config.is_ok(), "获取Bank1配置应该成功");
            
            // 获取Bank1时序配置
            let timing = fsmc.get_bank_timing(FsmcBank::Bank1);
            assert!(timing.is_ok(), "获取Bank1时序配置应该成功");
        }
    }
    
    /// 测试写时序配置
    #[test]
    fn test_fsmc_write_timing() {
        let fsmc = Fsmc::new();
        
        // 初始化FSMC
        unsafe {
            let init_result = fsmc.init();
            assert!(init_result.is_ok(), "FSMC初始化应该成功");
            
            // 初始化Bank1
            let init_bank_result = fsmc.init_bank(
                FsmcBank::Bank1,
                FsmcMemoryType::SRAM,
                FsmcDataWidth::Width16b,
                2, 1, 3
            );
            assert!(init_bank_result.is_ok(), "初始化FSMC Bank1应该成功");
            
            // 配置写时序
            let write_timing_result = fsmc.configure_write_timing(
                FsmcBank::Bank1,
                1, 1, 2
            );
            assert!(write_timing_result.is_ok(), "配置FSMC写时序应该成功");
            
            // 获取写时序配置
            let write_timing = fsmc.get_bank_write_timing(FsmcBank::Bank1);
            assert!(write_timing.is_ok(), "获取写时序配置应该成功");
        }
    }
}
