/// MPEG Audio 帧头解析常量
/// MPEG_VERSION_MAP[version_idx]: 3=MPEG1, 2=MPEG2, 0=MPEG2.5
const MPEG_VERSION_MAP: [u8; 4] = [2, 1, 0, 3];

/// 解析 MP3 文件头部的 Xing/Info 或 VBRI 标签来获取时长
/// 下载文件的前 512KB 数据，从中解析出总帧数和采样率，计算时长（秒）
#[tauri::command]
pub async fn get_mp3_duration(url: String) -> Result<f64, String> {
    let client = reqwest::Client::new();
    // 下载前 512KB 数据，足够包含 Xing/VBRI 头部
    let response = client
        .get(&url)
        .header("Range", "bytes=0-524288")
        .header("Referer", "https://www.dizzylab.net")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .send()
        .await
        .map_err(|e| format!("音频请求失败: {}", e))?;

    let status = response.status().as_u16();
    if status >= 400 {
        return Err(format!("音频请求失败, HTTP {}", status));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("读取音频数据失败: {}", e))?;

    let data = bytes.as_ref();
    if data.len() < 100 {
        return Err("音频数据不足".to_string());
    }

    // 查找第一个有效的 MPEG 帧同步头 (0xFFE0-0xFFFE)
    let mut pos = 0;
    let mut first_header: Option<u32> = None;
    let mut first_header_pos = 0;

    while pos < data.len() - 1 {
        if data[pos] == 0xFF && (data[pos + 1] & 0xE0) == 0xE0 {
            let h = u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
            // 验证帧头有效性
            let version_bits = (h >> 19) & 0x03;
            let layer_bits = (h >> 17) & 0x03;
            let bitrate_bits = (h >> 12) & 0x0F;
            let sample_rate_bits = (h >> 10) & 0x03;

            // Valid: version != 01 (reserved), layer != 00 (reserved), bitrate != 1111 (bad), samplerate != 11 (reserved)
            if version_bits != 0x01
                && layer_bits != 0x00
                && bitrate_bits != 0x0F
                && sample_rate_bits != 0x03
            {
                first_header = Some(h);
                first_header_pos = pos;
                break;
            }
        }
        pos += 1;
    }

    let header = first_header.ok_or("未找到有效的 MPEG 帧同步头")?;

    // 解析帧头获取采样率
    let version_idx = ((header >> 19) & 0x03) as usize;
    let sample_rate_idx = ((header >> 10) & 0x03) as usize;
    let padding = ((header >> 9) & 0x01) as usize;

    let mpeg_version = MPEG_VERSION_MAP[version_idx];
    let sample_rate = match mpeg_version {
        3 => {
            // MPEG 1
            match sample_rate_idx {
                0 => 44100u32,
                1 => 48000,
                2 => 32000,
                _ => return Err("无效的采样率索引".to_string()),
            }
        }
        2 => {
            // MPEG 2
            match sample_rate_idx {
                0 => 22050u32,
                1 => 24000,
                2 => 16000,
                _ => return Err("无效的采样率索引".to_string()),
            }
        }
        _ => {
            // MPEG 2.5
            match sample_rate_idx {
                0 => 11025u32,
                1 => 12000,
                2 => 8000,
                _ => return Err("无效的采样率索引".to_string()),
            }
        }
    };

    // 计算帧大小（用于跳过帧头查找 Xing/VBRI）
    let bitrate_idx = ((header >> 12) & 0x0F) as usize;
    let bitrates: [u32; 15] = [
        0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320,
    ];
    let bitrate = bitrates.get(bitrate_idx).copied().unwrap_or(128) * 1000;

    let frame_size = if mpeg_version == 3 {
        (144 * bitrate / sample_rate + padding as u32) as usize
    } else {
        (72 * bitrate / sample_rate + padding as u32) as usize
    };

    // 在第一个帧头之后查找 Xing/Info 或 VBRI 头部
    let side_bytes = if mpeg_version == 3 { 32 } else { 17 };
    let has_crc = ((header >> 16) & 0x01) == 0;
    let xing_offset = if has_crc { side_bytes + 2 } else { side_bytes };

    let search_start = first_header_pos + 4 + xing_offset;
    let search_end = (first_header_pos + frame_size).min(data.len());

    if search_start >= data.len() {
        return Err("数据不足以查找 Xing 头部".to_string());
    }

    // 在第一个帧的数据区域中搜索 "Xing" 或 "Info" 标签
    let search_range = search_start..search_end.min(data.len());
    let slice = &data[search_range];

    // 查找 Xing/Info 头部 (4 字节标识)
    let mut xing_pos: Option<usize> = None;
    for i in 0..slice.len().saturating_sub(4) {
        if slice[i..i + 4] == *b"Xing" || slice[i..i + 4] == *b"Info" {
            xing_pos = Some(search_start + i);
            break;
        }
    }

    if let Some(xp) = xing_pos {
        // 找到 Xing/Info 头部
        let xing_offset_in_data = xp - search_start;
        let xing_data_start = xing_offset_in_data + 4; // 跳过 "Xing"/"Info"

        if xing_data_start + 4 > slice.len() {
            return Err("Xing 头部数据不足".to_string());
        }

        // Xing 头部标志 (4 字节)
        let flags = u32::from_be_bytes([
            slice[xing_data_start],
            slice[xing_data_start + 1],
            slice[xing_data_start + 2],
            slice[xing_data_start + 3],
        ]);

        let offset = xing_data_start + 4;

        // 标志位 0x0001: 包含帧数
        let num_frames = if flags & 0x0001 != 0 {
            if offset + 4 > slice.len() {
                return Err("Xing 帧数数据不足".to_string());
            }
            u32::from_be_bytes([
                slice[offset],
                slice[offset + 1],
                slice[offset + 2],
                slice[offset + 3],
            ])
        } else {
            return Err("Xing 头部不包含帧数信息".to_string());
        };

        // 计算时长: 帧数 * 每帧采样数 / 采样率
        let samples_per_frame = if mpeg_version == 3 { 1152 } else { 576 };
        let duration_secs = num_frames as f64 * samples_per_frame as f64 / sample_rate as f64;

        if duration_secs > 0.0 {
            return Ok((duration_secs * 10.0).round() / 10.0); // 保留一位小数
        }
    }

    // 尝试查找 VBRI 头部（通常在文件开头 32-64 字节处）
    if data.len() > 100 {
        for i in 0..(data.len().saturating_sub(4)).min(200) {
            if &data[i..i + 4] == b"VBRI" {
                if i + 26 > data.len() {
                    return Err("VBRI 头部数据不足".to_string());
                }
                let vbri_frames =
                    u32::from_be_bytes([data[i + 18], data[i + 19], data[i + 20], data[i + 21]]);
                let samples_per_frame = if mpeg_version == 3 { 1152 } else { 576 };
                let duration_secs =
                    vbri_frames as f64 * samples_per_frame as f64 / sample_rate as f64;
                if duration_secs > 0.0 {
                    return Ok((duration_secs * 10.0).round() / 10.0);
                }
                break;
            }
        }
    }

    // 如果 Xing/VBRI 都未找到，尝试通过文件总大小估算
    // 获取 Content-Length
    let head_response = client
        .head(&url)
        .header("Referer", "https://www.dizzylab.net")
        .send()
        .await
        .map_err(|e| format!("HEAD 请求失败: {}", e))?;

    let total_size = head_response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    if total_size > 0 && bitrate > 0 {
        let duration_secs = total_size as f64 * 8.0 / bitrate as f64;
        if duration_secs > 0.0 {
            return Ok((duration_secs * 10.0).round() / 10.0);
        }
    }

    Err("无法从 mp3 头部解析时长信息".to_string())
}
