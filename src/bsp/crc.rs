//! CRC模块
//! 提供循环冗余校验功能封装

#![allow(unused)]

// 导入内部生成的设备驱动库
use stm32f103::*;

/// CRC结构体
pub struct Crc;

impl Crc {
    /// 创建新的CRC实例
    pub const fn new() -> Self {
        Self
    }
    
    /// 获取CRC寄存器块
    unsafe fn crc() -> &'static mut stm32f103::crc::RegisterBlock {
        &mut *(0x40023000 as *mut stm32f103::crc::RegisterBlock)
    }
    
    /// 获取RCC寄存器块
    unsafe fn rcc() -> &'static mut stm32f103::rcc::RegisterBlock {
        &mut *(0x40021000 as *mut stm32f103::rcc::RegisterBlock)
    }
    
    /// 初始化CRC
    pub unsafe fn init(&self) {
        let rcc = Crc::rcc();
        
        // 启用CRC时钟
        rcc.ahbenr().modify(|_, w: &mut stm32f103::rcc::ahbenr::W| w
            .crcen().set_bit()
        );
    }
    
    /// 重置CRC计算单元
    pub unsafe fn reset(&self) {
        let crc = Crc::crc();
        crc.cr().write(|w: &mut stm32f103::crc::cr::W| w
            .reset().set_bit()
        );
    }
    
    /// 计算8位数据的CRC
    pub unsafe fn calculate8(&self, data: u8) -> u32 {
        let crc = Crc::crc();
        crc.dr().write(|w: &mut stm32f103::crc::dr::W| w
            .dr().bits(data as u32)
        );
        crc.dr().read().dr().bits()
    }
    
    /// 计算16位数据的CRC
    pub unsafe fn calculate16(&self, data: u16) -> u32 {
        let crc = Crc::crc();
        crc.dr().write(|w: &mut stm32f103::crc::dr::W| w
            .dr().bits(data as u32)
        );
        crc.dr().read().dr().bits()
    }
    
    /// 计算32位数据的CRC
    pub unsafe fn calculate32(&self, data: u32) -> u32 {
        let crc = Crc::crc();
        crc.dr().write(|w: &mut stm32f103::crc::dr::W| w
            .dr().bits(data)
        );
        crc.dr().read().dr().bits()
    }
    
    /// 计算数据块的CRC
    pub unsafe fn calculate_block(&self, data: &[u8]) -> u32 {
        // 重置CRC计算单元
        self.reset();
        
        let crc = Crc::crc();
        
        // 处理每个字节
        for &byte in data {
            crc.dr().write(|w: &mut stm32f103::crc::dr::W| w
                .dr().bits(byte as u32)
            );
        }
        
        crc.dr().read().dr().bits()
    }
    
    /// 获取当前CRC值
    pub unsafe fn get_crc(&self) -> u32 {
        let crc = Crc::crc();
        crc.dr().read().dr().bits()
    }
    
    /// 写入独立数据寄存器
    pub unsafe fn write_idr(&self, data: u8) {
        let crc = Crc::crc();
        crc.idr().write(|w: &mut stm32f103::crc::idr::W| w
            .idr().bits(data)
        );
    }
    
    /// 读取独立数据寄存器
    pub unsafe fn read_idr(&self) -> u8 {
        let crc = Crc::crc();
        crc.idr().read().idr().bits()
    }
}

/// 预定义的CRC实例
pub const CRC: Crc = Crc::new();