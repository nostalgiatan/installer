// SeeSea Self-Contained Installer - Utils Module
// 模块名称: utils
// 职责范围: 提供安装器所需的通用工具函数
// 期望实现计划: 
// 1. 实现目录创建功能
// 2. 实现文件复制功能
// 3. 实现命令执行功能
// 4. 实现文件权限设置功能
// 5. 实现日志辅助功能
// 已实现功能: 目录创建、文件复制、命令执行
// 使用依赖: anyhow, log, std::fs, std::process, std::path, walkdir, fs_extra
// 主要接口: create_directory, copy_files, execute_command
// 注意事项: 支持跨平台，处理不同平台的路径格式

use anyhow::Result;
use log::{debug, error};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use walkdir::WalkDir;
use fs_extra::dir::CopyOptions;

/// 创建目录，如果目录已存在则忽略
pub fn create_directory(path: &Path) -> Result<()> {
    debug!("Creating directory: {:?}", path);
    
    if !path.exists() {
        fs::create_dir_all(path)?;
        debug!("Directory created: {:?}", path);
    } else {
        debug!("Directory already exists: {:?}", path);
    }
    
    Ok(())
}

/// 复制文件和目录
pub fn copy_files(source: &Path, destination: &Path) -> Result<()> {
    debug!("Copying files from {:?} to {:?}", source, destination);
    
    if !source.exists() {
        anyhow::bail!("Source path does not exist: {:?}", source);
    }
    
    // 创建目标目录
    create_directory(destination)?;
    
    // 设置复制选项
    let mut options = CopyOptions::new();
    options.overwrite = true;
    options.skip_exist = false;
    options.copy_inside = true;
    
    // 复制文件
    fs_extra::dir::copy(source, destination, &options)?;
    
    debug!("Files copied successfully");
    
    Ok(())
}

/// 执行命令
pub fn execute_command(command: &str, working_dir: Option<&Path>) -> Result<()> {
    debug!("Executing command: {}", command);
    if let Some(dir) = working_dir {
        debug!("Working directory: {:?}", dir);
    }
    
    // 在Windows上使用cmd.exe执行命令，在Unix上使用sh执行命令
    let (shell, shell_args) = if cfg!(windows) {
        ("cmd.exe", ["/C", command])
    } else {
        ("sh", ["-c", command])
    };
    
    let mut cmd = Command::new(shell);
    cmd.args(&shell_args);
    
    // 设置工作目录
    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }
    
    // 设置输出
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());
    
    // 执行命令
    let status = cmd.status()?;
    
    if !status.success() {
        error!("Command failed with status: {:?}", status);
        anyhow::bail!("Command execution failed: {}", command);
    }
    
    debug!("Command executed successfully");
    
    Ok(())
}

/// 获取文件大小
pub fn get_file_size(path: &Path) -> Result<u64> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.len())
}

/// 检查文件是否存在
pub fn file_exists(path: &Path) -> bool {
    path.exists()
}

/// 删除文件
pub fn delete_file(path: &Path) -> Result<()> {
    debug!("Deleting file: {:?}", path);
    
    if path.exists() {
        fs::remove_file(path)?;
        debug!("File deleted: {:?}", path);
    } else {
        debug!("File does not exist: {:?}", path);
    }
    
    Ok(())
}

/// 删除目录
pub fn delete_directory(path: &Path) -> Result<()> {
    debug!("Deleting directory: {:?}", path);
    
    if path.exists() {
        fs::remove_dir_all(path)?;
        debug!("Directory deleted: {:?}", path);
    } else {
        debug!("Directory does not exist: {:?}", path);
    }
    
    Ok(())
}

/// 列出目录中的所有文件
pub fn list_files(path: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    for entry in WalkDir::new(path) {
        let entry = entry?;
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }
    
    Ok(files)
}

/// 替换文件中的字符串
pub fn replace_in_file(path: &Path, from: &str, to: &str) -> Result<()> {
    debug!("Replacing '{}' with '{}' in file: {:?}", from, to, path);
    
    let content = fs::read_to_string(path)?;
    let new_content = content.replace(from, to);
    fs::write(path, new_content)?;
    
    debug!("String replaced successfully");
    
    Ok(())
}
