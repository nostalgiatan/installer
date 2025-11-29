// SeeSea Self-Contained Installer - Version Module
// 模块名称: version
// 职责范围: 处理版本管理相关功能，包括版本检测、比较和验证
// 期望实现计划:
// 1. 实现版本号解析
// 2. 实现版本号比较
// 3. 实现版本检测功能
// 4. 实现版本验证功能
// 已实现功能: 版本号解析和比较
// 使用依赖: anyhow, log, std::fs
// 主要接口: Version::parse, Version::compare, get_current_version
// 注意事项: 支持语义化版本号格式，如1.0.0, 2.1.3-beta

use anyhow::Result;
use log::debug;
use std::fmt;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

/// 版本号结构体
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Version {
    /// 主版本号
    pub major: u32,
    /// 次版本号
    pub minor: u32,
    /// 修订版本号
    pub patch: u32,
    /// 预发布版本标识符
    pub pre_release: Option<String>,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.pre_release {
            Some(pre) => write!(f, "{}.{}.{}-{}", self.major, self.minor, self.patch, pre),
            None => write!(f, "{}.{}.{}", self.major, self.minor, self.patch),
        }
    }
}

impl Version {
    /// 解析版本号字符串
    pub fn parse(version_str: &str) -> Result<Self> {
        debug!("Parsing version string: {version_str}");
        
        // 先按 '-' 分割主版本号和预发布版本
        let version_parts: Vec<&str> = version_str.splitn(2, '-').collect();
        let main_version = version_parts[0];
        let pre_release = version_parts.get(1).map(|s| s.to_string());
        
        // 分割主版本号的各个部分
        let main_parts: Vec<&str> = main_version.split('.').collect();
        
        if main_parts.len() < 3 {
            anyhow::bail!("Invalid version format: {version_str}");
        }
        
        let major = main_parts[0].parse::<u32>()?;
        let minor = main_parts[1].parse::<u32>()?;
        let patch = main_parts[2].parse::<u32>()?;
        
        Ok(Self {
            major,
            minor,
            patch,
            pre_release,
        })
    }
    
    /// 比较两个版本号
    pub fn compare(&self, other: &Self) -> i32 {
        if self.major > other.major {
            return 1;
        } else if self.major < other.major {
            return -1;
        }
        
        if self.minor > other.minor {
            return 1;
        } else if self.minor < other.minor {
            return -1;
        }
        
        if self.patch > other.patch {
            return 1;
        } else if self.patch < other.patch {
            return -1;
        }
        
        // 比较预发布版本
        match (&self.pre_release, &other.pre_release) {
            (None, None) => 0,
            (Some(_), None) => -1, // 正式版本比预发布版本新
            (None, Some(_)) => 1,  // 正式版本比预发布版本新
            (Some(a), Some(b)) => {
                if a < b {
                    -1
                } else if a > b {
                    1
                } else {
                    0
                }
            }
        }
    }
}

/// 获取当前安装的版本号
pub fn get_current_version(install_dir: &Path) -> Result<Option<Version>> {
    let version_file = install_dir.join("version.txt");
    
    if !version_file.exists() {
        debug!("Version file not found: {version_file:?}");
        return Ok(None);
    }
    
    let mut file = File::open(version_file)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    
    let version_str = content.trim();
    if version_str.is_empty() {
        debug!("Version file is empty");
        return Ok(None);
    }
    
    let version = Version::parse(version_str)?;
    debug!("Current version: {version:?}");
    
    Ok(Some(version))
}

/// 保存版本号到安装目录
pub fn save_version(install_dir: &Path, version: &Version) -> Result<()> {
    let version_file = install_dir.join("version.txt");
    
    let mut file = File::create(version_file)?;
    file.write_all(version.to_string().as_bytes())?;
    
    debug!("Saved version: {version} to version.txt");
    Ok(())
}

/// 检查版本是否需要更新
pub fn check_update(current_version: Option<Version>, new_version: &Version, force: bool) -> bool {
    if force {
        debug!("Forcing update, ignoring version check");
        return true;
    }
    
    match current_version {
        None => {
            debug!("No current version found, proceeding with update");
            true
        }
        Some(current) => {
            let comparison = current.compare(new_version);
            debug!("Current version: {current}, New version: {new_version}, Comparison result: {comparison}");
            comparison < 0
        }
    }
}
