//! RTC模块
//! 提供实时时钟功能封装

#![allow(unused)]

// 导入内部生成的设备驱动库
use stm32f103::*;

/// RTC结构体
pub struct Rtc;

impl Rtc {
    /// 创建新的RTC实例
    pub const fn new() -> Self {
        Self
    }
    
    /// 获取RTC寄存器块
    unsafe fn rtc(&self) -> &'static mut stm32f103::rtc::RegisterBlock {
        &mut *(0x40002800 as *mut stm32f103::rtc::RegisterBlock)
    }
    
    /// 获取RCC寄存器块
    unsafe fn rcc(&self) -> &'static mut stm32f103::rcc::RegisterBlock {
        &mut *(0x40021000 as *mut stm32f103::rcc::RegisterBlock)
    }
    
    /// 获取PWR寄存器块
    unsafe fn pwr(&self) -> &'static mut stm32f103::pwr::RegisterBlock {
        &mut *(0x40007000 as *mut stm32f103::pwr::RegisterBlock)
    }
    
    /// 初始化RTC
    pub unsafe fn init(&self, prescaler: u32) {
        let rcc = self.rcc();
        let pwr = self.pwr();
        
        // 启用PWR和BKP时钟
        rcc.apb1enr().modify(|_, w: &mut stm32f103::rcc::apb1enr::W| w
            .pwren().set_bit()
            .bkpen().set_bit()
        );
        
        // 使能对备份域的访问
        pwr.cr().modify(|_, w: &mut stm32f103::pwr::cr::W| w
            .dbp().set_bit()
        );
        
        // 重置备份域
        rcc.bdcr().modify(|_, w: &mut stm32f103::rcc::bdcr::W| w
            .bdrst().set_bit()
        );
        rcc.bdcr().modify(|_, w: &mut stm32f103::rcc::bdcr::W| w
            .bdrst().clear_bit()
        );
        
        // 启用LSE振荡器
        rcc.bdcr().modify(|_, w: &mut stm32f103::rcc::bdcr::W| w
            .lseon().set_bit()
        );
        
        // 等待LSE就绪
        while rcc.bdcr().read().lserdy().bit_is_clear() {
            core::hint::spin_loop();
        }
        
        // 选择LSE作为RTC时钟源
        rcc.bdcr().modify(|_, w: &mut stm32f103::rcc::bdcr::W| w
            .rtcsel().bits(0b10)
        );
        
        // 启用RTC时钟
        rcc.bdcr().modify(|_, w: &mut stm32f103::rcc::bdcr::W| w
            .rtcen().set_bit()
        );
        
        // 进入配置模式
        self.enter_config_mode();
        
        // 设置预分频值
        let rtc = self.rtc();
        rtc.prlh().write(|w: &mut stm32f103::rtc::prlh::W| unsafe { w.bits((prescaler >> 16) & 0x0F) });
        rtc.prll().write(|w: &mut stm32f103::rtc::prll::W| unsafe { w.bits(prescaler & 0xFFFF) });
        
        // 退出配置模式
        self.exit_config_mode();
        
        // 等待RTC寄存器同步
        self.wait_for_sync();
        
        // 等待RTC寄存器写入操作完成
        self.wait_for_last_task();
    }
    
    /// 进入配置模式
    pub unsafe fn enter_config_mode(&self) {
        let rtc = self.rtc();
        rtc.crl().modify(|_, w: &mut stm32f103::rtc::crl::W| w
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
        rtc.crl().modify(|_, w: &mut stm32f103::rtc::crl::W| w
            .cnf().clear_bit()
        );
    }
    
    /// 等待RTC寄存器同步
    pub unsafe fn wait_for_sync(&self) {
        let rtc = self.rtc();
        rtc.crl().modify(|_, w: &mut stm32f103::rtc::crl::W| w
            .rsf().clear_bit()
        );
        while rtc.crl().read().rsf().bit_is_clear() {
            core::hint::spin_loop();
        }
    }
    
    /// 等待RTC寄存器写入操作完成
    pub unsafe fn wait_for_last_task(&self) {
        let rtc = self.rtc();
        // rtoff是只读字段，由硬件自动设置
        // 等待rtoff位被硬件置位，表示RTC操作完成
        while rtc.crl().read().rtoff().bit_is_clear() {
            core::hint::spin_loop();
        }
    }
    
    /// 设置RTC计数器值
    pub unsafe fn set_counter(&self, counter: u32) {
        let rtc = self.rtc();
        self.enter_config_mode();
        
        rtc.cnth().write(|w: &mut stm32f103::rtc::cnth::W| unsafe { w.bits((counter >> 16) & 0xFFFF) });
        rtc.cntl().write(|w: &mut stm32f103::rtc::cntl::W| unsafe { w.bits(counter & 0xFFFF) });
        
        self.exit_config_mode();
        self.wait_for_last_task();
    }
    
    /// 获取RTC计数器值
    pub unsafe fn get_counter(&self) -> u32 {
        self.wait_for_sync();
        
        let rtc = self.rtc();
        let cnth = rtc.cnth().read().bits();
        let cntl = rtc.cntl().read().bits();
        
        ((cnth as u32) << 16) | (cntl as u32)
    }
    
    /// 设置RTC闹钟值
    pub unsafe fn set_alarm(&self, alarm: u32) {
        let rtc = self.rtc();
        self.enter_config_mode();
        
        rtc.alrh().write(|w: &mut stm32f103::rtc::alrh::W| unsafe { w.bits((alarm >> 16) & 0xFFFF) });
        rtc.alrl().write(|w: &mut stm32f103::rtc::alrl::W| unsafe { w.bits(alarm & 0xFFFF) });
        
        self.exit_config_mode();
        self.wait_for_last_task();
    }
    
    /// 获取RTC闹钟值
    pub unsafe fn get_alarm(&self) -> u32 {
        // 注意：ALRH和ALRL是只写寄存器，不能读取
        // 这个方法实际上无法获取当前闹钟值，返回0作为占位
        0
    }
    
    /// 启用RTC秒中断
    pub unsafe fn enable_second_interrupt(&self) {
        let rtc = self.rtc();
        self.enter_config_mode();
        
        rtc.crh().modify(|_, w: &mut stm32f103::rtc::crh::W| w
            .secie().set_bit()
        );
        
        self.exit_config_mode();
        self.wait_for_last_task();
    }
    
    /// 禁用RTC秒中断
    pub unsafe fn disable_second_interrupt(&self) {
        let rtc = self.rtc();
        self.enter_config_mode();
        
        rtc.crh().modify(|_, w: &mut stm32f103::rtc::crh::W| w
            .secie().clear_bit()
        );
        
        self.exit_config_mode();
        self.wait_for_last_task();
    }
    
    /// 启用RTC闹钟中断
    pub unsafe fn enable_alarm_interrupt(&self) {
        let rtc = self.rtc();
        self.enter_config_mode();
        
        rtc.crh().modify(|_, w: &mut stm32f103::rtc::crh::W| w
            .alrie().set_bit()
        );
        
        self.exit_config_mode();
        self.wait_for_last_task();
    }
    
    /// 禁用RTC闹钟中断
    pub unsafe fn disable_alarm_interrupt(&self) {
        let rtc = self.rtc();
        self.enter_config_mode();
        
        rtc.crh().modify(|_, w: &mut stm32f103::rtc::crh::W| w
            .alrie().clear_bit()
        );
        
        self.exit_config_mode();
        self.wait_for_last_task();
    }
    
    /// 启用RTC溢出中断
    pub unsafe fn enable_overflow_interrupt(&self) {
        let rtc = self.rtc();
        self.enter_config_mode();
        
        rtc.crh().modify(|_, w: &mut stm32f103::rtc::crh::W| w
            .owie().set_bit()
        );
        
        self.exit_config_mode();
        self.wait_for_last_task();
    }
    
    /// 禁用RTC溢出中断
    pub unsafe fn disable_overflow_interrupt(&self) {
        let rtc = self.rtc();
        self.enter_config_mode();
        
        rtc.crh().modify(|_, w: &mut stm32f103::rtc::crh::W| w
            .owie().clear_bit()
        );
        
        self.exit_config_mode();
        self.wait_for_last_task();
    }
    
    /// 清除RTC秒中断标志
    pub unsafe fn clear_second_flag(&self) {
        let rtc = self.rtc();
        rtc.crl().modify(|_, w: &mut stm32f103::rtc::crl::W| w
            .secf().clear_bit()
        );
    }
    
    /// 清除RTC闹钟中断标志
    pub unsafe fn clear_alarm_flag(&self) {
        let rtc = self.rtc();
        rtc.crl().modify(|_, w: &mut stm32f103::rtc::crl::W| w
            .alrf().clear_bit()
        );
    }
    
    /// 清除RTC溢出中断标志
    pub unsafe fn clear_overflow_flag(&self) {
        let rtc = self.rtc();
        rtc.crl().modify(|_, w: &mut stm32f103::rtc::crl::W| w
            .owf().clear_bit()
        );
    }
    
    /// 检查RTC秒中断标志
    pub unsafe fn get_second_flag(&self) -> bool {
        let rtc = self.rtc();
        rtc.crl().read().secf().bit_is_set()
    }
    
    /// 检查RTC闹钟中断标志
    pub unsafe fn get_alarm_flag(&self) -> bool {
        let rtc = self.rtc();
        rtc.crl().read().alrf().bit_is_set()
    }
    
    /// 检查RTC溢出中断标志
    pub unsafe fn get_overflow_flag(&self) -> bool {
        let rtc = self.rtc();
        rtc.crl().read().owf().bit_is_set()
    }
}

/// 预定义的RTC实例
pub const RTC: Rtc = Rtc::new();
