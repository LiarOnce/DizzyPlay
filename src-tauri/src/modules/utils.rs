use std::path::PathBuf;

pub fn get_app_subdir(subdir: &str) -> Result<PathBuf, String> {
    let exe_path = std::env::current_exe().map_err(|e| format!("获取可执行文件路径失败: {}", e))?;
    let exe_dir = exe_path
        .parent()
        .ok_or_else(|| "无法获取可执行文件所在目录".to_string())?;
    Ok(exe_dir.join(subdir))
}

pub fn url_to_filename(url: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    url.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

pub fn guess_extension(url: &str) -> &'static str {
    let url_lower = url.to_lowercase();
    if url_lower.contains(".png") {
        ".png"
    } else if url_lower.contains(".gif") {
        ".gif"
    } else if url_lower.contains(".webp") {
        ".webp"
    } else if url_lower.contains(".bmp") {
        ".bmp"
    } else if url_lower.contains(".svg") {
        ".svg"
    } else {
        ".jpg"
    }
}

pub fn build_cookies(csrf_token: &str, session_id: &str) -> Vec<String> {
    let mut cookies = Vec::new();
    if !csrf_token.is_empty() {
        cookies.push(format!("csrftoken={}", csrf_token));
    }
    if !session_id.is_empty() {
        cookies.push(format!("sessionid={}", session_id));
    }
    cookies
}

pub fn create_dizzylab_client() -> reqwest::Client {
    reqwest::Client::new()
}

pub fn add_dizzylab_headers(request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    request
        .header("Referer", "https://www.dizzylab.net")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
}

pub fn add_cookie_header(
    request: reqwest::RequestBuilder,
    csrf_token: &str,
    session_id: &str,
) -> reqwest::RequestBuilder {
    let cookies = build_cookies(csrf_token, session_id);
    if !cookies.is_empty() {
        request.header("Cookie", cookies.join("; "))
    } else {
        request
    }
}
