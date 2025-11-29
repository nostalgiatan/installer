// SeeSea Self-Contained Installer - Packager Module
// 模块名称: packager
// 职责范围: 处理安装文件的zstd打包和解包
// 期望实现计划: 
// 1. 实现zstd压缩功能
// 2. 实现zstd解压功能
// 3. 实现目录打包功能
// 4. 实现目录解压功能
// 已实现功能: zstd压缩和解压功能
// 使用依赖: zstd, anyhow, log, std::fs, std::path, walkdir
// 主要接口: pack_directory, unpack_directory
// 注意事项: 使用zstd算法进行高效压缩

use anyhow::Result;
use log::{debug, info};
use std::fs::{File, create_dir_all};
use std::io::{Read, Write};
use std::path::Path;
use walkdir::WalkDir;
use zstd::stream::{Encoder, Decoder};

/// 打包目录为zstd压缩文件
pub fn pack_directory(source_dir: &Path, output_file: &Path) -> Result<()> {
    info!("Packaging directory {source_dir:?} to {output_file:?} using zstd");
    
    // 创建输出文件
    let output = File::create(output_file)?;
    
    // 创建zstd编码器
    let mut encoder = Encoder::new(output, 19)?; // 使用最高压缩级别
    
    // 遍历目录并添加文件
    let mut file_count = 0;
    for entry in WalkDir::new(source_dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let file_path = entry.path();
            let relative_path = file_path.strip_prefix(source_dir)?;
            
            // 写入文件路径长度和路径
            let path_str = relative_path.to_string_lossy();
            let path_len = path_str.len() as u32;
            encoder.write_all(&path_len.to_le_bytes())?;
            encoder.write_all(path_str.as_bytes())?;
            
            // 写入文件内容
            let mut file = File::open(file_path)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            
            // 写入文件大小
            let file_size = buffer.len() as u64;
            encoder.write_all(&file_size.to_le_bytes())?;
            
            // 写入文件内容
            encoder.write_all(&buffer)?;
            
            file_count += 1;
            debug!("Added file: {relative_path:?}");
        }
    }
    
    // 完成编码
    encoder.finish()?;
    
    info!("Successfully packaged {file_count} files to {output_file:?}");
    Ok(())
}

/// 从zstd压缩文件解压到目录
pub fn unpack_directory(input_file: &Path, output_dir: &Path) -> Result<()> {
    info!("Unpacking {input_file:?} to {output_dir:?} using zstd");
    
    // 创建输出目录
    create_dir_all(output_dir)?;
    
    // 打开输入文件
    let input = File::open(input_file)?;
    
    // 创建zstd解码器
    let mut decoder = Decoder::new(input)?;
    
    // 读取并解压文件
    let mut file_count = 0;
    loop {
        // 读取文件路径长度
        let mut path_len_buf = [0u8; 4];
        let read_len = decoder.read(&mut path_len_buf)?;
        if read_len == 0 {
            break; // 文件结束
        }
        
        let path_len = u32::from_le_bytes(path_len_buf) as usize;
        
        // 读取文件路径
        let mut path_buf = vec![0u8; path_len];
        decoder.read_exact(&mut path_buf)?;
        let path_str = String::from_utf8(path_buf)?;
        let file_path = output_dir.join(&path_str);
        
        // 创建父目录
        if let Some(parent) = file_path.parent() {
            create_dir_all(parent)?;
        }
        
        // 读取文件大小
        let mut file_size_buf = [0u8; 8];
        decoder.read_exact(&mut file_size_buf)?;
        let file_size = u64::from_le_bytes(file_size_buf) as usize;
        
        // 读取文件内容
        let mut file_content = vec![0u8; file_size];
        decoder.read_exact(&mut file_content)?;
        
        // 写入文件
        let mut output_file = File::create(&file_path)?;
        output_file.write_all(&file_content)?;
        
        file_count += 1;
        debug!("Extracted file: {file_path:?}");
    }
    
    info!("Successfully unpacked {file_count} files to {output_dir:?}");
    Ok(())
}

/// 压缩单个文件为zstd格式
pub fn compress_file(input_file: &Path, output_file: &Path) -> Result<()> {
    info!("Compressing file {input_file:?} to {output_file:?} using zstd");
    
    // 打开输入文件
    let mut input = File::open(input_file)?;
    
    // 创建输出文件
    let output = File::create(output_file)?;
    
    // 创建zstd编码器
    let mut encoder = Encoder::new(output, 19)?; // 使用最高压缩级别
    
    // 复制文件内容
    let mut buffer = Vec::new();
    input.read_to_end(&mut buffer)?;
    encoder.write_all(&buffer)?;
    
    // 完成编码
    encoder.finish()?;
    
    info!("Successfully compressed file {input_file:?} to {output_file:?}");
    Ok(())
}

/// 解压单个zstd文件
pub fn decompress_file(input_file: &Path, output_file: &Path) -> Result<()> {
    info!("Decompressing file {input_file:?} to {output_file:?} using zstd");
    
    // 打开输入文件
    let input = File::open(input_file)?;
    
    // 创建zstd解码器
    let mut decoder = Decoder::new(input)?;
    
    // 创建输出文件
    let mut output = File::create(output_file)?;
    
    // 复制文件内容
    let mut buffer = Vec::new();
    decoder.read_to_end(&mut buffer)?;
    output.write_all(&buffer)?;
    
    info!("Successfully decompressed file {input_file:?} to {output_file:?}");
    Ok(())
}
