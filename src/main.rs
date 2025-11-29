// SeeSea Self-Contained Installer
// 模块名称: seesea-installer
// 职责范围: 提供跨平台的自包含安装程序，支持自定义安装命令
// 期望实现计划: 
// 1. 实现命令行参数解析
// 2. 实现配置管理
// 3. 实现安装逻辑核心功能
// 4. 实现平台特定代码
// 5. 实现日志记录
// 已实现功能: 命令行参数解析框架
// 使用依赖: clap, anyhow, log, env_logger
// 主要接口: main函数
// 注意事项: 支持Windows、Linux、macOS平台

use clap::Parser;
use anyhow::Result;
use log::info;

mod config;
mod cli;
mod installer;
mod platform;
mod utils;

use crate::cli::Args;

fn main() -> Result<()> {
    // 初始化日志
    env_logger::init();
    
    // 解析命令行参数
    let args = Args::parse();
    
    info!("Starting SeeSea Installer v{}", env!("CARGO_PKG_VERSION"));
    info!("Command: {}", args.command);
    info!("Config file: {}", args.config);
    
    // 加载配置
    let config = config::load_config(&args.config)?;
    
    // 创建安装器实例
    let mut installer = installer::Installer::new(config, &args)?;
    
    // 执行命令
    match args.command.as_str() {
        "install" => installer.install()?,
        "uninstall" => installer.uninstall()?,
        "repair" => installer.repair()?,
        _ => anyhow::bail!("Unknown command: {}", args.command),
    }
    
    info!("SeeSea Installer completed successfully");
    Ok(())
}
