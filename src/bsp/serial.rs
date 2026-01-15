//! UART/USART模块
//! 提供串口通信功能，支持中断接收和多种配置选项

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

/// 数据位长度
#[derive(Debug, Clone, Copy)]
pub enum WordLength {
    /// 8位数据位
    Bits8,
    /// 9位数据位
    Bits9,
}

/// 停止位配置
#[derive(Debug, Clone, Copy)]
pub enum StopBits {
    /// 0.5位停止位
    Bits0_5,
    /// 1位停止位
    Bits1,
    /// 1.5位停止位
    Bits1_5,
    /// 2位停止位
    Bits2,
}

/// 奇偶校验配置
#[derive(Debug, Clone, Copy)]
pub enum Parity {
    /// 无校验
    None,
    /// 偶校验
    Even,
    /// 奇校验
    Odd,
}

/// 硬件流控制配置
#[derive(Debug, Clone, Copy)]
pub enum HardwareFlowControl {
    /// 无硬件流控
    None,
    /// RTS流控
    RTS,
    /// CTS流控
    CTS,
    /// RTS和CTS流控
    RtsCts,
}

/// 同步模式时钟配置
#[derive(Debug, Clone, Copy)]
pub enum SyncClock {
    /// 禁用同步时钟
    Disable,
    /// 启用同步时钟
    Enable,
}

/// 同步模式时钟极性
#[derive(Debug, Clone, Copy)]
pub enum SyncClockPolarity {
    /// 时钟空闲状态为低电平
    Low,
    /// 时钟空闲状态为高电平
    High,
}

/// 同步模式时钟相位
#[derive(Debug, Clone, Copy)]
pub enum SyncClockPhase {
    /// 第1个时钟边沿采样
    Edge1,
    /// 第2个时钟边沿采样
    Edge2,
}

/// 同步模式最后位配置
#[derive(Debug, Clone, Copy)]
pub enum SyncLastBit {
    /// 最后一位数据不输出时钟脉冲
    Disable,
    /// 最后一位数据输出时钟脉冲
    Enable,
}

/// 唤醒模式
#[derive(Debug, Clone, Copy)]
pub enum WakeUpMode {
    /// 空闲线唤醒
    IdleLine,
    /// 地址标记唤醒
    AddressMark,
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
    
    /// 清空缓冲区
    pub fn clear(&self) {
        self.head.store(0, Ordering::Relaxed);
        self.tail.store(0, Ordering::Relaxed);
        self.overflow.store(false, Ordering::Relaxed);
    }
}

/// 串口初始化配置结构体
#[derive(Debug, Clone, Copy)]
pub struct SerialConfig {
    /// 波特率
    pub baud_rate: BaudRate,
    /// 数据位长度
    pub word_length: WordLength,
    /// 停止位配置
    pub stop_bits: StopBits,
    /// 奇偶校验配置
    pub parity: Parity,
    /// 硬件流控制配置
    pub hw_flow_control: HardwareFlowControl,
    /// 同步模式时钟配置
    pub sync_clock: SyncClock,
    /// 同步模式时钟极性
    pub sync_cpol: SyncClockPolarity,
    /// 同步模式时钟相位
    pub sync_cpha: SyncClockPhase,
    /// 同步模式最后位配置
    pub sync_last_bit: SyncLastBit,
    /// 唤醒模式
    pub wakeup_mode: WakeUpMode,
    /// 是否启用接收中断
    pub rx_interrupt: bool,
    /// 是否启用空闲中断
    pub idle_interrupt: bool,
    /// 是否启用发送中断
    pub tx_interrupt: bool,
    /// 是否启用发送完成中断
    pub tc_interrupt: bool,
    /// 是否启用错误中断
    pub error_interrupt: bool,
}

impl Default for SerialConfig {
    fn default() -> Self {
        Self {
            baud_rate: BaudRate::B115200,
            word_length: WordLength::Bits8,
            stop_bits: StopBits::Bits1,
            parity: Parity::None,
            hw_flow_control: HardwareFlowControl::None,
            sync_clock: SyncClock::Disable,
            sync_cpol: SyncClockPolarity::Low,
            sync_cpha: SyncClockPhase::Edge1,
            sync_last_bit: SyncLastBit::Disable,
            wakeup_mode: WakeUpMode::IdleLine,
            rx_interrupt: false,
            idle_interrupt: false,
            tx_interrupt: false,
            tc_interrupt: false,
            error_interrupt: false,
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
        
        // 精确计算波特率，参考标准库实现
        let integer_divider = fck / (16 * baud as u32);
        let fractional_divider = ((fck % (16 * baud as u32)) * 16 + baud as u32 / 2) / baud as u32;
        
        (integer_divider << 4) | fractional_divider
    }
    
    /// 初始化串口
    pub fn init(&self, config: SerialConfig) {
        let rcc = unsafe { &mut *(0x40021000 as *mut Rcc) };
        let usart = self.get_usart();
        
        // 1. 启用串口时钟
        unsafe {
            match self.port {
                SerialPort::USART1 => {
                    rcc.apb2enr().modify(|_, w| w.usart1en().set_bit());
                }
                SerialPort::USART2 => {
                    rcc.apb1enr().modify(|_, w| w.usart2en().set_bit());
                }
                SerialPort::USART3 => {
                    rcc.apb1enr().modify(|_, w| w.usart3en().set_bit());
                }
            }
        }
        
        // 2. 配置波特率
        let brr = match config.baud_rate {
            BaudRate::B9600 => self.baud_rate_value(BaudRate::B9600),
            BaudRate::B19200 => self.baud_rate_value(BaudRate::B19200),
            BaudRate::B38400 => self.baud_rate_value(BaudRate::B38400),
            BaudRate::B57600 => self.baud_rate_value(BaudRate::B57600),
            BaudRate::B115200 => self.baud_rate_value(BaudRate::B115200),
        };
        
        unsafe {
            usart.brr().write(|w| w.bits(brr));
        }
        
        // 3. 配置CR1寄存器
        unsafe {
            usart.cr1().write(|w| {
                let mut cr1 = w;
                
                // 启用USART
                cr1 = cr1.ue().set_bit();
                // 启用发送
                cr1 = cr1.te().set_bit();
                // 启用接收
                cr1 = cr1.re().set_bit();
                
                // 配置数据位长度
                match config.word_length {
                    WordLength::Bits8 => cr1 = cr1.m().clear_bit(),
                    WordLength::Bits9 => cr1 = cr1.m().set_bit(),
                }
                
                // 配置奇偶校验
                match config.parity {
                    Parity::None => {
                        cr1 = cr1.pce().clear_bit();
                    }
                    Parity::Even => {
                        cr1 = cr1.pce().set_bit();
                        cr1 = cr1.ps().clear_bit();
                    }
                    Parity::Odd => {
                        cr1 = cr1.pce().set_bit();
                        cr1 = cr1.ps().set_bit();
                    }
                }
                
                // 配置唤醒模式
                match config.wakeup_mode {
                    WakeUpMode::IdleLine => cr1 = cr1.wake().clear_bit(),
                    WakeUpMode::AddressMark => cr1 = cr1.wake().set_bit(),
                }
                
                // 配置中断
                if config.rx_interrupt {
                    cr1 = cr1.rxneie().set_bit();
                }
                
                if config.idle_interrupt {
                    cr1 = cr1.idleie().set_bit();
                }
                
                if config.tx_interrupt {
                    cr1 = cr1.txeie().set_bit();
                }
                
                if config.tc_interrupt {
                    cr1 = cr1.tcie().set_bit();
                }
                
                cr1
            });
        }
        
        // 4. 配置CR2寄存器
        unsafe {
            usart.cr2().write(|w| {
                let mut cr2 = w;
                
                // 配置停止位
                let stop_bits = match config.stop_bits {
                    StopBits::Bits0_5 => 0b00,
                    StopBits::Bits1 => 0b01,
                    StopBits::Bits1_5 => 0b10,
                    StopBits::Bits2 => 0b11,
                };
                cr2 = cr2.stop().bits(stop_bits);
                
                // 配置同步模式时钟
                match config.sync_clock {
                    SyncClock::Disable => cr2 = cr2.clken().clear_bit(),
                    SyncClock::Enable => cr2 = cr2.clken().set_bit(),
                }
                
                // 配置同步模式时钟极性
                match config.sync_cpol {
                    SyncClockPolarity::Low => cr2 = cr2.cpol().clear_bit(),
                    SyncClockPolarity::High => cr2 = cr2.cpol().set_bit(),
                }
                
                // 配置同步模式时钟相位
                match config.sync_cpha {
                    SyncClockPhase::Edge1 => cr2 = cr2.cpha().clear_bit(),
                    SyncClockPhase::Edge2 => cr2 = cr2.cpha().set_bit(),
                }
                
                // 配置同步模式最后位
                match config.sync_last_bit {
                    SyncLastBit::Disable => cr2 = cr2.lbcl().clear_bit(),
                    SyncLastBit::Enable => cr2 = cr2.lbcl().set_bit(),
                }
                
                cr2
            });
        }
        
        // 5. 配置CR3寄存器
        unsafe {
            usart.cr3().write(|w| {
                let mut cr3 = w;
                
                // 配置硬件流控制
                match config.hw_flow_control {
                    HardwareFlowControl::None => {
                        cr3 = cr3.rtse().clear_bit().ctse().clear_bit();
                    }
                    HardwareFlowControl::RTS => {
                        cr3 = cr3.rtse().set_bit().ctse().clear_bit();
                    }
                    HardwareFlowControl::CTS => {
                        cr3 = cr3.rtse().clear_bit().ctse().set_bit();
                    }
                    HardwareFlowControl::RtsCts => {
                        cr3 = cr3.rtse().set_bit().ctse().set_bit();
                    }
                }
                
                // 配置错误中断
                if config.error_interrupt {
                    cr3 = cr3.eie().set_bit();
                }
                
                cr3
            });
        }
    }
    
    /// 初始化串口（使用默认配置）
    pub fn init_default(&self) {
        self.init(SerialConfig::default());
    }
    
    /// 启用串口
    pub fn enable(&self) {
        let usart = self.get_usart();
        unsafe {
            usart.cr1().modify(|_, w| w.ue().set_bit());
        }
    }
    
    /// 禁用串口
    pub fn disable(&self) {
        let usart = self.get_usart();
        unsafe {
            usart.cr1().modify(|_, w| w.ue().clear_bit());
        }
    }
    
    /// 启用发送中断
    pub fn enable_tx_interrupt(&self) {
        let usart = self.get_usart();
        unsafe {
            usart.cr1().modify(|_, w| w.txeie().set_bit());
        }
    }
    
    /// 禁用发送中断
    pub fn disable_tx_interrupt(&self) {
        let usart = self.get_usart();
        unsafe {
            usart.cr1().modify(|_, w| w.txeie().clear_bit());
        }
    }
    
    /// 启用接收中断
    pub fn enable_rx_interrupt(&self) {
        let usart = self.get_usart();
        unsafe {
            usart.cr1().modify(|_, w| w.rxneie().set_bit());
        }
    }
    
    /// 禁用接收中断
    pub fn disable_rx_interrupt(&self) {
        let usart = self.get_usart();
        unsafe {
            usart.cr1().modify(|_, w| w.rxneie().clear_bit());
        }
    }
    
    /// 启用空闲中断
    pub fn enable_idle_interrupt(&self) {
        let usart = self.get_usart();
        unsafe {
            usart.cr1().modify(|_, w| w.idleie().set_bit());
        }
    }
    
    /// 禁用空闲中断
    pub fn disable_idle_interrupt(&self) {
        let usart = self.get_usart();
        unsafe {
            usart.cr1().modify(|_, w| w.idleie().clear_bit());
        }
    }
    
    /// 启用错误中断
    pub fn enable_error_interrupt(&self) {
        let usart = self.get_usart();
        unsafe {
            usart.cr3().modify(|_, w| w.eie().set_bit());
        }
    }
    
    /// 禁用错误中断
    pub fn disable_error_interrupt(&self) {
        let usart = self.get_usart();
        unsafe {
            usart.cr3().modify(|_, w| w.eie().clear_bit());
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
    
    /// 处理错误中断
    pub fn handle_error_interrupt(&self) {
        let usart = self.get_usart();
        let sr = usart.sr().read();
        
        // 清除错误标志
        if sr.ore().bit_is_set() || sr.ne().bit_is_set() || sr.fe().bit_is_set() {
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
    
    /// 从接收缓冲区读取多个字节
    pub fn read_from_buffer_multiple(&self, buffer: &mut [u8]) -> usize {
        let mut read_count = 0;
        
        if let Some(rx_buffer) = &self.rx_buffer {
            for byte in buffer.iter_mut() {
                if let Some(data) = rx_buffer.pop() {
                    *byte = data;
                    read_count += 1;
                } else {
                    break;
                }
            }
        }
        
        read_count
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
            usart.dr().write(|w| w.bits(byte as u32));
        }
        
        // 等待发送完成
        while usart.sr().read().tc().bit_is_clear() {
            core::hint::spin_loop();
        }
    }
    
    /// 发送多个字节
    pub fn write_bytes(&self, bytes: &[u8]) {
        for &byte in bytes {
            self.write_byte(byte);
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
    
    /// 检查是否有发送缓冲区为空
    pub fn is_tx_buffer_empty(&self) -> bool {
        let usart = self.get_usart();
        usart.sr().read().txe().bit_is_set()
    }
    
    /// 检查发送是否完成
    pub fn is_tx_complete(&self) -> bool {
        let usart = self.get_usart();
        usart.sr().read().tc().bit_is_set()
    }
    
    /// 获取状态寄存器
    pub fn get_status(&self) -> u32 {
        let usart = self.get_usart();
        usart.sr().read().bits()
    }
    
    /// 清除状态标志位
    pub fn clear_status_flags(&self) {
        let usart = self.get_usart();
        unsafe {
            // 读取DR寄存器清除状态标志
            let _ = usart.dr().read();
        }
    }
    
    /// 发送Break信号
    pub fn send_break(&self) {
        let usart = self.get_usart();
        unsafe {
            usart.cr1().modify(|_, w| w.sbk().set_bit());
        }
    }
    
    /// 设置串口地址（用于多机通信）
    pub fn set_address(&self, address: u8) {
        let usart = self.get_usart();
        unsafe {
            usart.cr2().modify(|_, w| w.add().bits(address & 0x0F));
        }
    }
    
    /// 启用LIN模式
    pub fn enable_lin_mode(&self) {
        let usart = self.get_usart();
        unsafe {
            // LIN模式在当前实现中未完全支持
            // 这里仅作为占位符
        }
    }
    
    /// 禁用LIN模式
    pub fn disable_lin_mode(&self) {
        let usart = self.get_usart();
        unsafe {
            // LIN模式在当前实现中未完全支持
            // 这里仅作为占位符
        }
    }
    
    /// 启用半双工模式
    pub fn enable_half_duplex(&self) {
        let usart = self.get_usart();
        unsafe {
            usart.cr3().modify(|_, w| w.hdsel().set_bit());
        }
    }
    
    /// 禁用半双工模式
    pub fn disable_half_duplex(&self) {
        let usart = self.get_usart();
        unsafe {
            usart.cr3().modify(|_, w| w.hdsel().clear_bit());
        }
    }
    
    /// 获取CR1寄存器值
    pub fn get_cr1(&self) -> u32 {
        let usart = self.get_usart();
        usart.cr1().read().bits()
    }
    
    /// 获取CR2寄存器值
    pub fn get_cr2(&self) -> u32 {
        let usart = self.get_usart();
        usart.cr2().read().bits()
    }
    
    /// 获取CR3寄存器值
    pub fn get_cr3(&self) -> u32 {
        let usart = self.get_usart();
        usart.cr3().read().bits()
    }
    
    /// 重置串口配置
    pub fn reset(&self) {
        let usart = self.get_usart();
        unsafe {
            // 禁用USART
            usart.cr1().modify(|_, w| w.ue().clear_bit());
            // 重置寄存器
            usart.cr1().reset();
            usart.cr2().reset();
            usart.cr3().reset();
            // 清除状态标志
            let _ = usart.sr().read();
            let _ = usart.dr().read();
        }
    }
    
    /// 启用SmartCard模式
    pub fn enable_smartcard_mode(&self) {
        let usart = self.get_usart();
        unsafe {
            // 启用SmartCard模式
            usart.cr3().modify(|_, w| w.scen().set_bit());
        }
    }
    
    /// 禁用SmartCard模式
    pub fn disable_smartcard_mode(&self) {
        let usart = self.get_usart();
        unsafe {
            usart.cr3().modify(|_, w| w.scen().clear_bit());
        }
    }
    
    /// 启用SmartCard NACK
    pub fn enable_smartcard_nack(&self) {
        let usart = self.get_usart();
        unsafe {
            usart.cr3().modify(|_, w| w.nack().set_bit());
        }
    }
    
    /// 禁用SmartCard NACK
    pub fn disable_smartcard_nack(&self) {
        let usart = self.get_usart();
        unsafe {
            usart.cr3().modify(|_, w| w.nack().clear_bit());
        }
    }
    
    /// 启用IrDA模式
    pub fn enable_irda_mode(&self) {
        let usart = self.get_usart();
        unsafe {
            usart.cr3().modify(|_, w| w.iren().set_bit());
        }
    }
    
    /// 禁用IrDA模式
    pub fn disable_irda_mode(&self) {
        let usart = self.get_usart();
        unsafe {
            usart.cr3().modify(|_, w| w.iren().clear_bit());
        }
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