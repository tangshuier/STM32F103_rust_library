//! UART/USART模块
//! 提供串口通信功能，支持中断接收

// 屏蔽未使用代码警告
#![allow(unused)]

use core::fmt;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use core::cell::UnsafeCell;

// 导入内部生成的设备驱动库
use stm32f103::*;

/// 串口波特率枚举
#[derive(Debug, Clone, Copy)]
pub enum BaudRate {
    B9600,
    B19200,
    B38400,
    B57600,
    B115200,
}

/// 串口枚举
#[derive(Debug, Clone, Copy)]
pub enum SerialPort {
    USART1,
    USART2,
    USART3,
}

/// 串口接收缓冲区大小
const RX_BUFFER_SIZE: usize = 256;

/// 串口接收缓冲区
pub struct RxBuffer {
    buffer: UnsafeCell<[u8; RX_BUFFER_SIZE]>,
    head: AtomicUsize,
    tail: AtomicUsize,
    overflow: AtomicBool,
}

/// 实现 Send trait，允许 RxBuffer 在线程间安全传递
unsafe impl Send for RxBuffer {}

/// 实现 Sync trait，允许多个线程同时访问 RxBuffer
unsafe impl Sync for RxBuffer {}

impl RxBuffer {
    /// 创建新的接收缓冲区
    pub const fn new() -> Self {
        Self {
            buffer: UnsafeCell::new([0; RX_BUFFER_SIZE]),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            overflow: AtomicBool::new(false),
        }
    }
    
    /// 向缓冲区添加一个字节
    pub fn push(&self, byte: u8) {
        let head = self.head.load(Ordering::Relaxed);
        let next_head = (head + 1) % RX_BUFFER_SIZE;
        
        if next_head != self.tail.load(Ordering::Relaxed) {
            unsafe {
                let buffer = &mut *self.buffer.get();
                buffer[head] = byte;
            }
            self.head.store(next_head, Ordering::Relaxed);
            self.overflow.store(false, Ordering::Relaxed);
        } else {
            self.overflow.store(true, Ordering::Relaxed);
        }
    }
    
    /// 从缓冲区读取一个字节
    pub fn pop(&self) -> Option<u8> {
        let tail = self.tail.load(Ordering::Relaxed);
        
        if tail != self.head.load(Ordering::Relaxed) {
            let byte = unsafe {
                let buffer = &*self.buffer.get();
                buffer[tail]
            };
            let next_tail = (tail + 1) % RX_BUFFER_SIZE;
            self.tail.store(next_tail, Ordering::Relaxed);
            Some(byte)
        } else {
            None
        }
    }
    
    /// 检查缓冲区是否为空
    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Relaxed) == self.tail.load(Ordering::Relaxed)
    }
    
    /// 检查缓冲区是否已满
    pub fn is_full(&self) -> bool {
        let head = self.head.load(Ordering::Relaxed);
        let next_head = (head + 1) % RX_BUFFER_SIZE;
        next_head == self.tail.load(Ordering::Relaxed)
    }
    
    /// 检查是否发生溢出
    pub fn has_overflow(&self) -> bool {
        self.overflow.load(Ordering::Relaxed)
    }
    
    /// 清除溢出标志
    pub fn clear_overflow(&self) {
        self.overflow.store(false, Ordering::Relaxed);
    }
    
    /// 获取缓冲区中的字节数
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        
        if head >= tail {
            head - tail
        } else {
            RX_BUFFER_SIZE - (tail - head)
        }
    }
}

/// 串口结构体
pub struct Serial {
    port: SerialPort,
    rx_buffer: Option<&'static RxBuffer>,
}

impl SerialPort {
    /// 获取串口寄存器
    fn get_usart(&self) -> &'static mut Usart1 {
        match self {
            SerialPort::USART1 => unsafe { &mut *(0x40013800 as *mut Usart1) },
            SerialPort::USART2 => unsafe { &mut *(0x40004400 as *mut Usart1) },
            SerialPort::USART3 => unsafe { &mut *(0x40004800 as *mut Usart1) },
        }
    }
    
    /// 获取串口时钟使能位
    const fn clock_en_bit(&self) -> u32 {
        match self {
            SerialPort::USART1 => 1 << 14,  // APB2
            SerialPort::USART2 => 1 << 17,  // APB1
            SerialPort::USART3 => 1 << 18,  // APB1
        }
    }
    
    /// 获取时钟寄存器
    fn clock_reg(&self) -> &'static mut Rcc {
        unsafe { &mut *(0x40021000 as *mut Rcc) }
    }
}

impl Serial {
    /// 创建新的串口实例
    pub const fn new(port: SerialPort) -> Self {
        Self {
            port,
            rx_buffer: None,
        }
    }
    
    /// 创建带接收缓冲区的串口实例
    pub const fn new_with_buffer(port: SerialPort, buffer: &'static RxBuffer) -> Self {
        Self {
            port,
            rx_buffer: Some(buffer),
        }
    }
    
    /// 获取USART寄存器
    fn get_usart(&self) -> &'static mut Usart1 {
        self.port.get_usart()
    }
    
    /// 获取波特率寄存器值
    fn baud_rate_value(&self, baud: BaudRate) -> u32 {
        // 获取串口时钟频率
        // USART1挂载在APB2上，时钟频率为72MHz
        // USART2和USART3挂载在APB1上，时钟频率为36MHz
        let fck = match self.port {
            SerialPort::USART1 => 72_000_000,
            SerialPort::USART2 | SerialPort::USART3 => 36_000_000,
        };
        
        match baud {
            BaudRate::B9600 => fck / 9600,
            BaudRate::B19200 => fck / 19200,
            BaudRate::B38400 => fck / 38400,
            BaudRate::B57600 => fck / 57600,
            BaudRate::B115200 => fck / 115200,
        }
    }
    
    /// 初始化串口（无中断）
    pub fn init(&self, baud: BaudRate) {
        let rcc = unsafe { &mut *(0x40021000 as *mut Rcc) };
        let usart = self.get_usart();
        
        // 1. 启用串口时钟
        unsafe {
            match self.port {
                SerialPort::USART1 => {
                    rcc.apb2enr().modify(|_, w: &mut stm32f103::rcc::apb2enr::W| w.usart1en().set_bit());
                }
                SerialPort::USART2 => {
                    rcc.apb1enr().modify(|_, w: &mut stm32f103::rcc::apb1enr::W| w.usart2en().set_bit());
                }
                SerialPort::USART3 => {
                    rcc.apb1enr().modify(|_, w: &mut stm32f103::rcc::apb1enr::W| w.usart3en().set_bit());
                }
            }
        }
        
        // 2. 配置波特率
        let brr = self.baud_rate_value(baud);
        unsafe {
            usart.brr().write(|w: &mut stm32f103::usart1::brr::W| w.bits(brr));
        }
        
        // 3. 配置串口参数：8位数据，1位停止位，无校验
        // CR1: 启用USART，8位数据，无奇偶校验，禁用中断
        unsafe {
            usart.cr1().write(|w: &mut stm32f103::usart1::cr1::W| {
                w.ue().set_bit()
                    .te().set_bit()
                    .re().set_bit()
            });
        }
        // CR2: 1位停止位
        unsafe {
            usart.cr2().write(|w: &mut stm32f103::usart1::cr2::W| w.bits(0));
        }
        // CR3: 无硬件流控
        unsafe {
            usart.cr3().write(|w: &mut stm32f103::usart1::cr3::W| w.bits(0));
        }
    }
    
    /// 初始化串口（带接收中断）
    pub fn init_with_interrupt(&self, baud: BaudRate) {
        let rcc = unsafe { &mut *(0x40021000 as *mut Rcc) };
        let usart = self.get_usart();
        
        // 1. 启用串口时钟
        unsafe {
            match self.port {
                SerialPort::USART1 => {
                    rcc.apb2enr().modify(|_, w: &mut stm32f103::rcc::apb2enr::W| w.usart1en().set_bit());
                }
                SerialPort::USART2 => {
                    rcc.apb1enr().modify(|_, w: &mut stm32f103::rcc::apb1enr::W| w.usart2en().set_bit());
                }
                SerialPort::USART3 => {
                    rcc.apb1enr().modify(|_, w: &mut stm32f103::rcc::apb1enr::W| w.usart3en().set_bit());
                }
            }
        }
        
        // 2. 配置波特率
        let brr = self.baud_rate_value(baud);
        unsafe {
            usart.brr().write(|w: &mut stm32f103::usart1::brr::W| w.bits(brr));
        }
        
        // 3. 配置串口参数：8位数据，1位停止位，无校验，启用接收中断
        // CR1: 启用USART，8位数据，无奇偶校验，启用接收中断
        unsafe {
            usart.cr1().write(|w: &mut stm32f103::usart1::cr1::W| {
                w.ue().set_bit()
                    .te().set_bit()
                    .re().set_bit()
                    .rxneie().set_bit()
            });
        }
        // CR2: 1位停止位
        unsafe {
            usart.cr2().write(|w: &mut stm32f103::usart1::cr2::W| w.bits(0));
        }
        // CR3: 无硬件流控
        unsafe {
            usart.cr3().write(|w: &mut stm32f103::usart1::cr3::W| w.bits(0));
        }
    }
    
    /// 初始化串口（带空闲中断）
    pub fn init_with_idle_interrupt(&self, baud: BaudRate) {
        let rcc = unsafe { &mut *(0x40021000 as *mut Rcc) };
        let usart = self.get_usart();
        
        // 1. 启用串口时钟
        unsafe {
            match self.port {
                SerialPort::USART1 => {
                    rcc.apb2enr().modify(|_, w: &mut stm32f103::rcc::apb2enr::W| w.usart1en().set_bit());
                }
                SerialPort::USART2 => {
                    rcc.apb1enr().modify(|_, w: &mut stm32f103::rcc::apb1enr::W| w.usart2en().set_bit());
                }
                SerialPort::USART3 => {
                    rcc.apb1enr().modify(|_, w: &mut stm32f103::rcc::apb1enr::W| w.usart3en().set_bit());
                }
            }
        }
        
        // 2. 配置波特率
        let brr = self.baud_rate_value(baud);
        unsafe {
            usart.brr().write(|w: &mut stm32f103::usart1::brr::W| w.bits(brr));
        }
        
        // 3. 配置串口参数：8位数据，1位停止位，无校验，启用接收中断和空闲中断
        // CR1: 启用USART，8位数据，无奇偶校验，启用接收中断和空闲中断
        unsafe {
            usart.cr1().write(|w: &mut stm32f103::usart1::cr1::W| {
                w.ue().set_bit()
                    .te().set_bit()
                    .re().set_bit()
                    .rxneie().set_bit()
                    .idleie().set_bit()
            });
        }
        // CR2: 1位停止位
        unsafe {
            usart.cr2().write(|w: &mut stm32f103::usart1::cr2::W| w.bits(0));
        }
        // CR3: 无硬件流控
        unsafe {
            usart.cr3().write(|w: &mut stm32f103::usart1::cr3::W| w.bits(0));
        }
    }
    
    /// 启用接收中断
    pub fn enable_rx_interrupt(&self) {
        let usart = self.get_usart();
        unsafe {
            usart.cr1().modify(|_, w: &mut stm32f103::usart1::cr1::W| w.rxneie().set_bit());
        }
    }
    
    /// 禁用接收中断
    pub fn disable_rx_interrupt(&self) {
        let usart = self.get_usart();
        unsafe {
            usart.cr1().modify(|_, w: &mut stm32f103::usart1::cr1::W| w.rxneie().clear_bit());
        }
    }
    
    /// 启用空闲中断
    pub fn enable_idle_interrupt(&self) {
        let usart = self.get_usart();
        unsafe {
            usart.cr1().modify(|_, w: &mut stm32f103::usart1::cr1::W| w.idleie().set_bit());
        }
    }
    
    /// 禁用空闲中断
    pub fn disable_idle_interrupt(&self) {
        let usart = self.get_usart();
        unsafe {
            usart.cr1().modify(|_, w: &mut stm32f103::usart1::cr1::W| w.idleie().clear_bit());
        }
    }
    
    /// 处理接收中断
    pub fn handle_rx_interrupt(&self) {
        let usart = self.get_usart();
        
        // 检查是否有接收数据
        if usart.sr().read().rxne().bit_is_set() {
            let byte = (usart.dr().read().bits() & 0xFF) as u8;
            
            // 如果有接收缓冲区，将数据添加到缓冲区
            if let Some(buffer) = &self.rx_buffer {
                buffer.push(byte);
            }
        }
    }
    
    /// 处理空闲中断
    pub fn handle_idle_interrupt(&self) {
        let usart = self.get_usart();
        
        // 清除空闲中断标志（读取SR后读取DR）
        if usart.sr().read().idle().bit_is_set() {
            // 读取DR寄存器清除空闲标志
            unsafe {
                let _ = usart.dr().read();
            }
        }
    }
    
    /// 从接收缓冲区读取一个字节
    pub fn read_from_buffer(&self) -> Option<u8> {
        if let Some(buffer) = &self.rx_buffer {
            buffer.pop()
        } else {
            None
        }
    }
    
    /// 检查接收缓冲区是否有数据
    pub fn has_data(&self) -> bool {
        if let Some(buffer) = &self.rx_buffer {
            !buffer.is_empty()
        } else {
            self.is_data_available()
        }
    }
    
    /// 获取接收缓冲区中的字节数
    pub fn buffer_len(&self) -> usize {
        if let Some(buffer) = &self.rx_buffer {
            buffer.len()
        } else {
            0
        }
    }
    
    /// 检查接收缓冲区是否溢出
    pub fn buffer_overflow(&self) -> bool {
        if let Some(buffer) = &self.rx_buffer {
            buffer.has_overflow()
        } else {
            false
        }
    }
    
    /// 发送一个字节
    pub fn write_byte(&self, byte: u8) {
        let usart = self.get_usart();
        
        // 等待发送缓冲区为空
        while usart.sr().read().txe().bit_is_clear() {
            core::hint::spin_loop();
        }
        
        // 发送数据
        unsafe {
            usart.dr().write(|w: &mut stm32f103::usart1::dr::W| w.bits(byte as u32));
        }
        
        // 等待发送完成
        while usart.sr().read().tc().bit_is_clear() {
            core::hint::spin_loop();
        }
    }
    
    /// 发送字符串
    pub fn write_str(&self, s: &str) {
        for &byte in s.as_bytes() {
            self.write_byte(byte);
        }
    }
    
    /// 接收一个字节
    pub fn read_byte(&self) -> u8 {
        let usart = self.get_usart();
        
        // 等待接收数据
        while usart.sr().read().rxne().bit_is_clear() {
            core::hint::spin_loop();
        }
        
        // 读取数据
        unsafe {
            (usart.dr().read().bits() & 0xFF) as u8
        }
    }
    
    /// 检查是否有数据可读
    pub fn is_data_available(&self) -> bool {
        let usart = self.get_usart();
        usart.sr().read().rxne().bit_is_set()
    }
}

/// 实现fmt::Write特性，支持使用write!宏
impl fmt::Write for Serial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c)?;
        }
        Ok(())
    }
    
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.write_byte(c as u8);
        Ok(())
    }
}

/// 预定义的串口接收缓冲区
pub static USART1_RX_BUFFER: RxBuffer = RxBuffer::new();
pub static USART2_RX_BUFFER: RxBuffer = RxBuffer::new();
pub static USART3_RX_BUFFER: RxBuffer = RxBuffer::new();

/// 预定义的串口常量（无缓冲区）
pub const USART1: Serial = Serial::new(SerialPort::USART1);
pub const USART2: Serial = Serial::new(SerialPort::USART2);
pub const USART3: Serial = Serial::new(SerialPort::USART3);

/// 预定义的串口常量（带缓冲区）
pub const USART1_WITH_BUFFER: Serial = Serial::new_with_buffer(SerialPort::USART1, &USART1_RX_BUFFER);
pub const USART2_WITH_BUFFER: Serial = Serial::new_with_buffer(SerialPort::USART2, &USART2_RX_BUFFER);
pub const USART3_WITH_BUFFER: Serial = Serial::new_with_buffer(SerialPort::USART3, &USART3_RX_BUFFER);