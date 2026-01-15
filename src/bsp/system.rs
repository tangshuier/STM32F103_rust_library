//! 系统初始化模块
//! 提供STM32F103C8T6的系统时钟初始化功能

#![allow(unused)]

use cortex_m::asm;
use cortex_m::peripheral;
use crate::bsp::delay;

// 导入内部生成的设备驱动库
use stm32f103::*;

// 定义常量
const HSE_STARTUP_TIMEOUT: u32 = 0x05000;

/// 系统初始化函数
/// 初始化STM32F103C8T6的系统时钟到72MHz
/// 
/// 配置：
/// - 使用HSE（外部高速时钟，8MHz）作为系统时钟源
/// - 通过PLL倍频到72MHz
/// - SYSCLK = 72MHz
/// - HCLK = 72MHz
/// - PCLK1 = 36MHz
/// - PCLK2 = 72MHz
pub unsafe fn init() {
    let rcc = unsafe { &mut *(0x40021000 as *mut stm32f103::rcc::RegisterBlock) };
    let flash = unsafe { &mut *(0x40022000 as *mut stm32f103::flash::RegisterBlock) };
    let scb: &mut cortex_m::peripheral::SCB = unsafe { &mut *(cortex_m::peripheral::SCB::PTR as *mut _) };
    
    // 1. 重置RCC时钟配置到默认状态
    
    // 设置HSION位（启用内部高速时钟作为备用）
    rcc.cr().modify(|_, w: &mut stm32f103::rcc::cr::W| w.hsion().set_bit());
    
    // 等待HSI就绪
    let mut timeout = 1000;
    while rcc.cr().read().hsirdy().bit_is_clear() && timeout > 0 {
        timeout -= 1;
    }
    
    // 重置SW, HPRE, PPRE1, PPRE2, ADCPRE和MCO位
    rcc.cfgr().modify(|_, w: &mut stm32f103::rcc::cfgr::W| w.bits(0xF8FF0000));
    
    // 重置HSEON, CSSON和PLLON位
    rcc.cr().modify(|_, w: &mut stm32f103::rcc::cr::W| w.bits(0xFEF6FFFF));
    
    // 重置HSEBYP位
    rcc.cr().modify(|_, w: &mut stm32f103::rcc::cr::W| w.bits(0xFFFBFFFF));
    
    // 重置PLLSRC, PLLXTPRE, PLLMUL和USBPRE/OTGFSPRE位
    rcc.cfgr().modify(|_, w: &mut stm32f103::rcc::cfgr::W| w.bits(0xFF80FFFF));
    
    // 禁用所有中断和清除挂起位
    rcc.cir().write(|w: &mut stm32f103::rcc::cir::W| w.bits(0x009F0000));
    
    // 2. 配置Flash
    // 启用预取缓冲区
    flash.acr().modify(|_, w: &mut stm32f103::flash::acr::W| w.prftbe().set_bit());
    
    // 设置Flash延迟为2个等待周期（针对72MHz）
    flash.acr().modify(|_, w: &mut stm32f103::flash::acr::W| w.latency().bits(0x02));
    
    // 3. 启用HSE（外部8MHz晶振）
    rcc.cr().modify(|_, w: &mut stm32f103::rcc::cr::W| w.hseon().set_bit());
    
    // 等待HSE就绪或超时
    timeout = HSE_STARTUP_TIMEOUT;
    while rcc.cr().read().hserdy().bit_is_clear() && timeout > 0 {
        timeout -= 1;
    }
    
    // 如果HSE启动成功，则配置为72MHz
    if rcc.cr().read().hserdy().bit_is_set() {
        // 4. 配置总线预分频
        // HCLK = SYSCLK
        rcc.cfgr().modify(|_, w: &mut stm32f103::rcc::cfgr::W| w.hpre().bits(0x0));
        
        // PCLK2 = HCLK
        rcc.cfgr().modify(|_, w: &mut stm32f103::rcc::cfgr::W| w.ppre2().bits(0x0));
        
        // PCLK1 = HCLK / 2
        rcc.cfgr().modify(|_, w: &mut stm32f103::rcc::cfgr::W| w.ppre1().bits(0x4));
        
        // 5. 配置PLL
        // HSE作为PLL输入，倍频系数为9，PLLCLK = 8MHz * 9 = 72MHz
        rcc.cfgr().modify(|_, w: &mut stm32f103::rcc::cfgr::W| {
            w.pllsrc().set_bit()
                .pllmul().bits(0x7)
        });
        
        // 6. 启用PLL
        rcc.cr().modify(|_, w: &mut stm32f103::rcc::cr::W| w.pllon().set_bit());
        
        // 等待PLL就绪
        while rcc.cr().read().pllrdy().bit_is_clear() {};
        
        // 7. 选择PLL作为系统时钟源
        rcc.cfgr().modify(|_, w: &mut stm32f103::rcc::cfgr::W| w.sw().bits(0x2));
        
        // 等待系统时钟切换到PLL
        while rcc.cfgr().read().sws().bits() != 0x02 {};
    }
    
    // 8. 设置向量表偏移
    scb.vtor.write(0x08000000);
    
    // 9. 初始化延时模块
    delay::init_systick();
}