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

    let client = crate::utils::create_dizzylab_client();
    let response = crate::utils::add_dizzylab_headers(client.get(&url)).send()
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

    let client = crate::utils::create_dizzylab_client();
    let response = crate::utils::add_dizzylab_headers(client.post(&url))
        .header("Content-Type", &content_type)
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
    let client = crate::utils::create_dizzylab_client();
    let response = crate::utils::add_dizzylab_headers(client.get(&url))
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
    let music_cache_dir = crate::utils::get_app_subdir("cache/music")?;

    std::fs::create_dir_all(&music_cache_dir)
        .map_err(|e| format!("创建音乐缓存目录失败: {}", e))?;

    let filename = crate::utils::url_to_filename(&url);
    let mp3_name = format!("{}.mp3", filename);
    let file_path = music_cache_dir.join(&mp3_name);

    if file_path.exists() {
        if let Ok(data) = std::fs::read(&file_path) {
            let is_valid = data.len() >= 4
                && (data[0] == 0x49 && data[1] == 0x44 && data[2] == 0x33
                    || data[0] == 0xFF && (data[1] & 0xF0) == 0xF0);
            if is_valid {
                return Ok(file_path.to_string_lossy().to_string());
            }
            println!("[MusicCache] 已有缓存文件无效，重新下载: {:?}", file_path);
            let _ = std::fs::remove_file(&file_path);
        }
    }

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::REFERER,
        reqwest::header::HeaderValue::from_static(crate::globalvars::DIZZYLAB_REFERER),
    );
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static(crate::globalvars::DIZZYLAB_USER_AGENT),
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .default_headers(headers)
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("下载音乐文件失败: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("下载音乐文件失败, HTTP {}", status));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("读取音乐文件数据失败: {}", e))?;

    if bytes.len() < 4 {
        return Err("下载的音乐文件无效（数据不足）".to_string());
    }

    // 检查是否为有效的 MP3 文件（ID3 或 MPEG 帧头）
    let is_mp3 = bytes[0] == 0x49 && bytes[1] == 0x44 && bytes[2] == 0x33 // ID3
        || bytes[0] == 0xFF && (bytes[1] & 0xF0) == 0xF0; // MPEG 帧同步
    if !is_mp3 {
        return Err("下载的文件不是有效的 MP3 格式".to_string());
    }
    
    // 写入原始二进制数据
    std::fs::write(&file_path, &bytes).map_err(|e| format!("写入音乐缓存文件失败: {}", e))?;

    println!("[MusicCache] 已缓存音乐: {} -> {:?}", url, file_path);
    Ok(file_path.to_string_lossy().to_string())
}

// ─── 辅助函数 ───────────────────────────────────────────────
