use std::path::PathBuf;

/// 检测是否处于便携模式
/// 便携模式下用户数据存储在可执行文件所在目录
fn is_portable() -> bool {
    let check_dir = |dir: &std::path::Path| dir.join("portable.txt").exists();

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            if check_dir(exe_dir) {
                return true;
            }
            // `tauri dev` 时 exe 在 target/debug/ 下，向上查找项目根
            if let Ok(cwd) = std::env::current_dir() {
                for ancestor in exe_dir.ancestors() {
                    if ancestor == cwd || ancestor == cwd.join("src-tauri") {
                        break;
                    }
                    if check_dir(ancestor) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// 获取应用子目录路径
/// 便携模式下返回 exe 所在目录下的子目录，否则返回系统标准目录
pub fn get_app_subdir(subdir: &str) -> Result<PathBuf, String> {
    if is_portable() {
        let exe_path =
            std::env::current_exe().map_err(|e| format!("获取可执行文件路径失败: {}", e))?;
        let exe_dir = exe_path
            .parent()
            .ok_or_else(|| "无法获取可执行文件所在目录".to_string())?;
        return Ok(exe_dir.join(subdir));
    }

    let base = if cfg!(target_os = "windows") {
        dirs::data_dir().ok_or_else(|| "无法获取用户数据目录".to_string())?
    } else {
        dirs::config_dir().ok_or_else(|| "无法获取用户配置目录".to_string())?
    };
    Ok(base.join("DizzyPlay").join(subdir))
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
