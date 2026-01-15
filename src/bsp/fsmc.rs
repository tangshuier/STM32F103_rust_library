//! FSMC模块
//! 提供灵活的静态存储器控制器功能封装

#![allow(unused)]

// 导入内部生成的设备驱动库
use stm32f103::*;

/// FSMC存储区域枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FsmcBank {
    Bank1 = 0,    // 存储区域1
    Bank2 = 1,    // 存储区域2
    Bank3 = 2,    // 存储区域3
    Bank4 = 3,    // 存储区域4
}

/// FSMC存储器类型枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FsmcMemoryType {
    SRAM = 0,              // SRAM
    PSRAM = 1,             // PSRAM
    NorFlash = 2,         // NOR Flash
    NandFlash = 3,        // NAND Flash
}

/// FSMC数据总线宽度枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FsmcDataWidth {
    Width8b = 0,     // 8位
    Width16b = 1,    // 16位
}

/// FSMC结构体
pub struct Fsmc;

impl Fsmc {
    /// 创建新的FSMC实例
    pub const fn new() -> Self {
        Self
    }
    
    /// 获取FSMC寄存器块
    unsafe fn fsmc() -> &'static mut stm32f103::fsmc::RegisterBlock {
        &mut *(0xA0000000 as *mut stm32f103::fsmc::RegisterBlock)
    }
    
    /// 初始化FSMC存储区域
    /// 
    /// # 参数
    /// * `bank` - 存储区域
    /// * `mem_type` - 存储器类型
    /// * `data_width` - 数据总线宽度
    /// * `address_setup_time` - 地址建立时间 (HCLK周期数)
    /// * `address_hold_time` - 地址保持时间 (HCLK周期数)
    /// * `data_setup_time` - 数据建立时间 (HCLK周期数)
    pub unsafe fn init_bank(
        &self,
        bank: FsmcBank,
        mem_type: FsmcMemoryType,
        data_width: FsmcDataWidth,
        address_setup_time: u8,
        address_hold_time: u8,
        data_setup_time: u8
    ) {
        let fsmc = Fsmc::fsmc();
        
        // 获取对应的BCR和BTR寄存器
        match bank {
            FsmcBank::Bank1 => {
                // 重置BCR寄存器
                fsmc.bcr1().write(|w: &mut stm32f103::fsmc::bcr1::W| unsafe { w.bits(0x00000000) });
                
                // 配置存储器类型和数据总线宽度
                fsmc.bcr1().write(|w: &mut stm32f103::fsmc::bcr1::W| unsafe { 
                    w.bits(
                        ((mem_type as u32) << 4) |
                        ((data_width as u32) << 1) |
                        (1 << 0) // 启用存储区域
                    ) 
                });
                
                // 配置时序参数
                fsmc.btr1().write(|w: &mut stm32f103::fsmc::btr1::W| unsafe { 
                    w.bits(
                        ((address_setup_time as u32) << 0) |
                        ((address_hold_time as u32) << 4) |
                        ((data_setup_time as u32) << 8)
                    ) 
                });
            },
            FsmcBank::Bank2 => {
                // 重置BCR寄存器
                fsmc.bcr2().write(|w: &mut stm32f103::fsmc::bcr2::W| unsafe { w.bits(0x00000000) });
                
                // 配置存储器类型和数据总线宽度
                fsmc.bcr2().write(|w: &mut stm32f103::fsmc::bcr2::W| unsafe { 
                    w.bits(
                        ((mem_type as u32) << 4) |
                        ((data_width as u32) << 1) |
                        (1 << 0) // 启用存储区域
                    ) 
                });
                
                // 配置时序参数
                fsmc.btr2().write(|w: &mut stm32f103::fsmc::btr2::W| unsafe { 
                    w.bits(
                        ((address_setup_time as u32) << 0) |
                        ((address_hold_time as u32) << 4) |
                        ((data_setup_time as u32) << 8)
                    ) 
                });
            },
            FsmcBank::Bank3 => {
                // 重置BCR寄存器
                fsmc.bcr3().write(|w: &mut stm32f103::fsmc::bcr3::W| unsafe { w.bits(0x00000000) });
                
                // 配置存储器类型和数据总线宽度
                fsmc.bcr3().write(|w: &mut stm32f103::fsmc::bcr3::W| unsafe { 
                    w.bits(
                        ((mem_type as u32) << 4) |
                        ((data_width as u32) << 1) |
                        (1 << 0) // 启用存储区域
                    ) 
                });
                
                // 配置时序参数
                fsmc.btr3().write(|w: &mut stm32f103::fsmc::btr3::W| unsafe { 
                    w.bits(
                        ((address_setup_time as u32) << 0) |
                        ((address_hold_time as u32) << 4) |
                        ((data_setup_time as u32) << 8)
                    ) 
                });
            },
            FsmcBank::Bank4 => {
                // 重置BCR寄存器
                fsmc.bcr4().write(|w: &mut stm32f103::fsmc::bcr4::W| unsafe { w.bits(0x00000000) });
                
                // 配置存储器类型和数据总线宽度
                fsmc.bcr4().write(|w: &mut stm32f103::fsmc::bcr4::W| unsafe { 
                    w.bits(
                        ((mem_type as u32) << 4) |
                        ((data_width as u32) << 1) |
                        (1 << 0) // 启用存储区域
                    ) 
                });
                
                // 配置时序参数
                fsmc.btr4().write(|w: &mut stm32f103::fsmc::btr4::W| unsafe { 
                    w.bits(
                        ((address_setup_time as u32) << 0) |
                        ((address_hold_time as u32) << 4) |
                        ((data_setup_time as u32) << 8)
                    ) 
                });
            },
        }
    }
    
    /// 配置FSMC存储区域的写时序
    /// 
    /// # 参数
    /// * `bank` - 存储区域
    /// * `address_setup_time` - 地址建立时间 (HCLK周期数)
    /// * `address_hold_time` - 地址保持时间 (HCLK周期数)
    /// * `data_setup_time` - 数据建立时间 (HCLK周期数)
    pub unsafe fn configure_write_timing(
        &self,
        bank: FsmcBank,
        address_setup_time: u8,
        address_hold_time: u8,
        data_setup_time: u8
    ) {
        let fsmc = Fsmc::fsmc();
        
        // 获取对应的BWTR寄存器
        match bank {
            FsmcBank::Bank1 => {
                // 配置写时序参数
                fsmc.bwtr1().write(|w: &mut stm32f103::fsmc::bwtr1::W| unsafe { 
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
                fsmc.bwtr2().write(|w: &mut stm32f103::fsmc::bwtr2::W| unsafe { 
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
                fsmc.bwtr3().write(|w: &mut stm32f103::fsmc::bwtr3::W| unsafe { 
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
                fsmc.bwtr4().write(|w: &mut stm32f103::fsmc::bwtr4::W| unsafe { 
                    w.bits(
                        ((address_setup_time as u32) << 0) |
                        ((address_hold_time as u32) << 4) |
                        ((data_setup_time as u32) << 8) |
                        (1 << 16) // 启用写时序配置
                    ) 
                });
            },
        }
    }
    
    /// 启用存储区域
    /// 
    /// # 参数
    /// * `bank` - 存储区域
    pub unsafe fn enable_bank(&self, bank: FsmcBank) {
        let fsmc = Fsmc::fsmc();
        
        match bank {
            FsmcBank::Bank1 => {
                fsmc.bcr1().modify(|r: &stm32f103::fsmc::bcr1::R, w: &mut stm32f103::fsmc::bcr1::W| unsafe { 
                    w.bits(r.bits() | (1 << 0)) 
                });
            },
            FsmcBank::Bank2 => {
                fsmc.bcr2().modify(|r: &stm32f103::fsmc::bcr2::R, w: &mut stm32f103::fsmc::bcr2::W| unsafe { 
                    w.bits(r.bits() | (1 << 0)) 
                });
            },
            FsmcBank::Bank3 => {
                fsmc.bcr3().modify(|r: &stm32f103::fsmc::bcr3::R, w: &mut stm32f103::fsmc::bcr3::W| unsafe { 
                    w.bits(r.bits() | (1 << 0)) 
                });
            },
            FsmcBank::Bank4 => {
                fsmc.bcr4().modify(|r: &stm32f103::fsmc::bcr4::R, w: &mut stm32f103::fsmc::bcr4::W| unsafe { 
                    w.bits(r.bits() | (1 << 0)) 
                });
            },
        }
    }
    
    /// 禁用存储区域
    /// 
    /// # 参数
    /// * `bank` - 存储区域
    pub unsafe fn disable_bank(&self, bank: FsmcBank) {
        let fsmc = Fsmc::fsmc();
        
        match bank {
            FsmcBank::Bank1 => {
                fsmc.bcr1().modify(|r: &stm32f103::fsmc::bcr1::R, w: &mut stm32f103::fsmc::bcr1::W| unsafe { 
                    w.bits(r.bits() & !(1 << 0)) 
                });
            },
            FsmcBank::Bank2 => {
                fsmc.bcr2().modify(|r: &stm32f103::fsmc::bcr2::R, w: &mut stm32f103::fsmc::bcr2::W| unsafe { 
                    w.bits(r.bits() & !(1 << 0)) 
                });
            },
            FsmcBank::Bank3 => {
                fsmc.bcr3().modify(|r: &stm32f103::fsmc::bcr3::R, w: &mut stm32f103::fsmc::bcr3::W| unsafe { 
                    w.bits(r.bits() & !(1 << 0)) 
                });
            },
            FsmcBank::Bank4 => {
                fsmc.bcr4().modify(|r: &stm32f103::fsmc::bcr4::R, w: &mut stm32f103::fsmc::bcr4::W| unsafe { 
                    w.bits(r.bits() & !(1 << 0)) 
                });
            },
        }
    }
    
    /// 读取存储区域配置
    /// 
    /// # 参数
    /// * `bank` - 存储区域
    /// 
    /// # 返回值
    /// 存储区域配置
    pub unsafe fn get_bank_config(&self, bank: FsmcBank) -> u32 {
        let fsmc = Fsmc::fsmc();
        
        match bank {
            FsmcBank::Bank1 => fsmc.bcr1().read().bits(),
            FsmcBank::Bank2 => fsmc.bcr2().read().bits(),
            FsmcBank::Bank3 => fsmc.bcr3().read().bits(),
            FsmcBank::Bank4 => fsmc.bcr4().read().bits(),
        }
    }
    
    /// 读取存储区域时序配置
    /// 
    /// # 参数
    /// * `bank` - 存储区域
    /// 
    /// # 返回值
    /// 存储区域时序配置
    pub unsafe fn get_bank_timing(&self, bank: FsmcBank) -> u32 {
        let fsmc = Fsmc::fsmc();
        
        match bank {
            FsmcBank::Bank1 => fsmc.btr1().read().bits(),
            FsmcBank::Bank2 => fsmc.btr2().read().bits(),
            FsmcBank::Bank3 => fsmc.btr3().read().bits(),
            FsmcBank::Bank4 => fsmc.btr4().read().bits(),
        }
    }
    
    /// 读取存储区域写时序配置
    /// 
    /// # 参数
    /// * `bank` - 存储区域
    /// 
    /// # 返回值
    /// 存储区域写时序配置
    pub unsafe fn get_bank_write_timing(&self, bank: FsmcBank) -> u32 {
        let fsmc = Fsmc::fsmc();
        
        match bank {
            FsmcBank::Bank1 => fsmc.bwtr1().read().bits(),
            FsmcBank::Bank2 => fsmc.bwtr2().read().bits(),
            FsmcBank::Bank3 => fsmc.bwtr3().read().bits(),
            FsmcBank::Bank4 => fsmc.bwtr4().read().bits(),
        }
    }
}

/// 预定义的FSMC实例
pub const FSMC: Fsmc = Fsmc::new();