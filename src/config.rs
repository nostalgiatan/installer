// SeeSea Self-Contained Installer - Config Module
// 模块名称: config
// 职责范围: 处理安装配置文件的加载、解析和验证
// 期望实现计划: 
// 1. 定义配置结构
// 2. 实现配置文件加载
// 3. 实现配置验证
// 4. 实现默认配置生成
// 已实现功能: 配置结构定义、配置文件加载
// 使用依赖: toml, serde, anyhow, std::fs
// 主要接口: load_config, Config struct
// 注意事项: 配置文件使用TOML格式，支持平台特定配置

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use anyhow::Result;
use log::debug;

/// 项目信息配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProjectConfig {
    /// 项目名称
    pub name: String,
    /// 项目版本
    pub version: String,
    /// 项目描述
    pub description: Option<String>,
    /// 项目作者
    pub author: Option<String>,
    /// 项目主页
    pub homepage: Option<String>,
    /// 许可证
    pub license: Option<String>,
}

/// 安装选项配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InstallOptions {
    /// 默认安装目录
    pub default_dir: String,
    /// 是否创建桌面快捷方式
    pub create_desktop_shortcut: bool,
    /// 是否创建开始菜单快捷方式
    pub create_start_menu_shortcut: bool,
    /// 是否添加到PATH环境变量
    pub add_to_path: bool,
    /// 是否创建卸载程序
    pub create_uninstaller: bool,
    /// 是否启用静默安装
    pub silent: bool,
    /// 默认安装语言
    pub language: Option<String>,
    /// 日志级别
    pub log_level: Option<String>,
    /// 安装组件列表
    pub components: Option<Vec<String>>,
    /// 预安装脚本
    pub pre_install_script: Option<String>,
    /// 后安装脚本
    pub post_install_script: Option<String>,
    /// 预卸载脚本
    pub pre_uninstall_script: Option<String>,
    /// 后卸载脚本
    pub post_uninstall_script: Option<String>,
    /// 是否创建系统服务
    pub create_service: bool,
    /// 服务名称
    pub service_name: Option<String>,
    /// 服务描述
    pub service_description: Option<String>,
    /// 是否自动检查更新
    pub auto_check_updates: bool,
    /// 更新通道（stable, beta, alpha）
    pub update_channel: Option<String>,
    /// 更新时需要保留的配置文件列表
    pub preserve_configs: Option<Vec<String>>,
    /// 是否启用自动备份
    pub backup_enabled: bool,
    /// 备份保留数量
    pub backup_retention: Option<u32>,
}

/// 自定义命令配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CommandConfig {
    /// 命令名称
    pub name: String,
    /// 命令描述
    pub description: Option<String>,
    /// 命令执行的程序路径
    pub program: String,
    /// 命令参数
    pub args: Vec<String>,
    /// 工作目录
    pub working_dir: Option<String>,
    /// 是否在后台执行
    pub background: bool,
    /// 环境变量
    pub env: Option<Vec<(String, String)>>,
    /// 执行用户
    pub user: Option<String>,
    /// 执行组
    pub group: Option<String>,
    /// 超时时间（秒）
    pub timeout: Option<u32>,
    /// 重试次数
    pub retries: Option<u32>,
    /// 重试间隔（秒）
    pub retry_delay: Option<u32>,
    /// 成功退出码
    pub success_codes: Option<Vec<i32>>,
    /// 是否忽略错误
    pub ignore_errors: bool,
}

/// 依赖配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DependencyConfig {
    /// 依赖名称
    pub name: String,
    /// 依赖版本
    pub version: String,
    /// 依赖类型: runtime, development, optional
    pub kind: String,
    /// 安装命令
    pub install_command: Option<String>,
    /// 卸载命令
    pub uninstall_command: Option<String>,
    /// 检查命令
    pub check_command: Option<String>,
    /// 依赖的平台
    pub platforms: Option<Vec<String>>,
    /// 依赖的组件
    pub components: Option<Vec<String>>,
    /// 是否可选
    pub optional: bool,
    /// 依赖的其他依赖
    pub depends_on: Option<Vec<String>>,
}

/// 平台特定配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PlatformConfig {
    /// Windows平台默认安装目录
    pub windows_default_dir: Option<String>,
    /// Linux平台默认安装目录
    pub linux_default_dir: Option<String>,
    /// macOS平台默认安装目录
    pub macos_default_dir: Option<String>,
    /// FreeBSD平台默认安装目录
    pub freebsd_default_dir: Option<String>,
    /// NetBSD平台默认安装目录
    pub netbsd_default_dir: Option<String>,
    /// OpenBSD平台默认安装目录
    pub openbsd_default_dir: Option<String>,
}

/// 组件配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ComponentConfig {
    /// 组件名称
    pub name: String,
    /// 组件描述
    pub description: Option<String>,
    /// 组件版本
    pub version: Option<String>,
    /// 组件是否默认安装
    pub default: bool,
    /// 组件的文件列表
    pub files: Option<Vec<String>>,
    /// 组件的依赖
    pub depends_on: Option<Vec<String>>,
    /// 组件的平台
    pub platforms: Option<Vec<String>>,
}

/// 插件配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PluginConfig {
    /// 插件名称
    pub name: String,
    /// 插件路径
    pub path: String,
    /// 插件配置
    pub config: Option<toml::Value>,
}

/// 主配置结构
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    /// 项目信息
    pub project: ProjectConfig,
    /// 全局安装选项
    pub install_options: InstallOptions,
    /// 平台特定配置
    pub platform: Option<PlatformConfig>,
    /// 自定义命令列表
    pub commands: Vec<CommandConfig>,
    /// 依赖列表
    pub dependencies: Option<Vec<DependencyConfig>>,
    /// 组件列表
    pub components: Option<Vec<ComponentConfig>>,
    /// 插件列表
    pub plugins: Option<Vec<PluginConfig>>,
    /// 安装程序版本
    pub installer_version: Option<String>,
    /// 安装程序名称
    pub installer_name: Option<String>,
}

/// 加载配置文件
pub fn load_config(config_path: &str) -> Result<Config> {
    debug!("Loading config from: {config_path}");
    
    // 尝试多种路径查找配置文件
    let mut paths_to_try = Vec::new();
    paths_to_try.push(config_path.to_string());
    paths_to_try.push(format!("../{config_path}"));
    paths_to_try.push(format!("../../{config_path}"));
    
    let mut file = None;
    let mut used_path = String::new();
    
    for path in &paths_to_try {
        debug!("Trying config path: {path}");
        if let Ok(f) = File::open(path) {
            file = Some(f);
            used_path = path.clone();
            break;
        }
    }
    
    let mut file = file.ok_or_else(|| anyhow::anyhow!("Could not find config file at any of the tried paths: {:?}", paths_to_try))?;
    
    debug!("Found config file at: {used_path}");
    
    // 读取文件内容
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    // 解析TOML配置
    let config: Config = toml::from_str(&contents)?;
    
    debug!("Config loaded successfully: {config:?}");
    
    // 验证配置
    validate_config(&config)?;
    
    Ok(config)
}

/// 验证配置
fn validate_config(config: &Config) -> Result<()> {
    // 验证项目名称和版本
    if config.project.name.is_empty() {
        anyhow::bail!("Project name cannot be empty");
    }
    
    if config.project.version.is_empty() {
        anyhow::bail!("Project version cannot be empty");
    }
    
    // 验证安装选项
    if config.install_options.default_dir.is_empty() {
        anyhow::bail!("Default install directory cannot be empty");
    }
    
    // 验证命令配置
    for (index, command) in config.commands.iter().enumerate() {
        if command.name.is_empty() {
            anyhow::bail!("Command name cannot be empty at index {index}");
        }
        
        if command.program.is_empty() {
            anyhow::bail!("Command program cannot be empty for command '{0}'", command.name);
        }
        
        // 验证命令超时时间
        if let Some(timeout) = command.timeout {
            if timeout == 0 {
                anyhow::bail!("Command timeout cannot be zero for command '{0}'", command.name);
            }
        }
        
        // 验证命令重试次数
        if let Some(retries) = command.retries {
            if retries > 10 {
                anyhow::bail!("Command retries cannot exceed 10 for command '{0}'", command.name);
            }
        }
    }
    
    // 验证组件配置
    if let Some(components) = &config.components {
        // 检查组件名称唯一性
        let mut component_names = std::collections::HashSet::new();
        for component in components {
            if !component_names.insert(&component.name) {
                anyhow::bail!("Duplicate component name: {0}", component.name);
            }
            
            // 检查组件依赖是否存在
            if let Some(depends_on) = &component.depends_on {
                for dep in depends_on {
                    let dep_exists = components.iter().any(|c| c.name == *dep);
                    if !dep_exists {
                        anyhow::bail!("Component {0} depends on non-existent component: {1}", component.name, dep);
                    }
                }
            }
        }
    }
    
    // 验证插件配置
    if let Some(plugins) = &config.plugins {
        // 检查插件名称唯一性
        let mut plugin_names = std::collections::HashSet::new();
        for plugin in plugins {
            if !plugin_names.insert(&plugin.name) {
                anyhow::bail!("Duplicate plugin name: {0}", plugin.name);
            }
            
            // 检查插件路径是否存在
            if !std::path::Path::new(&plugin.path).exists() {
                anyhow::bail!("Plugin path does not exist: {0}", plugin.path);
            }
        }
    }
    
    // 验证依赖配置
    if let Some(dependencies) = &config.dependencies {
        // 检查依赖名称唯一性
        let mut dependency_names = std::collections::HashSet::new();
        for dependency in dependencies {
            if !dependency_names.insert(&dependency.name) {
                anyhow::bail!("Duplicate dependency name: {0}", dependency.name);
            }
        }
    }
    
    Ok(())
}

/// 生成默认配置
pub fn generate_default_config() -> Config {
    Config {
        project: ProjectConfig {
            name: "seesea".to_string(),
            version: "1.0.0".to_string(),
            description: Some("SeeSea Project".to_string()),
            author: None,
            homepage: None,
            license: Some("MIT".to_string()),
        },
        install_options: InstallOptions {
            default_dir: "C:\\Program Files\\SeeSea".to_string(),
            create_desktop_shortcut: true,
            create_start_menu_shortcut: true,
            add_to_path: true,
            create_uninstaller: true,
            silent: false,
            language: Some("en".to_string()),
            log_level: Some("info".to_string()),
            components: None,
            pre_install_script: None,
            post_install_script: None,
            pre_uninstall_script: None,
            post_uninstall_script: None,
            create_service: false,
            service_name: None,
            service_description: None,
            auto_check_updates: true,
            update_channel: Some("stable".to_string()),
            preserve_configs: None,
            backup_enabled: true,
            backup_retention: Some(5),
        },
        platform: None,
        commands: Vec::new(),
        dependencies: None,
        components: None,
        plugins: None,
        installer_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        installer_name: Some("seesea-installer".to_string()),
    }
}
