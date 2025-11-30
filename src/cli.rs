// SeeSea Self-Contained Installer - CLI Module
// 模块名称: cli
// 职责范围: 处理命令行参数解析
// 已实现功能: Args结构体定义
// 使用依赖: clap
// 主要接口: Args结构体
// 注意事项: 用于命令行参数解析

use clap::Parser;

/// 命令行参数结构体
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// 安装配置文件路径
    #[arg(short, long, default_value = "install.toml")]
    pub config: String,
    
    /// 安装目录
    #[arg(short, long)]
    pub install_dir: Option<String>,
    
    /// 启用调试日志
    #[arg(short, long)]
    pub debug: bool,
    
    /// 执行的命令: install, uninstall, repair, update
    #[arg(default_value = "install")]
    pub command: String,
    
    /// 仅检查更新，不执行更新
    #[arg(long)]
    pub check: bool,
    
    /// 指定备份目录
    #[arg(long)]
    pub backup_dir: Option<String>,
    
    /// 强制更新，忽略版本检查
    #[arg(long)]
    pub force: bool,
}
