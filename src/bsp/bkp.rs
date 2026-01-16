//! BKP（备份寄存器）模块
//! 提供备份寄存器的封装和操作，用于在系统复位时保存重要数据

// 屏蔽未使用代码警告
#![allow(unused)]

// 使用内部生成的设备驱动库
use library::*;

/// BKP错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BkpError {
    /// 无效的寄存器编号
    InvalidRegisterNumber,
    /// 无效的校准值
    InvalidCalibrationValue,
    /// 初始化失败
    InitializationFailed,
    /// 访问被拒绝
    AccessDenied,
}

/// BKP状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BkpStatus {
    /// BKP准备就绪
    Ready,
    /// BKP正在初始化
    Initializing,
    /// BKP出现错误
    Error,
    /// 访问被拒绝
    AccessDenied,
}

/// BKP数据寄存器编号枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BkpDataRegister {
    /// 数据寄存器1
    DR1 = 1,
    /// 数据寄存器2
    DR2 = 2,
    /// 数据寄存器3
    DR3 = 3,
    /// 数据寄存器4
    DR4 = 4,
    /// 数据寄存器5
    DR5 = 5,
    /// 数据寄存器6
    DR6 = 6,
    /// 数据寄存器7
    DR7 = 7,
    /// 数据寄存器8
    DR8 = 8,
    /// 数据寄存器9
    DR9 = 9,
    /// 数据寄存器10
    DR10 = 10,
}

/// BKP结构体
#[derive(Debug, Clone, Copy)]
pub struct Bkp;

impl Bkp {
    /// 创建新的BKP实例
    pub const fn new() -> Self {
        Self
    }
    
    /// 获取BKP寄存器块的不可变引用
    pub unsafe fn bkp_reg(&self) -> &'static bkp::RegisterBlock {
        &*(0x40006C00 as *const bkp::RegisterBlock)
    }
    
    /// 获取BKP寄存器块的可变引用
    pub unsafe fn bkp_reg_mut(&self) -> &'static mut bkp::RegisterBlock {
        &mut *(0x40006C00 as *mut bkp::RegisterBlock)
    }
    
    /// 获取RCC寄存器块的可变引用
    pub unsafe fn rcc_reg_mut(&self) -> &'static mut rcc::RegisterBlock {
        &mut *(0x40021000 as *mut rcc::RegisterBlock)
    }
    
    /// 获取PWR寄存器块的可变引用
    pub unsafe fn pwr_reg_mut(&self) -> &'static mut pwr::RegisterBlock {
        &mut *(0x40007000 as *mut pwr::RegisterBlock)
    }
    
    /// 初始化BKP
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// - 初始化后，BKP将可以访问备份域
    pub unsafe fn init(&self) -> Result<(), BkpError> {
        let rcc = self.rcc_reg_mut();
        let pwr = self.pwr_reg_mut();
        
        // 启用PWR和BKP时钟
        rcc.apb1enr().modify(|_, w| w
            .pwren().set_bit()
            .bkpen().set_bit()
        );
        
        // 使能对备份域的访问
        pwr.cr().modify(|_, w| w
            .dbp().set_bit()
        );
        
        Ok(())
    }
    
    /// 写入备份数据寄存器
    /// 
    /// # 安全
    /// - 调用者必须确保BKP已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `register`：数据寄存器编号
    /// - `value`：要写入的值
    pub unsafe fn write_data_register(&self, register: BkpDataRegister, value: u16) -> Result<(), BkpError> {
        let bkp = self.bkp_reg_mut();
        
        // 根据寄存器编号选择对应的寄存器写入
        match register {
            BkpDataRegister::DR1 => bkp.dr1().write(|w| w.d1().bits(value)),
            BkpDataRegister::DR2 => bkp.dr2().write(|w| w.d2().bits(value)),
            BkpDataRegister::DR3 => bkp.dr3().write(|w| w.d3().bits(value)),
            BkpDataRegister::DR4 => bkp.dr4().write(|w| w.d4().bits(value)),
            BkpDataRegister::DR5 => bkp.dr5().write(|w| w.d5().bits(value)),
            BkpDataRegister::DR6 => bkp.dr6().write(|w| w.d6().bits(value)),
            BkpDataRegister::DR7 => bkp.dr7().write(|w| w.d7().bits(value)),
            BkpDataRegister::DR8 => bkp.dr8().write(|w| w.d8().bits(value)),
            BkpDataRegister::DR9 => bkp.dr9().write(|w| w.d9().bits(value)),
            BkpDataRegister::DR10 => bkp.dr10().write(|w| w.d10().bits(value)),
        };
        
        Ok(())
    }
    
    /// 通过编号写入备份数据寄存器
    /// 
    /// # 安全
    /// - 调用者必须确保BKP已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `register_num`：数据寄存器编号（1-10）
    /// - `value`：要写入的值
    pub unsafe fn write_data_register_by_num(&self, register_num: u8, value: u16) -> Result<(), BkpError> {
        // 检查参数范围
        if register_num < 1 || register_num > 10 {
            return Err(BkpError::InvalidRegisterNumber);
        }
        
        let register = match register_num {
            1 => BkpDataRegister::DR1,
            2 => BkpDataRegister::DR2,
            3 => BkpDataRegister::DR3,
            4 => BkpDataRegister::DR4,
            5 => BkpDataRegister::DR5,
            6 => BkpDataRegister::DR6,
            7 => BkpDataRegister::DR7,
            8 => BkpDataRegister::DR8,
            9 => BkpDataRegister::DR9,
            10 => BkpDataRegister::DR10,
            _ => unreachable!(),
        };
        
        self.write_data_register(register, value)
    }
    
    /// 读取备份数据寄存器
    /// 
    /// # 安全
    /// - 调用者必须确保BKP已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `register`：数据寄存器编号
    /// 
    /// # 返回值
    /// 寄存器中的值
    pub unsafe fn read_data_register(&self, register: BkpDataRegister) -> Result<u16, BkpError> {
        let bkp = self.bkp_reg();
        
        // 根据寄存器编号选择对应的寄存器读取
        let result = match register {
            BkpDataRegister::DR1 => bkp.dr1().read().d1().bits(),
            BkpDataRegister::DR2 => bkp.dr2().read().d2().bits(),
            BkpDataRegister::DR3 => bkp.dr3().read().d3().bits(),
            BkpDataRegister::DR4 => bkp.dr4().read().d4().bits(),
            BkpDataRegister::DR5 => bkp.dr5().read().d5().bits(),
            BkpDataRegister::DR6 => bkp.dr6().read().d6().bits(),
            BkpDataRegister::DR7 => bkp.dr7().read().d7().bits(),
            BkpDataRegister::DR8 => bkp.dr8().read().d8().bits(),
            BkpDataRegister::DR9 => bkp.dr9().read().d9().bits(),
            BkpDataRegister::DR10 => bkp.dr10().read().d10().bits(),
        };
        
        Ok(result)
    }
    
    /// 通过编号读取备份数据寄存器
    /// 
    /// # 安全
    /// - 调用者必须确保BKP已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `register_num`：数据寄存器编号（1-10）
    /// 
    /// # 返回值
    /// 寄存器中的值
    pub unsafe fn read_data_register_by_num(&self, register_num: u8) -> Result<u16, BkpError> {
        // 检查参数范围
        if register_num < 1 || register_num > 10 {
            return Err(BkpError::InvalidRegisterNumber);
        }
        
        let register = match register_num {
            1 => BkpDataRegister::DR1,
            2 => BkpDataRegister::DR2,
            3 => BkpDataRegister::DR3,
            4 => BkpDataRegister::DR4,
            5 => BkpDataRegister::DR5,
            6 => BkpDataRegister::DR6,
            7 => BkpDataRegister::DR7,
            8 => BkpDataRegister::DR8,
            9 => BkpDataRegister::DR9,
            10 => BkpDataRegister::DR10,
            _ => unreachable!(),
        };
        
        self.read_data_register(register)
    }
    
    /// 设置RTC校准值
    /// 
    /// # 安全
    /// - 调用者必须确保BKP已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 参数
    /// - `calibration`：校准值，范围：0-127
    pub unsafe fn set_rtc_calibration(&self, calibration: u8) -> Result<(), BkpError> {
        // 检查参数范围
        if calibration > 0x7F {
            return Err(BkpError::InvalidCalibrationValue);
        }
        
        let bkp = self.bkp_reg_mut();
        bkp.rtccr().write(|w| w
            .cal().bits(calibration)
        );
        
        Ok(())
    }
    
    /// 获取RTC校准值
    /// 
    /// # 安全
    /// - 调用者必须确保BKP已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// 当前的RTC校准值
    pub unsafe fn get_rtc_calibration(&self) -> Result<u8, BkpError> {
        let bkp = self.bkp_reg();
        let cal_value = bkp.rtccr().read().cal().bits();
        
        Ok(cal_value)
    }
    
    /// 启用RTC输出
    /// 
    /// # 安全
    /// - 调用者必须确保BKP已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    pub unsafe fn enable_rtc_output(&self) -> Result<(), BkpError> {
        let bkp = self.bkp_reg_mut();
        bkp.cr().modify(|_, w| w
            .rtco().set_bit()
        );
        
        Ok(())
    }
    
    /// 禁用RTC输出
    /// 
    /// # 安全
    /// - 调用者必须确保BKP已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    pub unsafe fn disable_rtc_output(&self) -> Result<(), BkpError> {
        let bkp = self.bkp_reg_mut();
        bkp.cr().modify(|_, w| w
            .rtco().clear_bit()
        );
        
        Ok(())
    }
    
    /// 检查RTC输出是否启用
    /// 
    /// # 安全
    /// - 调用者必须确保BKP已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// RTC输出是否启用
    pub unsafe fn is_rtc_output_enabled(&self) -> Result<bool, BkpError> {
        let bkp = self.bkp_reg();
        let is_enabled = bkp.cr().read().rtco().bit_is_set();
        
        Ok(is_enabled)
    }
    
    /// 检查侵入检测标志
    /// 
    /// # 安全
    /// - 调用者必须确保BKP已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// 侵入检测标志是否被设置
    pub unsafe fn get_tamper_flag(&self) -> Result<bool, BkpError> {
        let bkp = self.bkp_reg();
        let flag = bkp.csr().read().tampf().bit_is_set();
        
        Ok(flag)
    }
    
    /// 清除侵入检测标志
    /// 
    /// # 安全
    /// - 调用者必须确保BKP已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    pub unsafe fn clear_tamper_flag(&self) -> Result<(), BkpError> {
        let bkp = self.bkp_reg_mut();
        bkp.csr().write(|w| w
            .ctampf().set_bit()
        );
        
        Ok(())
    }
    
    /// 启用侵入检测中断
    /// 
    /// # 安全
    /// - 调用者必须确保BKP已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    pub unsafe fn enable_tamper_interrupt(&self) -> Result<(), BkpError> {
        let bkp = self.bkp_reg_mut();
        bkp.csr().modify(|_, w| w
            .tpie().set_bit()
        );
        
        Ok(())
    }
    
    /// 禁用侵入检测中断
    /// 
    /// # 安全
    /// - 调用者必须确保BKP已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    pub unsafe fn disable_tamper_interrupt(&self) -> Result<(), BkpError> {
        let bkp = self.bkp_reg_mut();
        bkp.csr().modify(|_, w| w
            .tpie().clear_bit()
        );
        
        Ok(())
    }
    
    /// 检查侵入检测中断是否启用
    /// 
    /// # 安全
    /// - 调用者必须确保BKP已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// 侵入检测中断是否启用
    pub unsafe fn is_tamper_interrupt_enabled(&self) -> Result<bool, BkpError> {
        let bkp = self.bkp_reg();
        let is_enabled = bkp.csr().read().tpie().bit_is_set();
        
        Ok(is_enabled)
    }
    
    /// 启用侵入检测引脚滤波
    /// 
    /// # 安全
    /// - 调用者必须确保BKP已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    pub unsafe fn enable_tamper_filter(&self) -> Result<(), BkpError> {
        let bkp = self.bkp_reg_mut();
        bkp.cr().modify(|_, w| w
            .tampflt().set_bit()
        );
        
        Ok(())
    }
    
    /// 禁用侵入检测引脚滤波
    /// 
    /// # 安全
    /// - 调用者必须确保BKP已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    pub unsafe fn disable_tamper_filter(&self) -> Result<(), BkpError> {
        let bkp = self.bkp_reg_mut();
        bkp.cr().modify(|_, w| w
            .tampflt().clear_bit()
        );
        
        Ok(())
    }
    
    /// 检查侵入检测引脚滤波是否启用
    /// 
    /// # 安全
    /// - 调用者必须确保BKP已经初始化
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// 侵入检测引脚滤波是否启用
    pub unsafe fn is_tamper_filter_enabled(&self) -> Result<bool, BkpError> {
        let bkp = self.bkp_reg();
        let is_enabled = bkp.cr().read().tampflt().bit_is_set();
        
        Ok(is_enabled)
    }
    
    /// 获取BKP状态
    /// 
    /// # 安全
    /// - 调用者必须确保在正确的上下文中调用此函数
    /// 
    /// # 返回值
    /// BKP当前状态
    pub unsafe fn get_status(&self) -> BkpStatus {
        let rcc = self.rcc_reg_mut();
        let pwr = self.pwr_reg_mut();
        
        // 检查时钟是否启用
        let rcc_apb1enr = rcc.apb1enr().read();
        let pwr_enabled = rcc_apb1enr.pwren().bit_is_set();
        let bkp_enabled = rcc_apb1enr.bkpen().bit_is_set();
        
        if !pwr_enabled || !bkp_enabled {
            return BkpStatus::Initializing;
        }
        
        // 检查备份域访问是否启用
        let pwr_cr = pwr.cr().read();
        let dbp_enabled = pwr_cr.dbp().bit_is_set();
        
        if !dbp_enabled {
            return BkpStatus::AccessDenied;
        }
        
        BkpStatus::Ready
    }
}

/// 预定义的BKP实例
pub const BKP: Bkp = Bkp::new();

/// 测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试BKP初始化
    #[test]
    fn test_bkp_init() {
        let bkp = Bkp::new();
        
        // 初始化BKP
        unsafe {
            let result = bkp.init();
            assert!(result.is_ok(), "BKP初始化失败");
        }
        
        // 检查状态
        unsafe {
            let status = bkp.get_status();
            assert_eq!(status, BkpStatus::Ready, "BKP状态错误");
        }
    }
    
    /// 测试BKP数据寄存器读写
    #[test]
    fn test_bkp_data_register() {
        let bkp = Bkp::new();
        
        // 初始化BKP
        unsafe {
            let result = bkp.init();
            assert!(result.is_ok(), "BKP初始化失败");
        }
        
        // 测试写入和读取数据寄存器
        let test_value: u16 = 0x1234;
        unsafe {
            // 使用枚举写入
            let write_result = bkp.write_data_register(BkpDataRegister::DR1, test_value);
            assert!(write_result.is_ok(), "写入数据寄存器失败");
            
            // 使用枚举读取
            let read_result = bkp.read_data_register(BkpDataRegister::DR1);
            assert!(read_result.is_ok(), "读取数据寄存器失败");
            assert_eq!(read_result.unwrap(), test_value, "读取值与写入值不匹配");
        }
        
        // 测试通过编号读写
        let test_value2: u16 = 0x5678;
        unsafe {
            let write_result = bkp.write_data_register_by_num(2, test_value2);
            assert!(write_result.is_ok(), "通过编号写入数据寄存器失败");
            
            let read_result = bkp.read_data_register_by_num(2);
            assert!(read_result.is_ok(), "通过编号读取数据寄存器失败");
            assert_eq!(read_result.unwrap(), test_value2, "通过编号读取值与写入值不匹配");
        }
    }
    
    /// 测试BKP RTC相关功能
    #[test]
    fn test_bkp_rtc_functions() {
        let bkp = Bkp::new();
        
        // 初始化BKP
        unsafe {
            let result = bkp.init();
            assert!(result.is_ok(), "BKP初始化失败");
        }
        
        // 测试RTC校准值设置和获取
        let test_cal_value: u8 = 0x40;
        unsafe {
            let write_result = bkp.set_rtc_calibration(test_cal_value);
            assert!(write_result.is_ok(), "设置RTC校准值失败");
            
            let read_result = bkp.get_rtc_calibration();
            assert!(read_result.is_ok(), "获取RTC校准值失败");
            assert_eq!(read_result.unwrap(), test_cal_value, "RTC校准值读取错误");
        }
        
        // 测试RTC输出启用和禁用
        unsafe {
            // 启用RTC输出
            let enable_result = bkp.enable_rtc_output();
            assert!(enable_result.is_ok(), "启用RTC输出失败");
            
            // 检查是否启用
            let is_enabled = bkp.is_rtc_output_enabled();
            assert!(is_enabled.is_ok(), "检查RTC输出状态失败");
            assert!(is_enabled.unwrap(), "RTC输出应该已启用");
            
            // 禁用RTC输出
            let disable_result = bkp.disable_rtc_output();
            assert!(disable_result.is_ok(), "禁用RTC输出失败");
            
            // 检查是否禁用
            let is_disabled = bkp.is_rtc_output_enabled();
            assert!(is_disabled.is_ok(), "检查RTC输出状态失败");
            assert!(!is_disabled.unwrap(), "RTC输出应该已禁用");
        }
    }
    
    /// 测试BKP侵入检测功能
    #[test]
    fn test_bkp_tamper_functions() {
        let bkp = Bkp::new();
        
        // 初始化BKP
        unsafe {
            let result = bkp.init();
            assert!(result.is_ok(), "BKP初始化失败");
        }
        
        // 测试侵入检测中断启用和禁用
        unsafe {
            // 启用侵入检测中断
            let enable_result = bkp.enable_tamper_interrupt();
            assert!(enable_result.is_ok(), "启用侵入检测中断失败");
            
            // 检查是否启用
            let is_enabled = bkp.is_tamper_interrupt_enabled();
            assert!(is_enabled.is_ok(), "检查侵入检测中断状态失败");
            assert!(is_enabled.unwrap(), "侵入检测中断应该已启用");
            
            // 禁用侵入检测中断
            let disable_result = bkp.disable_tamper_interrupt();
            assert!(disable_result.is_ok(), "禁用侵入检测中断失败");
            
            // 检查是否禁用
            let is_disabled = bkp.is_tamper_interrupt_enabled();
            assert!(is_disabled.is_ok(), "检查侵入检测中断状态失败");
            assert!(!is_disabled.unwrap(), "侵入检测中断应该已禁用");
        }
        
        // 测试侵入检测标志清除
        unsafe {
            // 清除侵入检测标志
            let clear_result = bkp.clear_tamper_flag();
            assert!(clear_result.is_ok(), "清除侵入检测标志失败");
            
            // 检查标志是否已清除
            let flag = bkp.get_tamper_flag();
            assert!(flag.is_ok(), "获取侵入检测标志失败");
            assert!(!flag.unwrap(), "侵入检测标志应该已清除");
        }
    }
    
    /// 测试BKP滤波功能
    #[test]
    fn test_bkp_filter_functions() {
        let bkp = Bkp::new();
        
        // 初始化BKP
        unsafe {
            let result = bkp.init();
            assert!(result.is_ok(), "BKP初始化失败");
        }
        
        // 测试侵入检测引脚滤波启用和禁用
        unsafe {
            // 启用滤波
            let enable_result = bkp.enable_tamper_filter();
            assert!(enable_result.is_ok(), "启用侵入检测滤波失败");
            
            // 检查是否启用
            let is_enabled = bkp.is_tamper_filter_enabled();
            assert!(is_enabled.is_ok(), "检查侵入检测滤波状态失败");
            assert!(is_enabled.unwrap(), "侵入检测滤波应该已启用");
            
            // 禁用滤波
            let disable_result = bkp.disable_tamper_filter();
            assert!(disable_result.is_ok(), "禁用侵入检测滤波失败");
            
            // 检查是否禁用
            let is_disabled = bkp.is_tamper_filter_enabled();
            assert!(is_disabled.is_ok(), "检查侵入检测滤波状态失败");
            assert!(!is_disabled.unwrap(), "侵入检测滤波应该已禁用");
        }
    }
    
    /// 测试BKP错误处理
    #[test]
    fn test_bkp_error_handling() {
        let bkp = Bkp::new();
        
        // 初始化BKP
        unsafe {
            let result = bkp.init();
            assert!(result.is_ok(), "BKP初始化失败");
        }
        
        // 测试无效的寄存器编号
        unsafe {
            let result = bkp.write_data_register_by_num(0, 0x1234);
            assert!(result.is_err(), "应该拒绝无效的寄存器编号");
            assert_eq!(result.unwrap_err(), BkpError::InvalidRegisterNumber, "错误类型不匹配");
            
            let result = bkp.write_data_register_by_num(11, 0x1234);
            assert!(result.is_err(), "应该拒绝无效的寄存器编号");
            assert_eq!(result.unwrap_err(), BkpError::InvalidRegisterNumber, "错误类型不匹配");
        }
        
        // 测试无效的校准值
        unsafe {
            let result = bkp.set_rtc_calibration(0x80);
            assert!(result.is_err(), "应该拒绝无效的校准值");
            assert_eq!(result.unwrap_err(), BkpError::InvalidCalibrationValue, "错误类型不匹配");
        }
    }
}
