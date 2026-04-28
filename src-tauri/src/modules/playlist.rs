/// 保存播放列表到 user/playlist.json
#[tauri::command]
pub fn save_playlist(data: String) -> Result<String, String> {
    let config_dir = crate::user_configs::get_user_config_dir()?;
    std::fs::create_dir_all(&config_dir).map_err(|e| format!("创建用户配置目录失败: {}", e))?;

    let file_path = config_dir.join("playlist.json");
    std::fs::write(&file_path, &data).map_err(|e| format!("写入播放列表文件失败: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

/// 从 user/playlist.json 加载播放列表
#[tauri::command]
pub fn load_playlist() -> Result<String, String> {
    let config_dir = crate::user_configs::get_user_config_dir()?;
    let file_path = config_dir.join("playlist.json");

    if !file_path.exists() {
        return Ok(String::new());
    }

    std::fs::read_to_string(&file_path).map_err(|e| format!("读取播放列表文件失败: {}", e))
}
