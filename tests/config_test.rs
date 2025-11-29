// SeeSea Installer - Config Module Tests
// 测试配置管理模块的功能

use seesea_installer::config;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_load_config() {
    // 创建临时目录
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("installer.toml");
    
    // 创建测试配置文件
        let config_content = r#"
[project]
name = "test-project"
version = "1.0.0"
description = "Test project"
author = "Test Author"

[install_options]
default_dir = "/opt/test"
create_desktop_shortcut = true
create_start_menu_shortcut = true
add_to_path = true
create_uninstaller = true
silent = false
create_service = false
auto_check_updates = true
update_channel = "stable"
backup_enabled = true
backup_retention = 5

[[commands]]
name = "test-command"
program = "/bin/echo"
args = ["hello", "world"]
working_dir = "/tmp"
background = false
ignore_errors = false
"#;
    
    let mut file = File::create(&config_path).unwrap();
    file.write_all(config_content.as_bytes()).unwrap();
    
    // 加载配置
    let config = config::load_config(config_path.to_str().unwrap()).unwrap();
    
    // 验证配置
    assert_eq!(config.project.name, "test-project");
    assert_eq!(config.project.version, "1.0.0");
    assert_eq!(config.project.description.unwrap(), "Test project");
    assert_eq!(config.project.author.unwrap(), "Test Author");
    
    assert_eq!(config.install_options.default_dir, "/opt/test");
    assert!(config.install_options.create_desktop_shortcut);
    assert!(config.install_options.create_start_menu_shortcut);
    assert!(config.install_options.add_to_path);
    assert!(config.install_options.create_uninstaller);
    
    assert_eq!(config.commands.len(), 1);
    assert_eq!(config.commands[0].name, "test-command");
    assert_eq!(config.commands[0].program, "/bin/echo");
    assert_eq!(config.commands[0].args, ["hello", "world"]);
    assert_eq!(config.commands[0].working_dir.clone().unwrap(), "/tmp");
    assert!(!config.commands[0].background);
}

#[test]
fn test_validate_config() {
    // 创建无效配置（缺少项目名称）
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("invalid_config.toml");
    
    let config_content = r#"
[project]
version = "1.0.0"

[install_options]
default_dir = "/opt/test"
create_desktop_shortcut = true
create_start_menu_shortcut = true
add_to_path = true
create_uninstaller = true
silent = false
create_service = false
auto_check_updates = true
update_channel = "stable"
backup_enabled = true
backup_retention = 5

[commands]
"#;
    let mut file = File::create(&config_path).unwrap();
    file.write_all(config_content.as_bytes()).unwrap();
    
    // 验证无效配置应该失败
    let result = config::load_config(config_path.to_str().unwrap());
    assert!(result.is_err());
}

#[test]
fn test_generate_default_config() {
    // 生成默认配置
    let default_config = config::generate_default_config();
    
    // 验证默认配置
    assert_eq!(default_config.project.name, "seesea");
    assert_eq!(default_config.project.version, "1.0.0");
    assert_eq!(default_config.project.description.unwrap(), "SeeSea Project");
    assert!(default_config.project.author.is_none());
    
    assert_eq!(default_config.install_options.default_dir, "C:\\Program Files\\SeeSea");
    assert!(default_config.install_options.create_desktop_shortcut);
    assert!(default_config.install_options.create_start_menu_shortcut);
    assert!(default_config.install_options.add_to_path);
    assert!(default_config.install_options.create_uninstaller);
    
    assert!(default_config.commands.is_empty());
    assert!(default_config.dependencies.is_none());
}

#[test]
fn test_platform_specific_config() {
    // 创建包含平台特定配置的测试文件
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("platform_config.toml");
    
    let config_content = r#"
[project]
name = "test-platform"
version = "1.0.0"

[install_options]
default_dir = "/opt/test"
create_desktop_shortcut = false
create_start_menu_shortcut = false
add_to_path = false
create_uninstaller = false
silent = false
create_service = false
auto_check_updates = true
update_channel = "stable"
backup_enabled = true
backup_retention = 5

[platform.windows]
default_dir = "C:\\Program Files\\Test"
create_desktop_shortcut = true
create_start_menu_shortcut = true
add_to_path = true
create_uninstaller = true
silent = true
create_service = false
auto_check_updates = true
update_channel = "stable"
backup_enabled = true
backup_retention = 5

[platform.linux]
default_dir = "/usr/local/test"
create_desktop_shortcut = true
create_start_menu_shortcut = true
add_to_path = true
create_uninstaller = true
silent = true
create_service = false
auto_check_updates = true
update_channel = "stable"
backup_enabled = true
backup_retention = 5

[platform.macos]
default_dir = "/Applications/Test"
create_desktop_shortcut = true
create_start_menu_shortcut = true
add_to_path = true
create_uninstaller = true
silent = true
create_service = false
auto_check_updates = true
update_channel = "stable"
backup_enabled = true
backup_retention = 5

# 空的commands数组
[[commands]]
name = "placeholder"
program = "/bin/true"
args = []
background = false
ignore_errors = false
"#;
    
    let mut file = File::create(&config_path).unwrap();
    file.write_all(config_content.as_bytes()).unwrap();
    
    // 加载配置
    let config = config::load_config(config_path.to_str().unwrap()).unwrap();
    
    // 验证全局配置
    assert_eq!(config.project.name, "test-platform");
    assert_eq!(config.install_options.default_dir, "/opt/test");
    assert!(!config.install_options.create_desktop_shortcut);
    
    // 验证平台特定配置存在
    assert!(config.platform.is_some());
    let platform_config = config.platform.unwrap();
    
    // 验证Windows配置
    assert!(platform_config.windows.is_some());
    let windows_config = platform_config.windows.unwrap();
    assert_eq!(windows_config.default_dir, "C:\\Program Files\\Test");
    assert!(windows_config.create_desktop_shortcut);
    
    // 验证Linux配置
    assert!(platform_config.linux.is_some());
    let linux_config = platform_config.linux.unwrap();
    assert_eq!(linux_config.default_dir, "/usr/local/test");
    assert!(linux_config.create_desktop_shortcut);
    
    // 验证macOS配置
    assert!(platform_config.macos.is_some());
    let macos_config = platform_config.macos.unwrap();
    assert_eq!(macos_config.default_dir, "/Applications/Test");
    assert!(macos_config.create_desktop_shortcut);
}
