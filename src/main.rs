#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use core::sync::atomic::{AtomicBool, Ordering};

// 外部中断处理函数声明
extern "C" {
    fn Reset();
}

// 默认中断处理函数
#[no_mangle]
pub unsafe extern "C" fn DefaultHandler() {
    // 什么都不做，只是一个占位符
}

// 全局中断触发标志
static INTERRUPT_TRIGGERED: AtomicBool = AtomicBool::new(false);

// 导入BSP模块
pub mod bsp;

// 使用BSP模块
use crate::bsp::gpio;
use crate::bsp::system;
use crate::bsp::delay;
use crate::bsp::serial;

#[entry]
fn main() -> ! {
    unsafe {
        // 1. 初始化系统时钟到72MHz
        system::init();
        
        // 2. 获取LED引脚（GPIOA.0）
        let led = gpio::PA0;
        
        // 3. 配置GPIOA.0为推挽输出
        led.into_push_pull_output();
        
        // 4. 配置USART3的GPIO引脚（PB10=TX, PB11=RX）
        let pb10 = gpio::PB10;
        let pb11 = gpio::PB11;
        pb10.into_alternate_push_pull();
        pb11.into_floating_input();
        
        // 5. 初始化USART3串口（波特率115200）
            let usart3 = serial::USART3;
            usart3.init(serial::BaudRate::B115200);
            
            // 6. 启用接收中断
            // 启用USART3的接收中断
            let usart3_cr1 = 0x4000480C as *mut u32;
            *usart3_cr1 |= 1 << 5;
            
            // 启用NVIC中的USART3中断（中断编号39，对应ISER1的第7位）
            let nvic_iser1 = 0xE000E104 as *mut u32;
            *nvic_iser1 |= 1 << 7;
    }

    // 主循环，闪烁LED并通过串口发送信息，同时处理接收数据
    loop {
        unsafe {
            let led = gpio::PA0;
            let usart3 = serial::USART3;
            
            // 点亮LED（低电平点亮）
            led.set_low();
            
            // 发送信息到串口
            usart3.write_str("LED ON\r\n");
            
            // 检查并处理接收的数据（轮询方式）
            while usart3.is_data_available() {
                let byte = usart3.read_byte();
                usart3.write_byte(byte);
            }
            
            // 发送中断状态
            usart3.write_str("Interrupt enabled\r\n");
            
            // 使用SysTick实现精确的1秒延时
            delay::delay_ms(1000);
            
            // 熄灭LED（高电平熄灭）
            led.set_high();
            
            // 发送信息到串口
            usart3.write_str("LED OFF\r\n");
            
            // 检查并处理接收的数据（轮询方式）
            while usart3.is_data_available() {
                let byte = usart3.read_byte();
                usart3.write_byte(byte);
            }
            
            // 使用SysTick实现精确的1秒延时
            delay::delay_ms(1000);
        }
    }
}

/// USART3 中断处理函数
/// 
/// 处理串口3的接收中断
#[no_mangle]
pub unsafe extern "C" fn USART3() {
    // 1. 读取状态寄存器，检查中断源
    let usart3_sr = 0x40004800 as *const u32;
    let sr_value = *usart3_sr;
    
    // 2. 处理接收中断
    if (sr_value & (1 << 5)) != 0 {
        // 3. 读取数据寄存器，清除中断标志
        let usart3_dr = 0x40004804 as *mut u32;
        let _ = *usart3_dr;
    }
}



