// SeeSea Self-Contained Installer Library
// 模块名称: seesea-installer
// 职责范围: 提供安装器的公共API
// 已实现功能: 配置管理模块导出
// 使用依赖: config, installer, platform, utils
// 主要接口: config, installer, platform, utils模块
// 注意事项: 用于集成测试和外部调用

pub mod cli;
pub mod config;
pub mod installer;
pub mod packager;
pub mod platform;
pub mod utils;

// 重新导出主要类型和函数
pub use cli::Args;
pub use config::{Config, InstallOptions, load_config, generate_default_config};
pub use installer::Installer;
pub use packager::{pack_directory, unpack_directory, compress_file, decompress_file};
pub use platform::PlatformImpl;
