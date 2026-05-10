/// 获取 HTML 内容
#[tauri::command]
pub async fn proxy_html_page(url: String) -> Result<String, String> {
    let client = crate::utils::create_dizzylab_client();
    let response = crate::utils::add_dizzylab_headers(client.get(&url)).send()
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
