// SeeSea Self-Contained Installer - Installer Module
// 模块名称: installer
// 职责范围: 实现安装器的核心逻辑，包括安装、卸载、修复功能
// 期望实现计划: 
// 1. 定义Installer结构体
// 2. 实现安装逻辑
// 3. 实现卸载逻辑
// 4. 实现修复逻辑
// 5. 实现平台特定功能调用
// 已实现功能: Installer结构体定义、基础安装流程
// 使用依赖: config, platform, utils, anyhow, log, std::fs, std::path
// 主要接口: Installer::new, install, uninstall, repair
// 注意事项: 支持Windows、Linux、macOS平台，使用平台特定实现

use crate::config::{Config, InstallOptions};
use crate::platform::PlatformImpl;
use crate::utils::{create_directory, copy_files, execute_command};
use crate::Args;
use anyhow::Result;
use log::{info, debug, warn};
use std::fs;
use std::path::{Path, PathBuf};
use std::env;

/// 安装器结构体
pub struct Installer {
    /// 配置信息
    pub config: Config,
    /// 命令行参数
    pub args: Args,
    /// 安装目录
    pub install_dir: PathBuf,
    /// 平台特定实现
    pub platform: PlatformImpl,
    /// 安装选项
    pub install_options: InstallOptions,
}

impl Installer {
    /// 创建新的安装器实例
    pub fn new(config: Config, args: &Args) -> Result<Self> {
        debug!("Creating installer instance");
        
        // 确定安装目录
        let install_dir = match &args.install_dir {
            Some(dir) => PathBuf::from(dir),
            None => PathBuf::from(&config.install_options.default_dir),
        };
        
        // 获取平台特定实现
        let platform = PlatformImpl::new()?;
        
        // 获取平台特定安装选项
        let install_options = platform.get_install_options(&config)?;
        
        debug!("Installer instance created with install_dir: {:?}", install_dir);
        
        Ok(Self {
            config,
            args: args.clone(),
            install_dir,
            platform,
            install_options,
        })
    }
    
    /// 执行安装
    pub fn install(&mut self) -> Result<()> {
        info!("Starting installation process");
        debug!("Install options: {:?}", self.install_options);
        
        // 1. 检查系统要求
        info!("Checking system requirements");
        self.platform.check_system_requirements(&self.config)?;
        
        // 2. 创建安装目录
        info!("Creating install directory: {:?}", self.install_dir);
        create_directory(&self.install_dir)?;
        
        // 3. 复制安装文件
        info!("Copying installation files");
        self.copy_install_files()?;
        
        // 4. 安装依赖
        if let Some(deps) = &self.config.dependencies {
            if !deps.is_empty() {
                info!("Installing dependencies");
                self.install_dependencies()?;
            }
        }
        
        // 5. 创建快捷方式
        if self.install_options.create_desktop_shortcut {
            info!("Creating desktop shortcut");
            self.platform.create_desktop_shortcut(&self.config, &self.install_dir)?;
        }
        
        if self.install_options.create_start_menu_shortcut {
            info!("Creating start menu shortcut");
            self.platform.create_start_menu_shortcut(&self.config, &self.install_dir)?;
        }
        
        // 6. 添加到PATH环境变量
        if self.install_options.add_to_path {
            info!("Adding to PATH environment variable");
            self.platform.add_to_path(&self.install_dir)?;
        }
        
        // 7. 创建卸载程序
        if self.install_options.create_uninstaller {
            info!("Creating uninstaller");
            self.platform.create_uninstaller(&self.config, &self.install_dir)?;
        }
        
        // 8. 执行自定义安装后命令
        info!("Running post-install commands");
        self.run_post_install_commands()?;
        
        info!("Installation completed successfully");
        Ok(())
    }
    
    /// 执行卸载
    pub fn uninstall(&mut self) -> Result<()> {
        info!("Starting uninstallation process");
        
        // 1. 执行自定义卸载前命令
        info!("Running pre-uninstall commands");
        self.run_pre_uninstall_commands()?;
        
        // 2. 删除快捷方式
        info!("Removing shortcuts");
        self.platform.remove_shortcuts(&self.config)?;
        
        // 3. 从PATH环境变量中移除
        info!("Removing from PATH environment variable");
        self.platform.remove_from_path(&self.install_dir)?;
        
        // 4. 删除安装目录
        info!("Removing install directory: {:?}", self.install_dir);
        if self.install_dir.exists() {
            fs::remove_dir_all(&self.install_dir)?;
        }
        
        // 5. 删除卸载程序
        info!("Removing uninstaller");
        self.platform.remove_uninstaller(&self.config)?;
        
        info!("Uninstallation completed successfully");
        Ok(())
    }
    
    /// 执行修复
    pub fn repair(&mut self) -> Result<()> {
        info!("Starting repair process");
        
        // 1. 检查安装目录是否存在
        if !self.install_dir.exists() {
            anyhow::bail!("Install directory does not exist: {:?}", self.install_dir);
        }
        
        // 2. 重新复制安装文件
        info!("Re-copying installation files");
        self.copy_install_files()?;
        
        // 3. 重新创建快捷方式
        info!("Re-creating shortcuts");
        if self.install_options.create_desktop_shortcut {
            self.platform.create_desktop_shortcut(&self.config, &self.install_dir)?;
        }
        
        if self.install_options.create_start_menu_shortcut {
            self.platform.create_start_menu_shortcut(&self.config, &self.install_dir)?;
        }
        
        // 4. 确保在PATH环境变量中
        if self.install_options.add_to_path {
            info!("Ensuring in PATH environment variable");
            self.platform.add_to_path(&self.install_dir)?;
        }
        
        info!("Repair completed successfully");
        Ok(())
    }
    
    /// 复制安装文件
    fn copy_install_files(&self) -> Result<()> {
        // 获取当前可执行文件目录
        let exe_path = env::current_exe()?;
        let exe_dir = exe_path.parent().ok_or_else(|| anyhow::anyhow!("Failed to get executable directory"))?;
        
        // 复制安装文件到目标目录
        // 这里假设安装文件在installer目录下的payload子目录中
        let payload_dir = exe_dir.join("payload");
        if payload_dir.exists() {
            debug!("Copying files from {:?} to {:?}", payload_dir, self.install_dir);
            copy_files(&payload_dir, &self.install_dir)?;
        } else {
            warn!("Payload directory not found: {:?}", payload_dir);
        }
        
        Ok(())
    }
    
    /// 安装依赖
    fn install_dependencies(&self) -> Result<()> {
        if let Some(deps) = &self.config.dependencies {
            for dep in deps {
                info!("Installing dependency: {}", dep.name);
                debug!("Dependency config: {:?}", dep);
                
                if let Some(cmd) = &dep.install_command {
                    debug!("Executing dependency install command: {}", cmd);
                    execute_command(cmd, None)?;
                } else {
                    debug!("No install command specified for dependency: {}", dep.name);
                }
            }
        }
        
        Ok(())
    }
    
    /// 执行安装后命令
    fn run_post_install_commands(&self) -> Result<()> {
        // 这里可以添加自定义的安装后命令执行逻辑
        // 例如执行配置文件中定义的命令
        Ok(())
    }
    
    /// 执行卸载前命令
    fn run_pre_uninstall_commands(&self) -> Result<()> {
        // 这里可以添加自定义的卸载前命令执行逻辑
        // 例如停止正在运行的服务
        Ok(())
    }
}
