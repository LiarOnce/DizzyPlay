use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;
use tauri::Emitter;

/// 全局取消标志注册表
/// key: task_id (由前端生成), value: 是否请求取消
static CANCEL_TOKENS: LazyLock<Mutex<HashMap<String, bool>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// 获取默认下载目录
/// - 便携模式（主程序目录下有 portable.txt）：返回主程序目录下的 downloads 文件夹
/// - 非便携模式：返回系统下载目录下的 DizzyPlay 子目录
///   - Windows: %USERPROFILE%\Downloads\DizzyPlay
///   - Linux/macOS: ~/Downloads/DizzyPlay
#[tauri::command]
pub fn get_default_download_dir() -> Result<String, String> {
    // 检查便携模式
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let portable_flag = exe_dir.join("portable.txt");
            if portable_flag.exists() {
                let dir = exe_dir.join("downloads");
                return Ok(dir.to_string_lossy().to_string());
            }
        }
    }

    let dir = dirs::home_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
        .join("Downloads")
        .join("DizzyPlay");
    Ok(dir.to_string_lossy().to_string())
}

/// 下载链接信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DownloadLink {
    pub label: String, // 显示标签，如 "MP3标准音质 (128kbps) - 32MB"
    pub url: String,   // 完整下载 URL
}

/// 获取专辑下载页面的 HTML（通过代理）
/// 需要 csrftoken 和 sessionid 用于 cookie 认证
#[tauri::command]
pub async fn fetch_disc_page_html(
    disc_id: String,
    csrf_token: String,
    session_id: String,
) -> Result<String, String> {
    let url = format!("https://www.dizzylab.net/d/{}/", disc_id);
    println!(
        "[Download] 获取专辑页面: {} (csrf: {}, session: {})",
        url,
        !csrf_token.is_empty(),
        !session_id.is_empty()
    );

    let client = crate::utils::create_dizzylab_client();
    let request = crate::utils::add_cookie_header(
        crate::utils::add_dizzylab_headers(client.get(&url)),
        &csrf_token,
        &session_id,
    );

    let response = request
        .send()
        .await
        .map_err(|e| format!("请求专辑页面失败: {}", e))?;

    let status = response.status().as_u16();
    if status >= 400 {
        return Err(format!("获取专辑页面失败, HTTP {}", status));
    }

    let html = response
        .text()
        .await
        .map_err(|e| format!("读取页面内容失败: {}", e))?;

    Ok(html)
}

/// 从专辑页面 HTML 中解析下载链接
/// 查找 div.dropdown-menu > a.dropdown-item，href 以 /albums/download/ 开头
#[tauri::command]
pub fn parse_download_links(html: String) -> Result<Vec<DownloadLink>, String> {
    let mut links = Vec::new();

    // 优先解析下拉菜单中的多格式下载链接
    let mut search_start = 0;
    loop {
        let a_start = html[search_start..].find(r#"<a"#);
        if a_start.is_none() {
            break;
        }
        let a_start = search_start + a_start.unwrap();

        let a_end = html[a_start..].find("</a>");
        if a_end.is_none() {
            break;
        }
        let a_end = a_start + a_end.unwrap() + 4;

        let a_tag = &html[a_start..a_end];

        if !a_tag.contains("dropdown-item") {
            search_start = a_end;
            continue;
        }

        let href = extract_attribute(a_tag, "href");
        if let Some(href) = href {
            if href.starts_with("/albums/download/") {
                let full_url = format!("https://www.dizzylab.net{}", href);
                let content_start = a_tag.find('>').map(|i| i + 1).unwrap_or(0);
                let content = if content_start < a_tag.len() {
                    let raw = &a_tag[content_start..a_tag.len() - 4];
                    let text = raw.split('<').next().unwrap_or("").trim().to_string();
                    text
                } else {
                    String::new()
                };
                links.push(DownloadLink {
                    label: content,
                    url: full_url,
                });
            }
        }
        search_start = a_end;
    }

    // 没有下拉链接时，尝试解析单个下载链接（download_gift）
    if links.is_empty() {
        search_start = 0;
        loop {
            let a_start = html[search_start..].find(r#"<a"#);
            if a_start.is_none() {
                break;
            }
            let a_start = search_start + a_start.unwrap();

            let a_end = html[a_start..].find("</a>");
            if a_end.is_none() {
                break;
            }
            let a_end = a_start + a_end.unwrap() + 4;

            let a_tag = &html[a_start..a_end];

            let href = extract_attribute(a_tag, "href");
            if let Some(href) = href {
                if href.starts_with("/albums/download_gift/") {
                    let full_url = format!("https://www.dizzylab.net{}", href);
                    links.push(DownloadLink {
                        label: "下载商品".to_string(),
                        url: full_url,
                    });
                }
            }
            search_start = a_end;
        }
    }

    println!("[Download] 解析到 {} 个下载链接", links.len());
    Ok(links)
}

/// 从 HTML 标签中提取属性值
fn extract_attribute(tag: &str, attr: &str) -> Option<String> {
    let pattern = format!(r#"{}="#, attr);
    let attr_start = tag.find(&pattern)?;
    let value_start = attr_start + pattern.len();
    let quote = tag[value_start..].chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let value_end = tag[value_start + 1..].find(quote)?;
    Some(tag[value_start + 1..value_start + 1 + value_end].to_string())
}

/// 取消下载任务
/// 设置取消标志，download_file 中的循环会检测到并提前退出
#[tauri::command]
pub fn cancel_download(task_id: String) -> Result<String, String> {
    if let Ok(mut map) = CANCEL_TOKENS.lock() {
        map.insert(task_id.clone(), true);
        println!("[Download] 已请求取消任务: {}", task_id);
    }
    Ok(format!("已请求取消: {}", task_id))
}

/// 下载文件到指定路径（支持断点续传和暂停）
///
/// # 参数
/// - `url`: 下载 URL
/// - `save_path`: 保存目录
/// - `csrf_token`: CSRF Token
/// - `session_id`: Session ID
/// - `task_id`: 任务唯一标识（用于暂停/取消）
/// - `offset`: 已下载的字节数（续传时使用）
///
/// 暂停机制：
/// 1. 前端调用 `cancel_download(task_id)` 设置取消标志
/// 2. 下载循环每次迭代检查取消标志，若为 true 则返回 "paused" 错误
/// 3. 前端捕获到 "paused" 错误，将任务标记为 PAUSED
///
/// 续传机制：
/// 1. 前端记录已下载字节数（downloadedBytes）
/// 2. 继续时调用 `download_file` 传入 offset=downloadedBytes
/// 3. Rust 发送 Range 请求头，从断点处继续下载
/// 4. 以追加模式打开已有文件，从断点处继续写入
#[tauri::command]
pub async fn download_file(
    url: String,
    save_path: String,
    csrf_token: String,
    session_id: String,
    task_id: String,
    offset: u64,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    println!(
        "[Download] 开始下载: {} (task_id: {}, offset: {})",
        url, task_id, offset
    );

    // 注册取消标志（初始为 false）
    {
        if let Ok(mut map) = CANCEL_TOKENS.lock() {
            map.insert(task_id.clone(), false);
        }
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3600)) // 1小时超时
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let mut request = crate::utils::add_cookie_header(
        crate::utils::add_dizzylab_headers(client.get(&url)),
        &csrf_token,
        &session_id,
    );

    // 断点续传：如果 offset > 0，发送 Range 请求头
    if offset > 0 {
        request = request.header("Range", format!("bytes={}-", offset));
        println!("[Download] 断点续传, Range: bytes={}-", offset);
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("下载请求失败: {}", e))?;

    let status = response.status().as_u16();
    if status >= 400 {
        return Err(format!("下载失败, HTTP {}", status));
    }

    // 获取总大小
    // 如果是续传，total_size 可能来自 Content-Range: bytes {start}-{end}/{total}
    let total_size = if offset > 0 {
        response
            .headers()
            .get("content-range")
            .and_then(|v| v.to_str().ok())
            .and_then(|cr| {
                // Content-Range: bytes {start}-{end}/{total}
                cr.rsplit('/').next()?.parse::<u64>().ok()
            })
            .unwrap_or(0)
    } else {
        response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0)
    };

    // 获取文件名（从 Content-Disposition 或 URL 中提取）
    let filename = response
        .headers()
        .get("content-disposition")
        .and_then(|v| v.to_str().ok())
        .and_then(|cd| {
            cd.split(';').find_map(|part| {
                let part = part.trim();
                if part.starts_with("filename=") {
                    Some(part["filename=".len()..].trim_matches('"').to_string())
                } else {
                    None
                }
            })
        })
        .unwrap_or_else(|| {
            // 从 URL 中提取文件名
            let name = url
                .rsplit('/')
                .next()
                .unwrap_or("download")
                .split('?')
                .next()
                .unwrap_or("download")
                .to_string();
            // 如果提取的文件名是空字符串或路径片段，使用默认名
            if name.is_empty() || name.contains('.') == false && name.len() < 5 {
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                format!("download_{}.zip", ts)
            } else {
                name
            }
        });

    // 如果 Content-Disposition 提取的文件名为空，也使用默认名
    let filename = if filename.is_empty() {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        format!("download_{}.zip", ts)
    } else {
        filename
    };

    let file_path = std::path::PathBuf::from(&save_path).join(&filename);

    // 确保目录存在
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    // 检查 file_path 是否是一个已存在的目录
    if file_path.is_dir() {
        return Err(format!("保存路径是一个目录，无法写入文件: {:?}", file_path));
    }

    // 流式写入文件（带进度回调）
    // 续传时使用 append 模式，新下载时使用 create 模式
    use std::io::Write;
    let file = if offset > 0 {
        std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&file_path)
            .map_err(|e| format!("打开文件失败: {}", e))?
    } else {
        std::fs::File::create(&file_path).map_err(|e| format!("创建文件失败: {}", e))?
    };
    let mut writer = std::io::BufWriter::new(file);

    let mut downloaded: u64 = offset;
    let mut last_emitted_pct: u32 = if total_size > 0 {
        ((offset as f64 / total_size as f64) * 100.0) as u32
    } else {
        0
    };
    let mut stream = response.bytes_stream();

    use futures_util::StreamExt;
    while let Some(chunk_result) = stream.next().await {
        // 检查取消标志（暂停请求）
        {
            if let Ok(map) = CANCEL_TOKENS.lock() {
                if let Some(&cancelled) = map.get(&task_id) {
                    if cancelled {
                        // 清除取消标志
                        drop(map);
                        if let Ok(mut map) = CANCEL_TOKENS.lock() {
                            map.remove(&task_id);
                        }
                        writer
                            .flush()
                            .map_err(|e| format!("刷新文件缓冲区失败: {}", e))?;
                        println!("[Download] 任务已暂停: {}", task_id);
                        return Err("下载已暂停".to_string());
                    }
                }
            }
        }

        let chunk = chunk_result.map_err(|e| format!("读取下载流失败: {}", e))?;
        writer
            .write_all(&chunk)
            .map_err(|e| format!("写入文件失败: {}", e))?;
        downloaded += chunk.len() as u64;

        // 计算并发送进度
        if total_size > 0 {
            let pct = ((downloaded as f64 / total_size as f64) * 100.0) as u32;
            if pct != last_emitted_pct || downloaded >= total_size {
                last_emitted_pct = pct;
                let _ = app_handle.emit(
                    "download-progress",
                    serde_json::json!({
                        "downloaded": downloaded,
                        "total": total_size,
                        "percent": pct,
                    }),
                );
            }
        } else {
            if downloaded - last_emitted_pct as u64 >= 1024 * 1024 {
                last_emitted_pct = (downloaded / (1024 * 1024)) as u32;
                let _ = app_handle.emit(
                    "download-progress",
                    serde_json::json!({
                        "downloaded": downloaded,
                        "total": 0,
                        "percent": 0,
                    }),
                );
            }
        }
    }

    writer
        .flush()
        .map_err(|e| format!("刷新文件缓冲区失败: {}", e))?;

    // 清除取消标志
    if let Ok(mut map) = CANCEL_TOKENS.lock() {
        map.remove(&task_id);
    }

    let size_mb = downloaded as f64 / (1024.0 * 1024.0);
    println!("[Download] 下载完成: {:?} ({:.2} MB)", file_path, size_mb);

    Ok(file_path.to_string_lossy().to_string())
}

/// 在文件管理器中打开指定路径
#[tauri::command]
pub fn open_download_folder(path: String) -> Result<String, String> {
    let dir = std::path::PathBuf::from(&path);
    let dir_to_open = if dir.is_dir() {
        dir
    } else if let Some(parent) = dir.parent() {
        parent.to_path_buf()
    } else {
        dir
    };

    println!("[Download] 打开文件夹: {:?}", dir_to_open);

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&dir_to_open)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&dir_to_open)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&dir_to_open)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }

    Ok(dir_to_open.to_string_lossy().to_string())
}
