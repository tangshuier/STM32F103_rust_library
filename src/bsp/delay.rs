//! 延时和计时功能模块
//! 提供基于SysTick的高精度延时和计时API

// 屏蔽未使用代码警告
#![allow(unused)]

use core::sync::atomic::Ordering;
use core::arch::asm;

/// 初始化系统滴答定时器（如果尚未初始化）
/// 
/// 配置SysTick为1kHz，根据实际系统时钟频率计算重装载值，不使用中断
/// 
/// # Safety
/// 直接访问硬件寄存器，需要确保在正确的上下文中调用
pub unsafe fn init_systick() {
    // 检查SYSTICK是否已初始化
    let csr = core::ptr::read_volatile(0xE000E010 as *const u32);
    if (csr & 0x01) == 0 {
        // 读取系统时钟配置，确定当前系统时钟频率
        let rcc_cfgr = core::ptr::read_volatile(0x40021004 as *const u32);
        let sysclk_source = rcc_cfgr & 0x0C;
        
        // 根据系统时钟源计算重装载值
        let reload_value = if sysclk_source == 0x08 {
            // PLL作为系统时钟源（72MHz）
            71999 // (72MHz / 1kHz) - 1
        } else {
            // HSI作为系统时钟源（8MHz）
            7999 // (8MHz / 1kHz) - 1
        };
        
        // 配置SYSTICK为1kHz
        core::ptr::write_volatile(0xE000E014 as *mut u32, reload_value);
        // 清空当前值
        core::ptr::write_volatile(0xE000E018 as *mut u32, 0);
        // 启用SYSTICK，使用处理器时钟，**不**启用中断
        core::ptr::write_volatile(0xE000E010 as *mut u32, 0x05); // 0x05 = ENABLE + CLKSOURCE，不设置TICKINT位
    }
}

/// 基于系统时钟的延时函数（微秒）
/// 
/// 实现高精度的微秒级延时，结合SysTick和空循环
/// 对于大于1ms的延时，使用SysTick的COUNTFLAG标志
/// 对于小于1ms的延时，使用精确的空循环
/// 
/// # Arguments
/// * `us` - 延时时间，单位：微秒
/// 
/// # Safety
/// 直接访问硬件寄存器，需要确保在正确的上下文中调用
pub unsafe fn delay_us(us: u32) {
    // 确保SYSTICK已初始化
    init_systick();
    
    if us == 0 {
        return;
    }
    
    // 对于大于1ms的延时，使用SysTick的COUNTFLAG标志
    if us >= 1000 {
        let ms = us / 1000;
        for _ in 0..ms {
            // 等待SysTick计数完成
            while (core::ptr::read_volatile(0xE000E010 as *const u32) & (1 << 16)) == 0 {
                core::sync::atomic::compiler_fence(Ordering::SeqCst);
            }
        }
        
        // 处理剩余的微秒
        let remaining_us = us % 1000;
        if remaining_us > 0 {
            delay_us_precise(remaining_us);
        }
    } else {
        // 对于小于1ms的延时，使用精确的空循环
        delay_us_precise(us);
    }
}

/// 精确的微秒级延时，基于空循环
/// 
/// 假设CPU频率为72MHz，每个循环大约需要14ns
/// 1us = 72个循环
/// 
/// # Arguments
/// * `us` - 延时时间，单位：微秒
/// 
/// # Safety
/// 使用内联汇编，需要确保在正确的上下文中调用
#[inline(always)]
unsafe fn delay_us_precise(us: u32) {
    // 根据72MHz系统时钟计算循环次数
    // 每个循环大约需要14ns (1/72MHz)
    // 1us = 72个循环
    let cycles_per_us = 72;
    let total_cycles = us as u32 * cycles_per_us;
    
    // 使用内联汇编实现精确的空循环
    asm!(
        "mov r0, {cycles}",
        "0:",
        "subs r0, r0, #1",
        "bne 0b",
        cycles = in(reg) total_cycles,
        options(nomem, nostack, preserves_flags),
    );
}

/// 基于系统时钟的延时函数（毫秒）
/// 
/// 使用SysTick的COUNTFLAG标志实现精确的毫秒级延时，不依赖中断
/// 
/// # Arguments
/// * `ms` - 延时时间，单位：毫秒
/// 
/// # Safety
/// 直接访问硬件寄存器，需要确保在正确的上下文中调用
pub unsafe fn delay_ms(ms: u32) {
    // 确保SYSTICK已初始化
    init_systick();
    
    // 使用SysTick的COUNTFLAG标志实现延时
    for _ in 0..ms {
        // 等待SysTick计数完成
        while (core::ptr::read_volatile(0xE000E010 as *const u32) & (1 << 16)) == 0 {
            core::sync::atomic::compiler_fence(Ordering::SeqCst);
        }
        // COUNTFLAG被读取后会自动清零，无需额外操作
    }
}

/// 基于系统时钟的超时函数，返回是否超时
/// 
/// # Arguments
/// * `timeout_us` - 超时时间，单位：微秒
/// * `condition` - 要检查的条件，返回true表示条件满足
/// 
/// # Returns
/// * `true` - 超时
/// * `false` - 未超时，条件已满足
/// 
/// # Safety
/// 直接访问硬件寄存器，需要确保在正确的上下文中调用
pub unsafe fn wait_with_timeout<F>(timeout_us: u32, condition: F) -> bool
where
    F: Fn() -> bool,
{
    // 确保SYSTICK已初始化
    init_systick();
    
    // 使用基于SysTick的延时实现超时
    let start_tick = core::ptr::read_volatile(0xE000E018 as *const u32);
    let timeout_ticks = timeout_us / 1000; // 转换为毫秒
    
    // 等待条件满足或超时
    loop {
        if condition() {
            return false; // 条件满足，未超时
        }
        
        let current_tick = core::ptr::read_volatile(0xE000E018 as *const u32);
        if current_tick.wrapping_sub(start_tick) >= timeout_ticks {
            return true; // 超时
        }
        
        core::sync::atomic::compiler_fence(Ordering::SeqCst);
    }
}
