#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

// 导入BSP模块
pub mod bsp;

// 使用BSP模块
use crate::bsp::gpio;

#[entry]
fn main() -> ! {
    unsafe {
        // 获取LED引脚（GPIOA.0）
        let led = gpio::PA0;
        
        // 配置GPIOA.0为推挽输出
        led.into_push_pull_output();
        
        // 配置USART3的GPIO引脚（PB10=TX, PB11=RX）
        let pb10 = gpio::PB10;
        let pb11 = gpio::PB11;
        pb10.into_alternate_push_pull();
        pb11.into_floating_input();
        
        // 主循环，闪烁LED
        loop {
            // 点亮LED（低电平点亮）
            led.set_low();
            
            // 简单延时（使用忙等待）
            for _ in 0..1000000 {
                core::hint::spin_loop();
            }
            
            // 熄灭LED（高电平熄灭）
            led.set_high();
            
            // 简单延时（使用忙等待）
            for _ in 0..1000000 {
                core::hint::spin_loop();
            }
        }
    }
}
