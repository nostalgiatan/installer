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

// 明确指定这是一个控制台应用程序
#![cfg_attr(windows, windows_subsystem = "console")]

use clap::Parser;
use anyhow::Result;
use log::{info, error};

mod config;
mod cli;
mod installer;
mod platform;
mod utils;
mod version;

use crate::cli::Args;

fn main() -> Result<()> {
    // 初始化日志
    env_logger::init();
    
    // 打印欢迎信息
    println!("SeeSea Installer v{}", env!("CARGO_PKG_VERSION"));
    println!("Starting installation process...");
    println!("Press Ctrl+C to cancel.");
    println!();
    
    // 解析命令行参数
    let args = Args::parse();
    
    println!("Command: {}", args.command);
    println!("Config file: {}", args.config);
    println!();
    
    info!("Starting SeeSea Installer v{}", env!("CARGO_PKG_VERSION"));
    info!("Command: {}", args.command);
    info!("Config file: {}", args.config);
    
    // 简单测试模式：如果配置文件不存在，只显示控制台窗口
    if let Err(e) = config::load_config(&args.config) {
        println!("警告: 无法加载配置文件: {e}");
        println!("进入测试模式，仅显示控制台窗口...");
        
        // 等待用户输入，防止命令行窗口立即关闭
        println!("\n按任意键退出...");
        let _ = std::io::stdin().read_line(&mut String::new());
        return Ok(());
    }
    
    // 加载配置
    let config = config::load_config(&args.config)?;
    
    // 创建安装器实例
    let mut installer = installer::Installer::new(config, &args)?;
    
    // 执行命令
    let result = match args.command.as_str() {
        "install" => {
            println!("是否继续安装？(y/n/update): ");
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).expect("无法读取输入");
            
            let input = input.trim().to_lowercase();
            if input == "y" || input == "yes" {
                installer.install()
            } else if input == "update" {
                println!("执行更新操作...");
                installer.update()
            } else {
                println!("安装已取消");
                Ok(())
            }
        },
        "uninstall" => installer.uninstall(),
        "repair" => installer.repair(),
        "update" => installer.update(),
        _ => anyhow::bail!("Unknown command: {}", args.command),
    };
    
    // 处理执行结果
    match &result {
        Ok(_) => {
            info!("SeeSea Installer completed successfully");
        },
        Err(e) => {
            error!("SeeSea Installer failed with error: {e}");
            println!("\n错误: {e}");
        }
    }
    
    // 等待用户输入，防止命令行窗口立即关闭
    println!("\nPress any key to exit...");
    let _ = std::io::stdin().read_line(&mut String::new());
    
    result
}
