// SeeSea Installer - Version Module Tests
// 测试版本管理模块的功能

use seesea_installer::version::{Version, get_current_version, save_version, check_update};
use tempfile::tempdir;

#[test]
fn test_version_parse() {
    // 测试正常版本号解析
    let version = Version::parse("1.0.0").unwrap();
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 0);
    assert_eq!(version.patch, 0);
    assert_eq!(version.pre_release, None);
    
    // 测试带有预发布版本的解析
    let version = Version::parse("2.1.3-beta").unwrap();
    assert_eq!(version.major, 2);
    assert_eq!(version.minor, 1);
    assert_eq!(version.patch, 3);
    assert_eq!(version.pre_release, Some("beta".to_string()));
    
    // 测试带有构建元数据的解析
    let version = Version::parse("3.2.1-alpha.1").unwrap();
    assert_eq!(version.major, 3);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 1);
    assert_eq!(version.pre_release, Some("alpha.1".to_string()));
}

#[test]
fn test_version_compare() {
    // 测试相同版本
    let v1 = Version::parse("1.0.0").unwrap();
    let v2 = Version::parse("1.0.0").unwrap();
    assert_eq!(v1.compare(&v2), 0);
    
    // 测试主版本不同
    let v1 = Version::parse("1.0.0").unwrap();
    let v2 = Version::parse("2.0.0").unwrap();
    assert_eq!(v1.compare(&v2), -1);
    assert_eq!(v2.compare(&v1), 1);
    
    // 测试次版本不同
    let v1 = Version::parse("1.0.0").unwrap();
    let v2 = Version::parse("1.1.0").unwrap();
    assert_eq!(v1.compare(&v2), -1);
    assert_eq!(v2.compare(&v1), 1);
    
    // 测试修订版本不同
    let v1 = Version::parse("1.0.0").unwrap();
    let v2 = Version::parse("1.0.1").unwrap();
    assert_eq!(v1.compare(&v2), -1);
    assert_eq!(v2.compare(&v1), 1);
    
    // 测试预发布版本比较
    let v1 = Version::parse("1.0.0-beta").unwrap();
    let v2 = Version::parse("1.0.0").unwrap();
    assert_eq!(v1.compare(&v2), -1);
    assert_eq!(v2.compare(&v1), 1);
    
    // 测试不同预发布版本比较
    let v1 = Version::parse("1.0.0-alpha").unwrap();
    let v2 = Version::parse("1.0.0-beta").unwrap();
    assert_eq!(v1.compare(&v2), -1);
    assert_eq!(v2.compare(&v1), 1);
}

#[test]
fn test_version_to_string() {
    // 测试正常版本号转换
    let version = Version::parse("1.0.0").unwrap();
    assert_eq!(version.to_string(), "1.0.0");
    
    // 测试带有预发布版本的转换
    let version = Version::parse("2.1.3-beta").unwrap();
    assert_eq!(version.to_string(), "2.1.3-beta");
    
    // 测试带有构建元数据的转换
    let version = Version::parse("3.2.1-alpha.1").unwrap();
    assert_eq!(version.to_string(), "3.2.1-alpha.1");
}

#[test]
fn test_version_file_operations() {
    // 创建临时目录
    let temp_dir = tempdir().unwrap();
    let install_dir = temp_dir.path();
    
    // 测试获取不存在的版本文件
    let current_version = get_current_version(install_dir).unwrap();
    assert_eq!(current_version, None);
    
    // 测试保存版本
    let version = Version::parse("1.0.0").unwrap();
    save_version(install_dir, &version).unwrap();
    
    // 测试获取版本
    let current_version = get_current_version(install_dir).unwrap();
    assert_eq!(current_version, Some(version.clone()));
    
    // 测试更新版本
    let new_version = Version::parse("1.1.0").unwrap();
    save_version(install_dir, &new_version).unwrap();
    
    // 测试获取更新后的版本
    let current_version = get_current_version(install_dir).unwrap();
    assert_eq!(current_version, Some(new_version));
}

#[test]
fn test_check_update() {
    // 测试没有当前版本时需要更新
    let current_version = None;
    let new_version = Version::parse("1.0.0").unwrap();
    assert!(check_update(current_version, &new_version, false));
    
    // 测试当前版本低于新版本时需要更新
    let current_version = Some(Version::parse("1.0.0").unwrap());
    let new_version = Version::parse("1.1.0").unwrap();
    assert!(check_update(current_version, &new_version, false));
    
    // 测试当前版本等于新版本时不需要更新
    let current_version = Some(Version::parse("1.0.0").unwrap());
    let new_version = Version::parse("1.0.0").unwrap();
    assert!(!check_update(current_version, &new_version, false));
    
    // 测试当前版本高于新版本时不需要更新
    let current_version = Some(Version::parse("1.1.0").unwrap());
    let new_version = Version::parse("1.0.0").unwrap();
    assert!(!check_update(current_version, &new_version, false));
    
    // 测试强制更新
    let current_version = Some(Version::parse("1.1.0").unwrap());
    let new_version = Version::parse("1.0.0").unwrap();
    assert!(check_update(current_version, &new_version, true));
}
