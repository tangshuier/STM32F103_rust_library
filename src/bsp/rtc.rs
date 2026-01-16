//! RTC模块
//! 提供实时时钟功能封装，适配标准库API

#![allow(unused)]

// 导入内部生成的设备驱动库
use library::generic::*;
use library::rtc::{self, RegisterBlock as RtcRegisterBlock};
use library::rcc::{self, RegisterBlock as RccRegisterBlock};
use library::pwr::{self, RegisterBlock as PwrRegisterBlock};

/// RTC中断类型枚举
pub enum RtcInterrupt {
    Overflow = 0x0004,
    Alarm = 0x0002,
    Second = 0x0001,
}

/// RTC标志类型枚举
pub enum RtcFlag {
    Rtoff = 0x0020,
    Rsf = 0x0008,
    Overflow = 0x0004,
    Alarm = 0x0002,
    Second = 0x0001,
}

/// RTC结构体
pub struct Rtc;

impl Rtc {
    /// 创建新的RTC实例
    pub const fn new() -> Self {
        Self
    }
    
    /// RTC寄存器基地址
    const RTC_BASE: u32 = 0x40002800;
    /// RCC寄存器基地址
    const RCC_BASE: u32 = 0x40021000;
    /// PWR寄存器基地址
    const PWR_BASE: u32 = 0x40007000;
    
    /// 获取RTC寄存器块
    unsafe fn rtc(&self) -> &'static mut RtcRegisterBlock {
        &mut *(Self::RTC_BASE as *mut RtcRegisterBlock)
    }
    
    /// 获取RCC寄存器块
    unsafe fn rcc(&self) -> &'static mut RccRegisterBlock {
        &mut *(Self::RCC_BASE as *mut RccRegisterBlock)
    }
    
    /// 获取PWR寄存器块
    unsafe fn pwr(&self) -> &'static mut PwrRegisterBlock {
        &mut *(Self::PWR_BASE as *mut PwrRegisterBlock)
    }
    
    /// 初始化RTC
    pub unsafe fn init(&self, prescaler: u32) {
        let rcc = self.rcc();
        let pwr = self.pwr();
        
        // 启用PWR和BKP时钟
        rcc.apb1enr().modify(|_, w| w
            .pwren().set_bit()
            .bkpen().set_bit()
        );
        
        // 使能对备份域的访问
        pwr.cr().modify(|_, w| w
            .dbp().set_bit()
        );
        
        // 重置备份域
        rcc.bdcr().modify(|_, w| w
            .bdrst().set_bit()
        );
        rcc.bdcr().modify(|_, w| w
            .bdrst().clear_bit()
        );
        
        // 启用LSE振荡器
        rcc.bdcr().modify(|_, w| w
            .lseon().set_bit()
        );
        
        // 等待LSE就绪
        while rcc.bdcr().read().lserdy().bit_is_clear() {
            core::hint::spin_loop();
        }
        
        // 选择LSE作为RTC时钟源
        rcc.bdcr().modify(|_, w| w
            .rtcsel().bits(0b10)
        );
        
        // 启用RTC时钟
        rcc.bdcr().modify(|_, w| w
            .rtcen().set_bit()
        );
        
        // 等待RTC寄存器同步
        self.wait_for_synchro();
        
        // 设置预分频值
        self.set_prescaler(prescaler);
        
        // 等待RTC寄存器写入操作完成
        self.wait_for_last_task();
    }
    
    /// 进入配置模式
    pub unsafe fn enter_config_mode(&self) {
        let rtc = self.rtc();
        rtc.crl().modify(|_, w| w
            .cnf().set_bit()
        );
        // 等待配置模式进入
        while rtc.crl().read().cnf().bit_is_clear() {
            core::hint::spin_loop();
        }
    }
    
    /// 退出配置模式
    pub unsafe fn exit_config_mode(&self) {
        let rtc = self.rtc();
        rtc.crl().modify(|_, w| w
            .cnf().clear_bit()
        );
    }
    
    /// 等待RTC寄存器同步（标准库命名）
    pub unsafe fn wait_for_synchro(&self) {
        let rtc = self.rtc();
        rtc.crl().modify(|_, w| w
            .rsf().clear_bit()
        );
        while rtc.crl().read().rsf().bit_is_clear() {
            core::hint::spin_loop();
        }
    }
    
    /// 等待RTC寄存器写入操作完成
    pub unsafe fn wait_for_last_task(&self) {
        let rtc = self.rtc();
        // 等待rtoff位被硬件置位，表示RTC操作完成
        while rtc.crl().read().rtoff().bit_is_clear() {
            core::hint::spin_loop();
        }
    }
    
    /// 设置RTC预分频值
    pub unsafe fn set_prescaler(&self, prescaler: u32) {
        self.enter_config_mode();
        
        let rtc = self.rtc();
        rtc.prlh().write(|w| unsafe { w.bits((prescaler >> 16) & 0x0F) });
        rtc.prll().write(|w| unsafe { w.bits(prescaler & 0xFFFF) });
        
        self.exit_config_mode();
        self.wait_for_last_task();
    }
    
    /// 设置RTC计数器值
    pub unsafe fn set_counter(&self, counter: u32) {
        self.enter_config_mode();
        
        let rtc = self.rtc();
        rtc.cnth().write(|w| unsafe { w.bits((counter >> 16) & 0xFFFF) });
        rtc.cntl().write(|w| unsafe { w.bits(counter & 0xFFFF) });
        
        self.exit_config_mode();
        self.wait_for_last_task();
    }
    
    /// 获取RTC计数器值
    pub unsafe fn get_counter(&self) -> u32 {
        self.wait_for_synchro();
        
        let rtc = self.rtc();
        let cnth = rtc.cnth().read().bits();
        let cntl = rtc.cntl().read().bits();
        
        ((cnth as u32) << 16) | (cntl as u32)
    }
    
    /// 设置RTC闹钟值
    pub unsafe fn set_alarm(&self, alarm: u32) {
        self.enter_config_mode();
        
        let rtc = self.rtc();
        rtc.alrh().write(|w| unsafe { w.bits((alarm >> 16) & 0xFFFF) });
        rtc.alrl().write(|w| unsafe { w.bits(alarm & 0xFFFF) });
        
        self.exit_config_mode();
        self.wait_for_last_task();
    }
    
    /// 获取RTC闹钟值
    pub unsafe fn get_alarm(&self) -> u32 {
        // 注意：ALRH和ALRL是只写寄存器，不能读取
        // 这个方法实际上无法获取当前闹钟值，返回0作为占位
        0
    }
    
    /// 获取RTC分频器值
    pub unsafe fn get_divider(&self) -> u32 {
        self.wait_for_synchro();
        
        let rtc = self.rtc();
        let divh = rtc.divh().read().bits();
        let divl = rtc.divl().read().bits();
        
        ((divh as u32) << 8) | (divl as u32)
    }
    
    /// 配置RTC中断
    pub unsafe fn it_config(&self, interrupt: RtcInterrupt, new_state: bool) {
        self.enter_config_mode();
        
        let rtc = self.rtc();
        match interrupt {
            RtcInterrupt::Overflow => {
                if new_state {
                    rtc.crh().modify(|_, w| w.owie().set_bit());
                } else {
                    rtc.crh().modify(|_, w| w.owie().clear_bit());
                }
            },
            RtcInterrupt::Alarm => {
                if new_state {
                    rtc.crh().modify(|_, w| w.alrie().set_bit());
                } else {
                    rtc.crh().modify(|_, w| w.alrie().clear_bit());
                }
            },
            RtcInterrupt::Second => {
                if new_state {
                    rtc.crh().modify(|_, w| w.secie().set_bit());
                } else {
                    rtc.crh().modify(|_, w| w.secie().clear_bit());
                }
            },
        }
        
        self.exit_config_mode();
        self.wait_for_last_task();
    }
    
    /// 获取RTC标志状态
    pub unsafe fn get_flag_status(&self, flag: RtcFlag) -> bool {
        let rtc = self.rtc();
        let crl = rtc.crl().read();
        
        match flag {
            RtcFlag::Rtoff => crl.rtoff().bit_is_set(),
            RtcFlag::Rsf => crl.rsf().bit_is_set(),
            RtcFlag::Overflow => crl.owf().bit_is_set(),
            RtcFlag::Alarm => crl.alrf().bit_is_set(),
            RtcFlag::Second => crl.secf().bit_is_set(),
        }
    }
    
    /// 清除RTC标志
    pub unsafe fn clear_flag(&self, flag: RtcFlag) {
        let rtc = self.rtc();
        match flag {
            RtcFlag::Overflow => {
                rtc.crl().modify(|_, w| w.owf().clear_bit());
            },
            RtcFlag::Alarm => {
                rtc.crl().modify(|_, w| w.alrf().clear_bit());
            },
            RtcFlag::Second => {
                rtc.crl().modify(|_, w| w.secf().clear_bit());
            },
            _ => {}, // 其他标志不需要清除或自动清除
        }
    }
    
    /// 获取RTC中断状态
    pub unsafe fn get_it_status(&self, interrupt: RtcInterrupt) -> bool {
        let rtc = self.rtc();
        let crl = rtc.crl().read();
        let crh = rtc.crh().read();
        
        match interrupt {
            RtcInterrupt::Overflow => crh.owie().bit_is_set() && crl.owf().bit_is_set(),
            RtcInterrupt::Alarm => crh.alrie().bit_is_set() && crl.alrf().bit_is_set(),
            RtcInterrupt::Second => crh.secie().bit_is_set() && crl.secf().bit_is_set(),
        }
    }
    
    /// 清除RTC中断挂起位
    pub unsafe fn clear_it_pending_bit(&self, interrupt: RtcInterrupt) {
        let flag = match interrupt {
            RtcInterrupt::Overflow => RtcFlag::Overflow,
            RtcInterrupt::Alarm => RtcFlag::Alarm,
            RtcInterrupt::Second => RtcFlag::Second,
        };
        self.clear_flag(flag);
    }
}

/// 预定义的RTC实例
pub const RTC: Rtc = Rtc::new();
