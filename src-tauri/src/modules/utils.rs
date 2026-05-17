use std::path::PathBuf;
use std::sync::OnceLock;

/// 检测是否处于便携模式
/// 便携模式下用户数据存储在可执行文件所在目录
pub(crate) fn is_portable() -> bool {
    let check_dir = |dir: &std::path::Path| dir.join("portable.txt").exists();

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            if check_dir(exe_dir) {
                return true;
            }
            // tauri dev / cargo build (debug) 时自动使用便携模式
            let path_str = exe.to_string_lossy();
            if path_str.contains("/target/debug/") || path_str.contains("\\target\\debug\\") {
                return true;
            }
            // 向上查找项目根的 portable.txt（保持向后兼容）
            if let Ok(cwd) = std::env::current_dir() {
                for ancestor in exe_dir.ancestors() {
                    if check_dir(ancestor) {
                        return true;
                    }
                    if ancestor == cwd || ancestor == cwd.join("src-tauri") {
                        break;
                    }
                }
            }
        }
    }
    false
}

/// 计算应用根目录（不含子目录名）
fn compute_app_root() -> Result<PathBuf, String> {
    if is_portable() {
        let exe_path =
            std::env::current_exe().map_err(|e| format!("获取可执行文件路径失败: {}", e))?;
        let exe_dir = exe_path
            .parent()
            .ok_or_else(|| "无法获取可执行文件所在目录".to_string())?;
        Ok(exe_dir.to_path_buf())
    } else {
        let base = if cfg!(target_os = "windows") {
            dirs::data_dir().ok_or_else(|| "无法获取用户数据目录".to_string())?
        } else {
            dirs::config_dir().ok_or_else(|| "无法获取用户配置目录".to_string())?
        };
        Ok(base.join("DizzyPlay"))
    }
}

/// 获取应用子目录路径
/// 便携模式下返回 exe 所在目录下的子目录，否则返回系统标准目录
/// 结果被缓存，后续调用不会重复计算
pub fn get_app_subdir(subdir: &str) -> Result<PathBuf, String> {
    static APP_ROOT: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    let root = APP_ROOT.get_or_init(compute_app_root);
    match root {
        Ok(r) => Ok(r.join(subdir)),
        Err(e) => Err(e.clone()),
    }
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
        .header("Referer", crate::globalvars::DIZZYLAB_REFERER)
        .header("User-Agent", crate::globalvars::DIZZYLAB_USER_AGENT)
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
