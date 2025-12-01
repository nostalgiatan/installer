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

use crate::config::{Config, InstallOptions, ComponentConfig};
use crate::platform::PlatformImpl;
use crate::utils::{create_directory, execute_command, copy_files};
use crate::version::{Version, get_current_version, save_version, check_update, get_latest_version_from_github};
use crate::Args;
use anyhow::Result;
use log::{info, debug, warn};
use std::fs;
use std::path::{Path, PathBuf};
use std::env;
use std::collections::HashMap;

/// 组件状态
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ComponentStatus {
    /// 已安装
    Installed,
    /// 未安装
    NotInstalled,
    /// 安装中
    Installing,
    /// 卸载中
    Uninstalling,
    /// 损坏
    Broken,
}

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
    /// 组件状态
    pub component_status: HashMap<String, ComponentStatus>,
    /// 临时目录
    pub temp_dir: PathBuf,
    /// 已安装的文件列表，用于回滚
    pub installed_files: Vec<PathBuf>,
    /// 已安装的组件列表，用于回滚
    pub installed_components: Vec<String>,
    /// 已创建的快捷方式列表
    #[allow(dead_code)]
    pub created_shortcuts: Vec<PathBuf>,
    /// 是否已创建卸载程序，用于回滚
    pub created_uninstaller: bool,
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
        
        // 创建临时目录
        let temp_dir = std::env::temp_dir().join("seesea-installer");
        create_directory(&temp_dir)?;
        
        // 初始化组件状态
        let mut component_status = HashMap::new();
        if let Some(components) = &config.components {
            for component in components {
                component_status.insert(component.name.clone(), ComponentStatus::NotInstalled);
            }
        }
        
        debug!("Installer instance created with install_dir: {install_dir:?}");
        debug!("Temporary directory: {temp_dir:?}");
        debug!("Initial component count: {}", component_status.len());
        
        // 初始化已安装文件列表
        let installed_files: Vec<PathBuf> = Vec::new();
        
        // 初始化已安装组件列表
        let installed_components: Vec<String> = Vec::new();
        
        // 初始化已创建快捷方式列表
        let created_shortcuts: Vec<PathBuf> = Vec::new();
        
        // 初始化创建卸载程序状态
        let created_uninstaller = false;
        
        Ok(Self {
            config,
            args: args.clone(),
            install_dir,
            platform,
            install_options,
            component_status,
            temp_dir,
            installed_files,
            installed_components,
            created_shortcuts,
            created_uninstaller,
        })
    }
    
    /// 执行安装
    pub fn install(&mut self) -> Result<()> {
        // 打印欢迎信息
        println!("\x1b[1;36m========================================\x1b[0m");
        println!("\x1b[1;36m      SeeSea Installer v{}\x1b[0m", env!("CARGO_PKG_VERSION"));
        println!("\x1b[1;36m========================================\x1b[0m");
        println!("\x1b[1;32m✓\x1b[0m Starting installation process");
        println!("\x1b[1;32m✓\x1b[0m Install directory: {}", self.install_dir.display());
        println!("\x1b[1;32m✓\x1b[0m Install options: {install_options:?}", install_options = self.install_options);
        println!();
        
        info!("Starting installation process");
        debug!("Install options: {install_options:?}", install_options = self.install_options);
        
        // 安装过程中发生错误时，执行回滚
        let result = self.install_internal();
        
        if let Err(e) = &result {
            println!();
            println!("\x1b[1;31m✗\x1b[0m Installation failed!");
            println!("\x1b[1;31m✗\x1b[0m Error: {e:?}");
            println!("\x1b[1;33m→\x1b[0m Starting rollback...");
            info!("Installation failed, starting rollback...");
            debug!("Error: {e:?}");
            if let Err(rollback_err) = self.rollback() {
                warn!("Rollback failed: {rollback_err:?}");
                println!("\x1b[1;31m✗\x1b[0m Rollback failed: {rollback_err:?}");
            } else {
                println!("\x1b[1;32m✓\x1b[0m Rollback completed");
            }
            // 清理临时文件
            if let Err(cleanup_err) = self.cleanup() {
                warn!("Cleanup failed: {cleanup_err:?}");
                println!("\x1b[1;31m✗\x1b[0m Cleanup failed: {cleanup_err:?}");
            } else {
                println!("\x1b[1;32m✓\x1b[0m Cleanup completed");
            }
        } else {
            // 安装成功，清理临时文件
            println!();
            println!("\x1b[1;32m✓\x1b[0m Cleaning up temporary files");
            info!("Cleaning up temporary files");
            self.cleanup()?;
            
            println!("\x1b[1;36m========================================\x1b[0m");
            println!("\x1b[1;32m✓\x1b[0m Installation completed successfully!");
            println!("\x1b[1;32m✓\x1b[0m SeeSea has been installed to: {}", self.install_dir.display());
            println!("\x1b[1;36m========================================\x1b[0m");
            info!("Installation completed successfully");
        }
        
        result
    }
    
    /// 内部安装方法，包含实际安装逻辑
    fn install_internal(&mut self) -> Result<()> {
        // 1. 执行预安装脚本
        if let Some(pre_script) = &self.install_options.pre_install_script {
            info!("Running pre-install script");
            execute_command(pre_script, Some(&self.temp_dir))?;
        }
        
        // 2. 检查系统要求
        info!("Checking system requirements");
        self.platform.check_system_requirements(&self.config)?;
        
        // 3. 创建安装目录
        info!("Creating install directory: {install_dir:?}", install_dir = self.install_dir);
        create_directory(&self.install_dir)?;
        
        // 4. 安装组件
        info!("Installing components");
        self.install_components()?;
        
        // 5. 复制安装文件
        info!("Copying installation files");
        self.copy_install_files()?;
        
        // 6. 安装依赖
        info!("Installing dependencies");
        self.install_dependencies()?;
        
        // 7. 创建快捷方式
        if self.install_options.create_desktop_shortcut {
            info!("Creating desktop shortcut");
            self.platform.create_desktop_shortcut(&self.config, &self.install_dir)?;
        }
        
        if self.install_options.create_start_menu_shortcut {
            info!("Creating start menu shortcut");
            self.platform.create_start_menu_shortcut(&self.config, &self.install_dir)?;
        }
        
        // 8. 创建系统服务
        if self.install_options.create_service {
            info!("Creating system service");
            self.create_service()?;
        }
        
        // 9. 创建卸载程序
        if self.install_options.create_uninstaller {
            info!("Creating uninstaller");
            self.platform.create_uninstaller(&self.config, &self.install_dir)?;
            self.created_uninstaller = true;
        }
        
        // 11. 执行自定义安装后命令
        info!("Running post-install commands");
        self.run_post_install_commands()?;
        
        // 12. 执行后安装脚本
        if let Some(post_script) = &self.install_options.post_install_script {
            info!("Running post-install script");
            execute_command(post_script, Some(&self.install_dir))?;
        }
        
        Ok(())
    }
    
    /// 回滚安装
    fn rollback(&mut self) -> Result<()> {
        info!("Performing rollback...");
        
        // 1. 回滚创建卸载程序
        if self.created_uninstaller {
            info!("Rolling back uninstaller");
            if let Err(e) = self.platform.remove_uninstaller(&self.config) {
                warn!("Failed to rollback uninstaller: {e:?}");
            }
            self.created_uninstaller = false;
        }
        
        // 2. 回滚创建快捷方式
        info!("Rolling back shortcuts");
        if let Err(e) = self.platform.remove_shortcuts(&self.config) {
            warn!("Failed to rollback shortcuts: {e:?}");
        }
        
        // 3. 删除已安装的文件
        info!("Rolling back installed files");
        for file_path in &self.installed_files {
            if file_path.exists() {
                if let Err(e) = std::fs::remove_file(file_path) {
                    warn!("Failed to remove file: {file_path:?}, error: {e:?}");
                }
            }
        }
        self.installed_files.clear();
        
        // 4. 删除安装目录
        info!("Rolling back install directory");
        if self.install_dir.exists() {
            if let Err(e) = std::fs::remove_dir_all(&self.install_dir) {
                warn!("Failed to remove install directory: {install_dir:?}, error: {e:?}", install_dir = self.install_dir);
            }
        }
        
        info!("Rollback completed");
        Ok(())
    }
    
    /// 安装组件
    fn install_components(&mut self) -> Result<()> {
        debug!("Installing components");
        
        if let Some(components) = &self.config.components {
            // 构建组件依赖图和组件映射
            let mut dependency_graph: HashMap<String, Vec<String>> = HashMap::new();
            let mut component_map: HashMap<String, &ComponentConfig> = HashMap::new();
            
            for component in components {
                component_map.insert(component.name.clone(), component);
                dependency_graph.insert(component.name.clone(), component.depends_on.clone().unwrap_or(Vec::new()));
            }
            
            // 拓扑排序组件
            let sorted_components = self.topological_sort(&dependency_graph)?;
            info!("Installing {} components in order: {sorted_components:?}", sorted_components.len());
            
            // 按照拓扑排序顺序安装组件
            for (index, component_name) in sorted_components.iter().enumerate() {
                let component = component_map.get(component_name).unwrap();
                info!("Installing component {}/{1}: {2}", index + 1, sorted_components.len(), component.name);
                self.component_status.insert(component.name.clone(), ComponentStatus::Installing);
                
                // 安装组件文件
                if let Some(files) = &component.files {
                    debug!("Installing {} files for component: {1}", files.len(), component.name);
                    
                    for file in files {
                        let src_path = Path::new(file);
                        if src_path.exists() {
                            let dest_path = self.install_dir.join(src_path.file_name().unwrap());
                            fs::copy(src_path, &dest_path)?;
                            // 添加到已安装文件列表
                            self.installed_files.push(dest_path.clone());
                            debug!("Copied component file: {src_path:?} -> {dest_path:?}");
                        } else {
                            warn!("Component file not found: {src_path:?}");
                        }
                    }
                }
                
                self.component_status.insert(component.name.clone(), ComponentStatus::Installed);
                // 添加到已安装组件列表
                self.installed_components.push(component.name.clone());
                info!("Component {}/{1} installed successfully: {2}", index + 1, sorted_components.len(), component.name);
            }
        } else {
            debug!("No components to install");
        }
        
        Ok(())
    }
    
    /// 拓扑排序
    fn topological_sort(&self, graph: &HashMap<String, Vec<String>>) -> Result<Vec<String>> {
        debug!("Performing topological sort on component dependencies");
        
        // 计算每个节点的入度
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        for (node, edges) in graph {
            if !in_degree.contains_key(node) {
                in_degree.insert(node.clone(), 0);
            }
            for edge in edges {
                *in_degree.entry(edge.clone()).or_insert(0) += 1;
            }
        }
        
        // 初始化队列，将入度为0的节点加入队列
        let mut queue: Vec<String> = Vec::new();
        for (node, degree) in &in_degree {
            if *degree == 0 {
                queue.push(node.clone());
            }
        }
        
        // 执行拓扑排序
        let mut result: Vec<String> = Vec::new();
        while !queue.is_empty() {
            let node = queue.remove(0);
            result.push(node.clone());
            
            // 遍历当前节点的所有邻接节点，减少它们的入度
            if let Some(neighbors) = graph.get(&node) {
                for neighbor in neighbors {
                    let degree = in_degree.get_mut(neighbor).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push(neighbor.clone());
                    }
                }
            }
        }
        
        // 检查是否存在环
        if result.len() != graph.len() {
            anyhow::bail!("Component dependency graph contains a cycle");
        }
        
        debug!("Topological sort result: {result:?}");
        Ok(result)
    }
    
    /// 创建系统服务
    fn create_service(&self) -> Result<()> {
        debug!("Creating system service");
        // 系统服务创建逻辑
        // 目前仅作为占位符，后续实现平台特定的服务创建
        Ok(())
    }
    
    /// 清理临时文件
    fn cleanup(&self) -> Result<()> {
        debug!("Cleaning up temporary files");
        if self.temp_dir.exists() {
            fs::remove_dir_all(&self.temp_dir)?;
        }
        Ok(())
    }
    
    /// 备份安装目录
    fn backup_installation(&self, backup_dir: Option<&Path>) -> Result<PathBuf> {
        info!("Backing up current installation");
        
        // 确定备份目录
        let backup_path = match backup_dir {
            Some(dir) => dir.to_path_buf(),
            None => {
                let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
                self.temp_dir.join(format!("backup_{timestamp}"))
            }
        };
        
        // 创建备份目录
        create_directory(&backup_path)?;
        debug!("Backup directory: {backup_path:?}");
        
        // 复制安装目录内容到备份目录
        copy_files(&self.install_dir, &backup_path)?;
        
        info!("Backup completed successfully: {backup_path:?}");
        Ok(backup_path)
    }
    
    /// 恢复安装目录
    fn restore_installation(&self, backup_dir: &Path) -> Result<()> {
        info!("Restoring installation from backup");
        debug!("Backup directory: {backup_dir:?}");
        
        // 删除当前安装目录
        if self.install_dir.exists() {
            fs::remove_dir_all(&self.install_dir)?;
        }
        
        // 复制备份内容到安装目录
        copy_files(backup_dir, &self.install_dir)?;
        
        info!("Restore completed successfully");
        Ok(())
    }
    
    /// 执行更新
    pub fn update(&mut self) -> Result<()> {
        info!("Starting update process");
        debug!("Update options: check={}, backup_dir={:?}, force={}", 
               self.args.check, self.args.backup_dir, self.args.force);
        
        // 解析当前版本
        let current_version = get_current_version(&self.install_dir)?;
        
        // 从GitHub获取最新版本
        let new_version = get_latest_version_from_github()?;
        
        // 仅检查更新
        if self.args.check {
            info!("Checking for updates...");
            match current_version {
                Some(version) => {
                    let comparison = version.compare(&new_version);
                    if comparison < 0 {
                        info!("Update available: {version} -> {new_version}");
                    } else if comparison == 0 {
                        info!("Already on the latest version: {version}");
                    } else {
                        info!("Current version is newer than available version: {version} -> {new_version}");
                    }
                }
                None => {
                    info!("No current version found, update available: {new_version}");
                }
            }
            return Ok(());
        }
        
        // 检查是否需要更新
        if !check_update(current_version.clone(), &new_version, self.args.force) {
            info!("No update needed, current version is already up to date: {}", 
                  current_version.as_ref().map(|v| v.to_string()).unwrap_or("unknown".to_string()));
            return Ok(());
        }
        
        // 备份当前安装
        let backup_dir = self.args.backup_dir.as_ref().map(Path::new);
        let backup_path = self.backup_installation(backup_dir)?;
        
        // 更新过程中发生错误时，执行回滚
        let result = self.update_internal(&new_version);
        
        if let Err(e) = &result {
            info!("Update failed, starting rollback from backup: {backup_path:?}");
            debug!("Error: {e:?}");
            if let Err(rollback_err) = self.restore_installation(&backup_path) {
                warn!("Rollback failed: {rollback_err:?}");
            }
            // 清理临时文件
            if let Err(cleanup_err) = self.cleanup() {
                warn!("Cleanup failed: {cleanup_err:?}");
            }
        } else {
            // 更新成功，清理临时文件
            info!("Cleaning up temporary files");
            self.cleanup()?;
            info!("Update completed successfully");
        }
        
        result
    }
    
    /// 内部更新方法，包含实际更新逻辑
    fn update_internal(&mut self, new_version: &Version) -> Result<()> {
        info!("Starting internal update process");
        debug!("New version: {new_version:?}");
        
        // 1. 执行预安装脚本
        if let Some(pre_script) = &self.install_options.pre_install_script {
            info!("Running pre-install script");
            execute_command(pre_script, Some(&self.temp_dir))?;
        }
        
        // 2. 检查系统要求
        info!("Checking system requirements");
        self.platform.check_system_requirements(&self.config)?;
        
        // 3. 安装依赖
        if let Some(deps) = &self.config.dependencies {
            if !deps.is_empty() {
                info!("Installing dependencies");
                self.install_dependencies()?;
            }
        }
        
        // 4. 安装组件
        info!("Installing components");
        self.install_components()?;
        
        // 5. 复制安装文件
        info!("Copying installation files");
        self.copy_install_files()?;
        
        // 6. 更新快捷方式
        if self.install_options.create_desktop_shortcut {
            info!("Updating desktop shortcut");
            self.platform.remove_shortcuts(&self.config)?;
            self.platform.create_desktop_shortcut(&self.config, &self.install_dir)?;
        }
        
        if self.install_options.create_start_menu_shortcut {
            info!("Updating start menu shortcut");
            self.platform.create_start_menu_shortcut(&self.config, &self.install_dir)?;
        }
        
        // 7. 确保在PATH环境变量中
        if self.install_options.add_to_path {
            info!("Ensuring in PATH environment variable");
            self.platform.add_to_path(&self.install_dir)?;
        }
        
        // 8. 更新服务配置
        if self.install_options.create_service {
            info!("Updating system service");
            self.create_service()?;
        }
        
        // 9. 更新卸载程序
        if self.install_options.create_uninstaller {
            info!("Updating uninstaller");
            self.platform.create_uninstaller(&self.config, &self.install_dir)?;
        }
        
        // 10. 执行自定义安装后命令
        info!("Running post-install commands");
        self.run_post_install_commands()?;
        
        // 11. 执行后安装脚本
        if let Some(post_script) = &self.install_options.post_install_script {
            info!("Running post-install script");
            execute_command(post_script, Some(&self.install_dir))?;
        }
        
        // 12. 保存新的版本号
        save_version(&self.install_dir, new_version)?;
        
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
        
        // 4. 卸载Python包
        info!("Uninstalling Python packages");
        
        if cfg!(target_os = "linux") {
            // Linux平台：使用虚拟环境中的pip命令卸载
            let venv_dir = Path::new("/etc/seesea/venv");
            let venv_pip = venv_dir.join("bin").join("pip");
            
            if venv_pip.exists() {
                // 卸载seesea包，忽略错误
                info!("Uninstalling seesea package using virtual environment pip");
                println!("执行命令: {} uninstall -y seesea", venv_pip.display());
                let status = std::process::Command::new(venv_pip.clone())
                    .args(["uninstall", "-y", "seesea"])
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit())
                    .status();
                println!("命令执行状态: {status:?}");
                
                // 卸载seesea-core包，忽略错误
                info!("Uninstalling seesea-core package using virtual environment pip");
                println!("执行命令: {} uninstall -y seesea-core", venv_pip.display());
                let status = std::process::Command::new(venv_pip)
                    .args(["uninstall", "-y", "seesea-core"])
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit())
                    .status();
                println!("命令执行状态: {status:?}");
            } else {
                warn!("Virtual environment pip not found, skipping Python package uninstallation");
            }
            
            // 5. 删除虚拟环境目录
            let see_sea_dir = Path::new("/etc/seesea");
            if see_sea_dir.exists() {
                info!("Removing virtual environment directory: {:?}", see_sea_dir);
                fs::remove_dir_all(see_sea_dir)?;
            }
            
            // 6. 删除命令导出文件
            let seesea_cmd = Path::new("/usr/local/bin/seesea");
            if seesea_cmd.exists() {
                info!("Removing command export file: {:?}", seesea_cmd);
                std::fs::remove_file(seesea_cmd)?;
            }
        } else {
            // Windows和macOS平台：使用系统pip命令卸载
            let pip_cmd = if cfg!(target_os = "windows") {
                "pip" 
            } else {
                "pip3"
            };
            
            // 卸载seesea包，忽略错误
            info!("Uninstalling seesea package");
            println!("执行命令: {pip_cmd} uninstall -y seesea");
            let status = std::process::Command::new(pip_cmd)
                .args(["uninstall", "-y", "seesea"])
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status();
            println!("命令执行状态: {status:?}");
            
            // 卸载seesea-core包，忽略错误
            info!("Uninstalling seesea-core package");
            println!("执行命令: {pip_cmd} uninstall -y seesea-core");
            let status = std::process::Command::new(pip_cmd)
                .args(["uninstall", "-y", "seesea-core"])
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status();
            println!("命令执行状态: {status:?}");
        }
        
        // 7. 删除安装目录
        info!("Removing install directory: {install_dir:?}", install_dir = self.install_dir);
        if self.install_dir.exists() {
            // 先保存uninstaller路径，因为我们需要在删除目录前删除它
            let uninstaller_path = self.install_dir.join("uninstall.exe");
            
            // 8. 删除卸载程序
            info!("Removing uninstaller");
            self.platform.remove_uninstaller(&self.config)?;
            
            // 9. 删除安装目录
            // 先删除uninstall.exe，因为它正在运行
            if uninstaller_path.exists() {
                std::fs::remove_file(&uninstaller_path)?;
            }
            
            // 删除剩余的安装目录
            fs::remove_dir_all(&self.install_dir)?;
        } else {
            // 安装目录不存在，只删除卸载程序信息
            info!("Install directory not found, only removing uninstaller information");
            self.platform.remove_uninstaller(&self.config)?;
        }
        
        info!("Uninstallation completed successfully");
        Ok(())
    }
    
    /// 执行修复
    pub fn repair(&mut self) -> Result<()> {
        info!("Starting repair process");
        
        // 1. 检查安装目录是否存在
        if !self.install_dir.exists() {
            anyhow::bail!("Install directory does not exist: {install_dir:?}", install_dir = self.install_dir);
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
    fn copy_install_files(&mut self) -> Result<()> {
        // 获取当前可执行文件目录
        let exe_path = env::current_exe()?;
        let exe_dir = exe_path.parent().ok_or_else(|| anyhow::anyhow!("Failed to get executable directory"))?;
        
        // 尝试多种路径查找building目录
        let mut building_paths = Vec::new();
        
        // 当前可执行文件所在目录的building子目录
        building_paths.push(exe_dir.join("building"));
        
        // 当前目录
        building_paths.push(PathBuf::from("building"));
        
        // 上级目录
        building_paths.push(PathBuf::from("../building"));
        building_paths.push(PathBuf::from("../../building"));
        
        // 系统安装目录
        building_paths.push(PathBuf::from("/opt/seesea-installer/building"));
        building_paths.push(PathBuf::from("C:\\Program Files\\SeeSea-Installer\\building"));
        building_paths.push(PathBuf::from("/Applications/SeeSea-Installer/building"));
        
        // Linux deb包特定目录结构：building在lib/seesea-installer目录下
        building_paths.push(exe_dir.join("../lib/seesea-installer/building"));
        building_paths.push(PathBuf::from("/usr/lib/seesea-installer/building"));
        building_paths.push(PathBuf::from("/lib/seesea-installer/building"));
        // 添加更多可能的路径
        building_paths.push(PathBuf::from("/usr/local/lib/seesea-installer/building"));
        building_paths.push(PathBuf::from("/opt/seesea-installer/building"));
        
        // 查找存在的building目录
        let mut found_building_dir = None;
        for path in &building_paths {
            if path.exists() && path.is_dir() {
                found_building_dir = Some(path);
                break;
            }
        }
        
        if let Some(building_dir) = found_building_dir {
            debug!("Copying files from {building_dir:?} to {install_dir:?}", install_dir = self.install_dir);
            
            // 遍历building目录下的所有文件
            for entry in std::fs::read_dir(building_dir)? {
                let entry = entry?;
                let src_path = entry.path();
                if src_path.is_file() {
                    let dest_path = self.install_dir.join(src_path.file_name().unwrap());
                    
                    // 复制文件
                    std::fs::copy(&src_path, &dest_path)?;
                    
                    // 添加到已安装文件列表
                    self.installed_files.push(dest_path.clone());
                    debug!("Copied file: {src_path:?} -> {dest_path:?}");
                }
            }
        } else {
            warn!("Building directory not found at any of the tried paths: {building_paths:?}");
            anyhow::bail!("Building directory not found");
        }
        
        Ok(())
    }
    
    /// 安装依赖
    fn install_dependencies(&self) -> Result<()> {
        // 检查Python环境
        info!("Checking Python environment");
        let python_cmd = if cfg!(target_os = "windows") {
            "python" 
        } else {
            "python3"
        };
        
        let python_check = execute_command(format!("{python_cmd} --version").as_str(), None);
        if python_check.is_err() {
            anyhow::bail!("Python is not installed or not in PATH");
        }
        
        // 检查pip环境
        info!("Checking pip environment");
        let pip_cmd = if cfg!(target_os = "windows") {
            "pip" 
        } else {
            "pip3"
        };
        
        let pip_check = execute_command(format!("{pip_cmd} --version").as_str(), None);
        if pip_check.is_err() {
            anyhow::bail!("pip is not installed or not in PATH");
        }
        
        // 收集所有whl文件
        let mut whl_files = Vec::new();
        for entry in std::fs::read_dir(&self.install_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
                if file_name.ends_with(".whl") {
                    whl_files.push(path);
                    info!("Found whl file: {file_name}");
                }
            }
        }
        
        if whl_files.is_empty() {
            warn!("No whl files found in install directory");
            return Ok(());
        }
        
        // 根据平台执行不同的安装逻辑
        if cfg!(target_os = "linux") {
            // Linux平台：使用虚拟环境安装
            info!("Installing on Linux platform");
            
            // 创建安装目录
            let install_base_dir = Path::new("/etc/seesea");
            create_directory(install_base_dir)?;
            
            // 创建虚拟环境
            let venv_dir = install_base_dir.join("venv");
            if !venv_dir.exists() {
                info!("Creating virtual environment at: {venv_dir:?}");
                execute_command(format!("{python_cmd} -m venv {}", venv_dir.to_str().unwrap()).as_str(), None)?;
            }
            
            // 虚拟环境中的pip命令
            let venv_pip = venv_dir.join("bin").join("pip");
            
            // 安装所有whl文件
            for whl_file in &whl_files {
                info!("Installing whl file in virtual environment: {whl_file:?}");
                execute_command(format!("{} install {}", venv_pip.to_str().unwrap(), whl_file.to_str().unwrap()).as_str(), None)?;
            }
            
            // 创建bash脚本，导出seesea命令
            let bash_script_path = Path::new("/usr/local/bin/seesea");
            let bash_script_content = format!("#!/bin/bash\n\n{}/bin/seesea \"$@\"\n", venv_dir.to_str().unwrap());
            
            info!("Creating bash script at: {bash_script_path:?}");
            std::fs::write(bash_script_path, bash_script_content)?;
            
            // 设置脚本执行权限
            execute_command(format!("chmod +x {}", bash_script_path.to_str().unwrap()).as_str(), None)?;
            
        } else if cfg!(target_os = "windows") {
            // Windows平台：直接安装
            info!("Installing on Windows platform");
            
            for whl_file in &whl_files {
                info!("Installing whl file: {whl_file:?}");
                execute_command(format!("{pip_cmd} install {}", whl_file.to_str().unwrap()).as_str(), None)?;
            }
            
        } else if cfg!(target_os = "macos") {
            // macOS平台：直接安装
            info!("Installing on macOS platform");
            
            for whl_file in &whl_files {
                info!("Installing whl file: {whl_file:?}");
                execute_command(format!("{pip_cmd} install {}", whl_file.to_str().unwrap()).as_str(), None)?;
            }
        }
        
        info!("All dependencies installed successfully");
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
