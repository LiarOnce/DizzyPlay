fn get_cache_dir() -> Result<std::path::PathBuf, String> {
    crate::utils::get_app_subdir("cache/datas")
}

fn get_image_cache_dir() -> Result<std::path::PathBuf, String> {
    crate::utils::get_app_subdir("cache/images")
}

fn get_music_cache_dir() -> Result<std::path::PathBuf, String> {
    crate::utils::get_app_subdir("cache/music")
}

// ─── JSON 数据缓存 ──────────────────────────────────────────

/// 保存缓存数据到程序同目录下的 cache/datas 文件夹
/// 返回保存的文件路径
#[tauri::command]
pub fn save_cache(key: String, data: String) -> Result<String, String> {
    let cache_dir = get_cache_dir()?;

    std::fs::create_dir_all(&cache_dir).map_err(|e| format!("创建缓存目录失败: {}", e))?;

    let safe_key: String = key
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    let file_path = cache_dir.join(format!("{}.json", safe_key));
    std::fs::write(&file_path, &data).map_err(|e| format!("写入缓存文件失败: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

/// 从程序同目录下的 cache/datas 文件夹加载缓存数据
/// 如果缓存文件不存在或已过期（超过指定秒数），返回空字符串
#[tauri::command]
pub fn load_cache(key: String, max_age_secs: u64) -> Result<String, String> {
    let cache_dir = get_cache_dir()?;

    let safe_key: String = key
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    let file_path = cache_dir.join(format!("{}.json", safe_key));

    if !file_path.exists() {
        return Ok(String::new());
    }

    if let Ok(metadata) = std::fs::metadata(&file_path) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(elapsed) = modified.elapsed() {
                if elapsed.as_secs() > max_age_secs {
                    let _ = std::fs::remove_file(&file_path);
                    return Ok(String::new());
                }
            }
        }
    }

    std::fs::read_to_string(&file_path).map_err(|e| format!("读取缓存文件失败: {}", e))
}

/// 删除指定键名的缓存文件
#[tauri::command]
pub fn delete_cache(key: String) -> Result<String, String> {
    let cache_dir = get_cache_dir()?;

    let safe_key: String = key
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    let file_path = cache_dir.join(format!("{}.json", safe_key));

    if file_path.exists() {
        std::fs::remove_file(&file_path).map_err(|e| format!("删除缓存文件失败: {}", e))?;
        Ok(format!("已删除缓存: {}", safe_key))
    } else {
        Ok(format!("缓存不存在: {}", safe_key))
    }
}

// ─── 图片缓存 ───────────────────────────────────────────────

/// 保存图片缓存到 cache/images 目录
/// 返回保存的文件路径
#[tauri::command]
pub fn save_image_cache(url: String, data: Vec<u8>) -> Result<String, String> {
    let img_cache_dir = get_image_cache_dir()?;

    std::fs::create_dir_all(&img_cache_dir).map_err(|e| format!("创建图片缓存目录失败: {}", e))?;

    let filename = crate::utils::url_to_filename(&url);
    let ext = crate::utils::guess_extension(&url);
    let file_path = img_cache_dir.join(format!("{}{}", filename, ext));

    std::fs::write(&file_path, &data).map_err(|e| format!("写入图片缓存文件失败: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

/// 从 cache/images 目录加载图片缓存
/// 如果缓存文件存在且未过期，返回 base64 编码的图片数据（含 data: URI 前缀）
/// 否则返回空字符串
#[tauri::command]
pub fn load_image_cache(url: String, max_age_secs: u64) -> Result<String, String> {
    let img_cache_dir = get_image_cache_dir()?;

    let filename = crate::utils::url_to_filename(&url);
    let ext = crate::utils::guess_extension(&url);
    let file_path = img_cache_dir.join(format!("{}{}", filename, ext));

    if !file_path.exists() {
        return Ok(String::new());
    }

    if let Ok(metadata) = std::fs::metadata(&file_path) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(elapsed) = modified.elapsed() {
                if elapsed.as_secs() > max_age_secs {
                    let _ = std::fs::remove_file(&file_path);
                    return Ok(String::new());
                }
            }
        }
    }

    let bytes = std::fs::read(&file_path).map_err(|e| format!("读取图片缓存文件失败: {}", e))?;
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);

    let mime = match ext {
        ".png" => "image/png",
        ".gif" => "image/gif",
        ".webp" => "image/webp",
        ".bmp" => "image/bmp",
        ".svg" => "image/svg+xml",
        _ => "image/jpeg",
    };

    Ok(format!("data:{};base64,{}", mime, b64))
}

// ─── 音乐缓存 ───────────────────────────────────────────────

/// 从 cache/music 目录加载音乐缓存
/// 如果缓存文件存在，返回 data:audio/mpeg;base64,... 数据 URI；否则返回空字符串
#[tauri::command]
pub fn load_music_cache(url: String) -> Result<String, String> {
    let music_cache_dir = get_music_cache_dir()?;

    let filename = crate::utils::url_to_filename(&url);
    let file_path = music_cache_dir.join(format!("{}.mp3", filename));

    if !file_path.exists() {
        return Ok(String::new());
    }

    let data = std::fs::read(&file_path).map_err(|e| format!("读取音乐缓存文件失败: {}", e))?;

    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);

    Ok(format!("data:audio/mpeg;base64,{}", b64))
}

// ─── 缓存管理 ───────────────────────────────────────────────

/// 递归遍历目录计算总大小
fn walk_dir(path: std::path::PathBuf) -> Result<u64, String> {
    let mut size = 0u64;
    let entries = std::fs::read_dir(&path).map_err(|e| format!("读取目录失败: {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let meta = entry
            .metadata()
            .map_err(|e| format!("获取元数据失败: {}", e))?;
        if meta.is_file() {
            size += meta.len();
        } else if meta.is_dir() {
            size += walk_dir(entry.path())?;
        }
    }
    Ok(size)
}

/// 获取缓存目录大小
#[tauri::command]
pub fn get_cache_dir_size(dir_type: String) -> Result<f64, String> {
    let dir_path = match dir_type.as_str() {
        "datas" => get_cache_dir()?,
        "images" => get_image_cache_dir()?,
        "music" => get_music_cache_dir()?,
        _ => return Err(format!("未知的缓存类型: {}", dir_type)),
    };

    if !dir_path.exists() {
        return Ok(0.0);
    }

    let total_size = walk_dir(dir_path)?;
    Ok(total_size as f64)
}

/// 删除指定缓存目录中的所有文件
#[tauri::command]
pub fn clear_cache_dir(dir_type: String) -> Result<String, String> {
    let dir_path = match dir_type.as_str() {
        "datas" => get_cache_dir()?,
        "images" => get_image_cache_dir()?,
        "music" => get_music_cache_dir()?,
        "all" => {
            let datas_dir = get_cache_dir()?;
            let images_dir = get_image_cache_dir()?;
            let music_dir = get_music_cache_dir()?;

            if datas_dir.exists() {
                std::fs::remove_dir_all(&datas_dir)
                    .map_err(|e| format!("删除 datas 缓存目录失败: {}", e))?;
            }
            if images_dir.exists() {
                std::fs::remove_dir_all(&images_dir)
                    .map_err(|e| format!("删除 images 缓存目录失败: {}", e))?;
            }
            if music_dir.exists() {
                std::fs::remove_dir_all(&music_dir)
                    .map_err(|e| format!("删除 music 缓存目录失败: {}", e))?;
            }
            return Ok("已清除所有缓存".to_string());
        }
        _ => return Err(format!("未知的缓存类型: {}", dir_type)),
    };

    if !dir_path.exists() {
        return Ok(format!("{} 缓存目录不存在", dir_type));
    }

    let entries = std::fs::read_dir(&dir_path).map_err(|e| format!("读取目录失败: {}", e))?;
    let mut count = 0u32;
    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();
        if path.is_file() {
            std::fs::remove_file(&path).map_err(|e| format!("删除文件失败: {}", e))?;
            count += 1;
        } else if path.is_dir() {
            std::fs::remove_dir_all(&path).map_err(|e| format!("删除子目录失败: {}", e))?;
        }
    }

    Ok(format!("已清除 {} 缓存，共删除 {} 个文件", dir_type, count))
}
