use crate::log::log_info;
use std::path::Path;

fn sanitize_archive_path(path: &str) -> Option<String> {
    let path = path.replace('\\', "/");
    let components: Vec<&str> = path.split('/').collect();
    let mut safe = Vec::new();
    for component in components {
        match component {
            "." | "" => continue,
            ".." => return None,
            _ => safe.push(component),
        }
    }
    if safe.is_empty() {
        return None;
    }
    Some(safe.join("/"))
}

/// 解压 zip 压缩包到同级目录，可选删除原压缩包
#[tauri::command]
pub fn extract_archive(archive_path: String, delete_after: bool) -> Result<String, String> {
    let path = Path::new(&archive_path);

    if !path.exists() {
        return Err(format!("文件不存在: {}", archive_path));
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    if ext != "zip" {
        return Err(format!("不支持的文件格式: .{}，仅支持 .zip", ext));
    }

    let output_dir = path.parent().unwrap_or(Path::new("."));

    let file =
        std::fs::File::open(path).map_err(|e| format!("打开压缩文件失败: {}", e))?;
    let mut zip_archive =
        zip::ZipArchive::new(file).map_err(|e| format!("读取 zip 文件失败: {}", e))?;

    for i in 0..zip_archive.len() {
        let mut entry = zip_archive
            .by_index(i)
            .map_err(|e| format!("读取压缩条目失败: {}", e))?;

        let raw_name = entry.name().to_string();
        let safe_name = sanitize_archive_path(&raw_name)
            .ok_or_else(|| format!("压缩条目包含非法路径: {}", raw_name))?;
        let target = output_dir.join(&safe_name);

        if entry.is_dir() {
            std::fs::create_dir_all(&target)
                .map_err(|e| format!("创建目录失败 {}: {}", safe_name, e))?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("创建目录失败: {}", e))?;
            }
            let mut output_file =
                std::fs::File::create(&target).map_err(|e| format!("创建文件失败: {}", e))?;
            std::io::copy(&mut entry, &mut output_file)
                .map_err(|e| format!("解压文件失败 {}: {}", safe_name, e))?;
        }
    }

    log_info(&format!("[Extract] 解压完成: {:?} → {:?}", path, output_dir));

    if delete_after {
        std::fs::remove_file(path)
            .map_err(|e| format!("删除压缩包失败: {}", e))?;
        log_info(&format!("[Extract] 已删除压缩包: {:?}", path));
    }

    Ok(output_dir.to_string_lossy().to_string())
}
