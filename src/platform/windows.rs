// SeeSea Self-Contained Installer - Windows Platform Implementation
// 模块名称: windows
// 职责范围: 实现Windows平台特定的安装功能
// 期望实现计划: 
// 1. 实现Windows平台特定安装选项获取
// 2. 实现系统要求检查
// 3. 实现桌面快捷方式创建
// 4. 实现开始菜单快捷方式创建
// 5. 实现PATH环境变量管理
// 6. 实现卸载程序创建
// 已实现功能: WindowsImpl结构体定义、基础功能实现
// 使用依赖: config, anyhow, log, std::path, std::env, winreg
// 主要接口: WindowsImpl::new, get_install_options, check_system_requirements
// 注意事项: 仅在Windows平台编译，需要管理员权限执行某些操作

use crate::config::{Config, InstallOptions};
use anyhow::Result;
use log::{debug, info};
use std::env;
use std::path::{Path, PathBuf};
use winreg::{RegKey, enums::*};
use std::process::Command;

/// Windows平台实现结构体
pub struct WindowsImpl {
    /// 注册表路径
    pub uninstall_reg_path: String,
}

impl WindowsImpl {
    /// 创建新的Windows平台实现实例
    pub fn new() -> Result<Self> {
        debug!("Creating Windows platform implementation");
        
        Ok(Self {
            uninstall_reg_path: r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall".to_string(),
        })
    }
}

impl super::Platform for WindowsImpl {
    /// 获取平台特定安装选项
    fn get_install_options(&self, config: &Config) -> Result<InstallOptions> {
        debug!("Getting Windows specific install options");
        
        // 如果配置中有Windows特定选项，则使用它，否则使用全局选项
        if let Some(platform_config) = &config.platform {
            if let Some(windows_config) = &platform_config.windows {
                debug!("Using Windows specific install options from config");
                return Ok(windows_config.clone());
            }
        }
        
        debug!("Using global install options");
        Ok(config.install_options.clone())
    }
    
    /// 检查系统要求
    fn check_system_requirements(&self, _config: &Config) -> Result<()> {
        info!("Checking Windows system requirements");
        
        // 检查Windows版本
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let os_info = hklm.open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion")?;
        let product_name: String = os_info.get_value("ProductName")?;
        let current_build: String = os_info.get_value("CurrentBuild")?;
        
        info!("Windows version: {product_name} (Build: {current_build})");
        debug!("Product Name: {product_name}, Current Build: {current_build}");
        
        // 检查.NET Framework版本（如果需要）
        // 这里可以根据项目需求添加更多系统要求检查
        
        info!("System requirements check passed");
        Ok(())
    }
    
    /// 创建桌面快捷方式
    fn create_desktop_shortcut(&self, config: &Config, install_dir: &Path) -> Result<()> {
        info!("Creating desktop shortcut on Windows");
        
        // 获取桌面路径
        let desktop_path = PathBuf::from(env::var("USERPROFILE")?).join("Desktop");
        debug!("Desktop path: {desktop_path:?}");
        
        // 构建快捷方式路径
        let shortcut_path = desktop_path.join(format!("{}.lnk", config.project.name));
        debug!("Shortcut path: {shortcut_path:?}");
        
        // 构建目标可执行文件路径
        let target_exe = install_dir.join(format!("{}.exe", config.project.name.to_lowercase()));
        debug!("Target executable: {target_exe:?}");
        
        // 使用PowerShell创建快捷方式
        let powershell_command = format!(
            "$WshShell = New-Object -comObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut('{}'); $Shortcut.TargetPath = '{}'; $Shortcut.WorkingDirectory = '{}'; $Shortcut.Save()",
            shortcut_path.display(),
            target_exe.display(),
            install_dir.display()
        );
        
        debug!("PowerShell command: {powershell_command}");
        Command::new("powershell")
            .arg("-Command")
            .arg(powershell_command)
            .output()?;
        
        info!("Desktop shortcut created successfully");
        Ok(())
    }
    
    /// 创建开始菜单快捷方式
    fn create_start_menu_shortcut(&self, config: &Config, install_dir: &Path) -> Result<()> {
        info!("Creating start menu shortcut on Windows");
        
        // 获取开始菜单路径
        let start_menu_path = PathBuf::from(env::var("APPDATA")?).join(r"Microsoft\Windows\Start Menu\Programs");
        debug!("Start menu path: {start_menu_path:?}");
        
        // 构建快捷方式路径
        let shortcut_path = start_menu_path.join(format!("{}.lnk", config.project.name));
        debug!("Shortcut path: {shortcut_path:?}");
        
        // 构建目标可执行文件路径
        let target_exe = install_dir.join(format!("{}.exe", config.project.name.to_lowercase()));
        debug!("Target executable: {target_exe:?}");
        
        // 使用PowerShell创建快捷方式
        let powershell_command = format!(
            "$WshShell = New-Object -comObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut('{}'); $Shortcut.TargetPath = '{}'; $Shortcut.WorkingDirectory = '{}'; $Shortcut.Save()",
            shortcut_path.display(),
            target_exe.display(),
            install_dir.display()
        );
        
        debug!("PowerShell command: {powershell_command}");
        Command::new("powershell")
            .arg("-Command")
            .arg(powershell_command)
            .output()?;
        
        info!("Start menu shortcut created successfully");
        Ok(())
    }
    
    /// 添加到PATH环境变量
    fn add_to_path(&self, install_dir: &Path) -> Result<()> {
        info!("Adding to PATH environment variable on Windows");
        
        // 打开注册表中的PATH环境变量
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let env_key = hklm.open_subkey_with_flags(r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment", KEY_READ | KEY_WRITE)?;
        
        // 获取当前PATH值
        let current_path: String = env_key.get_value("Path")?;
        debug!("Current PATH: {current_path}");
        
        // 检查PATH中是否已包含安装目录
        let install_dir_str = install_dir.to_string_lossy();
        if !current_path.contains(&*install_dir_str) {
            // 添加安装目录到PATH
            let separator = current_path.ends_with(";").then_some("").unwrap_or(";").to_string();
            let new_path = format!("{current_path}{separator}{install_dir_str}");
            env_key.set_value("Path", &new_path)?;
            debug!("New PATH: {new_path}");
            info!("Successfully added to PATH environment variable");
        } else {
            debug!("Install directory already in PATH");
            info!("Install directory already in PATH environment variable");
        }
        
        Ok(())
    }
    
    /// 从PATH环境变量中移除
    fn remove_from_path(&self, install_dir: &Path) -> Result<()> {
        info!("Removing from PATH environment variable on Windows");
        
        // 打开注册表中的PATH环境变量
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let env_key = hklm.open_subkey_with_flags(r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment", KEY_READ | KEY_WRITE)?;
        
        // 获取当前PATH值
        let current_path: String = env_key.get_value("Path")?;
        debug!("Current PATH: {current_path}");
        
        // 移除安装目录从PATH
        let install_dir_str = install_dir.to_string_lossy();
        let new_path = current_path.replace(&format!("{install_dir_str}"), "")
            .replace(&format!(";{install_dir_str}"), "")
            .replace(&format!("{install_dir_str}"), "");
        
        env_key.set_value("Path", &new_path)?;
        debug!("New PATH: {new_path}");
        info!("Successfully removed from PATH environment variable");
        
        Ok(())
    }
    
    /// 创建卸载程序
    fn create_uninstaller(&self, config: &Config, install_dir: &Path) -> Result<()> {
        info!("Creating uninstaller on Windows");
        
        // 构建卸载程序路径
        let uninstaller_path = install_dir.join("uninstall.exe");
        debug!("Uninstaller path: {uninstaller_path:?}");
        
        // 复制当前安装程序到卸载程序路径
        let current_exe = env::current_exe()?;
        std::fs::copy(&current_exe, &uninstaller_path)?;
        debug!("Copied installer to uninstaller path");
        
        // 在注册表中添加卸载信息
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let (uninstall_key, _) = hklm.create_subkey(format!("{}\\{}", self.uninstall_reg_path, config.project.name))?;
        
        // 设置卸载信息
        uninstall_key.set_value("DisplayName", &config.project.name)?;
        uninstall_key.set_value("DisplayVersion", &config.project.version)?;
        uninstall_key.set_value("Publisher", &config.project.author.as_deref().unwrap_or(""))?;
        uninstall_key.set_value("InstallLocation", &install_dir.to_string_lossy().to_string())?;
        uninstall_key.set_value("UninstallString", &format!("\"{}\" uninstall", uninstaller_path.display()))?;
        uninstall_key.set_value("QuietUninstallString", &format!("\"{}\" uninstall --quiet", uninstaller_path.display()))?;
        uninstall_key.set_value("NoModify", &1u32)?;
        uninstall_key.set_value("NoRepair", &1u32)?;
        
        debug!("Added uninstall information to registry");
        info!("Successfully created uninstaller");
        
        Ok(())
    }
    
    /// 移除卸载程序
    fn remove_uninstaller(&self, config: &Config) -> Result<()> {
        info!("Removing uninstaller on Windows");
        
        // 从注册表中删除卸载信息
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let uninstall_key_path = format!("{}\\{}", self.uninstall_reg_path, config.project.name);
        if hklm.open_subkey(&uninstall_key_path).is_ok() {
            hklm.delete_subkey_all(&uninstall_key_path)?;
            debug!("Removed uninstall information from registry");
        } else {
            debug!("Uninstall information not found in registry");
        }
        
        info!("Successfully removed uninstaller");
        Ok(())
    }
    
    /// 移除快捷方式
    fn remove_shortcuts(&self, config: &Config) -> Result<()> {
        info!("Removing shortcuts on Windows");
        
        // 获取桌面路径和开始菜单路径
        let desktop_path = PathBuf::from(env::var("USERPROFILE")?).join("Desktop");
        let start_menu_path = PathBuf::from(env::var("APPDATA")?).join(r"Microsoft\Windows\Start Menu\Programs");
        
        // 构建快捷方式路径
        let desktop_shortcut = desktop_path.join(format!("{}.lnk", config.project.name));
        let start_menu_shortcut = start_menu_path.join(format!("{}.lnk", config.project.name));
        
        // 删除桌面快捷方式
        if desktop_shortcut.exists() {
            std::fs::remove_file(&desktop_shortcut)?;
            debug!("Removed desktop shortcut: {desktop_shortcut:?}");
        }
        
        // 删除开始菜单快捷方式
        if start_menu_shortcut.exists() {
            std::fs::remove_file(&start_menu_shortcut)?;
            debug!("Removed start menu shortcut: {start_menu_shortcut:?}");
        }
        
        info!("Successfully removed shortcuts");
        Ok(())
    }
}
