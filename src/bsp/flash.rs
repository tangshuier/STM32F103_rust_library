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

/// FLASH状态枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlashStatus {
    Busy = 1,           // 忙
    ErrorPg = 2,        // 编程错误
    ErrorWrp = 3,       // 写保护错误
    Complete = 4,       // 操作完成
    Timeout = 5,        // 超时
}

/// FLASH结构体
pub struct FlashDriver;

impl FlashDriver {
    /// 创建新的FLASH实例
    pub const fn new() -> Self {
        Self
    }
    
    /// 获取FLASH实例
    fn flash(&self) -> &'static library::Flash {
        // 这个方法只在内部使用，外部无法直接访问，因此可以安全地使用unsafe
        unsafe {
            &*(library::Flash::PTR as *const library::Flash)
        }
    }
    
    /// 解锁FLASH
    pub unsafe fn unlock(&self) {
        let flash = self.flash();
        // 写入第一个密钥
        flash.keyr().write(|w| w.bits(FLASH_KEY1));
        // 写入第二个密钥
        flash.keyr().write(|w| w.bits(FLASH_KEY2));
    }
    
    /// 锁定FLASH
    pub unsafe fn lock(&self) {
        let flash = self.flash();
        flash.cr().modify(|_, w| w.lock().set_bit());
    }
    
    /// 解锁选项字节
    pub unsafe fn unlock_option_bytes(&self) {
        let flash = self.flash();
        // 写入第一个密钥
        flash.optkeyr().write(|w| w.bits(0x08192A3B));
        // 写入第二个密钥
        flash.optkeyr().write(|w| w.bits(0x4C5D6E7F));
    }
    
    /// 锁定选项字节
    pub unsafe fn lock_option_bytes(&self) {
        let flash = self.flash();
        flash.cr().modify(|_, w| w.lock().set_bit());
    }
    
    /// 设置FLASH等待周期
    pub unsafe fn set_latency(&self, latency: FlashLatency) {
        let flash = self.flash();
        flash.acr().modify(|_, w| {
            match latency {
                FlashLatency::Latency0 => w.latency().bits(0),
                FlashLatency::Latency1 => w.latency().bits(1),
                FlashLatency::Latency2 => w.latency().bits(2),
            }
        });
    }
    
    /// 启用FLASH预取缓冲区
    pub unsafe fn enable_prefetch(&self) {
        let flash = self.flash();
        flash.acr().modify(|_, w| w.prftbe().set_bit());
    }
    
    /// 禁用FLASH预取缓冲区
    pub unsafe fn disable_prefetch(&self) {
        let flash = self.flash();
        flash.acr().modify(|_, w| w.prftbe().clear_bit());
    }
    
    /// 启用/禁用FLASH半周期访问
    pub unsafe fn half_cycle_access_cmd(&self, enable: bool) {
        let flash = self.flash();
        if enable {
            flash.acr().modify(|_, w| w.hlfcya().set_bit());
        } else {
            flash.acr().modify(|_, w| w.hlfcya().clear_bit());
        }
    }
    
    /// 擦除选项字节
    pub unsafe fn erase_option_bytes(&self) {
        let flash = self.flash();
        
        // 等待忙标志清除
        while self.is_busy() {
            core::hint::spin_loop();
        }
        
        // 解锁选项字节
        self.unlock_option_bytes();
        
        // 设置选项字节擦除位
        flash.cr().modify(|_, w| w.opter().set_bit());
        // 设置开始位
        flash.cr().modify(|_, w| w.strt().set_bit());
        
        // 等待忙标志清除
        while self.is_busy() {
            core::hint::spin_loop();
        }
        
        // 清除选项字节擦除位
        flash.cr().modify(|_, w| w.opter().clear_bit());
    }
    
    /// 擦除FLASH扇区
    pub unsafe fn erase_sector(&self, sector: FlashSector) {
        let flash = self.flash();
        
        // 等待忙标志清除
        while self.is_busy() {
            core::hint::spin_loop();
        }
        
        // 计算扇区地址
        let sector_address = match sector {
            FlashSector::Sector0 => 0x08000000,
            FlashSector::Sector1 => 0x08004000,
            FlashSector::Sector2 => 0x08008000,
            FlashSector::Sector3 => 0x0800C000,
            FlashSector::Sector4 => 0x08010000,
            FlashSector::Sector5 => 0x08020000,
            FlashSector::Sector6 => 0x08040000,
            FlashSector::Sector7 => 0x08060000,
        };
        
        // 设置扇区擦除位
        flash.cr().modify(|_, w| w.per().set_bit());
        
        // 设置扇区地址
        flash.ar().write(|w| w.bits(sector_address));
        
        // 开始擦除
        flash.cr().modify(|_, w| w.strt().set_bit());
        
        // 等待忙标志清除
        while self.is_busy() {
            core::hint::spin_loop();
        }
        
        // 清除扇区擦除位
        flash.cr().modify(|_, w| w.per().clear_bit());
    }
    
    /// 整片擦除FLASH
    pub unsafe fn mass_erase(&self) {
        let flash = self.flash();
        
        // 等待忙标志清除
        while self.is_busy() {
            core::hint::spin_loop();
        }
        
        // 设置整片擦除位
        flash.cr().modify(|_, w| {
            w.mer().set_bit()
        });
        
        // 开始擦除
        flash.cr().modify(|_, w| {
            w.strt().set_bit()
        });
        
        // 等待忙标志清除
        while self.is_busy() {
            core::hint::spin_loop();
        }
        
        // 清除整片擦除位
        flash.cr().modify(|_, w| {
            w.mer().clear_bit()
        });
    }
    
    /// 擦除指定地址的页面
    pub unsafe fn erase_page(&self, page_address: u32) -> FlashStatus {
        let flash = self.flash();
        
        // 等待忙标志清除
        let status = self.wait_for_last_operation(0xFFFF);
        if status != FlashStatus::Complete {
            return status;
        }
        
        // 检查地址是否有效
        if page_address < 0x08000000 || page_address > 0x080FFFFF {
            return FlashStatus::ErrorPg;
        }
        
        // 设置页面擦除位
        flash.cr().modify(|_, w| w.per().set_bit());
        
        // 设置页面地址
        flash.ar().write(|w| w.bits(page_address));
        
        // 开始擦除
        flash.cr().modify(|_, w| w.strt().set_bit());
        
        // 等待操作完成
        let status = self.wait_for_last_operation(0xFFFF);
        
        // 清除页面擦除位
        flash.cr().modify(|_, w| w.per().clear_bit());
        
        status
    }
    
    /// 擦除所有页面（与mass_erase功能相同，为兼容标准库API而添加）
    pub unsafe fn erase_all_pages(&self) -> FlashStatus {
        let flash = self.flash();
        
        // 等待忙标志清除
        let status = self.wait_for_last_operation(0xFFFF);
        if status != FlashStatus::Complete {
            return status;
        }
        
        // 设置整片擦除位
        flash.cr().modify(|_, w| w.mer().set_bit());
        
        // 开始擦除
        flash.cr().modify(|_, w| w.strt().set_bit());
        
        // 等待操作完成
        let status = self.wait_for_last_operation(0xFFFF);
        
        // 清除整片擦除位
        flash.cr().modify(|_, w| w.mer().clear_bit());
        
        status
    }
    
    /// 写入半字到FLASH
    pub unsafe fn write_half_word(&self, address: u32, data: u16) -> FlashStatus {
        // 检查地址是否有效
        if address < 0x08000000 || address > 0x080FFFFF {
            return FlashStatus::ErrorPg;
        }
        
        let flash = self.flash();
        
        // 等待忙标志清除
        let status = self.wait_for_last_operation(0xFFFF);
        if status != FlashStatus::Complete {
            return status;
        }
        
        // 设置编程位
        flash.cr().modify(|_, w| {
            w.pg().set_bit()
        });
        
        // 写入数据
        *(address as *mut u16) = data;
        
        // 等待忙标志清除
        self.wait_for_last_operation(0xFFFF)
    }
    
    /// 写入字到FLASH
    pub unsafe fn write_word(&self, address: u32, data: u32) -> FlashStatus {
        // 检查地址是否有效
        if address < 0x08000000 || address > 0x080FFFFC {
            return FlashStatus::ErrorPg;
        }
        
        // 写入高半字
        let status = self.write_half_word(address, (data >> 16) as u16);
        if status != FlashStatus::Complete {
            return status;
        }
        
        // 写入低半字
        self.write_half_word(address + 2, data as u16)
    }
    
    /// 写入数据到FLASH
    pub unsafe fn write_data(&self, address: u32, data: &[u8]) -> FlashStatus {
        // 检查地址是否有效
        if address < 0x08000000 || address > 0x080FFFFF {
            return FlashStatus::ErrorPg;
        }
        
        let mut addr = address;
        let mut i = 0;
        
        // 对齐地址
        if (addr % 2) != 0 {
            // 写入单个字节
            if i >= data.len() {
                return FlashStatus::Complete;
            }
            
            let byte = data[0];
            let current_data = *(addr as *mut u8);
            let half_word = (current_data as u16) | ((byte as u16) << 8);
            
            let status = self.write_half_word(addr & !1, half_word);
            if status != FlashStatus::Complete {
                return status;
            }
            
            addr += 1;
            i = 1;
        }
        
        // 写入半字数据
        while i < data.len() - 1 {
            let half_word = ((data[i] as u16) << 8) | (data[i + 1] as u16);
            
            let status = self.write_half_word(addr, half_word);
            if status != FlashStatus::Complete {
                return status;
            }
            
            addr += 2;
            i += 2;
        }
        
        // 写入剩余字节
        if i < data.len() {
            let byte = data[i];
            let current_data = *((addr + 1) as *mut u8);
            let half_word = (byte as u16) | ((current_data as u16) << 8);
            
            let status = self.write_half_word(addr, half_word);
            if status != FlashStatus::Complete {
                return status;
            }
        }
        
        FlashStatus::Complete
    }
    
    /// 读取半字从FLASH
    pub unsafe fn read_half_word(&self, address: u32) -> u16 {
        // 检查地址是否有效
        assert!(address >= 0x08000000 && address <= 0x080FFFFE, "Invalid FLASH address");
        *(address as *mut u16)
    }
    
    /// 读取字从FLASH
    pub unsafe fn read_word(&self, address: u32) -> u32 {
        // 检查地址是否有效
        assert!(address >= 0x08000000 && address <= 0x080FFFFC, "Invalid FLASH address");
        *(address as *mut u32)
    }
    
    /// 读取数据从FLASH
    pub unsafe fn read_data(&self, address: u32, buffer: &mut [u8]) {
        // 检查地址范围是否有效
        assert!(address >= 0x08000000 && address + buffer.len() as u32 <= 0x08100000, "Invalid FLASH address range");
        
        let src = address as *const u8;
        let dst = buffer.as_mut_ptr();
        
        for i in 0..buffer.len() {
            *dst.add(i) = *src.add(i);
        }
    }
    
    /// 检查FLASH是否忙
    pub unsafe fn is_busy(&self) -> bool {
        let flash = self.flash();
        flash.sr().read().bsy().bit()
    }
    
    /// 检查编程错误
    pub unsafe fn has_program_error(&self) -> bool {
        let flash = self.flash();
        flash.sr().read().pgerr().bit()
    }
    
    /// 检查擦除错误
    pub unsafe fn has_erase_error(&self) -> bool {
        let flash = self.flash();
        flash.sr().read().wrprterr().bit()
    }
    
    /// 清除所有错误标志
    pub unsafe fn clear_error_flags(&self) {
        let flash = self.flash();
        flash.sr().write(|w| w
            .eop().clear_bit()
            .pgerr().clear_bit()
            .wrprterr().clear_bit()
        );
    }
    
    /// 获取选项字节值
    pub unsafe fn get_option_bytes(&self) -> u32 {
        let flash = self.flash();
        flash.obr().read().bits()
    }
    
    /// 检查写保护状态
    pub unsafe fn is_write_protected(&self, sector: FlashSector) -> bool {
        let flash = self.flash();
        (flash.wrpr().read().bits() & (1 << (sector as u32))) != 0
    }
    
    /// 编程选项字节数据
    pub unsafe fn program_option_byte_data(&self, address: u32, data: u8) {
        let flash = self.flash();
        
        // 等待忙标志清除
        while self.is_busy() {
            core::hint::spin_loop();
        }
        
        // 解锁选项字节
        self.unlock_option_bytes();
        
        // 设置选项字节编程位
        flash.cr().modify(|_, w| w.optpg().set_bit());
        
        // 写入数据
        *(address as *mut u8) = data;
        
        // 等待忙标志清除
        while self.is_busy() {
            core::hint::spin_loop();
        }
        
        // 清除选项字节编程位
        flash.cr().modify(|_, w| w.optpg().clear_bit());
    }
    
    /// 启用写保护
    pub unsafe fn enable_write_protection(&self, pages: u32) -> FlashStatus {
        let flash = self.flash();
        
        // 等待忙标志清除
        let status = self.wait_for_last_operation(0xFFFF);
        if status != FlashStatus::Complete {
            return status;
        }
        
        // 解锁选项字节
        self.unlock_option_bytes();
        
        // 设置选项字节编程位
        flash.cr().modify(|_, w| w.optpg().set_bit());
        
        // 写保护通过选项字节编程实现，需要写入特定地址
        // 这里实现的是简化版本，实际应用中需要根据具体硬件调整
        self.program_option_byte_data(0x1FFFF808, (pages & 0xFF) as u8);
        self.program_option_byte_data(0x1FFFF809, ((pages >> 8) & 0xFF) as u8);
        self.program_option_byte_data(0x1FFFF80A, ((pages >> 16) & 0xFF) as u8);
        self.program_option_byte_data(0x1FFFF80B, ((pages >> 24) & 0xFF) as u8);
        
        // 等待忙标志清除
        let status = self.wait_for_last_operation(0xFFFF);
        if status != FlashStatus::Complete {
            return status;
        }
        
        // 清除选项字节编程位
        flash.cr().modify(|_, w| w.optpg().clear_bit());
        
        FlashStatus::Complete
    }
    
    /// 获取预取缓冲区状态
    pub unsafe fn get_prefetch_buffer_status(&self) -> bool {
        let flash = self.flash();
        flash.acr().read().prftbs().bit()
    }
    
    /// 获取状态
    pub unsafe fn get_status(&self) -> FlashStatus {
        let flash = self.flash();
        let sr = flash.sr().read();
        
        if sr.bsy().bit() {
            FlashStatus::Busy
        } else if sr.pgerr().bit() {
            FlashStatus::ErrorPg
        } else if sr.wrprterr().bit() {
            FlashStatus::ErrorWrp
        } else if sr.eop().bit() {
            FlashStatus::Complete
        } else {
            FlashStatus::Complete
        }
    }
    
    /// 等待最后一次操作完成
    pub unsafe fn wait_for_last_operation(&self, timeout: u32) -> FlashStatus {
        let mut counter = timeout;
        
        while counter > 0 {
            let status = self.get_status();
            if status != FlashStatus::Busy {
                return status;
            }
            counter -= 1;
        }
        
        FlashStatus::Timeout
    }
    
    /// 读保护控制
    pub unsafe fn read_out_protection(&self, enable: bool) -> FlashStatus {
        let flash = self.flash();
        
        // 等待忙标志清除
        let status = self.wait_for_last_operation(0xFFFF);
        if status != FlashStatus::Complete {
            return status;
        }
        
        // 解锁选项字节
        self.unlock_option_bytes();
        
        if enable {
            // 启用读保护
            self.program_option_byte_data(0x1FFFF800, 0xAA);
            self.program_option_byte_data(0x1FFFF801, 0x55);
            self.program_option_byte_data(0x1FFFF802, 0xA5);
        } else {
            // 禁用读保护
            self.program_option_byte_data(0x1FFFF800, 0x99);
            self.program_option_byte_data(0x1FFFF801, 0x66);
            self.program_option_byte_data(0x1FFFF802, 0x96);
        }
        
        FlashStatus::Complete
    }
    
    /// 用户选项字节配置
    pub unsafe fn user_option_byte_config(&self, ob_iwdg: u16, ob_stop: u16, ob_stdby: u16) -> FlashStatus {
        let flash = self.flash();
        
        // 等待忙标志清除
        let status = self.wait_for_last_operation(0xFFFF);
        if status != FlashStatus::Complete {
            return status;
        }
        
        // 解锁选项字节
        self.unlock_option_bytes();
        
        // 擦除选项字节
        self.erase_option_bytes();
        
        // 编程用户选项字节
        let optbyte = ob_iwdg | ob_stop | ob_stdby;
        self.program_option_byte_data(0x1FFFF804, (optbyte & 0xFF) as u8);
        self.program_option_byte_data(0x1FFFF805, ((optbyte >> 8) & 0xFF) as u8);
        
        FlashStatus::Complete
    }
    
    /// 获取用户选项字节
    pub unsafe fn get_user_option_byte(&self) -> u32 {
        let flash = self.flash();
        flash.obr().read().bits()
    }
    
    /// 获取写保护选项字节
    pub unsafe fn get_write_protection_option_byte(&self) -> u32 {
        let flash = self.flash();
        flash.wrpr().read().bits()
    }
    
    /// 获取读保护状态
    pub unsafe fn get_read_out_protection_status(&self) -> bool {
        let flash = self.flash();
        flash.obr().read().rdprt().bit()
    }
    
    /// 配置FLASH中断
    pub unsafe fn it_config(&self, it: u32, enable: bool) {
        let flash = self.flash();
        if enable {
            flash.cr().modify(|_, w| w.bits(flash.cr().read().bits() | it));
        } else {
            flash.cr().modify(|_, w| w.bits(flash.cr().read().bits() & !it));
        }
    }
    
    /// 获取标志状态
    pub unsafe fn get_flag_status(&self, flag: u32) -> bool {
        let flash = self.flash();
        (flash.sr().read().bits() & flag) != 0
    }
    
    /// 清除标志
    pub unsafe fn clear_flag(&self, flag: u32) {
        let flash = self.flash();
        flash.sr().write(|w| w.bits(flash.sr().read().bits() & !flag));
    }
}

/// 预定义的FLASH实例
pub const FLASH: FlashDriver = FlashDriver::new();