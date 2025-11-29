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
use crate::version::{Version, get_current_version, save_version, check_update};
use crate::Args;
use anyhow::Result;
use log::{info, debug, warn};
use std::fs;
use std::path::{Path, PathBuf};
use std::env;
use std::collections::HashMap;
use libloading::{Library, Symbol};

/// 组件状态
#[derive(Debug, Clone)]
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
    /// 插件加载器
    pub plugins: Vec<Box<dyn Plugin>>,
    /// 已安装的文件列表，用于回滚
    pub installed_files: Vec<PathBuf>,
    /// 已安装的组件列表，用于回滚
    pub installed_components: Vec<String>,
    /// 已创建的快捷方式列表，用于回滚
    pub created_shortcuts: Vec<PathBuf>,
    /// 是否已添加到PATH，用于回滚
    pub added_to_path: bool,
    /// 是否已创建卸载程序，用于回滚
    pub created_uninstaller: bool,
}

/// 插件接口
pub trait Plugin: Send + Sync {
    /// 插件名称
    fn name(&self) -> &str;
    /// 初始化插件
    fn init(&mut self, config: &Config) -> Result<()>;
    /// 安装前调用
    fn pre_install(&self, installer: &mut Installer) -> Result<()>;
    /// 安装后调用
    fn post_install(&self, installer: &mut Installer) -> Result<()>;
    /// 卸载前调用
    fn pre_uninstall(&self, installer: &mut Installer) -> Result<()>;
    /// 卸载后调用
    fn post_uninstall(&self, installer: &mut Installer) -> Result<()>;
    /// 修复前调用
    fn pre_repair(&self, installer: &mut Installer) -> Result<()>;
    /// 修复后调用
    fn post_repair(&self, installer: &mut Installer) -> Result<()>;
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
        
        // 初始化插件列表
        let plugins: Vec<Box<dyn Plugin>> = Vec::new();
        
        debug!("Installer instance created with install_dir: {install_dir:?}");
        debug!("Temporary directory: {temp_dir:?}");
        debug!("Initial component count: {}", component_status.len());
        
        // 初始化已安装文件列表
        let installed_files: Vec<PathBuf> = Vec::new();
        
        // 初始化已安装组件列表
        let installed_components: Vec<String> = Vec::new();
        
        // 初始化已创建快捷方式列表
        let created_shortcuts: Vec<PathBuf> = Vec::new();
        
        // 初始化添加到PATH状态
        let added_to_path = false;
        
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
            plugins,
            installed_files,
            installed_components,
            created_shortcuts,
            added_to_path,
            created_uninstaller,
        })
    }
    
    /// 执行安装
    pub fn install(&mut self) -> Result<()> {
        info!("Starting installation process");
        debug!("Install options: {install_options:?}", install_options = self.install_options);
        
        // 安装过程中发生错误时，执行回滚
        let result = self.install_internal();
        
        if let Err(e) = &result {
            info!("Installation failed, starting rollback...");
            debug!("Error: {e:?}");
            if let Err(rollback_err) = self.rollback() {
                warn!("Rollback failed: {rollback_err:?}");
            }
            // 清理临时文件
            if let Err(cleanup_err) = self.cleanup() {
                warn!("Cleanup failed: {cleanup_err:?}");
            }
        } else {
            // 安装成功，清理临时文件
            info!("Cleaning up temporary files");
            self.cleanup()?;
            info!("Installation completed successfully");
        }
        
        result
    }
    
    /// 内部安装方法，包含实际安装逻辑
    fn install_internal(&mut self) -> Result<()> {
        // 1. 加载插件
        info!("Loading plugins");
        self.load_plugins()?;
        
        // 2. 执行插件的pre_install方法
        self.run_plugin_pre_install()?;
        
        // 3. 执行预安装脚本
        if let Some(pre_script) = &self.install_options.pre_install_script {
            info!("Running pre-install script");
            execute_command(pre_script, Some(&self.temp_dir))?;
        }
        
        // 4. 检查系统要求
        info!("Checking system requirements");
        self.platform.check_system_requirements(&self.config)?;
        
        // 5. 创建安装目录
        info!("Creating install directory: {install_dir:?}", install_dir = self.install_dir);
        create_directory(&self.install_dir)?;
        
        // 6. 安装依赖
        if let Some(deps) = &self.config.dependencies {
            if !deps.is_empty() {
                info!("Installing dependencies");
                self.install_dependencies()?;
            }
        }
        
        // 7. 安装组件
        info!("Installing components");
        self.install_components()?;
        
        // 8. 复制安装文件
        info!("Copying installation files");
        self.copy_install_files()?;
        
        // 9. 创建快捷方式
        if self.install_options.create_desktop_shortcut {
            info!("Creating desktop shortcut");
            self.platform.create_desktop_shortcut(&self.config, &self.install_dir)?;
        }
        
        if self.install_options.create_start_menu_shortcut {
            info!("Creating start menu shortcut");
            self.platform.create_start_menu_shortcut(&self.config, &self.install_dir)?;
        }
        
        // 10. 添加到PATH环境变量
        if self.install_options.add_to_path {
            info!("Adding to PATH environment variable");
            self.platform.add_to_path(&self.install_dir)?;
            self.added_to_path = true;
        }
        
        // 11. 创建系统服务
        if self.install_options.create_service {
            info!("Creating system service");
            self.create_service()?;
        }
        
        // 12. 创建卸载程序
        if self.install_options.create_uninstaller {
            info!("Creating uninstaller");
            self.platform.create_uninstaller(&self.config, &self.install_dir)?;
            self.created_uninstaller = true;
        }
        
        // 13. 执行自定义安装后命令
        info!("Running post-install commands");
        self.run_post_install_commands()?;
        
        // 14. 执行后安装脚本
        if let Some(post_script) = &self.install_options.post_install_script {
            info!("Running post-install script");
            execute_command(post_script, Some(&self.install_dir))?;
        }
        
        // 15. 执行插件的post_install方法
        self.run_plugin_post_install()?;
        
        Ok(())
    }
    
    /// 回滚安装
    fn rollback(&mut self) -> Result<()> {
        info!("Performing rollback...");
        
        // 1. 回滚添加到PATH环境变量
        if self.added_to_path {
            info!("Rolling back PATH environment variable");
            if let Err(e) = self.platform.remove_from_path(&self.install_dir) {
                warn!("Failed to rollback PATH: {e:?}");
            }
            self.added_to_path = false;
        }
        
        // 2. 回滚创建卸载程序
        if self.created_uninstaller {
            info!("Rolling back uninstaller");
            if let Err(e) = self.platform.remove_uninstaller(&self.config) {
                warn!("Failed to rollback uninstaller: {e:?}");
            }
            self.created_uninstaller = false;
        }
        
        // 3. 回滚创建快捷方式
        info!("Rolling back shortcuts");
        if let Err(e) = self.platform.remove_shortcuts(&self.config) {
            warn!("Failed to rollback shortcuts: {e:?}");
        }
        
        // 4. 删除已安装的文件
        info!("Rolling back installed files");
        for file_path in &self.installed_files {
            if file_path.exists() {
                if let Err(e) = std::fs::remove_file(file_path) {
                    warn!("Failed to remove file: {file_path:?}, error: {e:?}");
                }
            }
        }
        self.installed_files.clear();
        
        // 5. 删除安装目录
        info!("Rolling back install directory");
        if self.install_dir.exists() {
            if let Err(e) = std::fs::remove_dir_all(&self.install_dir) {
                warn!("Failed to remove install directory: {install_dir:?}, error: {e:?}", install_dir = self.install_dir);
            }
        }
        
        info!("Rollback completed");
        Ok(())
    }
    
    /// 加载插件
    fn load_plugins(&mut self) -> Result<()> {
        debug!("Loading plugins");
        
        if let Some(plugin_configs) = &self.config.plugins {
            for plugin_config in plugin_configs {
                info!("Loading plugin: {0}", plugin_config.name);
                debug!("Plugin path: {0}", plugin_config.path);
                
                // 加载动态库
                let lib = unsafe { Library::new(&plugin_config.path)? };
                
                // 定义插件初始化函数签名
                type PluginInit = unsafe fn() -> *mut dyn Plugin;
                
                // 获取插件初始化函数
                let init: Symbol<PluginInit> = unsafe { lib.get(b"plugin_init")? };
                
                // 调用初始化函数获取插件实例
                let plugin_ptr = unsafe { init() };
                let mut plugin = unsafe { Box::from_raw(plugin_ptr) };
                
                // 初始化插件
                plugin.init(&self.config)?;
                
                // 添加到插件列表
                self.plugins.push(plugin);
                
                info!("Plugin loaded successfully: {0}", plugin_config.name);
            }
        }
        
        Ok(())
    }
    
    /// 执行插件的pre_install方法
    fn run_plugin_pre_install(&mut self) -> Result<()> {
        debug!("Running plugin pre_install methods");
        
        // 临时取出插件列表，避免同时持有self的可变借用
        let mut plugins = std::mem::take(&mut self.plugins);
        
        for plugin in plugins.iter_mut() {
            plugin.pre_install(self)?;
        }
        
        // 将插件列表放回
        self.plugins = plugins;
        
        Ok(())
    }
    
    /// 执行插件的post_install方法
    fn run_plugin_post_install(&mut self) -> Result<()> {
        debug!("Running plugin post_install methods");
        
        // 临时取出插件列表，避免同时持有self的可变借用
        let mut plugins = std::mem::take(&mut self.plugins);
        
        for plugin in plugins.iter_mut() {
            plugin.post_install(self)?;
        }
        
        // 将插件列表放回
        self.plugins = plugins;
        
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
        
        // 解析当前版本和新版本
        let current_version = get_current_version(&self.install_dir)?;
        let new_version = Version::parse(&self.config.project.version)?;
        
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
        
        // 1. 加载插件
        info!("Loading plugins");
        self.load_plugins()?;
        
        // 2. 执行插件的pre_install方法
        self.run_plugin_pre_install()?;
        
        // 3. 执行预安装脚本
        if let Some(pre_script) = &self.install_options.pre_install_script {
            info!("Running pre-install script");
            execute_command(pre_script, Some(&self.temp_dir))?;
        }
        
        // 4. 检查系统要求
        info!("Checking system requirements");
        self.platform.check_system_requirements(&self.config)?;
        
        // 5. 安装依赖
        if let Some(deps) = &self.config.dependencies {
            if !deps.is_empty() {
                info!("Installing dependencies");
                self.install_dependencies()?;
            }
        }
        
        // 6. 安装组件
        info!("Installing components");
        self.install_components()?;
        
        // 7. 复制安装文件
        info!("Copying installation files");
        self.copy_install_files()?;
        
        // 8. 更新快捷方式
        if self.install_options.create_desktop_shortcut {
            info!("Updating desktop shortcut");
            self.platform.remove_shortcuts(&self.config)?;
            self.platform.create_desktop_shortcut(&self.config, &self.install_dir)?;
        }
        
        if self.install_options.create_start_menu_shortcut {
            info!("Updating start menu shortcut");
            self.platform.create_start_menu_shortcut(&self.config, &self.install_dir)?;
        }
        
        // 9. 确保在PATH环境变量中
        if self.install_options.add_to_path {
            info!("Ensuring in PATH environment variable");
            self.platform.add_to_path(&self.install_dir)?;
        }
        
        // 10. 更新服务配置
        if self.install_options.create_service {
            info!("Updating system service");
            self.create_service()?;
        }
        
        // 11. 更新卸载程序
        if self.install_options.create_uninstaller {
            info!("Updating uninstaller");
            self.platform.create_uninstaller(&self.config, &self.install_dir)?;
        }
        
        // 12. 执行自定义安装后命令
        info!("Running post-install commands");
        self.run_post_install_commands()?;
        
        // 13. 执行后安装脚本
        if let Some(post_script) = &self.install_options.post_install_script {
            info!("Running post-install script");
            execute_command(post_script, Some(&self.install_dir))?;
        }
        
        // 14. 保存新的版本号
        save_version(&self.install_dir, new_version)?;
        
        // 15. 执行插件的post_install方法
        self.run_plugin_post_install()?;
        
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
        info!("Removing install directory: {install_dir:?}", install_dir = self.install_dir);
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
        
        // 复制安装文件到目标目录
        // 这里假设安装文件在installer目录下的payload子目录中
        let payload_dir = exe_dir.join("payload");
        if payload_dir.exists() {
            debug!("Copying files from {payload_dir:?} to {install_dir:?}", install_dir = self.install_dir);
            
            // 遍历payload目录下的所有文件
            for entry in std::fs::read_dir(&payload_dir)? {
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
            warn!("Payload directory not found: {payload_dir:?}");
        }
        
        Ok(())
    }
    
    /// 安装依赖
    fn install_dependencies(&self) -> Result<()> {
        if let Some(deps) = &self.config.dependencies {
            for dep in deps {
                info!("Installing dependency: {0}", dep.name);
                debug!("Dependency config: {dep:?}");
                
                if let Some(cmd) = &dep.install_command {
                    debug!("Executing dependency install command: {cmd}");
                    execute_command(cmd, None)?;
                } else {
                    debug!("No install command specified for dependency: {0}", dep.name);
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
