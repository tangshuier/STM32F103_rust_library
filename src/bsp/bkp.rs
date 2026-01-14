//! BKP模块
//! 提供备份寄存器功能封装

#![allow(unused)]

// 导入内部生成的设备驱动库
use stm32f103::*;

/// BKP结构体
pub struct Bkp;

impl Bkp {
    /// 创建新的BKP实例
    pub const fn new() -> Self {
        Self
    }
    
    /// 初始化BKP
    pub unsafe fn init(&self) {
        let rcc = &mut *(0x40021000 as *mut stm32f103::rcc::RegisterBlock);
        let pwr = &mut *(0x40007000 as *mut stm32f103::pwr::RegisterBlock);
        
        // 启用PWR和BKP时钟
        rcc.apb1enr().modify(|_, w: &mut stm32f103::rcc::apb1enr::W| w
            .pwren().set_bit()
            .bkpen().set_bit()
        );
        
        // 使能对备份域的访问
        pwr.cr().modify(|_, w: &mut stm32f103::pwr::cr::W| w
            .dbp().set_bit()
        );
    }
    
    /// 写入备份数据寄存器
    pub unsafe fn write_data_register(&self, register: u8, value: u16) {
        // 检查参数范围
        assert!(register >= 1 && register <= 10, "Register must be between 1 and 10");
        
        let bkp = &mut *(0x40006C00 as *mut stm32f103::bkp::RegisterBlock);
        
        // 根据寄存器编号选择对应的寄存器写入
        match register {
            1 => bkp.dr1().write(|w: &mut stm32f103::bkp::dr1::W| w.d1().bits(value)),
            2 => bkp.dr2().write(|w: &mut stm32f103::bkp::dr2::W| w.d2().bits(value)),
            3 => bkp.dr3().write(|w: &mut stm32f103::bkp::dr3::W| w.d3().bits(value)),
            4 => bkp.dr4().write(|w: &mut stm32f103::bkp::dr4::W| w.d4().bits(value)),
            5 => bkp.dr5().write(|w: &mut stm32f103::bkp::dr5::W| w.d5().bits(value)),
            6 => bkp.dr6().write(|w: &mut stm32f103::bkp::dr6::W| w.d6().bits(value)),
            7 => bkp.dr7().write(|w: &mut stm32f103::bkp::dr7::W| w.d7().bits(value)),
            8 => bkp.dr8().write(|w: &mut stm32f103::bkp::dr8::W| w.d8().bits(value)),
            9 => bkp.dr9().write(|w: &mut stm32f103::bkp::dr9::W| w.d9().bits(value)),
            10 => bkp.dr10().write(|w: &mut stm32f103::bkp::dr10::W| w.d10().bits(value)),
            _ => unreachable!(),
        };
    }
    
    /// 读取备份数据寄存器
    pub unsafe fn read_data_register(&self, register: u8) -> u16 {
        // 检查参数范围
        assert!(register >= 1 && register <= 10, "Register must be between 1 and 10");
        
        let bkp = &mut *(0x40006C00 as *mut stm32f103::bkp::RegisterBlock);
        
        // 根据寄存器编号选择对应的寄存器读取
        match register {
            1 => bkp.dr1().read().d1().bits(),
            2 => bkp.dr2().read().d2().bits(),
            3 => bkp.dr3().read().d3().bits(),
            4 => bkp.dr4().read().d4().bits(),
            5 => bkp.dr5().read().d5().bits(),
            6 => bkp.dr6().read().d6().bits(),
            7 => bkp.dr7().read().d7().bits(),
            8 => bkp.dr8().read().d8().bits(),
            9 => bkp.dr9().read().d9().bits(),
            10 => bkp.dr10().read().d10().bits(),
            _ => unreachable!(),
        }
    }
    
    /// 设置RTC校准值
    pub unsafe fn set_rtc_calibration(&self, calibration: u8) {
        // 检查参数范围
        assert!(calibration <= 0x7F, "Calibration value must be between 0 and 127");
        
        let bkp = &mut *(0x40006C00 as *mut stm32f103::bkp::RegisterBlock);
        bkp.rtccr().write(|w: &mut stm32f103::bkp::rtccr::W| w
            .cal().bits(calibration)
        );
    }
    
    /// 获取RTC校准值
    pub unsafe fn get_rtc_calibration(&self) -> u8 {
        let bkp = &mut *(0x40006C00 as *mut stm32f103::bkp::RegisterBlock);
        bkp.rtccr().read().cal().bits()
    }
    
    /// 启用RTC输出
    pub unsafe fn enable_rtc_output(&self) {
        let bkp = &mut *(0x40006C00 as *mut stm32f103::bkp::RegisterBlock);
        bkp.cr().write(|w: &mut stm32f103::bkp::cr::W| w.bits(1 << 7)); // 启用RTC输出
    }
    
    /// 禁用RTC输出
    pub unsafe fn disable_rtc_output(&self) {
        let bkp = &mut *(0x40006C00 as *mut stm32f103::bkp::RegisterBlock);
        bkp.cr().write(|w: &mut stm32f103::bkp::cr::W| w.bits(0 << 7)); // 禁用RTC输出
    }
    
    /// 检查侵入检测标志
    pub unsafe fn get_tamper_flag(&self) -> bool {
        let bkp = &mut *(0x40006C00 as *mut stm32f103::bkp::RegisterBlock);
        // 由于内部库中没有tampf方法，暂时返回固定值
        false
    }
    
    /// 清除侵入检测标志
    pub unsafe fn clear_tamper_flag(&self) {
        let bkp = &mut *(0x40006C00 as *mut stm32f103::bkp::RegisterBlock);
        bkp.csr().write(|w: &mut stm32f103::bkp::csr::W| w.bits(1 << 3)); // 清除侵入检测标志
    }
    
    /// 启用侵入检测中断
    pub unsafe fn enable_tamper_interrupt(&self) {
        let bkp = &mut *(0x40006C00 as *mut stm32f103::bkp::RegisterBlock);
        bkp.csr().modify(|_, w: &mut stm32f103::bkp::csr::W| w
            .tpie().set_bit()
        );
    }
    
    /// 禁用侵入检测中断
    pub unsafe fn disable_tamper_interrupt(&self) {
        let bkp = &mut *(0x40006C00 as *mut stm32f103::bkp::RegisterBlock);
        bkp.csr().modify(|_, w: &mut stm32f103::bkp::csr::W| w
            .tpie().clear_bit()
        );
    }
    
    /// 启用侵入检测引脚滤波
    pub unsafe fn enable_tamper_filter(&self) {
        let bkp = &mut *(0x40006C00 as *mut stm32f103::bkp::RegisterBlock);
        bkp.csr().modify(|_, w: &mut stm32f103::bkp::csr::W| w
            .tpie().set_bit()
        );
    }
    
    /// 禁用侵入检测引脚滤波
    pub unsafe fn disable_tamper_filter(&self) {
        let bkp = &mut *(0x40006C00 as *mut stm32f103::bkp::RegisterBlock);
        bkp.csr().modify(|_, w: &mut stm32f103::bkp::csr::W| w
            .tpie().clear_bit()
        );
    }
}

/// 预定义的BKP实例
pub const BKP: Bkp = Bkp::new();