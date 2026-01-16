//! FLASH模块
//! 提供闪存读写和擦除功能封装

#![allow(unused)]

// 使用内部生成的设备驱动库
use library::*;

// 闪存密钥
const FLASH_KEY1: u32 = 0x45670123;
const FLASH_KEY2: u32 = 0xCDEF89AB;

/// FLASH擦除类型枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlashEraseType {
    Sector = 1,
    Mass = 2,
}

/// FLASH扇区枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlashSector {
    Sector0 = 0,    // 0x08000000 - 0x08003FFF (16KB)
    Sector1 = 1,    // 0x08004000 - 0x08007FFF (16KB)
    Sector2 = 2,    // 0x08008000 - 0x0800BFFF (16KB)
    Sector3 = 3,    // 0x0800C000 - 0x0800FFFF (16KB)
    Sector4 = 4,    // 0x08010000 - 0x0801FFFF (64KB)
    Sector5 = 5,    // 0x08020000 - 0x0803FFFF (128KB)
    Sector6 = 6,    // 0x08040000 - 0x0805FFFF (128KB)
    Sector7 = 7,    // 0x08060000 - 0x0807FFFF (128KB)
}

/// FLASH等待周期枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlashLatency {
    Latency0 = 0,    // 0等待周期
    Latency1 = 1,    // 1等待周期
    Latency2 = 2,    // 2等待周期
}

/// FLASH结构体
pub struct FlashDriver;

impl FlashDriver {
    /// 创建新的FLASH实例
    pub const fn new() -> Self {
        Self
    }
    
    /// 获取FLASH寄存器块
    unsafe fn get_flash(&self) -> &'static mut library::flash::RegisterBlock {
        &mut *(0x40022000 as *mut library::flash::RegisterBlock)
    }
    
    /// 解锁FLASH
    pub unsafe fn unlock(&self) {
        let flash = self.get_flash();
        // 写入第一个密钥
        flash.keyr().write(|w: &mut library::flash::keyr::W| unsafe { w.bits(FLASH_KEY1) });
        // 写入第二个密钥
        flash.keyr().write(|w: &mut library::flash::keyr::W| unsafe { w.bits(FLASH_KEY2) });
    }
    
    /// 锁定FLASH
    pub unsafe fn lock(&self) {
        let flash = self.get_flash();
        flash.cr().write(|w: &mut library::flash::cr::W| unsafe { w.bits(flash.cr().read().bits() | (1 << 7)) });
    }
    
    /// 解锁选项字节
    pub unsafe fn unlock_option_bytes(&self) {
        let flash = self.get_flash();
        // 写入第一个密钥
        flash.optkeyr().write(|w: &mut library::flash::optkeyr::W| unsafe { w.bits(0x08192A3B) });
        // 写入第二个密钥
        flash.optkeyr().write(|w: &mut library::flash::optkeyr::W| unsafe { w.bits(0x4C5D6E7F) });
    }
    
    /// 锁定选项字节
    pub unsafe fn lock_option_bytes(&self) {
        let flash = self.get_flash();
        flash.cr().write(|w: &mut library::flash::cr::W| unsafe { w.bits(flash.cr().read().bits() | (1 << 9)) });
    }
    
    /// 设置FLASH等待周期
    pub unsafe fn set_latency(&self, latency: FlashLatency) {
        let flash = self.get_flash();
        let mut value = flash.acr().read().bits();
        // 清除等待周期位
        value &= !(0x03 << 0);
        // 设置等待周期
        value |= (latency as u32) << 0;
        flash.acr().write(|w: &mut library::flash::acr::W| unsafe { w.bits(value) });
    }
    
    /// 启用FLASH预取缓冲区
    pub unsafe fn enable_prefetch(&self) {
        let flash = self.get_flash();
        flash.acr().write(|w: &mut library::flash::acr::W| unsafe { w.bits(flash.acr().read().bits() | (1 << 4)) });
    }
    
    /// 禁用FLASH预取缓冲区
    pub unsafe fn disable_prefetch(&self) {
        let flash = self.get_flash();
        flash.acr().write(|w: &mut library::flash::acr::W| unsafe { w.bits(flash.acr().read().bits() & !(1 << 4)) });
    }
    
    /// 擦除FLASH扇区
    pub unsafe fn erase_sector(&self, sector: FlashSector) {
        let flash = self.get_flash();
        
        // 等待忙标志清除
        while self.is_busy() {
            core::hint::spin_loop();
        }
        
        // 设置扇区擦除位
        flash.cr().write(|w: &mut library::flash::cr::W| unsafe { w.bits(flash.cr().read().bits() | (1 << 1)) });
        // 设置扇区编号
        flash.cr().write(|w: &mut library::flash::cr::W| unsafe { w.bits((flash.cr().read().bits() & !(0x0F << 3)) | ((sector as u32) & 0x0F) << 3) });
        // 开始擦除
        flash.cr().write(|w: &mut library::flash::cr::W| unsafe { w.bits(flash.cr().read().bits() | (1 << 6)) });
        
        // 等待忙标志清除
        while self.is_busy() {
            core::hint::spin_loop();
        }
        
        // 清除扇区擦除位
        flash.cr().write(|w: &mut library::flash::cr::W| unsafe { w.bits(flash.cr().read().bits() & !(1 << 1)) });
    }
    
    /// 整片擦除FLASH
    pub unsafe fn mass_erase(&self) {
        let flash = self.get_flash();
        
        // 等待忙标志清除
        while self.is_busy() {
            core::hint::spin_loop();
        }
        
        // 设置整片擦除位
        flash.cr().write(|w: &mut library::flash::cr::W| unsafe { w.bits(flash.cr().read().bits() | (1 << 2)) });
        // 开始擦除
        flash.cr().write(|w: &mut library::flash::cr::W| unsafe { w.bits(flash.cr().read().bits() | (1 << 6)) });
        
        // 等待忙标志清除
        while self.is_busy() {
            core::hint::spin_loop();
        }
        
        // 清除整片擦除位
        flash.cr().write(|w: &mut library::flash::cr::W| unsafe { w.bits(flash.cr().read().bits() & !(1 << 2)) });
    }
    
    /// 写入半字到FLASH
    pub unsafe fn write_half_word(&self, address: u32, data: u16) {
        let flash = self.get_flash();
        
        // 等待忙标志清除
        while self.is_busy() {
            core::hint::spin_loop();
        }
        
        // 设置编程位
        flash.cr().write(|w: &mut library::flash::cr::W| unsafe { w.bits(flash.cr().read().bits() | (1 << 0)) });
        
        // 写入数据
        *(address as *mut u16) = data;
        
        // 等待忙标志清除
        while self.is_busy() {
            core::hint::spin_loop();
        }
        
        // 清除编程位
        flash.cr().write(|w: &mut library::flash::cr::W| unsafe { w.bits(flash.cr().read().bits() & !(1 << 0)) });
    }
    
    /// 写入字到FLASH
    pub unsafe fn write_word(&self, address: u32, data: u32) {
        // 写入高半字
        self.write_half_word(address, (data >> 16) as u16);
        // 写入低半字
        self.write_half_word(address + 2, data as u16);
    }
    
    /// 写入数据到FLASH
    pub unsafe fn write_data(&self, address: u32, data: &[u8]) {
        let mut addr = address;
        let mut i = 0;
        
        // 对齐地址
        if (addr % 2) != 0 {
            // 写入单个字节
            let byte = data[0];
            let current_data = *(addr as *mut u8);
            let half_word = (current_data as u16) | ((byte as u16) << 8);
            self.write_half_word(addr & !1, half_word);
            addr += 1;
            i = 1;
        }
        
        // 写入半字数据
        while i < data.len() - 1 {
            let half_word = ((data[i] as u16) << 8) | (data[i + 1] as u16);
            self.write_half_word(addr, half_word);
            addr += 2;
            i += 2;
        }
        
        // 写入剩余字节
        if i < data.len() {
            let byte = data[i];
            let current_data = *((addr + 1) as *mut u8);
            let half_word = (byte as u16) | ((current_data as u16) << 8);
            self.write_half_word(addr, half_word);
        }
    }
    
    /// 读取半字从FLASH
    pub unsafe fn read_half_word(&self, address: u32) -> u16 {
        *(address as *mut u16)
    }
    
    /// 读取字从FLASH
    pub unsafe fn read_word(&self, address: u32) -> u32 {
        *(address as *mut u32)
    }
    
    /// 读取数据从FLASH
    pub unsafe fn read_data(&self, address: u32, buffer: &mut [u8]) {
        let src = address as *const u8;
        let dst = buffer.as_mut_ptr();
        
        for i in 0..buffer.len() {
            *dst.add(i) = *src.add(i);
        }
    }
    
    /// 检查FLASH是否忙
    pub unsafe fn is_busy(&self) -> bool {
        let flash = self.get_flash();
        (flash.sr().read().bits() & (1 << 0)) != 0
    }
    
    /// 检查编程错误
    pub unsafe fn has_program_error(&self) -> bool {
        let flash = self.get_flash();
        (flash.sr().read().bits() & (1 << 2)) != 0
    }
    
    /// 检查擦除错误
    pub unsafe fn has_erase_error(&self) -> bool {
        let flash = self.get_flash();
        (flash.sr().read().bits() & (1 << 3)) != 0
    }
    
    /// 清除所有错误标志
    pub unsafe fn clear_error_flags(&self) {
        let flash = self.get_flash();
        flash.sr().write(|w: &mut library::flash::sr::W| unsafe { w.bits(flash.sr().read().bits() & !(0x0E << 1)) });
    }
    
    /// 获取选项字节值
    pub unsafe fn get_option_bytes(&self) -> u32 {
        let flash = self.get_flash();
        flash.obr().read().bits()
    }
    
    /// 检查写保护状态
    pub unsafe fn is_write_protected(&self, sector: FlashSector) -> bool {
        let flash = self.get_flash();
        (flash.wrpr().read().bits() & (1 << (sector as u32))) != 0
    }
}

/// 预定义的FLASH实例
pub const FLASH: FlashDriver = FlashDriver::new();
