/// 获取 HTML 内容
#[tauri::command]
pub async fn proxy_html_page(url: String) -> Result<String, String> {
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
        .map_err(|e| format!("HTML 页面请求失败: {}", e))?;

    let status = response.status().as_u16();
    if status >= 400 {
        return Err(format!("HTML 页面请求失败, HTTP {}", status));
    }

    let html = response
        .text()
        .await
        .map_err(|e| format!("读取 HTML 响应失败: {}", e))?;

    Ok(html)
}
