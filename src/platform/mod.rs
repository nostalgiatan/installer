// SeeSea Self-Contained Installer - Platform Module
// 模块名称: platform
// 职责范围: 提供平台特定功能的抽象和实现
// 期望实现计划: 
// 1. 定义平台抽象接口
// 2. 实现Windows平台特定功能
// 3. 实现Linux平台特定功能
// 4. 实现macOS平台特定功能
// 5. 实现平台检测和实例化
// 已实现功能: 平台抽象接口定义、平台检测
// 使用依赖: config, anyhow, log, std::path
// 主要接口: PlatformImpl, Platform trait
// 注意事项: 支持Windows、Linux、macOS平台，使用条件编译

use crate::config::{Config, InstallOptions};
use anyhow::Result;
use log::debug;
use std::path::Path;

// 平台特定实现
#[cfg(windows)]
mod windows;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

/// 平台抽象接口
trait Platform {
    /// 获取平台特定安装选项
    fn get_install_options(&self, config: &Config) -> Result<InstallOptions>;
    
    /// 检查系统要求
    fn check_system_requirements(&self, config: &Config) -> Result<()>;
    
    /// 创建桌面快捷方式
    fn create_desktop_shortcut(&self, config: &Config, install_dir: &Path) -> Result<()>;
    
    /// 创建开始菜单快捷方式
    fn create_start_menu_shortcut(&self, config: &Config, install_dir: &Path) -> Result<()>;
    
    /// 添加到PATH环境变量
    fn add_to_path(&self, install_dir: &Path) -> Result<()>;
    
    /// 从PATH环境变量中移除
    fn remove_from_path(&self, install_dir: &Path) -> Result<()>;
    
    /// 创建卸载程序
    fn create_uninstaller(&self, config: &Config, install_dir: &Path) -> Result<()>;
    
    /// 删除快捷方式
    fn remove_shortcuts(&self, config: &Config) -> Result<()>;
    
    /// 删除卸载程序
    fn remove_uninstaller(&self, config: &Config) -> Result<()>;
}

/// 平台特定实现的包装器
pub enum PlatformImpl {
    /// Windows平台实现
    #[cfg(windows)]
    Windows(windows::WindowsImpl),
    
    /// Linux平台实现
    #[cfg(target_os = "linux")]
    Linux(linux::LinuxImpl),
    
    /// macOS平台实现
    #[cfg(target_os = "macos")]
    MacOS(macos::MacOSImpl),
}

impl PlatformImpl {
    /// 创建新的平台特定实现实例
    pub fn new() -> Result<Self> {
        debug!("Creating platform-specific implementation");
        
        #[cfg(windows)]
        let impl_ = Self::Windows(windows::WindowsImpl::new()?);
        
        #[cfg(target_os = "linux")]
        let impl_ = Self::Linux(linux::LinuxImpl::new()?);
        
        #[cfg(target_os = "macos")]
        let impl_ = Self::MacOS(macos::MacOSImpl::new()?);
        
        debug!("Platform-specific implementation created");
        Ok(impl_)
    }
    
    /// 获取平台特定安装选项
    pub fn get_install_options(&self, config: &Config) -> Result<InstallOptions> {
        match self {
            #[cfg(windows)]
            Self::Windows(impl_) => impl_.get_install_options(config),
            
            #[cfg(target_os = "linux")]
            Self::Linux(impl_) => impl_.get_install_options(config),
            
            #[cfg(target_os = "macos")]
            Self::MacOS(impl_) => impl_.get_install_options(config),
        }
    }
    
    /// 检查系统要求
    pub fn check_system_requirements(&self, config: &Config) -> Result<()> {
        match self {
            #[cfg(windows)]
            Self::Windows(impl_) => impl_.check_system_requirements(config),
            
            #[cfg(target_os = "linux")]
            Self::Linux(impl_) => impl_.check_system_requirements(config),
            
            #[cfg(target_os = "macos")]
            Self::MacOS(impl_) => impl_.check_system_requirements(config),
        }
    }
    
    /// 创建桌面快捷方式
    pub fn create_desktop_shortcut(&self, config: &Config, install_dir: &Path) -> Result<()> {
        match self {
            #[cfg(windows)]
            Self::Windows(impl_) => impl_.create_desktop_shortcut(config, install_dir),
            
            #[cfg(target_os = "linux")]
            Self::Linux(impl_) => impl_.create_desktop_shortcut(config, install_dir),
            
            #[cfg(target_os = "macos")]
            Self::MacOS(impl_) => impl_.create_desktop_shortcut(config, install_dir),
        }
    }
    
    /// 创建开始菜单快捷方式
    pub fn create_start_menu_shortcut(&self, config: &Config, install_dir: &Path) -> Result<()> {
        match self {
            #[cfg(windows)]
            Self::Windows(impl_) => impl_.create_start_menu_shortcut(config, install_dir),
            
            #[cfg(target_os = "linux")]
            Self::Linux(impl_) => impl_.create_start_menu_shortcut(config, install_dir),
            
            #[cfg(target_os = "macos")]
            Self::MacOS(impl_) => impl_.create_start_menu_shortcut(config, install_dir),
        }
    }
    
    /// 添加到PATH环境变量
    pub fn add_to_path(&self, install_dir: &Path) -> Result<()> {
        match self {
            #[cfg(windows)]
            Self::Windows(impl_) => impl_.add_to_path(install_dir),
            
            #[cfg(target_os = "linux")]
            Self::Linux(impl_) => impl_.add_to_path(install_dir),
            
            #[cfg(target_os = "macos")]
            Self::MacOS(impl_) => impl_.add_to_path(install_dir),
        }
    }
    
    /// 从PATH环境变量中移除
    pub fn remove_from_path(&self, install_dir: &Path) -> Result<()> {
        match self {
            #[cfg(windows)]
            Self::Windows(impl_) => impl_.remove_from_path(install_dir),
            
            #[cfg(target_os = "linux")]
            Self::Linux(impl_) => impl_.remove_from_path(install_dir),
            
            #[cfg(target_os = "macos")]
            Self::MacOS(impl_) => impl_.remove_from_path(install_dir),
        }
    }
    
    /// 创建卸载程序
    pub fn create_uninstaller(&self, config: &Config, install_dir: &Path) -> Result<()> {
        match self {
            #[cfg(windows)]
            Self::Windows(impl_) => impl_.create_uninstaller(config, install_dir),
            
            #[cfg(target_os = "linux")]
            Self::Linux(impl_) => impl_.create_uninstaller(config, install_dir),
            
            #[cfg(target_os = "macos")]
            Self::MacOS(impl_) => impl_.create_uninstaller(config, install_dir),
        }
    }
    
    /// 删除快捷方式
    pub fn remove_shortcuts(&self, config: &Config) -> Result<()> {
        match self {
            #[cfg(windows)]
            Self::Windows(impl_) => impl_.remove_shortcuts(config),
            
            #[cfg(target_os = "linux")]
            Self::Linux(impl_) => impl_.remove_shortcuts(config),
            
            #[cfg(target_os = "macos")]
            Self::MacOS(impl_) => impl_.remove_shortcuts(config),
        }
    }
    
    /// 删除卸载程序
    pub fn remove_uninstaller(&self, config: &Config) -> Result<()> {
        match self {
            #[cfg(windows)]
            Self::Windows(impl_) => impl_.remove_uninstaller(config),
            
            #[cfg(target_os = "linux")]
            Self::Linux(impl_) => impl_.remove_uninstaller(config),
            
            #[cfg(target_os = "macos")]
            Self::MacOS(impl_) => impl_.remove_uninstaller(config),
        }
    }
}
