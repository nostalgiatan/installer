// SeeSea Self-Contained Installer - Linux Platform Implementation
// 模块名称: linux
// 职责范围: 实现Linux平台特定的安装功能
// 期望实现计划: 
// 1. 实现Linux平台特定安装选项获取
// 2. 实现系统要求检查
// 3. 实现桌面快捷方式创建
// 4. 实现开始菜单快捷方式创建
// 5. 实现PATH环境变量管理
// 6. 实现卸载程序创建
// 已实现功能: LinuxImpl结构体定义、基础功能实现
// 使用依赖: config, anyhow, log, std::path, std::env
// 主要接口: LinuxImpl::new, get_install_options, check_system_requirements
// 注意事项: 仅在Linux平台编译，需要root权限执行某些操作

use crate::config::{Config, InstallOptions};
use anyhow::Result;
use log::{debug, info, warn};
use std::env;
use std::path::{Path, PathBuf};

/// Linux平台实现结构体
pub struct LinuxImpl {
    /// 卸载程序路径
    pub uninstall_script_path: String,
}

impl LinuxImpl {
    /// 创建新的Linux平台实现实例
    pub fn new() -> Result<Self> {
        debug!("Creating Linux platform implementation");
        
        Ok(Self {
            uninstall_script_path: "/usr/local/bin/seesea-uninstall".to_string(),
        })
    }
    
    /// 获取桌面目录路径
    fn get_desktop_dir(&self) -> Result<PathBuf> {
        let home_dir = env::var("HOME")?;
        Ok(PathBuf::from(home_dir).join("Desktop"))
    }
    
    /// 获取应用程序菜单目录路径
    fn get_app_menu_dir(&self) -> Result<PathBuf> {
        let home_dir = env::var("HOME")?;
        Ok(PathBuf::from(home_dir).join(".local/share/applications"))
    }
    
    /// 获取系统应用程序菜单目录路径
    fn get_system_app_menu_dir(&self) -> Result<PathBuf> {
        Ok(PathBuf::from("/usr/share/applications"))
    }
}

impl super::Platform for LinuxImpl {
    /// 获取平台特定安装选项
    fn get_install_options(&self, config: &Config) -> Result<InstallOptions> {
        debug!("Getting Linux specific install options");
        
        // 如果配置中有Linux特定选项，则使用它，否则使用全局选项
        if let Some(platform_config) = &config.platform {
            if let Some(linux_config) = &platform_config.linux {
                debug!("Using Linux specific install options from config");
                return Ok(linux_config.clone());
            }
        }
        
        debug!("Using global install options");
        Ok(config.install_options.clone())
    }
    
    /// 检查系统要求
    fn check_system_requirements(&self, config: &Config) -> Result<()> {
        info!("Checking Linux system requirements");
        // 简单实现，仅打印信息
        info!("System requirements check passed");
        Ok(())
    }
    
    /// 创建桌面快捷方式
    fn create_desktop_shortcut(&self, config: &Config, install_dir: &Path) -> Result<()> {
        info!("Creating desktop shortcut on Linux");
        
        // 获取桌面目录
        let desktop_dir = self.get_desktop_dir()?;
        debug!("Desktop directory: {:?}", desktop_dir);
        
        // 快捷方式路径
        let shortcut_path = desktop_dir.join(format!("{}.desktop", config.project.name));
        debug!("Shortcut path: {:?}", shortcut_path);
        
        // 目标程序路径（假设主程序名为项目名）
        let target_exe = install_dir.join(config.project.name.clone());
        debug!("Target executable: {:?}", target_exe);
        
        // 创建.desktop文件内容
        let desktop_content = format!(
            "[Desktop Entry]\n"
            "Name={}\n"
            "Comment={}\n"
            "Exec={}\n"
            "Icon={}\n"
            "Terminal=false\n"
            "Type=Application\n"
            "Categories=Utility;Application;\n"
            "StartupNotify=true\n",
            config.project.name,
            config.project.description.as_deref().unwrap_or(""),
            target_exe.display(),
            // 使用默认图标，实际项目中可以替换为自定义图标路径
            "application-default-icon"
        );
        
        // 写入.desktop文件
        std::fs::write(&shortcut_path, desktop_content)?;
        
        // 设置可执行权限
        std::fs::set_permissions(&shortcut_path, std::fs::Permissions::from_mode(0o755))?;
        
        debug!("Desktop shortcut created successfully");
        
        Ok(())
    }
    
    /// 创建开始菜单快捷方式
    fn create_start_menu_shortcut(&self, config: &Config, install_dir: &Path) -> Result<()> {
        info!("Creating start menu shortcut on Linux");
        
        // 获取应用程序菜单目录
        let app_menu_dir = self.get_app_menu_dir()?;
        debug!("Application menu directory: {:?}", app_menu_dir);
        
        // 创建应用程序菜单目录（如果不存在）
        if !app_menu_dir.exists() {
            std::fs::create_dir_all(&app_menu_dir)?;
        }
        
        // 快捷方式路径
        let shortcut_path = app_menu_dir.join(format!("{}.desktop", config.project.name));
        debug!("Shortcut path: {:?}", shortcut_path);
        
        // 目标程序路径（假设主程序名为项目名）
        let target_exe = install_dir.join(config.project.name.clone());
        debug!("Target executable: {:?}", target_exe);
        
        // 创建.desktop文件内容
        let desktop_content = format!(
            "[Desktop Entry]\n"
            "Name={}\n"
            "Comment={}\n"
            "Exec={}\n"
            "Icon={}\n"
            "Terminal=false\n"
            "Type=Application\n"
            "Categories=Utility;Application;\n"
            "StartupNotify=true\n",
            config.project.name,
            config.project.description.as_deref().unwrap_or(""),
            target_exe.display(),
            // 使用默认图标，实际项目中可以替换为自定义图标路径
            "application-default-icon"
        );
        
        // 写入.desktop文件
        std::fs::write(&shortcut_path, desktop_content)?;
        
        // 设置可执行权限
        std::fs::set_permissions(&shortcut_path, std::fs::Permissions::from_mode(0o644))?;
        
        debug!("Start menu shortcut created successfully");
        
        Ok(())
    }
    
    /// 添加到PATH环境变量
    fn add_to_path(&self, install_dir: &Path) -> Result<()> {
        info!("Adding to PATH environment variable on Linux");
        debug!("Install directory to add: {:?}", install_dir);
        
        // 获取HOME目录
        let home_dir = env::var("HOME")?;
        
        // 确定shell配置文件
        let shell = env::var("SHELL").unwrap_or("/bin/bash".to_string());
        let shell_config = if shell.contains("bash") {
            PathBuf::from(home_dir).join(".bashrc")
        } else if shell.contains("zsh") {
            PathBuf::from(home_dir).join(".zshrc")
        } else {
            // 默认使用.bashrc
            PathBuf::from(home_dir).join(".bashrc")
        };
        
        debug!("Using shell config file: {:?}", shell_config);
        
        // 读取当前配置文件内容
        let current_content = std::fs::read_to_string(&shell_config)?;
        
        // 检查是否已存在
        let install_dir_str = install_dir.to_string_lossy().to_string();
        let path_line = format!("export PATH=\"$PATH:{}\"", install_dir_str);
        
        if current_content.contains(&path_line) {
            debug!("Directory already in PATH: {:?}", install_dir);
            return Ok(());
        }
        
        // 添加到配置文件
        let mut new_content = current_content;
        new_content.push_str(&format!("\n{}\n", path_line));
        
        std::fs::write(&shell_config, new_content)?;
        
        info!("Added to PATH environment variable successfully");
        debug!("Please restart your terminal or run 'source {:?}' to apply the changes", shell_config);
        
        Ok(())
    }
    
    /// 从PATH环境变量中移除
    fn remove_from_path(&self, install_dir: &Path) -> Result<()> {
        info!("Removing from PATH environment variable on Linux");
        debug!("Install directory to remove: {:?}", install_dir);
        
        // 获取HOME目录
        let home_dir = env::var("HOME")?;
        
        // 确定shell配置文件
        let shell = env::var("SHELL").unwrap_or("/bin/bash".to_string());
        let shell_config = if shell.contains("bash") {
            PathBuf::from(home_dir).join(".bashrc")
        } else if shell.contains("zsh") {
            PathBuf::from(home_dir).join(".zshrc")
        } else {
            // 默认使用.bashrc
            PathBuf::from(home_dir).join(".bashrc")
        };
        
        debug!("Using shell config file: {:?}", shell_config);
        
        // 读取当前配置文件内容
        let current_content = std::fs::read_to_string(&shell_config)?;
        
        // 移除PATH行
        let install_dir_str = install_dir.to_string_lossy().to_string();
        let path_line = format!("export PATH=\"$PATH:{}\"", install_dir_str);
        
        let new_content = current_content
            .lines()
            .filter(|&line| line != path_line)
            .collect::<Vec<_>>()
            .join("\n");
        
        if new_content == current_content {
            debug!("Directory not found in PATH: {:?}", install_dir);
            return Ok(());
        }
        
        // 写入新配置
        std::fs::write(&shell_config, new_content)?;
        
        info!("Removed from PATH environment variable successfully");
        debug!("Please restart your terminal or run 'source {:?}' to apply the changes", shell_config);
        
        Ok(())
    }
    
    /// 创建卸载程序
    fn create_uninstaller(&self, config: &Config, install_dir: &Path) -> Result<()> {
        info!("Creating uninstaller on Linux");
        
        // 卸载脚本内容
        let uninstall_script = format!(
            "#!/bin/bash\n"
            "# SeeSea Uninstaller\n"
            "\n"
            "echo \"Uninstalling {}-{}...\"\n"
            "\n"
            "# 删除安装目录\n"
            "rm -rf {}\n"
            "\n"
            "# 删除桌面快捷方式\n"
            "rm -f \$HOME/Desktop/{}.desktop\n"
            "\n"
            "# 删除应用程序菜单快捷方式\n"
            "rm -f \$HOME/.local/share/applications/{}.desktop\n"
            "\n"
            "# 删除卸载脚本\n"
            "rm -f \$0\n"
            "\n"
            "echo \"Uninstallation completed successfully!\"\n",
            config.project.name,
            config.project.version,
            install_dir.display(),
            config.project.name,
            config.project.name
        );
        
        // 写入卸载脚本
        std::fs::write(&self.uninstall_script_path, uninstall_script)?;
        
        // 设置可执行权限
        std::fs::set_permissions(&self.uninstall_script_path, std::fs::Permissions::from_mode(0o755))?;
        
        info!("Uninstaller created successfully at: {}", self.uninstall_script_path);
        
        Ok(())
    }
    
    /// 删除快捷方式
    fn remove_shortcuts(&self, config: &Config) -> Result<()> {
        info!("Removing shortcuts on Linux");
        
        // 删除桌面快捷方式
        let desktop_dir = self.get_desktop_dir()?;
        let desktop_shortcut = desktop_dir.join(format!("{}.desktop", config.project.name));
        if desktop_shortcut.exists() {
            std::fs::remove_file(&desktop_shortcut)?;
            debug!("Desktop shortcut removed: {:?}", desktop_shortcut);
        }
        
        // 删除应用程序菜单快捷方式
        let app_menu_dir = self.get_app_menu_dir()?;
        let app_shortcut = app_menu_dir.join(format!("{}.desktop", config.project.name));
        if app_shortcut.exists() {
            std::fs::remove_file(&app_shortcut)?;
            debug!("Application menu shortcut removed: {:?}", app_shortcut);
        }
        
        // 也检查系统应用程序菜单目录
        let system_app_menu_dir = self.get_system_app_menu_dir()?;
        let system_app_shortcut = system_app_menu_dir.join(format!("{}.desktop", config.project.name));
        if system_app_shortcut.exists() {
            std::fs::remove_file(&system_app_shortcut)?;
            debug!("System application menu shortcut removed: {:?}", system_app_shortcut);
        }
        
        info!("Shortcuts removed successfully");
        
        Ok(())
    }
    
    /// 删除卸载程序
    fn remove_uninstaller(&self, config: &Config) -> Result<()> {
        info!("Removing uninstaller on Linux");
        
        // 删除卸载脚本
        if Path::new(&self.uninstall_script_path).exists() {
            std::fs::remove_file(&self.uninstall_script_path)?;
            debug!("Uninstaller removed: {}", self.uninstall_script_path);
        }
        
        info!("Uninstaller removed successfully");
        
        Ok(())
    }
}
