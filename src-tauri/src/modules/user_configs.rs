pub fn get_user_config_dir() -> Result<std::path::PathBuf, String> {
    crate::utils::get_app_subdir("user")
}

/// 保存用户配置到 user/config.json
#[tauri::command]
pub fn save_user_config(key: String, data: String) -> Result<String, String> {
    let config_dir = get_user_config_dir()?;
    std::fs::create_dir_all(&config_dir).map_err(|e| format!("创建用户配置目录失败: {}", e))?;

    let file_path = config_dir.join("config.json");

    let mut config: serde_json::Value = if file_path.exists() {
        let content = std::fs::read_to_string(&file_path)
            .map_err(|e| format!("读取用户配置文件失败: {}", e))?;
        serde_json::from_str(&content).unwrap_or(serde_json::Value::Object(serde_json::Map::new()))
    } else {
        serde_json::Value::Object(serde_json::Map::new())
    };

    if let serde_json::Value::Object(ref mut map) = config {
        map.insert(key, serde_json::Value::String(data));
    }

    let json_str =
        serde_json::to_string_pretty(&config).map_err(|e| format!("序列化配置失败: {}", e))?;
    std::fs::write(&file_path, &json_str).map_err(|e| format!("写入用户配置文件失败: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

/// 加载用户配置
#[tauri::command]
pub fn load_user_config(key: String) -> Result<String, String> {
    let config_dir = get_user_config_dir()?;
    let file_path = config_dir.join("config.json");

    if !file_path.exists() {
        return Ok(String::new());
    }

    let content =
        std::fs::read_to_string(&file_path).map_err(|e| format!("读取用户配置文件失败: {}", e))?;
    let config: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("解析用户配置文件失败: {}", e))?;

    if let serde_json::Value::Object(ref map) = config {
        if let Some(value) = map.get(&key) {
            if let serde_json::Value::String(s) = value {
                return Ok(s.clone());
            }
        }
    }

    Ok(String::new())
}
