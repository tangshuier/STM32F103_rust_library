//! DAC（数模转换器）模块
//! 提供数模转换器的封装和操作，用于将数字信号转换为模拟电压输出

// 屏蔽未使用代码警告
#![allow(unused)]

// 使用内部生成的设备驱动库
use library::*;

/// DAC错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DacError {
    /// 初始化失败
    InitializationFailed,
    /// 无效的通道
    InvalidChannel,
    /// 参数无效
    InvalidParameter,
    /// 无效的触发源
    InvalidTriggerSource,
    /// 通道未启用
    ChannelDisabled,
    /// 转换失败
    ConversionFailed,
    /// 无效的值
    InvalidValue,
    /// 未知错误
    UnknownError,
}

/// DAC状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DacStatus {
    /// DAC准备就绪
    Ready,
    /// DAC正在初始化
    Initializing,
    /// DAC出现错误
    Error,
    /// 通道1激活
    Channel1Active,
    /// 通道2激活
    Channel2Active,
    /// 两个通道都激活
    BothChannelsActive,
    /// DAC禁用
    Disabled,
}

/// DAC通道枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DacChannel {
    Channel1 = 0,
    Channel2 = 1,
}

/// DAC触发源枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DacTriggerSource {
    Software = 0,
    Timer2TRGO = 1,
    Timer4TRGO = 2,
    Timer5TRGO = 3,
    Timer6TRGO = 4,
    Timer7TRGO = 5,
    Exti9 = 6,
}

/// DAC数据对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DacDataAlignment {
    /// 右对齐
    Right = 0,
    /// 左对齐
    Left = 1,
}

/// DAC结构体
#[derive(Debug, Clone, Copy)]
pub struct Dac;

impl Dac {
    /// 创建新的DAC实例
    pub const fn new() -> Self {
        Self
    }
    
    /// 获取DAC寄存器块的不可变引用
    pub unsafe fn dac_reg(&self) -> &'static dac::RegisterBlock {
        &*(0x40007400 as *const dac::RegisterBlock)
    }
    
    /// 获取DAC寄存器块的可变引用
    pub unsafe fn dac_reg_mut(&self) -> &'static mut dac::RegisterBlock {
        &mut *(0x40007400 as *mut dac::RegisterBlock)
    }
    
    /// 获取RCC寄存器块的可变引用
    pub unsafe fn rcc_reg_mut(&self) -> &'static mut rcc::RegisterBlock {
        &mut *(0x40021000 as *mut rcc::RegisterBlock)
    }
    
    /// 初始化DAC
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(())：初始化成功
    /// - Err(DacError)：初始化失败
    pub unsafe fn init(&self) -> Result<(), DacError> {
        let rcc = self.rcc_reg_mut();
        
        // 启用DAC时钟
        rcc.apb1enr().modify(|_, w| w
            .dacen().set_bit()
        );
        
        Ok(())
    }
    
    /// 启用DAC通道
    /// 
    /// # 安全
    /// - 调用者必须确保DAC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `channel`：要启用的DAC通道
    /// 
    /// # 返回值
    /// - Ok(())：通道启用成功
    /// - Err(DacError)：通道启用失败
    pub unsafe fn enable_channel(&self, channel: DacChannel) -> Result<(), DacError> {
        let dac = self.dac_reg_mut();
        
        match channel {
            DacChannel::Channel1 => {
                dac.cr().modify(|_, w| w
                    .en1().set_bit()
                );
            }
            DacChannel::Channel2 => {
                dac.cr().modify(|_, w| w
                    .en2().set_bit()
                );
            }
        }
        
        Ok(())
    }
    
    /// 禁用DAC通道
    /// 
    /// # 安全
    /// - 调用者必须确保DAC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `channel`：要禁用的DAC通道
    /// 
    /// # 返回值
    /// - Ok(())：通道禁用成功
    /// - Err(DacError)：通道禁用失败
    pub unsafe fn disable_channel(&self, channel: DacChannel) -> Result<(), DacError> {
        let dac = self.dac_reg_mut();
        
        match channel {
            DacChannel::Channel1 => {
                dac.cr().modify(|_, w| w
                    .en1().clear_bit()
                );
            }
            DacChannel::Channel2 => {
                dac.cr().modify(|_, w| w
                    .en2().clear_bit()
                );
            }
        }
        
        Ok(())
    }
    
    /// 启用DAC通道触发
    /// 
    /// # 安全
    /// - 调用者必须确保DAC已经初始化
    /// - 调用者必须确保通道已经启用
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `channel`：要配置的DAC通道
    /// 
    /// # 返回值
    /// - Ok(())：触发启用成功
    /// - Err(DacError)：触发启用失败
    pub unsafe fn enable_trigger(&self, channel: DacChannel) -> Result<(), DacError> {
        let dac = self.dac_reg_mut();
        
        match channel {
            DacChannel::Channel1 => {
                dac.cr().modify(|_, w| w
                    .ten1().set_bit()
                );
            }
            DacChannel::Channel2 => {
                dac.cr().modify(|_, w| w
                    .ten2().set_bit()
                );
            }
        }
        
        Ok(())
    }
    
    /// 禁用DAC通道触发
    /// 
    /// # 安全
    /// - 调用者必须确保DAC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `channel`：要配置的DAC通道
    /// 
    /// # 返回值
    /// - Ok(())：触发禁用成功
    /// - Err(DacError)：触发禁用失败
    pub unsafe fn disable_trigger(&self, channel: DacChannel) -> Result<(), DacError> {
        let dac = self.dac_reg_mut();
        
        match channel {
            DacChannel::Channel1 => {
                dac.cr().modify(|_, w| w
                    .ten1().clear_bit()
                );
            }
            DacChannel::Channel2 => {
                dac.cr().modify(|_, w| w
                    .ten2().clear_bit()
                );
            }
        }
        
        Ok(())
    }
    
    /// 设置DAC通道触发源
    /// 
    /// # 安全
    /// - 调用者必须确保DAC已经初始化
    /// - 调用者必须确保通道已经启用
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `channel`：要配置的DAC通道
    /// - `source`：触发源
    /// 
    /// # 返回值
    /// - Ok(())：触发源设置成功
    /// - Err(DacError)：触发源设置失败
    pub unsafe fn set_trigger_source(&self, channel: DacChannel, source: DacTriggerSource) -> Result<(), DacError> {
        let dac = self.dac_reg_mut();
        
        match channel {
            DacChannel::Channel1 => {
                dac.cr().modify(|_, w| w
                    .tsel1().bits(source as u8)
                );
            }
            DacChannel::Channel2 => {
                dac.cr().modify(|_, w| w
                    .tsel2().bits(source as u8)
                );
            }
        }
        
        Ok(())
    }
    
    /// 启用DAC通道输出缓冲
    /// 
    /// # 安全
    /// - 调用者必须确保DAC已经初始化
    /// - 调用者必须确保通道已经启用
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `channel`：要配置的DAC通道
    /// 
    /// # 返回值
    /// - Ok(())：输出缓冲启用成功
    /// - Err(DacError)：输出缓冲启用失败
    pub unsafe fn enable_output_buffer(&self, channel: DacChannel) -> Result<(), DacError> {
        let dac = self.dac_reg_mut();
        
        match channel {
            DacChannel::Channel1 => {
                dac.cr().modify(|_, w| w
                    .boff1().clear_bit()
                );
            }
            DacChannel::Channel2 => {
                dac.cr().modify(|_, w| w
                    .boff2().clear_bit()
                );
            }
        }
        
        Ok(())
    }
    
    /// 禁用DAC通道输出缓冲
    /// 
    /// # 安全
    /// - 调用者必须确保DAC已经初始化
    /// - 调用者必须确保通道已经启用
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `channel`：要配置的DAC通道
    /// 
    /// # 返回值
    /// - Ok(())：输出缓冲禁用成功
    /// - Err(DacError)：输出缓冲禁用失败
    pub unsafe fn disable_output_buffer(&self, channel: DacChannel) -> Result<(), DacError> {
        let dac = self.dac_reg_mut();
        
        match channel {
            DacChannel::Channel1 => {
                dac.cr().modify(|_, w| w
                    .boff1().set_bit()
                );
            }
            DacChannel::Channel2 => {
                dac.cr().modify(|_, w| w
                    .boff2().set_bit()
                );
            }
        }
        
        Ok(())
    }
    
    /// 软件触发DAC转换
    /// 
    /// # 安全
    /// - 调用者必须确保DAC已经初始化
    /// - 调用者必须确保通道已经启用
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `channel`：要触发的DAC通道
    /// 
    /// # 返回值
    /// - Ok(())：软件触发成功
    /// - Err(DacError)：软件触发失败
    pub unsafe fn software_trigger(&self, channel: DacChannel) -> Result<(), DacError> {
        let dac = self.dac_reg_mut();
        
        match channel {
            DacChannel::Channel1 => {
                dac.swtrigr().write(|w| w
                    .swtrig1().set_bit()
                );
            }
            DacChannel::Channel2 => {
                dac.swtrigr().write(|w| w
                    .swtrig2().set_bit()
                );
            }
        }
        
        Ok(())
    }
    
    /// 设置DAC通道12位右对齐数据
    /// 
    /// # 安全
    /// - 调用者必须确保DAC已经初始化
    /// - 调用者必须确保通道已经启用
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `channel`：要设置的DAC通道
    /// - `value`：12位数据值（0-4095）
    /// 
    /// # 返回值
    /// - Ok(())：数据设置成功
    /// - Err(DacError)：数据设置失败
    pub unsafe fn set_channel_data(&self, channel: DacChannel, value: u16) -> Result<(), DacError> {
        let dac = self.dac_reg_mut();
        let value_clamped = if value > 4095 { 4095 } else { value };
        
        match channel {
            DacChannel::Channel1 => {
                dac.dhr12r1().write(|w| w
                    .dacc1dhr().bits(value_clamped)
                );
            }
            DacChannel::Channel2 => {
                dac.dhr12r2().write(|w| w
                    .dacc2dhr().bits(value_clamped)
                );
            }
        }
        
        Ok(())
    }
    
    /// 设置DAC通道12位左对齐数据
    /// 
    /// # 安全
    /// - 调用者必须确保DAC已经初始化
    /// - 调用者必须确保通道已经启用
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `channel`：要设置的DAC通道
    /// - `value`：12位数据值（0-4095）
    /// 
    /// # 返回值
    /// - Ok(())：数据设置成功
    /// - Err(DacError)：数据设置失败
    pub unsafe fn set_channel_data_left_aligned(&self, channel: DacChannel, value: u16) -> Result<(), DacError> {
        let dac = self.dac_reg_mut();
        let value_clamped = if value > 4095 { 4095 } else { value };
        
        match channel {
            DacChannel::Channel1 => {
                dac.dhr12l1().write(|w| w
                    .dacc1dhr().bits(value_clamped << 4)
                );
            }
            DacChannel::Channel2 => {
                dac.dhr12l2().write(|w| w
                    .dacc2dhr().bits(value_clamped << 4)
                );
            }
        }
        
        Ok(())
    }
    
    /// 设置DAC通道8位右对齐数据
    /// 
    /// # 安全
    /// - 调用者必须确保DAC已经初始化
    /// - 调用者必须确保通道已经启用
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `channel`：要设置的DAC通道
    /// - `value`：8位数据值（0-255）
    /// 
    /// # 返回值
    /// - Ok(())：数据设置成功
    /// - Err(DacError)：数据设置失败
    pub unsafe fn set_channel_data_8bit(&self, channel: DacChannel, value: u8) -> Result<(), DacError> {
        let dac = self.dac_reg_mut();
        
        match channel {
            DacChannel::Channel1 => {
                dac.dhr8r1().write(|w| w
                    .dacc1dhr().bits(value as u8)
                );
            }
            DacChannel::Channel2 => {
                dac.dhr8r2().write(|w| w
                    .dacc2dhr().bits(value as u8)
                );
            }
        }
        
        Ok(())
    }
    
    /// 设置双通道12位右对齐数据
    /// 
    /// # 安全
    /// - 调用者必须确保DAC已经初始化
    /// - 调用者必须确保两个通道都已经启用
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `value1`：通道1的12位数据值（0-4095）
    /// - `value2`：通道2的12位数据值（0-4095）
    /// 
    /// # 返回值
    /// - Ok(())：双通道数据设置成功
    /// - Err(DacError)：双通道数据设置失败
    pub unsafe fn set_dual_channel_data(&self, value1: u16, value2: u16) -> Result<(), DacError> {
        let dac = self.dac_reg_mut();
        let value1_clamped = if value1 > 4095 { 4095 } else { value1 };
        let value2_clamped = if value2 > 4095 { 4095 } else { value2 };
        
        dac.dhr12rd().write(|w| w
            .dacc1dhr().bits(value1_clamped)
            .dacc2dhr().bits(value2_clamped)
        );
        
        Ok(())
    }
    
    /// 获取DAC通道数据输出
    /// 
    /// # 安全
    /// - 调用者必须确保DAC已经初始化
    /// - 调用者必须确保通道已经启用
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `channel`：要获取的DAC通道
    /// 
    /// # 返回值
    /// - Ok(u16)：通道的当前输出值
    /// - Err(DacError)：获取输出值失败
    pub unsafe fn get_channel_output(&self, channel: DacChannel) -> Result<u16, DacError> {
        let dac = self.dac_reg();
        
        let result = match channel {
            DacChannel::Channel1 => {
                dac.dor1().read().dacc1dor().bits()
            }
            DacChannel::Channel2 => {
                dac.dor2().read().dacc2dor().bits()
            }
        };
        
        Ok(result)
    }
    
    /// 获取DAC状态
    /// 
    /// # 安全
    /// - 调用者必须确保DAC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// - Ok(DacStatus)：DAC的当前状态
    /// - Err(DacError)：获取状态失败
    pub unsafe fn get_status(&self) -> Result<DacStatus, DacError> {
        let dac = self.dac_reg();
        let cr = dac.cr().read();
        
        let channel1_enabled = cr.en1().bit_is_set();
        let channel2_enabled = cr.en2().bit_is_set();
        
        match (channel1_enabled, channel2_enabled) {
            (true, true) => Ok(DacStatus::BothChannelsActive),
            (true, false) => Ok(DacStatus::Channel1Active),
            (false, true) => Ok(DacStatus::Channel2Active),
            (false, false) => Ok(DacStatus::Disabled),
        }
    }
    
    /// 检查DAC通道是否启用
    /// 
    /// # 安全
    /// - 调用者必须确保DAC已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `channel`：要检查的DAC通道
    /// 
    /// # 返回值
    /// - Ok(bool)：通道是否启用
    /// - Err(DacError)：检查失败
    pub unsafe fn is_channel_enabled(&self, channel: DacChannel) -> Result<bool, DacError> {
        let dac = self.dac_reg();
        let cr = dac.cr().read();
        
        let result = match channel {
            DacChannel::Channel1 => cr.en1().bit_is_set(),
            DacChannel::Channel2 => cr.en2().bit_is_set(),
        };
        
        Ok(result)
    }
}

/// 预定义的DAC实例
pub const DAC: Dac = Dac::new();

/// 测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试DAC初始化和状态获取
    #[test]
    fn test_dac_init_status() {
        let dac = Dac::new();
        
        // 初始化DAC
        unsafe {
            let init_result = dac.init();
            assert!(init_result.is_ok(), "DAC初始化应该成功");
            
            let status = dac.get_status();
            assert!(status.is_ok(), "获取DAC状态应该成功");
            assert_eq!(status.unwrap(), DacStatus::Disabled, "DAC初始状态应该是Disabled");
        }
    }
    
    /// 测试DAC通道启用/禁用
    #[test]
    fn test_dac_channel_enable_disable() {
        let dac = Dac::new();
        
        // 初始化DAC
        unsafe {
            let init_result = dac.init();
            assert!(init_result.is_ok(), "DAC初始化应该成功");
            
            // 启用通道1
            let enable_result = dac.enable_channel(DacChannel::Channel1);
            assert!(enable_result.is_ok(), "启用DAC通道1应该成功");
            
            // 检查通道1是否启用
            let is_enabled = dac.is_channel_enabled(DacChannel::Channel1);
            assert!(is_enabled.is_ok(), "检查通道1状态应该成功");
            assert!(is_enabled.unwrap(), "通道1应该已启用");
            
            // 获取状态
            let status = dac.get_status();
            assert!(status.is_ok(), "获取DAC状态应该成功");
            assert_eq!(status.unwrap(), DacStatus::Channel1Active, "DAC状态应该是Channel1Active");
            
            // 禁用通道1
            let disable_result = dac.disable_channel(DacChannel::Channel1);
            assert!(disable_result.is_ok(), "禁用DAC通道1应该成功");
            
            // 检查通道1是否禁用
            let is_enabled = dac.is_channel_enabled(DacChannel::Channel1);
            assert!(is_enabled.is_ok(), "检查通道1状态应该成功");
            assert!(!is_enabled.unwrap(), "通道1应该已禁用");
        }
    }
    
    /// 测试DAC数据设置和获取
    #[test]
    fn test_dac_data_set_get() {
        let dac = Dac::new();
        
        // 初始化DAC
        unsafe {
            let init_result = dac.init();
            assert!(init_result.is_ok(), "DAC初始化应该成功");
            
            // 启用通道1
            let enable_result = dac.enable_channel(DacChannel::Channel1);
            assert!(enable_result.is_ok(), "启用DAC通道1应该成功");
            
            // 设置通道1数据
            let set_data_result = dac.set_channel_data(DacChannel::Channel1, 2048);
            assert!(set_data_result.is_ok(), "设置DAC通道1数据应该成功");
            
            // 使用软件触发转换
            let trigger_result = dac.software_trigger(DacChannel::Channel1);
            assert!(trigger_result.is_ok(), "软件触发DAC转换应该成功");
            
            // 获取通道1输出
            let output = dac.get_channel_output(DacChannel::Channel1);
            assert!(output.is_ok(), "获取DAC通道1输出应该成功");
        }
    }
    
    /// 测试DAC触发配置
    #[test]
    fn test_dac_trigger_config() {
        let dac = Dac::new();
        
        // 初始化DAC
        unsafe {
            let init_result = dac.init();
            assert!(init_result.is_ok(), "DAC初始化应该成功");
            
            // 启用通道1
            let enable_result = dac.enable_channel(DacChannel::Channel1);
            assert!(enable_result.is_ok(), "启用DAC通道1应该成功");
            
            // 启用触发
            let enable_trigger_result = dac.enable_trigger(DacChannel::Channel1);
            assert!(enable_trigger_result.is_ok(), "启用DAC通道1触发应该成功");
            
            // 设置触发源
            let set_trigger_source_result = dac.set_trigger_source(DacChannel::Channel1, DacTriggerSource::Software);
            assert!(set_trigger_source_result.is_ok(), "设置DAC通道1触发源应该成功");
            
            // 禁用触发
            let disable_trigger_result = dac.disable_trigger(DacChannel::Channel1);
            assert!(disable_trigger_result.is_ok(), "禁用DAC通道1触发应该成功");
        }
    }
}
