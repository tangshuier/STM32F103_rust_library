//! GPIO模块
//! 提供GPIO引脚的封装和操作

// 屏蔽未使用代码警告
#![allow(unused)]

// 使用内部生成的设备驱动库
use stm32f103::*;

/// GPIO速度枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GpioSpeed {
    Speed10MHz,   // 10MHz
    Speed2MHz,    // 2MHz
    Speed50MHz,   // 50MHz
}

/// GPIO模式枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GpioMode {
    /// 输入模式
    InputFloating,    // 浮空输入
    InputPullUp,      // 上拉输入
    InputPullDown,    // 下拉输入
    Analog,           // 模拟输入
    
    /// 输出模式
    OutputPushPull,   // 推挽输出
    OutputOpenDrain,  // 开漏输出
    
    /// 复用功能模式
    AlternatePushPull, // 复用推挽输出
    AlternateOpenDrain, // 复用开漏输出
}

/// GPIO端口枚举
#[derive(Debug, Clone, Copy)]
pub enum GpioPort {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

/// GPIO初始化结构体
#[derive(Debug, Clone, Copy)]
pub struct GpioInit {
    /// 引脚掩码，每个位代表一个引脚，1表示需要配置
    pub pin_mask: u16,
    /// 引脚速度
    pub speed: GpioSpeed,
    /// 引脚模式
    pub mode: GpioMode,
}

impl Default for GpioInit {
    /// 默认初始化GPIO_Init结构体
    fn default() -> Self {
        Self {
            pin_mask: 0x0000, // 默认不配置任何引脚
            speed: GpioSpeed::Speed50MHz, // 默认速度50MHz
            mode: GpioMode::InputFloating, // 默认模式为浮空输入
        }
    }
}

/// 初始化GPIO_Init结构体为默认值
pub fn gpio_struct_init(init: &mut GpioInit) {
    *init = GpioInit::default();
}

/// GPIO引脚结构体
#[derive(Debug, Clone, Copy)]
pub struct GpioPin {
    port: GpioPort,
    pin: u8,
}

/// GPIO端口结构体
#[derive(Debug, Clone, Copy)]
pub struct GpioPortStruct {
    port: GpioPort,
}

/// GPIO端口实现
impl GpioPort {
    /// 获取端口基地址
    fn base_addr(&self) -> u32 {
        match self {
            GpioPort::A => 0x4001_0800,
            GpioPort::B => 0x4001_0C00,
            GpioPort::C => 0x4001_1000,
            GpioPort::D => 0x4001_1400,
            GpioPort::E => 0x4001_1800,
            GpioPort::F => 0x4001_1C00,
            GpioPort::G => 0x4001_2000,
        }
    }
    
    /// 获取端口时钟使能位
    fn clock_en_bit(&self) -> u32 {
        match self {
            GpioPort::A => 1 << 2,
            GpioPort::B => 1 << 3,
            GpioPort::C => 1 << 4,
            GpioPort::D => 1 << 5,
            GpioPort::E => 1 << 6,
            GpioPort::F => 1 << 7,
            GpioPort::G => 1 << 8,
        }
    }
    
    /// 创建GpioPortStruct实例
    pub const fn into_struct(&self) -> GpioPortStruct {
        GpioPortStruct {
            port: *self,
        }
    }
}

/// GpioPortStruct实现
impl GpioPortStruct {
    /// 创建新的GpioPortStruct实例
    pub const fn new(port: GpioPort) -> Self {
        Self {
            port,
        }
    }
    
    /// 获取对应的GPIO端口寄存器块
    unsafe fn get_port(&self) -> &'static mut stm32f103::gpioa::RegisterBlock {
        match self.port {
            GpioPort::A => &mut *(0x40010800 as *mut stm32f103::gpioa::RegisterBlock),
            GpioPort::B => &mut *(0x40010C00 as *mut stm32f103::gpioa::RegisterBlock),
            GpioPort::C => &mut *(0x40011000 as *mut stm32f103::gpioa::RegisterBlock),
            GpioPort::D => &mut *(0x40011400 as *mut stm32f103::gpioa::RegisterBlock),
            GpioPort::E => &mut *(0x40011800 as *mut stm32f103::gpioa::RegisterBlock),
            GpioPort::F => &mut *(0x40011C00 as *mut stm32f103::gpioa::RegisterBlock),
            GpioPort::G => &mut *(0x40012000 as *mut stm32f103::gpioa::RegisterBlock),
        }
    }
    
    /// 启用端口时钟
    pub unsafe fn enable_clock(&self) {
        let rcc = &mut *(0x40021000 as *mut stm32f103::rcc::RegisterBlock);
        match self.port {
            GpioPort::A => {
                let mut value = rcc.apb2enr().read().bits();
                value |= (1 << 2);
                rcc.apb2enr().write(|w: &mut stm32f103::rcc::apb2enr::W| unsafe { w.bits(value) });
            },
            GpioPort::B => {
                let mut value = rcc.apb2enr().read().bits();
                value |= (1 << 3);
                rcc.apb2enr().write(|w: &mut stm32f103::rcc::apb2enr::W| unsafe { w.bits(value) });
            },
            GpioPort::C => {
                let mut value = rcc.apb2enr().read().bits();
                value |= (1 << 4);
                rcc.apb2enr().write(|w: &mut stm32f103::rcc::apb2enr::W| unsafe { w.bits(value) });
            },
            GpioPort::D => {
                let mut value = rcc.apb2enr().read().bits();
                value |= (1 << 5);
                rcc.apb2enr().write(|w: &mut stm32f103::rcc::apb2enr::W| unsafe { w.bits(value) });
            },
            GpioPort::E => {
                let mut value = rcc.apb2enr().read().bits();
                value |= (1 << 6);
                rcc.apb2enr().write(|w: &mut stm32f103::rcc::apb2enr::W| unsafe { w.bits(value) });
            },
            GpioPort::F => {
                let mut value = rcc.apb2enr().read().bits();
                value |= (1 << 7);
                rcc.apb2enr().write(|w: &mut stm32f103::rcc::apb2enr::W| unsafe { w.bits(value) });
            },
            GpioPort::G => {
                let mut value = rcc.apb2enr().read().bits();
                value |= (1 << 8);
                rcc.apb2enr().write(|w: &mut stm32f103::rcc::apb2enr::W| unsafe { w.bits(value) });
            },
        }
    }
    
    /// 读取整个端口的输入数据
    pub unsafe fn read_input(&self) -> u16 {
        let port = self.get_port();
        (port.idr().read().bits() & 0x0000FFFF) as u16
    }
    
    /// 读取整个端口的输出数据
    pub unsafe fn read_output(&self) -> u16 {
        let port = self.get_port();
        (port.odr().read().bits() & 0x0000FFFF) as u16
    }
    
    /// 写入整个端口的输出数据
    pub unsafe fn write_output(&self, value: u16) {
        let port = self.get_port();
        port.odr().write(|w: &mut stm32f103::gpioa::odr::W| unsafe { w.bits(value as u32) });
    }
    
    /// 设置端口的多个引脚为高电平
    pub unsafe fn set_pins(&self, pins: u16) {
        let port = self.get_port();
        port.bsrr().write(|w: &mut stm32f103::gpioa::bsrr::W| unsafe { w.bits(pins as u32) });
    }
    
    /// 重置端口的多个引脚为低电平
    pub unsafe fn reset_pins(&self, pins: u16) {
        let port = self.get_port();
        port.brr().write(|w: &mut stm32f103::gpioa::brr::W| unsafe { w.bits(pins as u32) });
    }
    
    /// 锁定端口的多个引脚配置
    pub unsafe fn lock_pins(&self, pins: u16) {
        let port = self.get_port();
        // 写入锁定键序列
        port.lckr().write(|w: &mut stm32f103::gpioa::lckr::W| unsafe { w.bits((1 << 16) | pins as u32) });
        port.lckr().write(|w: &mut stm32f103::gpioa::lckr::W| unsafe { w.bits(pins as u32) });
        port.lckr().write(|w: &mut stm32f103::gpioa::lckr::W| unsafe { w.bits((1 << 16) | pins as u32) });
        port.lckr().write(|w: &mut stm32f103::gpioa::lckr::W| unsafe { w.bits(pins as u32) });
    }
    
    /// 复位GPIO端口到默认状态
    pub unsafe fn deinit(&self) {
        let rcc = &mut *(0x40021000 as *mut stm32f103::rcc::RegisterBlock);
        
        // 使能复位
        match self.port {
            GpioPort::A => {
                rcc.apb2rstr().write(|w| unsafe { w.bits(1 << 2) });
                rcc.apb2rstr().write(|w| unsafe { w.bits(0) });
            },
            GpioPort::B => {
                rcc.apb2rstr().write(|w| unsafe { w.bits(1 << 3) });
                rcc.apb2rstr().write(|w| unsafe { w.bits(0) });
            },
            GpioPort::C => {
                rcc.apb2rstr().write(|w| unsafe { w.bits(1 << 4) });
                rcc.apb2rstr().write(|w| unsafe { w.bits(0) });
            },
            GpioPort::D => {
                rcc.apb2rstr().write(|w| unsafe { w.bits(1 << 5) });
                rcc.apb2rstr().write(|w| unsafe { w.bits(0) });
            },
            GpioPort::E => {
                rcc.apb2rstr().write(|w| unsafe { w.bits(1 << 6) });
                rcc.apb2rstr().write(|w| unsafe { w.bits(0) });
            },
            GpioPort::F => {
                rcc.apb2rstr().write(|w| unsafe { w.bits(1 << 7) });
                rcc.apb2rstr().write(|w| unsafe { w.bits(0) });
            },
            GpioPort::G => {
                rcc.apb2rstr().write(|w| unsafe { w.bits(1 << 8) });
                rcc.apb2rstr().write(|w| unsafe { w.bits(0) });
            },
        }
    }
    
    /// 使用GPIO初始化结构体配置端口
    pub unsafe fn init(&self, init: &GpioInit) {
        self.enable_clock();
        
        let port = self.get_port();
        let mut pin_mask = init.pin_mask;
        let mut pin_num = 0;
        
        // 遍历所有引脚
        while pin_num < 16 && pin_mask != 0 {
            if (pin_mask & 0x0001) != 0 {
                // 配置当前引脚
                let pin = GpioPin::new(self.port, pin_num);
                pin.into_mode(init.mode, init.speed);
            }
            
            // 处理下一个引脚
            pin_mask >>= 1;
            pin_num += 1;
        }
    }
}

/// GPIO重映射枚举
#[derive(Debug, Clone, Copy)]
pub enum GpioRemap {
    // SPI1重映射
    RemapSPI1,
    // I2C1重映射
    RemapI2C1,
    // USART1重映射
    RemapUSART1,
    // USART2重映射
    RemapUSART2,
    // USART3部分重映射
    PartialRemapUSART3,
    // USART3完全重映射
    FullRemapUSART3,
    // TIM1部分重映射
    PartialRemapTIM1,
    // TIM1完全重映射
    FullRemapTIM1,
    // TIM2部分重映射1
    PartialRemap1TIM2,
    // TIM2部分重映射2
    PartialRemap2TIM2,
    // TIM2完全重映射
    FullRemapTIM2,
    // TIM3部分重映射
    PartialRemapTIM3,
    // TIM3完全重映射
    FullRemapTIM3,
    // TIM4重映射
    RemapTIM4,
    // CAN1重映射1
    Remap1CAN1,
    // CAN1重映射2
    Remap2CAN1,
    // PD01重映射
    RemapPD01,
    // SWJ无JTRST重映射
    RemapSWJNoJTRST,
    // SWJ禁用JTAG重映射
    RemapSWJJTAGDisable,
    // SWJ完全禁用重映射
    RemapSWJDisable,
}

/// GPIO重映射配置函数
pub unsafe fn gpio_pin_remap_config(remap: GpioRemap, enable: bool) {
    let afio = &mut *(0x40010000 as *mut stm32f103::afio::RegisterBlock);
    
    match remap {
        GpioRemap::RemapSPI1 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00000001;
                } else {
                    value &= !0x00000001;
                }
                w.bits(value)
            });
        },
        GpioRemap::RemapI2C1 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00000002;
                } else {
                    value &= !0x00000002;
                }
                w.bits(value)
            });
        },
        GpioRemap::RemapUSART1 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00000004;
                } else {
                    value &= !0x00000004;
                }
                w.bits(value)
            });
        },
        GpioRemap::RemapUSART2 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00000008;
                } else {
                    value &= !0x00000008;
                }
                w.bits(value)
            });
        },
        GpioRemap::PartialRemapUSART3 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00140010;
                } else {
                    value &= !0x00140030;
                }
                w.bits(value)
            });
        },
        GpioRemap::FullRemapUSART3 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00140030;
                } else {
                    value &= !0x00140030;
                }
                w.bits(value)
            });
        },
        GpioRemap::PartialRemapTIM1 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00160040;
                } else {
                    value &= !0x001600C0;
                }
                w.bits(value)
            });
        },
        GpioRemap::FullRemapTIM1 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x001600C0;
                } else {
                    value &= !0x001600C0;
                }
                w.bits(value)
            });
        },
        GpioRemap::PartialRemap1TIM2 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00180100;
                } else {
                    value &= !0x00180300;
                }
                w.bits(value)
            });
        },
        GpioRemap::PartialRemap2TIM2 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00180200;
                } else {
                    value &= !0x00180300;
                }
                w.bits(value)
            });
        },
        GpioRemap::FullRemapTIM2 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00180300;
                } else {
                    value &= !0x00180300;
                }
                w.bits(value)
            });
        },
        GpioRemap::PartialRemapTIM3 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x001A0800;
                } else {
                    value &= !0x001A0C00;
                }
                w.bits(value)
            });
        },
        GpioRemap::FullRemapTIM3 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x001A0C00;
                } else {
                    value &= !0x001A0C00;
                }
                w.bits(value)
            });
        },
        GpioRemap::RemapTIM4 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00001000;
                } else {
                    value &= !0x00001000;
                }
                w.bits(value)
            });
        },
        GpioRemap::Remap1CAN1 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x001D4000;
                } else {
                    value &= !0x001D6000;
                }
                w.bits(value)
            });
        },
        GpioRemap::Remap2CAN1 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x001D6000;
                } else {
                    value &= !0x001D6000;
                }
                w.bits(value)
            });
        },
        GpioRemap::RemapPD01 => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00008000;
                } else {
                    value &= !0x00008000;
                }
                w.bits(value)
            });
        },
        GpioRemap::RemapSWJNoJTRST => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00300100;
                } else {
                    value &= !0x00300700;
                }
                w.bits(value)
            });
        },
        GpioRemap::RemapSWJJTAGDisable => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00300200;
                } else {
                    value &= !0x00300700;
                }
                w.bits(value)
            });
        },
        GpioRemap::RemapSWJDisable => {
            afio.mapr().modify(|r, w| {
                let mut value = r.bits();
                if enable {
                    value |= 0x00300400;
                } else {
                    value &= !0x00300700;
                }
                w.bits(value)
            });
        },
    }
}

/// 复位AFIO寄存器到默认状态
pub unsafe fn gpio_afio_deinit() {
    let rcc = &mut *(0x40021000 as *mut stm32f103::rcc::RegisterBlock);
    
    // 使能AFIO复位
    rcc.apb2rstr().write(|w| unsafe { w.bits(1 << 0) });
    rcc.apb2rstr().write(|w| unsafe { w.bits(0) });
}

/// 配置外部中断线
/// 该函数用于将指定GPIO端口的引脚映射到对应的外部中断线上
/// 注意：每个外部中断线(0-15)可以连接到不同端口的相同引脚号
pub unsafe fn gpio_exti_line_config(port_source: GpioPort, pin_source: u8) {
    let afio = &mut *(0x40010000 as *mut stm32f103::afio::RegisterBlock);
    
    // 确保pin_source在0-15范围内
    assert!(pin_source < 16, "Pin source must be between 0 and 15");
    
    // 将端口源转换为数字(0-6对应A-G)
    let port_num = match port_source {
        GpioPort::A => 0x00,
        GpioPort::B => 0x01,
        GpioPort::C => 0x02,
        GpioPort::D => 0x03,
        GpioPort::E => 0x04,
        GpioPort::F => 0x05,
        GpioPort::G => 0x06,
    };
    
    // 根据引脚号选择对应的EXTICR寄存器
    match pin_source / 4 {
        0 => {
            // EXTICR1 - 引脚0-3
            let pos = (pin_source % 4) * 4;
            let mask = 0x0F << pos;
            let value = (port_num << pos) & mask;
            
            afio.exticr1().modify(|r, w| {
                let mut bits = r.bits();
                bits &= !mask;
                bits |= value;
                w.bits(bits)
            });
        },
        1 => {
            // EXTICR2 - 引脚4-7
            let pos = (pin_source % 4) * 4;
            let mask = 0x0F << pos;
            let value = (port_num << pos) & mask;
            
            afio.exticr2().modify(|r, w| {
                let mut bits = r.bits();
                bits &= !mask;
                bits |= value;
                w.bits(bits)
            });
        },
        2 => {
            // EXTICR3 - 引脚8-11
            let pos = (pin_source % 4) * 4;
            let mask = 0x0F << pos;
            let value = (port_num << pos) & mask;
            
            afio.exticr3().modify(|r, w| {
                let mut bits = r.bits();
                bits &= !mask;
                bits |= value;
                w.bits(bits)
            });
        },
        3 => {
            // EXTICR4 - 引脚12-15
            let pos = (pin_source % 4) * 4;
            let mask = 0x0F << pos;
            let value = (port_num << pos) & mask;
            
            afio.exticr4().modify(|r, w| {
                let mut bits = r.bits();
                bits &= !mask;
                bits |= value;
                w.bits(bits)
            });
        },
        _ => unreachable!(),
    }
}

/// 功能状态枚举，用于控制外设的使能/禁用
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FunctionalState {
    Disable,
    Enable,
}

/// 配置事件输出
/// 该函数用于选择作为事件输出的GPIO引脚
pub unsafe fn gpio_event_output_config(port_source: GpioPort, pin_source: u8) {
    let afio = &mut *(0x40010000 as *mut stm32f103::afio::RegisterBlock);
    
    // 确保pin_source在0-15范围内
    assert!(pin_source < 16, "Pin source must be between 0 and 15");
    
    // 将端口源转换为数字(0-6对应A-G)
    let port_num = match port_source {
        GpioPort::A => 0x00,
        GpioPort::B => 0x01,
        GpioPort::C => 0x02,
        GpioPort::D => 0x03,
        GpioPort::E => 0x04,
        GpioPort::F => 0x05,
        GpioPort::G => 0x06,
    };
    
    // 配置EVCR寄存器
    afio.evcr().modify(|r, w| {
        let mut bits = r.bits();
        // 清除当前的引脚和端口选择
        bits &= !(0x0F | (0x07 << 4));
        // 设置新的引脚和端口
        bits |= pin_source as u32 | (port_num << 4);
        w.bits(bits)
    });
}

/// 控制事件输出功能的使能/禁用
pub unsafe fn gpio_event_output_cmd(new_state: FunctionalState) {
    let afio = &mut *(0x40010000 as *mut stm32f103::afio::RegisterBlock);
    
    afio.evcr().modify(|r, w| {
        let mut bits = r.bits();
        match new_state {
            FunctionalState::Enable => {
                bits |= (1 << 8); // 置位EVOE位
            },
            FunctionalState::Disable => {
                bits &= !(1 << 8); // 清除EVOE位
            },
        }
        w.bits(bits)
    });
}

/// 以太网媒体接口类型枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GpioEthMediaInterface {
    /// 媒体独立接口
    MediaInterfaceMii = 0,
    /// 简化媒体独立接口
    MediaInterfaceRmii = 1,
}

/// 配置以太网媒体接口
/// 该函数用于配置以太网MAC的媒体接口类型（MII或RMII）
pub unsafe fn gpio_eth_media_interface_config(interface: GpioEthMediaInterface) {
    let afio = &mut *(0x40010000 as *mut stm32f103::afio::RegisterBlock);
    
    afio.mapr().modify(|r, w| {
        let mut bits = r.bits();
        match interface {
            GpioEthMediaInterface::MediaInterfaceMii => {
                bits &= !(1 << 23); // 清除ETH_MII_RMII_SEL位，选择MII模式
            },
            GpioEthMediaInterface::MediaInterfaceRmii => {
                bits |= (1 << 23); // 置位ETH_MII_RMII_SEL位，选择RMII模式
            },
        }
        w.bits(bits)
    });
}

/// GPIO引脚实现
impl GpioPin {
    /// 创建新的GPIO引脚
    pub const fn new(port: GpioPort, pin: u8) -> Self {
        Self { port, pin }
    }
    
    /// 获取对应的GPIO端口寄存器块
    unsafe fn get_port(&self) -> &'static mut stm32f103::gpioa::RegisterBlock {
        match self.port {
            GpioPort::A => &mut *(0x40010800 as *mut stm32f103::gpioa::RegisterBlock),
            GpioPort::B => &mut *(0x40010C00 as *mut stm32f103::gpioa::RegisterBlock),
            GpioPort::C => &mut *(0x40011000 as *mut stm32f103::gpioa::RegisterBlock),
            GpioPort::D => &mut *(0x40011400 as *mut stm32f103::gpioa::RegisterBlock),
            GpioPort::E => &mut *(0x40011800 as *mut stm32f103::gpioa::RegisterBlock),
            GpioPort::F => &mut *(0x40011C00 as *mut stm32f103::gpioa::RegisterBlock),
            GpioPort::G => &mut *(0x40012000 as *mut stm32f103::gpioa::RegisterBlock),
        }
    }
    
    /// 启用端口时钟
    unsafe fn enable_clock(&self) {
        let rcc = &mut *(0x40021000 as *mut stm32f103::rcc::RegisterBlock);
        match self.port {
            GpioPort::A => {
                let mut value = rcc.apb2enr().read().bits();
                value |= (1 << 2);
                rcc.apb2enr().write(|w: &mut stm32f103::rcc::apb2enr::W| unsafe { w.bits(value) });
            },
            GpioPort::B => {
                let mut value = rcc.apb2enr().read().bits();
                value |= (1 << 3);
                rcc.apb2enr().write(|w: &mut stm32f103::rcc::apb2enr::W| unsafe { w.bits(value) });
            },
            GpioPort::C => {
                let mut value = rcc.apb2enr().read().bits();
                value |= (1 << 4);
                rcc.apb2enr().write(|w: &mut stm32f103::rcc::apb2enr::W| unsafe { w.bits(value) });
            },
            GpioPort::D => {
                let mut value = rcc.apb2enr().read().bits();
                value |= (1 << 5);
                rcc.apb2enr().write(|w: &mut stm32f103::rcc::apb2enr::W| unsafe { w.bits(value) });
            },
            GpioPort::E => {
                let mut value = rcc.apb2enr().read().bits();
                value |= (1 << 6);
                rcc.apb2enr().write(|w: &mut stm32f103::rcc::apb2enr::W| unsafe { w.bits(value) });
            },
            GpioPort::F => {
                let mut value = rcc.apb2enr().read().bits();
                value |= (1 << 7);
                rcc.apb2enr().write(|w: &mut stm32f103::rcc::apb2enr::W| unsafe { w.bits(value) });
            },
            GpioPort::G => {
                let mut value = rcc.apb2enr().read().bits();
                value |= (1 << 8);
                rcc.apb2enr().write(|w: &mut stm32f103::rcc::apb2enr::W| unsafe { w.bits(value) });
            },
        }
    }
    
    /// 配置GPIO引脚为指定模式和速度
    pub unsafe fn into_mode(&self, mode: GpioMode, speed: GpioSpeed) {
        self.enable_clock();
        
        let port = self.get_port();
        
        // 根据引脚号选择CRL或CRH寄存器
        if self.pin < 8 {
            let mut value = port.crl().read().bits();
            let pin_pos = self.pin % 8;
            let pin_mask = 0x0F << (pin_pos * 4);
            
            // 根据速度设置MODE位
            let mode_bits = match speed {
                GpioSpeed::Speed10MHz => 0b01,
                GpioSpeed::Speed2MHz => 0b10,
                GpioSpeed::Speed50MHz => 0b11,
            };
            
            // 根据模式设置配置值
            let config = match mode {
                // 输入模式
                GpioMode::InputFloating => 0b0100,    // CNF=01, MODE=00
                GpioMode::InputPullUp => 0b1000,      // CNF=10, MODE=00
                GpioMode::InputPullDown => 0b1000,    // CNF=10, MODE=00
                GpioMode::Analog => 0b0000,           // CNF=00, MODE=00
                
                // 输出模式
                GpioMode::OutputPushPull => 0b0000 | mode_bits,   // CNF=00, MODE=xx
                GpioMode::OutputOpenDrain => 0b0100 | mode_bits,  // CNF=01, MODE=xx
                
                // 复用功能模式
                GpioMode::AlternatePushPull => 0b1000 | mode_bits, // CNF=10, MODE=xx
                GpioMode::AlternateOpenDrain => 0b1100 | mode_bits, // CNF=11, MODE=xx
            };
            
            // 设置配置寄存器
            value = (value & !pin_mask) | (config << (pin_pos * 4));
            port.crl().write(|w: &mut stm32f103::gpioa::crl::W| unsafe { w.bits(value) });
        } else {
            let mut value = port.crh().read().bits();
            let pin_pos = self.pin % 8;
            let pin_mask = 0x0F << (pin_pos * 4);
            
            // 根据速度设置MODE位
            let mode_bits = match speed {
                GpioSpeed::Speed10MHz => 0b01,
                GpioSpeed::Speed2MHz => 0b10,
                GpioSpeed::Speed50MHz => 0b11,
            };
            
            // 根据模式设置配置值
            let config = match mode {
                // 输入模式
                GpioMode::InputFloating => 0b0100,    // CNF=01, MODE=00
                GpioMode::InputPullUp => 0b1000,      // CNF=10, MODE=00
                GpioMode::InputPullDown => 0b1000,    // CNF=10, MODE=00
                GpioMode::Analog => 0b0000,           // CNF=00, MODE=00
                
                // 输出模式
                GpioMode::OutputPushPull => 0b0000 | mode_bits,   // CNF=00, MODE=xx
                GpioMode::OutputOpenDrain => 0b0100 | mode_bits,  // CNF=01, MODE=xx
                
                // 复用功能模式
                GpioMode::AlternatePushPull => 0b1000 | mode_bits, // CNF=10, MODE=xx
                GpioMode::AlternateOpenDrain => 0b1100 | mode_bits, // CNF=11, MODE=xx
            };
            
            // 设置配置寄存器
            value = (value & !pin_mask) | (config << (pin_pos * 4));
            port.crh().write(|w: &mut stm32f103::gpioa::crh::W| unsafe { w.bits(value) });
        }
        
        // 处理上拉/下拉配置
        match mode {
            GpioMode::InputPullUp => {
                let mut value = port.odr().read().bits();
                value |= (1 << self.pin);
                port.odr().write(|w: &mut stm32f103::gpioa::odr::W| unsafe { w.bits(value) });
            },
            GpioMode::InputPullDown => {
                let mut value = port.odr().read().bits();
                value &= !(1 << self.pin);
                port.odr().write(|w: &mut stm32f103::gpioa::odr::W| unsafe { w.bits(value) });
            },
            _ => {},
        }
    }
    
    /// 配置为推挽输出（50MHz）
    pub unsafe fn into_push_pull_output(&self) {
        self.into_mode(GpioMode::OutputPushPull, GpioSpeed::Speed50MHz);
    }
    
    /// 配置为推挽输出
    pub unsafe fn into_push_pull_output_with_speed(&self, speed: GpioSpeed) {
        self.into_mode(GpioMode::OutputPushPull, speed);
    }
    
    /// 配置为开漏输出（50MHz）
    pub unsafe fn into_open_drain_output(&self) {
        self.into_mode(GpioMode::OutputOpenDrain, GpioSpeed::Speed50MHz);
    }
    
    /// 配置为开漏输出
    pub unsafe fn into_open_drain_output_with_speed(&self, speed: GpioSpeed) {
        self.into_mode(GpioMode::OutputOpenDrain, speed);
    }
    
    /// 配置为浮空输入
    pub unsafe fn into_floating_input(&self) {
        self.into_mode(GpioMode::InputFloating, GpioSpeed::Speed50MHz);
    }
    
    /// 配置为上拉输入
    pub unsafe fn into_pull_up_input(&self) {
        self.into_mode(GpioMode::InputPullUp, GpioSpeed::Speed50MHz);
    }
    
    /// 配置为下拉输入
    pub unsafe fn into_pull_down_input(&self) {
        self.into_mode(GpioMode::InputPullDown, GpioSpeed::Speed50MHz);
    }
    
    /// 配置为模拟输入
    pub unsafe fn into_analog_input(&self) {
        self.into_mode(GpioMode::Analog, GpioSpeed::Speed50MHz);
    }
    
    /// 配置为复用推挽输出（50MHz）
    pub unsafe fn into_alternate_push_pull(&self) {
        self.into_mode(GpioMode::AlternatePushPull, GpioSpeed::Speed50MHz);
    }
    
    /// 配置为复用推挽输出
    pub unsafe fn into_alternate_push_pull_with_speed(&self, speed: GpioSpeed) {
        self.into_mode(GpioMode::AlternatePushPull, speed);
    }
    
    /// 配置为复用开漏输出（50MHz）
    pub unsafe fn into_alternate_open_drain(&self) {
        self.into_mode(GpioMode::AlternateOpenDrain, GpioSpeed::Speed50MHz);
    }
    
    /// 配置为复用开漏输出
    pub unsafe fn into_alternate_open_drain_with_speed(&self, speed: GpioSpeed) {
        self.into_mode(GpioMode::AlternateOpenDrain, speed);
    }
    
    /// 设置引脚为高电平
    pub unsafe fn set_high(&self) {
        let port = self.get_port();
        port.bsrr().write(|w: &mut stm32f103::gpioa::bsrr::W| unsafe { w.bits(1 << self.pin) });
    }
    
    /// 设置引脚为低电平
    pub unsafe fn set_low(&self) {
        let port = self.get_port();
        port.brr().write(|w: &mut stm32f103::gpioa::brr::W| unsafe { w.bits(1 << self.pin) });
    }
    
    /// 切换引脚状态
    pub unsafe fn toggle(&self) {
        let port = self.get_port();
        let current = port.odr().read().bits();
        port.odr().write(|w: &mut stm32f103::gpioa::odr::W| unsafe { w.bits(current ^ (1 << self.pin)) });
    }
    
    /// 获取引脚输入状态（高电平返回true）
    pub unsafe fn is_high(&self) -> bool {
        let port = self.get_port();
        (port.idr().read().bits() & (1 << self.pin)) != 0
    }
    
    /// 获取引脚输入状态（低电平返回true）
    pub unsafe fn is_low(&self) -> bool {
        !self.is_high()
    }
    
    /// 获取引脚输入值（0或1）
    pub unsafe fn get_input(&self) -> u8 {
        if self.is_high() {
            1
        } else {
            0
        }
    }
    
    /// 获取引脚输出状态（高电平返回true）
    pub unsafe fn get_output(&self) -> bool {
        let port = self.get_port();
        (port.odr().read().bits() & (1 << self.pin)) != 0
    }
    
    /// 设置引脚的输出状态
    pub unsafe fn write_bit(&self, bit_val: bool) {
        let port = self.get_port();
        if bit_val {
            port.bsrr().write(|w| unsafe { w.bits(1 << self.pin) });
        } else {
            port.brr().write(|w| unsafe { w.bits(1 << self.pin) });
        }
    }
}

/// 预定义的GPIO引脚常量
pub const PA0: GpioPin = GpioPin::new(GpioPort::A, 0);
pub const PA1: GpioPin = GpioPin::new(GpioPort::A, 1);
pub const PA2: GpioPin = GpioPin::new(GpioPort::A, 2);
pub const PA3: GpioPin = GpioPin::new(GpioPort::A, 3);
pub const PA4: GpioPin = GpioPin::new(GpioPort::A, 4);
pub const PA5: GpioPin = GpioPin::new(GpioPort::A, 5);
pub const PA6: GpioPin = GpioPin::new(GpioPort::A, 6);
pub const PA7: GpioPin = GpioPin::new(GpioPort::A, 7);
pub const PA8: GpioPin = GpioPin::new(GpioPort::A, 8);
pub const PA9: GpioPin = GpioPin::new(GpioPort::A, 9);
pub const PA10: GpioPin = GpioPin::new(GpioPort::A, 10);
pub const PA11: GpioPin = GpioPin::new(GpioPort::A, 11);
pub const PA12: GpioPin = GpioPin::new(GpioPort::A, 12);
pub const PA13: GpioPin = GpioPin::new(GpioPort::A, 13);
pub const PA14: GpioPin = GpioPin::new(GpioPort::A, 14);
pub const PA15: GpioPin = GpioPin::new(GpioPort::A, 15);

pub const PB0: GpioPin = GpioPin::new(GpioPort::B, 0);
pub const PB1: GpioPin = GpioPin::new(GpioPort::B, 1);
pub const PB2: GpioPin = GpioPin::new(GpioPort::B, 2);
pub const PB3: GpioPin = GpioPin::new(GpioPort::B, 3);
pub const PB4: GpioPin = GpioPin::new(GpioPort::B, 4);
pub const PB5: GpioPin = GpioPin::new(GpioPort::B, 5);
pub const PB6: GpioPin = GpioPin::new(GpioPort::B, 6);
pub const PB7: GpioPin = GpioPin::new(GpioPort::B, 7);
pub const PB8: GpioPin = GpioPin::new(GpioPort::B, 8);
pub const PB9: GpioPin = GpioPin::new(GpioPort::B, 9);
pub const PB10: GpioPin = GpioPin::new(GpioPort::B, 10);
pub const PB11: GpioPin = GpioPin::new(GpioPort::B, 11);
pub const PB12: GpioPin = GpioPin::new(GpioPort::B, 12);
pub const PB13: GpioPin = GpioPin::new(GpioPort::B, 13);
pub const PB14: GpioPin = GpioPin::new(GpioPort::B, 14);
pub const PB15: GpioPin = GpioPin::new(GpioPort::B, 15);

pub const PC0: GpioPin = GpioPin::new(GpioPort::C, 0);
pub const PC1: GpioPin = GpioPin::new(GpioPort::C, 1);
pub const PC2: GpioPin = GpioPin::new(GpioPort::C, 2);
pub const PC3: GpioPin = GpioPin::new(GpioPort::C, 3);
pub const PC4: GpioPin = GpioPin::new(GpioPort::C, 4);
pub const PC5: GpioPin = GpioPin::new(GpioPort::C, 5);
pub const PC6: GpioPin = GpioPin::new(GpioPort::C, 6);
pub const PC7: GpioPin = GpioPin::new(GpioPort::C, 7);
pub const PC8: GpioPin = GpioPin::new(GpioPort::C, 8);
pub const PC9: GpioPin = GpioPin::new(GpioPort::C, 9);
pub const PC10: GpioPin = GpioPin::new(GpioPort::C, 10);
pub const PC11: GpioPin = GpioPin::new(GpioPort::C, 11);
pub const PC12: GpioPin = GpioPin::new(GpioPort::C, 12);
pub const PC13: GpioPin = GpioPin::new(GpioPort::C, 13);
pub const PC14: GpioPin = GpioPin::new(GpioPort::C, 14);
pub const PC15: GpioPin = GpioPin::new(GpioPort::C, 15);