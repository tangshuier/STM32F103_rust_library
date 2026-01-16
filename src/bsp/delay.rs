//! 延时和计时功能模块
//! 提供基于SysTick的高精度延时和计时API

// 屏蔽未使用代码警告
#![allow(unused)]

use core::sync::atomic::{AtomicU32, Ordering};
use core::arch::asm;
use core::time::Duration;

/// 系统运行时间计数器（毫秒）
static SYSTEM_TICK: AtomicU32 = AtomicU32::new(0);

/// SysTick重装载值
static mut SYSTICK_RELOAD: u32 = 0;

/// 系统时钟频率（Hz）
static mut SYSTEM_CLOCK: u32 = 72_000_000;

/// 初始化系统滴答定时器
/// 
/// 配置SysTick为1kHz，根据实际系统时钟频率计算重装载值
/// 
/// # Arguments
/// * `sysclk` - 系统时钟频率（Hz），如果为0则自动检测
/// 
/// # Safety
/// 直接访问硬件寄存器，需要确保在正确的上下文中调用
pub unsafe fn init_systick(sysclk: u32) {
    // 检查SYSTICK是否已初始化
    let csr = core::ptr::read_volatile(0xE000E010 as *const u32);
    
    // 确定系统时钟频率
    let actual_sysclk = if sysclk > 0 {
        sysclk
    } else {
        // 读取系统时钟配置，确定当前系统时钟频率
        let rcc_cfgr = core::ptr::read_volatile(0x40021004 as *const u32);
        let sysclk_source = rcc_cfgr & 0x0C;
        
        if sysclk_source == 0x08 {
            // PLL作为系统时钟源（72MHz）
            72_000_000
        } else {
            // HSI作为系统时钟源（8MHz）
            8_000_000
        }
    };
    
    SYSTEM_CLOCK = actual_sysclk;
    
    // 计算重装载值（1kHz）
    let reload_value = (actual_sysclk / 1000) - 1;
    SYSTICK_RELOAD = reload_value;
    
    if (csr & 0x01) == 0 {
        // 配置SYSTICK为1kHz
        core::ptr::write_volatile(0xE000E014 as *mut u32, reload_value);
        // 清空当前值
        core::ptr::write_volatile(0xE000E018 as *mut u32, 0);
        // 启用SYSTICK，使用处理器时钟，不启用中断
        core::ptr::write_volatile(0xE000E010 as *mut u32, 0x05); // 0x05 = ENABLE + CLKSOURCE，不设置TICKINT位
    }
}

/// SysTick中断处理函数
/// 
/// 用于递增系统运行时间计数器
/// 
/// # Safety
/// 直接访问硬件寄存器，需要确保在正确的上下文中调用
#[export_name = "SysTick_Handler"]
pub unsafe extern "C" fn systick_handler() {
    // 递增系统运行时间计数器
    SYSTEM_TICK.fetch_add(1, Ordering::SeqCst);
}

/// 获取系统运行时间（毫秒）
/// 
/// # Returns
/// 系统运行时间，单位：毫秒
pub fn get_uptime_ms() -> u32 {
    SYSTEM_TICK.load(Ordering::SeqCst)
}

/// 获取系统运行时间（微秒）
/// 
/// # Returns
/// 系统运行时间，单位：微秒
pub fn get_uptime_us() -> u64 {
    let ms = get_uptime_ms() as u64;
    let ticks = unsafe {
        let current_value = core::ptr::read_volatile(0xE000E018 as *const u32);
        let reload_value = SYSTICK_RELOAD;
        (reload_value - current_value) as u64
    };
    
    // 计算微秒数
    ms * 1000 + (ticks * 1000) / (unsafe { SYSTICK_RELOAD + 1 }) as u64
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
    if SYSTICK_RELOAD == 0 {
        init_systick(0);
    }
    
    if us == 0 {
        return;
    }
    
    // 对于大于1ms的延时，使用SysTick的COUNTFLAG标志
    if us >= 1000 {
        let ms = us / 1000;
        delay_ms(ms);
        
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
/// 根据当前系统时钟频率计算循环次数
/// 
/// # Arguments
/// * `us` - 延时时间，单位：微秒
/// 
/// # Safety
/// 使用内联汇编，需要确保在正确的上下文中调用
#[inline(always)]
unsafe fn delay_us_precise(us: u32) {
    // 根据系统时钟频率计算循环次数
    let cycles_per_us = SYSTEM_CLOCK / 1_000_000;
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
    if SYSTICK_RELOAD == 0 {
        init_systick(0);
    }
    
    // 使用SysTick的COUNTFLAG标志实现延时
    for _ in 0..ms {
        // 等待SysTick计数完成
        while (core::ptr::read_volatile(0xE000E010 as *const u32) & (1 << 16)) == 0 {
            core::sync::atomic::compiler_fence(Ordering::SeqCst);
        }
        // COUNTFLAG被读取后会自动清零，无需额外操作
    }
}

/// 基于系统时钟的延时函数（使用Duration）
/// 
/// # Arguments
/// * `duration` - 延时时间
/// 
/// # Safety
/// 直接访问硬件寄存器，需要确保在正确的上下文中调用
pub unsafe fn delay(duration: Duration) {
    let ms = duration.as_millis() as u32;
    let us = (duration.as_micros() % 1000) as u32;
    
    if ms > 0 {
        delay_ms(ms);
    }
    if us > 0 {
        delay_us(us);
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
    if SYSTICK_RELOAD == 0 {
        init_systick(0);
    }
    
    // 记录开始时间
    let start_time = get_uptime_us();
    
    // 等待条件满足或超时
    loop {
        if condition() {
            return false; // 条件满足，未超时
        }
        
        // 检查是否超时
        let current_time = get_uptime_us();
        if current_time.wrapping_sub(start_time) >= timeout_us as u64 {
            return true; // 超时
        }
        
        core::sync::atomic::compiler_fence(Ordering::SeqCst);
    }
}

/// 时间戳结构体
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp {
    /// 时间戳值（毫秒）
    value: u32,
}

impl Timestamp {
    /// 创建一个新的时间戳
    /// 
    /// # Returns
    /// 当前时间的时间戳
    pub fn now() -> Self {
        Self {
            value: get_uptime_ms(),
        }
    }
    
    /// 获取时间戳的值（毫秒）
    /// 
    /// # Returns
    /// 时间戳值，单位：毫秒
    pub fn as_millis(&self) -> u32 {
        self.value
    }
    
    /// 计算两个时间戳之间的时间差
    /// 
    /// # Arguments
    /// * `other` - 另一个时间戳
    /// 
    /// # Returns
    /// 时间差，单位：毫秒
    pub fn duration_since(&self, other: &Self) -> u32 {
        self.value.wrapping_sub(other.value)
    }
    
    /// 计算从现在开始的时间差
    /// 
    /// # Returns
    /// 从现在开始的时间差，单位：毫秒
    pub fn elapsed(&self) -> u32 {
        let now = Self::now();
        now.duration_since(self)
    }
}

/// 周期性定时器
pub struct PeriodicTimer {
    /// 周期（毫秒）
    period: u32,
    /// 上次触发时间
    last_trigger: Timestamp,
}

impl PeriodicTimer {
    /// 创建一个新的周期性定时器
    /// 
    /// # Arguments
    /// * `period` - 周期，单位：毫秒
    /// 
    /// # Returns
    /// 周期性定时器实例
    pub fn new(period: u32) -> Self {
        Self {
            period,
            last_trigger: Timestamp::now(),
        }
    }
    
    /// 检查定时器是否应该触发
    /// 
    /// # Returns
    /// * `true` - 定时器应该触发
    /// * `false` - 定时器不应该触发
    pub fn should_trigger(&mut self) -> bool {
        let now = Timestamp::now();
        let elapsed = now.duration_since(&self.last_trigger);
        
        if elapsed >= self.period {
            self.last_trigger = now;
            true
        } else {
            false
        }
    }
    
    /// 重置定时器
    pub fn reset(&mut self) {
        self.last_trigger = Timestamp::now();
    }
    
    /// 设置定时器周期
    /// 
    /// # Arguments
    /// * `period` - 新的周期，单位：毫秒
    pub fn set_period(&mut self, period: u32) {
        self.period = period;
        self.reset();
    }
}

/// 测试模块
#[cfg(test)]
mod tests {
    use super::*;
    use core::time::Duration;
    
    /// 测试系统运行时间计数器
    #[test]
    fn test_uptime() {
        // 初始化SysTick
        unsafe {
            init_systick(72_000_000);
        }
        
        // 记录开始时间
        let start_ms = get_uptime_ms();
        let start_us = get_uptime_us();
        
        // 延时一段时间
        unsafe {
            delay_ms(10);
        }
        
        // 检查运行时间是否正确
        let end_ms = get_uptime_ms();
        let end_us = get_uptime_us();
        
        assert!(end_ms >= start_ms + 10, "运行时间计算错误");
        assert!(end_us >= start_us + 10000, "微秒级运行时间计算错误");
    }
    
    /// 测试延时函数
    #[test]
    fn test_delay() {
        // 初始化SysTick
        unsafe {
            init_systick(72_000_000);
        }
        
        // 测试毫秒延时
        let start_ms = get_uptime_ms();
        unsafe {
            delay_ms(50);
        }
        let end_ms = get_uptime_ms();
        let elapsed_ms = end_ms.wrapping_sub(start_ms);
        assert!(elapsed_ms >= 50, "毫秒延时错误: {}ms", elapsed_ms);
        
        // 测试微秒延时
        let start_us = get_uptime_us();
        unsafe {
            delay_us(1000);
        }
        let end_us = get_uptime_us();
        let elapsed_us = end_us.wrapping_sub(start_us as u64);
        assert!(elapsed_us >= 1000, "微秒延时错误: {}us", elapsed_us);
        
        // 测试Duration延时
        let start_ms = get_uptime_ms();
        unsafe {
            delay(Duration::from_millis(20));
        }
        let end_ms = get_uptime_ms();
        let elapsed_ms = end_ms.wrapping_sub(start_ms);
        assert!(elapsed_ms >= 20, "Duration延时错误: {}ms", elapsed_ms);
    }
    
    /// 测试时间戳功能
    #[test]
    fn test_timestamp() {
        // 初始化SysTick
        unsafe {
            init_systick(72_000_000);
        }
        
        // 创建时间戳
        let ts1 = Timestamp::now();
        
        // 延时一段时间
        unsafe {
            delay_ms(10);
        }
        
        // 创建另一个时间戳
        let ts2 = Timestamp::now();
        
        // 测试时间差计算
        let diff = ts2.duration_since(&ts1);
        assert!(diff >= 10, "时间差计算错误: {}ms", diff);
        
        // 测试elapsed方法
        let elapsed = ts1.elapsed();
        assert!(elapsed >= 10, "elapsed方法错误: {}ms", elapsed);
    }
    
    /// 测试周期性定时器
    #[test]
    fn test_periodic_timer() {
        // 初始化SysTick
        unsafe {
            init_systick(72_000_000);
        }
        
        // 创建一个10ms周期的定时器
        let mut timer = PeriodicTimer::new(10);
        
        // 检查定时器是否不应该立即触发
        assert!(!timer.should_trigger(), "定时器不应该立即触发");
        
        // 延时一段时间
        unsafe {
            delay_ms(15);
        }
        
        // 检查定时器是否应该触发
        assert!(timer.should_trigger(), "定时器应该触发");
        
        // 检查定时器是否不应该再次触发
        assert!(!timer.should_trigger(), "定时器不应该再次触发");
        
        // 测试重置功能
        timer.reset();
        assert!(!timer.should_trigger(), "重置后定时器不应该立即触发");
        
        // 测试设置周期功能
        timer.set_period(20);
        unsafe {
            delay_ms(15);
        }
        assert!(!timer.should_trigger(), "新周期未到，定时器不应该触发");
        
        unsafe {
            delay_ms(10);
        }
        assert!(timer.should_trigger(), "新周期已到，定时器应该触发");
    }
    
    /// 测试超时函数
    #[test]
    fn test_wait_with_timeout() {
        // 初始化SysTick
        unsafe {
            init_systick(72_000_000);
        }
        
        // 测试条件满足的情况
        let result = unsafe {
            wait_with_timeout(10000, || true)
        };
        assert!(!result, "条件满足时不应该超时");
        
        // 测试超时的情况
        let result = unsafe {
            wait_with_timeout(1000, || false)
        };
        assert!(result, "条件不满足时应该超时");
    }
}
