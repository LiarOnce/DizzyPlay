/// 判断是否为闰年
fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// 追加日志到 user/DizzyPlay.log
/// 每条日志一行，格式: [时间戳] [级别] 消息
#[tauri::command]
pub fn append_log(level: String, message: String) -> Result<String, String> {
    let config_dir = crate::user_configs::get_user_config_dir()?;
    std::fs::create_dir_all(&config_dir).map_err(|e| format!("创建日志目录失败: {}", e))?;

    let file_path = config_dir.join("DizzyPlay.log");

    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("获取系统时间失败: {}", e))?;

    let total_secs = now.as_secs();
    let millis = now.subsec_millis();

    let secs_per_day: u64 = 86400;
    let days = total_secs / secs_per_day;
    let day_secs = total_secs % secs_per_day;

    let hours = day_secs / 3600;
    let minutes = (day_secs % 3600) / 60;
    let seconds = day_secs % 60;

    let mut y = 1970i64;
    let mut remaining_days = days as i64;
    loop {
        let days_in_year = if is_leap_year(y) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        y += 1;
    }
    let month_days = if is_leap_year(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 1usize;
    for &md in month_days.iter() {
        if remaining_days < md as i64 {
            break;
        }
        remaining_days -= md as i64;
        m += 1;
    }
    let d = remaining_days + 1;

    let timestamp = format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
        y, m, d, hours, minutes, seconds, millis
    );

    let log_line = format!("[{}] [{}] {}\n", timestamp, level, message);

    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)
        .map_err(|e| format!("打开日志文件失败: {}", e))?;
    file.write_all(log_line.as_bytes())
        .map_err(|e| format!("写入日志失败: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

/// 备份上一次运行的日志文件
/// 将 DizzyPlay.log 重命名为 DizzyPlay.log.bak（覆盖旧备份）
#[tauri::command]
pub fn rotate_log() -> Result<String, String> {
    let config_dir = crate::user_configs::get_user_config_dir()?;
    let log_path = config_dir.join("DizzyPlay.log");
    let bak_path = config_dir.join("DizzyPlay.log.bak");

    if log_path.exists() {
        // 删除旧的备份文件（如果存在）
        if bak_path.exists() {
            std::fs::remove_file(&bak_path).map_err(|e| format!("删除旧备份文件失败: {}", e))?;
        }
        // 重命名当前日志文件为备份
        std::fs::rename(&log_path, &bak_path).map_err(|e| format!("备份日志文件失败: {}", e))?;
        Ok(format!("日志已备份: {:?} -> {:?}", log_path, bak_path))
    } else {
        Ok("没有找到日志文件，无需备份".to_string())
    }
}
