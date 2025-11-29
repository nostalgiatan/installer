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
#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectConfig {
    /// 项目名称
    pub name: String,
    /// 项目版本
    pub version: String,
    /// 项目描述
    pub description: Option<String>,
    /// 项目作者
    pub author: Option<String>,
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
}

/// 自定义命令配置
#[derive(Debug, Deserialize, Serialize)]
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
}

/// 依赖配置
#[derive(Debug, Deserialize, Serialize)]
pub struct DependencyConfig {
    /// 依赖名称
    pub name: String,
    /// 依赖版本
    pub version: String,
    /// 依赖类型: runtime, development
    pub kind: String,
    /// 安装命令
    pub install_command: Option<String>,
}

/// 平台特定配置
#[derive(Debug, Deserialize, Serialize)]
pub struct PlatformConfig {
    /// Windows平台配置
    pub windows: Option<InstallOptions>,
    /// Linux平台配置
    pub linux: Option<InstallOptions>,
    /// macOS平台配置
    pub macos: Option<InstallOptions>,
}

/// 主配置结构
#[derive(Debug, Deserialize, Serialize)]
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
}

/// 加载配置文件
pub fn load_config(config_path: &str) -> Result<Config> {
    debug!("Loading config from: {}", config_path);
    
    // 打开配置文件
    let mut file = File::open(config_path)?;
    
    // 读取文件内容
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    // 解析TOML配置
    let config: Config = toml::from_str(&contents)?;
    
    debug!("Config loaded successfully: {:?}", config);
    
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
            anyhow::bail!("Command name cannot be empty at index {}", index);
        }
        
        if command.program.is_empty() {
            anyhow::bail!("Command program cannot be empty for command '{}'", command.name);
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
        },
        install_options: InstallOptions {
            default_dir: "C:\\Program Files\\SeeSea".to_string(),
            create_desktop_shortcut: true,
            create_start_menu_shortcut: true,
            add_to_path: true,
            create_uninstaller: true,
        },
        platform: None,
        commands: Vec::new(),
        dependencies: None,
    }
}
