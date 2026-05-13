#[path = "modules/utils.rs"]
mod utils;
#[path = "modules/api.rs"]
mod api;
#[path = "modules/cache.rs"]
mod cache;
#[path = "modules/download.rs"]
mod download;
#[path = "modules/extract.rs"]
mod extract;
#[path = "modules/html_proxy.rs"]
mod html_proxy;
#[path = "modules/log.rs"]
mod log;
#[path = "modules/music_utils.rs"]
mod music_utils;
#[path = "modules/playlist.rs"]
mod playlist;
#[path = "modules/user_configs.rs"]
mod user_configs;

// ─── 应用入口 ───────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            api::proxy_api_get,
            api::proxy_api_post,
            api::proxy_image,
            api::save_music_cache,
            html_proxy::proxy_html_page,
            cache::save_cache,
            cache::load_cache,
            cache::delete_cache,
            cache::save_image_cache,
            cache::load_image_cache,
            cache::load_music_cache,
            cache::get_cache_dir_size,
            cache::clear_cache_dir,
            music_utils::get_mp3_duration,
            user_configs::save_user_config,
            user_configs::load_user_config,
            playlist::save_playlist,
            playlist::load_playlist,
            log::append_log,
            log::rotate_log,
            download::fetch_disc_page_html,
            download::parse_download_links,
            download::download_file,
            download::open_download_folder,
            download::cancel_download,
            extract::extract_archive,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
