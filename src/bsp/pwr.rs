//! PWR模块
//! 提供电源控制功能封装

#![allow(unused)]

// 导入内部生成的设备驱动库
use library::*;

/// PWR结构体
pub struct Pwr;

impl Pwr {
    /// 创建新的PWR实例
    pub const fn new() -> Self {
        Self
    }
    
    /// 获取PWR寄存器块
    unsafe fn pwr(&self) -> &'static mut library::pwr::RegisterBlock {
        &mut *(0x40007000 as *mut library::pwr::RegisterBlock)
    }
    
    /// 获取RCC寄存器块
    unsafe fn rcc(&self) -> &'static mut library::rcc::RegisterBlock {
        &mut *(0x40021000 as *mut library::rcc::RegisterBlock)
    }
    
    /// 初始化PWR
    pub unsafe fn init(&self) {
        let rcc = self.rcc();
        
        // 启用PWR时钟
        rcc.apb1enr().modify(|_, w: &mut library::rcc::apb1enr::W| w
            .pwren().set_bit()
        );
    }
    
    /// 使能对备份域的访问
    pub unsafe fn enable_backup_domain_access(&self) {
        let pwr = self.pwr();
        pwr.cr().modify(|_, w: &mut library::pwr::cr::W| w
            .dbp().set_bit()
        );
    }
    
    /// 禁用对备份域的访问
    pub unsafe fn disable_backup_domain_access(&self) {
        let pwr = self.pwr();
        pwr.cr().modify(|_, w: &mut library::pwr::cr::W| w
            .dbp().clear_bit()
        );
    }
    
    /// 启用PVD（可编程电压监测器）
    pub unsafe fn enable_pvd(&self) {
        let pwr = self.pwr();
        pwr.cr().modify(|_, w: &mut library::pwr::cr::W| w
            .pvde().set_bit()
        );
    }
    
    /// 禁用PVD（可编程电压监测器）
    pub unsafe fn disable_pvd(&self) {
        let pwr = self.pwr();
        pwr.cr().modify(|_, w: &mut library::pwr::cr::W| w
            .pvde().clear_bit()
        );
    }
    
    /// 设置PVD阈值
    pub unsafe fn set_pvd_level(&self, level: u8) {
        let pwr = self.pwr();
        let level_clamped = if level > 7 { 7 } else { level };
        pwr.cr().modify(|_, w: &mut library::pwr::cr::W| w
            .pls().bits(level_clamped)
        );
    }
    
    /// 进入睡眠模式
    pub unsafe fn enter_sleep_mode(&self, wait_for_interrupt: bool) {
        if wait_for_interrupt {
            // WFI指令
            core::arch::asm!("wfi");
        } else {
            // WFE指令
            core::arch::asm!("wfe");
        }
    }
    
    /// 进入停止模式
    pub unsafe fn enter_stop_mode(&self, regulator_low_power: bool) {
        let pwr = self.pwr();
        
        // 设置LPDS位
        if regulator_low_power {
            pwr.cr().modify(|_, w: &mut library::pwr::cr::W| w
                .lpds().set_bit()
            );
        } else {
            pwr.cr().modify(|_, w: &mut library::pwr::cr::W| w
                .lpds().clear_bit()
            );
        }
        
        // 设置PDDS位为0（停止模式）
        pwr.cr().modify(|_, w: &mut library::pwr::cr::W| w
            .pdds().clear_bit()
        );
        
        // 设置CWUF位
        pwr.cr().modify(|_, w: &mut library::pwr::cr::W| w
            .cwuf().set_bit()
        );
        
        // WFI指令
        core::arch::asm!("wfi");
    }
    
    /// 进入待机模式
    pub unsafe fn enter_standby_mode(&self) {
        let pwr = self.pwr();
        
        // 设置PDDS位为1（待机模式）
        pwr.cr().modify(|_, w: &mut library::pwr::cr::W| w
            .pdds().set_bit()
        );
        
        // 设置CWUF位
        pwr.cr().modify(|_, w: &mut library::pwr::cr::W| w
            .cwuf().set_bit()
        );
        
        // WFI指令
        core::arch::asm!("wfi");
    }
    
    /// 清除Wake-Up标志
    pub unsafe fn clear_wakeup_flag(&self) {
        let pwr = self.pwr();
        pwr.cr().modify(|_, w: &mut library::pwr::cr::W| w
            .cwuf().set_bit()
        );
    }
    
    /// 清除待机标志
    pub unsafe fn clear_standby_flag(&self) {
        let pwr = self.pwr();
        pwr.cr().modify(|_, w: &mut library::pwr::cr::W| w
            .csbf().set_bit()
        );
    }
    
    /// 检查Wake-Up标志
    pub unsafe fn get_wakeup_flag(&self) -> bool {
        let pwr = self.pwr();
        pwr.csr().read().wuf().bit_is_set()
    }
    
    /// 检查待机标志
    pub unsafe fn get_standby_flag(&self) -> bool {
        let pwr = self.pwr();
        pwr.csr().read().sbf().bit_is_set()
    }
    
    /// 检查PVD输出
    pub unsafe fn get_pvd_output(&self) -> bool {
        let pwr = self.pwr();
        pwr.csr().read().pvdo().bit_is_set()
    }
}

/// 预定义的PWR实例
pub const PWR: Pwr = Pwr::new();
