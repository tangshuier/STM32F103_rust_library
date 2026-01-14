//! DAC模块
//! 提供数模转换器功能封装

#![allow(unused)]

// 导入内部生成的设备驱动库
use stm32f103::*;

/// DAC通道枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DacChannel {
    Channel1 = 0,
    Channel2 = 1,
}

/// DAC触发源枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DacTriggerSource {
    Software = 0,
    Timer2TRGO = 1,
    Timer4TRGO = 2,
    Timer5TRGO = 3,
    Timer6TRGO = 4,
    Timer7TRGO = 5,
    Exti9 = 6,
}

/// DAC结构体
pub struct Dac;

impl Dac {
    /// 创建新的DAC实例
    pub const fn new() -> Self {
        Self
    }
    
    /// 获取DAC寄存器块
    unsafe fn dac() -> &'static mut stm32f103::dac::RegisterBlock {
        &mut *(0x40007400 as *mut stm32f103::dac::RegisterBlock)
    }
    
    /// 获取RCC寄存器块
    unsafe fn rcc() -> &'static mut stm32f103::rcc::RegisterBlock {
        &mut *(0x40021000 as *mut stm32f103::rcc::RegisterBlock)
    }
    
    /// 初始化DAC
    pub unsafe fn init(&self) {
        let rcc = Dac::rcc();
        
        // 启用DAC时钟
        rcc.apb1enr().modify(|_, w: &mut stm32f103::rcc::apb1enr::W| w
            .dacen().set_bit()
        );
    }
    
    /// 启用DAC通道
    pub unsafe fn enable_channel(&self, channel: DacChannel) {
        let dac = Dac::dac();
        
        match channel {
            DacChannel::Channel1 => {
                dac.cr().modify(|_, w: &mut stm32f103::dac::cr::W| w
                    .en1().set_bit()
                );
            }
            DacChannel::Channel2 => {
                dac.cr().modify(|_, w: &mut stm32f103::dac::cr::W| w
                    .en2().set_bit()
                );
            }
        }
    }
    
    /// 禁用DAC通道
    pub unsafe fn disable_channel(&self, channel: DacChannel) {
        let dac = Dac::dac();
        
        match channel {
            DacChannel::Channel1 => {
                dac.cr().modify(|_, w: &mut stm32f103::dac::cr::W| w
                    .en1().clear_bit()
                );
            }
            DacChannel::Channel2 => {
                dac.cr().modify(|_, w: &mut stm32f103::dac::cr::W| w
                    .en2().clear_bit()
                );
            }
        }
    }
    
    /// 启用DAC通道触发
    pub unsafe fn enable_trigger(&self, channel: DacChannel) {
        let dac = Dac::dac();
        
        match channel {
            DacChannel::Channel1 => {
                dac.cr().modify(|_, w: &mut stm32f103::dac::cr::W| w
                    .ten1().set_bit()
                );
            }
            DacChannel::Channel2 => {
                dac.cr().modify(|_, w: &mut stm32f103::dac::cr::W| w
                    .ten2().set_bit()
                );
            }
        }
    }
    
    /// 禁用DAC通道触发
    pub unsafe fn disable_trigger(&self, channel: DacChannel) {
        let dac = Dac::dac();
        
        match channel {
            DacChannel::Channel1 => {
                dac.cr().modify(|_, w: &mut stm32f103::dac::cr::W| w
                    .ten1().clear_bit()
                );
            }
            DacChannel::Channel2 => {
                dac.cr().modify(|_, w: &mut stm32f103::dac::cr::W| w
                    .ten2().clear_bit()
                );
            }
        }
    }
    
    /// 设置DAC通道触发源
    pub unsafe fn set_trigger_source(&self, channel: DacChannel, source: DacTriggerSource) {
        let dac = Dac::dac();
        
        match channel {
            DacChannel::Channel1 => {
                dac.cr().modify(|_, w: &mut stm32f103::dac::cr::W| w
                    .tsel1().bits(source as u8)
                );
            }
            DacChannel::Channel2 => {
                dac.cr().modify(|_, w: &mut stm32f103::dac::cr::W| w
                    .tsel2().bits(source as u8)
                );
            }
        }
    }
    
    /// 启用DAC通道输出缓冲
    pub unsafe fn enable_output_buffer(&self, channel: DacChannel) {
        let dac = Dac::dac();
        
        match channel {
            DacChannel::Channel1 => {
                dac.cr().modify(|_, w: &mut stm32f103::dac::cr::W| w
                    .boff1().clear_bit()
                );
            }
            DacChannel::Channel2 => {
                dac.cr().modify(|_, w: &mut stm32f103::dac::cr::W| w
                    .boff2().clear_bit()
                );
            }
        }
    }
    
    /// 禁用DAC通道输出缓冲
    pub unsafe fn disable_output_buffer(&self, channel: DacChannel) {
        let dac = Dac::dac();
        
        match channel {
            DacChannel::Channel1 => {
                dac.cr().modify(|_, w: &mut stm32f103::dac::cr::W| w
                    .boff1().set_bit()
                );
            }
            DacChannel::Channel2 => {
                dac.cr().modify(|_, w: &mut stm32f103::dac::cr::W| w
                    .boff2().set_bit()
                );
            }
        }
    }
    
    /// 软件触发DAC转换
    pub unsafe fn software_trigger(&self, channel: DacChannel) {
        let dac = Dac::dac();
        
        match channel {
            DacChannel::Channel1 => {
                dac.swtrigr().write(|w: &mut stm32f103::dac::swtrigr::W| w
                    .swtrig1().set_bit()
                );
            }
            DacChannel::Channel2 => {
                dac.swtrigr().write(|w: &mut stm32f103::dac::swtrigr::W| w
                    .swtrig2().set_bit()
                );
            }
        }
    }
    
    /// 设置DAC通道12位右对齐数据
    pub unsafe fn set_channel_data(&self, channel: DacChannel, value: u16) {
        let dac = Dac::dac();
        let value_clamped = if value > 4095 { 4095 } else { value };
        
        match channel {
            DacChannel::Channel1 => {
                dac.dhr12r1().write(|w: &mut stm32f103::dac::dhr12r1::W| w
                    .dacc1dhr().bits(value_clamped)
                );
            }
            DacChannel::Channel2 => {
                dac.dhr12r2().write(|w: &mut stm32f103::dac::dhr12r2::W| w
                    .dacc2dhr().bits(value_clamped)
                );
            }
        }
    }
    
    /// 设置DAC通道12位左对齐数据
    pub unsafe fn set_channel_data_left_aligned(&self, channel: DacChannel, value: u16) {
        let dac = Dac::dac();
        let value_clamped = if value > 4095 { 4095 } else { value };
        
        match channel {
            DacChannel::Channel1 => {
                dac.dhr12l1().write(|w: &mut stm32f103::dac::dhr12l1::W| w
                    .dacc1dhr().bits(value_clamped << 4)
                );
            }
            DacChannel::Channel2 => {
                dac.dhr12l2().write(|w: &mut stm32f103::dac::dhr12l2::W| w
                    .dacc2dhr().bits(value_clamped << 4)
                );
            }
        }
    }
    
    /// 设置DAC通道8位右对齐数据
    pub unsafe fn set_channel_data_8bit(&self, channel: DacChannel, value: u8) {
        let dac = Dac::dac();
        
        match channel {
            DacChannel::Channel1 => {
                dac.dhr8r1().write(|w: &mut stm32f103::dac::dhr8r1::W| w
                    .dacc1dhr().bits(value as u8)
                );
            }
            DacChannel::Channel2 => {
                dac.dhr8r2().write(|w: &mut stm32f103::dac::dhr8r2::W| w
                    .dacc2dhr().bits(value as u8)
                );
            }
        }
    }
    
    /// 设置双通道12位右对齐数据
    pub unsafe fn set_dual_channel_data(&self, value1: u16, value2: u16) {
        let dac = Dac::dac();
        let value1_clamped = if value1 > 4095 { 4095 } else { value1 };
        let value2_clamped = if value2 > 4095 { 4095 } else { value2 };
        
        dac.dhr12rd().write(|w: &mut stm32f103::dac::dhr12rd::W| w
            .dacc1dhr().bits(value1_clamped)
            .dacc2dhr().bits(value2_clamped)
        );
    }
    
    /// 获取DAC通道数据输出
    pub unsafe fn get_channel_output(&self, channel: DacChannel) -> u16 {
        let dac = Dac::dac();
        
        match channel {
            DacChannel::Channel1 => {
                dac.dor1().read().dacc1dor().bits()
            }
            DacChannel::Channel2 => {
                dac.dor2().read().dacc2dor().bits()
            }
        }
    }
}

/// 预定义的DAC实例
pub const DAC: Dac = Dac::new();