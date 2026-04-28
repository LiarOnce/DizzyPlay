use serde::{Deserialize, Serialize};

/// 代理 API 请求的响应结构
#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyResponse {
    pub status: u16,
    pub body: serde_json::Value,
}

/// 图片代理响应结构（base64 编码）
#[derive(Debug, Serialize, Deserialize)]
pub struct ImageProxyResponse {
    pub data: String, // base64 编码的图片数据
    pub mime: String, // MIME 类型
}

/// POST 代理响应结构
#[derive(Debug, Serialize, Deserialize)]
pub struct PostProxyResponse {
    pub status: u16,
    pub body: String, // 原始响应文本，因为登录接口可能返回非 JSON
}

/// 通过 Rust 后端代理 HTTP GET 请求，绕过 CORS 限制
#[tauri::command]
pub async fn proxy_api_get(
    endpoint: String,
    params: String,
    token: String,
) -> Result<ProxyResponse, String> {
    let base_url = "https://www.dizzylab.net/apis/";

    // 构建 URL
    let mut url = format!("{}{}/?{}", base_url, endpoint, params);
    if !token.is_empty() {
        if params.is_empty() {
            url = format!("{}{}/?token={}", base_url, endpoint, token);
        } else {
            url = format!("{}{}/?{}&token={}", base_url, endpoint, params, token);
        }
    }

    // 发送 GET 请求
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("Referer", "https://www.dizzylab.net")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let status = response.status().as_u16();
    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    Ok(ProxyResponse { status, body })
}

/// 通过 Rust 后端代理 HTTP POST 请求，绕过 CORS 限制
/// 用于登录等需要 POST 操作的场景
#[tauri::command]
pub async fn proxy_api_post(
    endpoint: String,
    body: String,
    content_type: String,
) -> Result<PostProxyResponse, String> {
    let base_url = "https://www.dizzylab.net/apis/";
    let url = format!("{}{}/", base_url, endpoint);

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", &content_type)
        .header("Referer", "https://www.dizzylab.net")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .body(body)
        .send()
        .await
        .map_err(|e| format!("POST 请求失败: {}", e))?;

    let status = response.status().as_u16();
    let body_text = response
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    Ok(PostProxyResponse {
        status,
        body: body_text,
    })
}

/// 通过 Rust 后端代理图片请求，返回 base64 编码数据
/// 解决 cdn.dizzylab.net 的防盗链问题
#[tauri::command]
pub async fn proxy_image(url: String) -> Result<ImageProxyResponse, String> {
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("Referer", "https://www.dizzylab.net")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .send()
        .await
        .map_err(|e| format!("图片请求失败: {}", e))?;

    let status = response.status().as_u16();
    if status >= 400 {
        return Err(format!("图片请求失败, HTTP {}", status));
    }

    // 获取 Content-Type
    let mime = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/jpeg")
        .to_string();

    // 读取二进制数据并编码为 base64
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("读取图片数据失败: {}", e))?;

    use base64::Engine;
    let data = base64::engine::general_purpose::STANDARD.encode(&bytes);

    Ok(ImageProxyResponse { data, mime })
}

/// 保存音乐缓存到 cache/music 目录
/// 通过 reqwest 下载 mp3 文件并保存为原始二进制文件
/// 返回保存的文件路径
#[tauri::command]
pub async fn save_music_cache(url: String) -> Result<String, String> {
    let music_cache_dir = get_music_cache_dir()?;

    // 创建 cache/music 目录
    std::fs::create_dir_all(&music_cache_dir)
        .map_err(|e| format!("创建音乐缓存目录失败: {}", e))?;

    // 使用 URL 的哈希值作为文件名
    let filename = url_to_filename(&url);
    let file_path = music_cache_dir.join(format!("{}.mp3", filename));

    // 如果文件已存在，直接返回路径
    if file_path.exists() {
        return Ok(file_path.to_string_lossy().to_string());
    }

    // 下载音乐文件
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let response = client
        .get(&url)
        .header("Referer", "https://www.dizzylab.net")
        .send()
        .await
        .map_err(|e| format!("下载音乐文件失败: {}", e))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("读取音乐文件数据失败: {}", e))?;

    // 写入原始二进制数据
    std::fs::write(&file_path, &bytes).map_err(|e| format!("写入音乐缓存文件失败: {}", e))?;

    println!("[MusicCache] 已缓存音乐: {} -> {:?}", url, file_path);
    Ok(file_path.to_string_lossy().to_string())
}

// ─── 辅助函数 ───────────────────────────────────────────────

/// 获取音乐缓存目录
fn get_music_cache_dir() -> Result<std::path::PathBuf, String> {
    let exe_path = std::env::current_exe().map_err(|e| format!("获取可执行文件路径失败: {}", e))?;
    let exe_dir = exe_path
        .parent()
        .ok_or_else(|| "无法获取可执行文件所在目录".to_string())?;
    Ok(exe_dir.join("cache").join("music"))
}

/// 将 URL 转换为安全的文件名（使用哈希值）
fn url_to_filename(url: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    url.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
