use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
#[cfg(target_os = "linux")]
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicU16, Ordering};
#[cfg(target_os = "linux")]
use std::sync::Arc;
use std::thread;

static SERVER_PORT: AtomicU16 = AtomicU16::new(0);

pub fn get_port() -> u16 {
    SERVER_PORT.load(Ordering::Relaxed)
}

#[tauri::command]
pub fn get_audio_server_port() -> u16 {
    SERVER_PORT.load(Ordering::Relaxed)
}

pub fn start(base_dir: std::path::PathBuf) {
    #[cfg(target_os = "linux")]
    {
        let listener = TcpListener::bind("127.0.0.1:0")
            .expect("音频缓存 HTTP 服务器启动失败");
        let port = listener.local_addr().unwrap().port();
        SERVER_PORT.store(port, Ordering::Relaxed);

        let base = Arc::new(base_dir);

        thread::spawn(move || {
            for stream in listener.incoming() {
                if let Ok(stream) = stream {
                    let base = base.clone();
                    thread::spawn(move || {
                        handle_client(stream, &base);
                    });
                }
            }
        });
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = base_dir;
    }
}

#[cfg(target_os = "linux")]
fn handle_client(mut stream: TcpStream, base_dir: &std::path::Path) {
    let mut buf = [0u8; 4096];
    let n = match stream.read(&mut buf) {
        Ok(n) if n > 0 => n,
        _ => return,
    };
    let request = String::from_utf8_lossy(&buf[..n]);

    // 解析请求行: "GET /<filename> HTTP/1.1"
    let first_line = request.lines().next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 3 {
        return;
    }
    let method = parts[0];
    let path = parts[1];

    // 只处理 GET 和 HEAD
    if method != "GET" && method != "HEAD" {
        respond(stream, "405 Method Not Allowed");
        return;
    }

    // 解码 URL 路径 (strip leading /)
    let filename = percent_decode(path.trim_start_matches('/'));

    // 路径遍历安全检查
    if filename.contains("..") || filename.contains(std::path::MAIN_SEPARATOR) {
        respond(stream, "403 Forbidden");
        return;
    }

    let file_path = base_dir.join(&filename);
    if !file_path.starts_with(base_dir) || !file_path.exists() || !file_path.is_file() {
        respond(stream, "404 Not Found");
        return;
    }

    let file_len = match std::fs::metadata(&file_path) {
        Ok(m) => m.len(),
        Err(_) => {
            respond(stream, "404 Not Found");
            return;
        }
    };

    // 解析 Range 头
    let range_header = request
        .lines()
        .find(|l| l.to_ascii_lowercase().starts_with("range:"))
        .and_then(|l| l.split(':').nth(1))
        .map(|v| v.trim());

    if let Some(range_str) = range_header {
        if let Some(range) = parse_range(range_str, file_len) {
            let (start, end) = range;
            let content_len = end - start + 1;
            if method == "HEAD" {
                let resp = format!(
                    "HTTP/1.1 206 Partial Content\r\n\
                     Content-Type: audio/mpeg\r\n\
                     Content-Length: {}\r\n\
                     Content-Range: bytes {}-{}/{}\r\n\
                     Accept-Ranges: bytes\r\n\
                     Connection: close\r\n\r\n",
                    content_len, start, end, file_len
                );
                let _ = stream.write_all(resp.as_bytes());
            } else {
                let mut file = match File::open(&file_path) {
                    Ok(f) => f,
                    Err(_) => return,
                };
                if let Err(_) = file.seek(SeekFrom::Start(start)) {
                    return;
                }
                let resp = format!(
                    "HTTP/1.1 206 Partial Content\r\n\
                     Content-Type: audio/mpeg\r\n\
                     Content-Length: {}\r\n\
                     Content-Range: bytes {}-{}/{}\r\n\
                     Accept-Ranges: bytes\r\n\
                     Connection: close\r\n\r\n",
                    content_len, start, end, file_len
                );
                if let Err(_) = stream.write_all(resp.as_bytes()) {
                    return;
                }
                let mut remaining = content_len as usize;
                let mut buf = [0u8; 65536];
                while remaining > 0 {
                    let to_read = remaining.min(buf.len());
                    match file.read(&mut buf[..to_read]) {
                        Ok(0) => break,
                        Ok(n) => {
                            if let Err(_) = stream.write_all(&buf[..n]) {
                                return;
                            }
                            remaining -= n;
                        }
                        Err(_) => break,
                    }
                }
            }
            return;
        }
    }

    // 无 Range: 返回完整文件
    if method == "HEAD" {
        let resp = format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: audio/mpeg\r\n\
             Content-Length: {}\r\n\
             Accept-Ranges: bytes\r\n\
             Connection: close\r\n\r\n",
            file_len
        );
        let _ = stream.write_all(resp.as_bytes());
    } else {
        let resp = format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: audio/mpeg\r\n\
             Content-Length: {}\r\n\
             Accept-Ranges: bytes\r\n\
             Connection: close\r\n\r\n",
            file_len
        );
        if let Err(_) = stream.write_all(resp.as_bytes()) {
            return;
        }
        let mut file = match File::open(&file_path) {
            Ok(f) => f,
            Err(_) => return,
        };
        let mut buf = [0u8; 65536];
        loop {
            match file.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if let Err(_) = stream.write_all(&buf[..n]) {
                        return;
                    }
                }
                Err(_) => break,
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn respond(mut stream: TcpStream, status: &str) {
    let resp = format!(
        "HTTP/1.1 {}\r\n\
         Content-Length: 0\r\n\
         Connection: close\r\n\r\n",
        status
    );
    let _ = stream.write_all(resp.as_bytes());
}

#[cfg(target_os = "linux")]
fn parse_range(range_str: &str, file_len: u64) -> Option<(u64, u64)> {
    let range_str = range_str.strip_prefix("bytes=")?;
    let dash_pos = range_str.find('-')?;
    let start: u64 = range_str[..dash_pos].trim().parse().ok()?;
    let end_str = range_str[dash_pos + 1..].trim();
    let end: u64 = if end_str.is_empty() {
        file_len - 1
    } else {
        end_str.parse().ok()?
    };
    if start >= file_len || end >= file_len || end < start {
        return None;
    }
    Some((start, end))
}

#[cfg(target_os = "linux")]
fn percent_decode(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            } else {
                result.push('%');
                result.push_str(&hex);
            }
        } else {
            result.push(c);
        }
    }
    result
}
