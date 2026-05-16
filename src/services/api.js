/**
 * 1x1 透明 GIF 占位图，用于 Tauri 生产环境中缓存未命中时的回退
 */
const PLACEHOLDER_IMAGE =
  "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7";

/**
 * Tauri 环境下的同步图片缓存
 * key: 原始图片 URL, value: data: URI
 * 由 getCachedCoverUrl 异步填充，getCoverUrl 同步读取
 */
const tauriCoverCache = new Map();

import { buildCookieHeader, isTauri } from "../utils/format.js";

// JSON 数据缓存有效期（1天）
const CACHE_MAX_AGE_SECS = 86400;

// 图片缓存有效期（7天）
const IMAGE_CACHE_MAX_AGE_SECS = 7 * 24 * 3600;

// ============================================================
// 基础工具函数
// ============================================================

/**
 * Tauri invoke 快捷调用
 */
async function tauriInvoke(cmd, args) {
  const { invoke } = window.__TAURI_INTERNALS__;
  return invoke(cmd, args);
}

/**
 * 将 base64 字符串解码为 Uint8Array
 */
function base64ToBytes(base64) {
  const binaryStr = atob(base64);
  const bytes = new Uint8Array(binaryStr.length);
  for (let i = 0; i < binaryStr.length; i++) {
    bytes[i] = binaryStr.charCodeAt(i);
  }
  return bytes;
}

/**
 * 拼接 dizzylab 完整 URL（处理相对路径）
 */
function joinDizzylabUrl(path) {
  if (!path) return "";
  if (path.startsWith("http://") || path.startsWith("https://")) return path;
  return `https://www.dizzylab.net${path.startsWith("/") ? "" : "/"}${path}`;
}

/**
 * 获取 Vite 代理基础 URL
 */
function getProxyBase() {
  const useProxy =
    window.location.port === "1420" || window.location.hostname === "localhost";
  return useProxy
    ? `${window.location.origin}/apis/`
    : "https://www.dizzylab.net/apis/";
}

// ============================================================
// 统一存储抽象层（Tauri invoke / localStorage 回退）
// ============================================================

/**
 * 创建统一存储操作
 * @param {string} prefix - localStorage 键名前缀
 * @param {Object} tauriCmds - Tauri 命令名映射 { save, load, delete }
 * @param {Function} [serialize] - 序列化函数
 * @param {Function} [deserialize] - 反序列化函数
 */
function createStorage(
  prefix,
  tauriCmds,
  serialize = JSON.stringify,
  deserialize = JSON.parse,
) {
  return {
    async save(key, data) {
      const serialized = serialize(data);
      if (isTauri) {
        try {
          const result = await tauriInvoke(tauriCmds.save, {
            key,
            data: serialized,
          });
          return result;
        } catch (err) {
          console.warn(`[Storage] ${tauriCmds.save} 失败:`, err);
          return false;
        }
      }
      try {
        localStorage.setItem(`${prefix}_${key}`, serialized);
        if (tauriCmds.save === "save_cache") {
          localStorage.setItem(`${prefix}_${key}_time`, Date.now().toString());
        }
        return true;
      } catch (err) {
        console.warn(`[Storage] localStorage 保存失败:`, err);
        return false;
      }
    },

    async load(key, maxAgeSecs) {
      if (isTauri) {
        try {
          const args = { key };
          if (maxAgeSecs !== undefined) args.maxAgeSecs = maxAgeSecs;
          const data = await tauriInvoke(tauriCmds.load, args);
          if (data) return deserialize(data);
          return null;
        } catch (err) {
          console.warn(`[Storage] ${tauriCmds.load} 失败:`, err);
          return null;
        }
      }
      try {
        // 检查过期
        if (maxAgeSecs !== undefined) {
          const savedTime = localStorage.getItem(`${prefix}_${key}_time`);
          if (savedTime) {
            const elapsed = (Date.now() - parseInt(savedTime, 10)) / 1000;
            if (elapsed > maxAgeSecs) {
              localStorage.removeItem(`${prefix}_${key}`);
              localStorage.removeItem(`${prefix}_${key}_time`);
              return null;
            }
          }
        }
        const raw = localStorage.getItem(`${prefix}_${key}`);
        return raw ? deserialize(raw) : null;
      } catch (err) {
        console.warn(`[Storage] localStorage 读取失败:`, err);
        return null;
      }
    },

    async delete(key) {
      if (isTauri) {
        try {
          return await tauriInvoke(tauriCmds.delete, { key });
        } catch (err) {
          console.warn(`[Storage] ${tauriCmds.delete} 失败:`, err);
        }
        return;
      }
      localStorage.removeItem(`${prefix}_${key}`);
      if (tauriCmds.save === "save_cache") {
        localStorage.removeItem(`${prefix}_${key}_time`);
      }
    },
  };
}

// 创建各存储实例
const cacheStorage = createStorage("cache", {
  save: "save_cache",
  load: "load_cache",
  delete: "delete_cache",
});

const userConfigStorage = createStorage(
  "user",
  {
    save: "save_user_config",
    load: "load_user_config",
    delete: null,
  },
  (v) => v, // 用户配置直接传原始字符串，Rust 端负责 JSON 序列化
  (v) => v, // 用户配置直接返回字符串，Rust 端返回的就是纯字符串
);

const playlistStorage = createStorage("", {
  save: "save_playlist",
  load: "load_playlist",
  delete: null,
});

// ============================================================
// 封面图片相关
// ============================================================

/**
 * 获取封面图片的完整 URL
 * 在 Vite dev 环境中，将 cdn.dizzylab.net 的图片通过 /cdn 代理
 * 在 Tauri 生产环境中，优先使用同步缓存中的 data: URI，未命中时返回占位图
 * @param {string} cover - 封面路径或 URL
 * @returns {string} 完整的封面 URL
 */
export function getCoverUrl(cover) {
  if (!cover) return "";
  const fullUrl = joinDizzylabUrl(cover);

  // 浏览器环境：通过 Vite proxy 代理 cdn 图片
  if (!isTauri && fullUrl.includes("cdn.dizzylab.net")) {
    const useProxy =
      window.location.port === "1420" ||
      window.location.hostname === "localhost";
    if (useProxy) {
      return fullUrl.replace("https://cdn.dizzylab.net", "/cdn");
    }
  }

  // Tauri 环境：优先使用同步缓存中的 data: URI
  if (isTauri) {
    return tauriCoverCache.get(fullUrl) || PLACEHOLDER_IMAGE;
  }

  return fullUrl;
}

/**
 * 保存图片到本地缓存
 */
export async function saveImageCache(url, binaryData) {
  if (!isTauri) return false;
  try {
    const dataArray = Array.from(binaryData);
    const path = await tauriInvoke("save_image_cache", {
      url,
      data: dataArray,
    });
    console.log(`[ImageCache] 已保存: ${url.substring(0, 50)}... -> ${path}`);
    return path;
  } catch (err) {
    console.warn(`[ImageCache] 保存失败: ${url.substring(0, 50)}...`, err);
    return false;
  }
}

/**
 * 从本地缓存加载图片
 */
export async function loadImageCache(url) {
  if (!isTauri) return null;
  try {
    const filePath = await tauriInvoke("load_image_cache", {
      url,
      maxAgeSecs: IMAGE_CACHE_MAX_AGE_SECS,
    });
    if (filePath) {
      console.log(`[ImageCache] 命中缓存: ${url.substring(0, 50)}...`);
      return filePath;
    }
    console.log(`[ImageCache] 未命中: ${url.substring(0, 50)}...`);
    return null;
  } catch (err) {
    console.warn(`[ImageCache] 读取失败: ${url.substring(0, 50)}...`, err);
    return null;
  }
}

/**
 * 异步获取封面图片 URL 并缓存
 * @returns {Promise<string>} 封面图片 URL 或 base64 数据
 */
export async function getCachedCoverUrl(cover) {
  if (!cover) return "";

  // 浏览器环境：直接返回代理 URL
  if (!isTauri) return getCoverUrl(cover);

  // Tauri 环境
  const originalUrl = joinDizzylabUrl(cover);
  if (!originalUrl) return "";

  try {
    // 尝试从本地缓存加载
    const cachedDataUri = await loadImageCache(originalUrl);
    if (cachedDataUri) {
      tauriCoverCache.set(originalUrl, cachedDataUri);
      return cachedDataUri;
    }

    // 缓存未命中，通过 proxy_image 获取
    const result = await tauriInvoke("proxy_image", { url: originalUrl });
    if (result?.data) {
      const bytes = base64ToBytes(result.data);
      // 后台保存缓存
      saveImageCache(originalUrl, bytes).catch((e) =>
        console.warn("[ImageCache] 保存缓存失败:", e),
      );
      const dataUri = `data:${result.mime};base64,${result.data}`;
      tauriCoverCache.set(originalUrl, dataUri);
      return dataUri;
    }
  } catch (err) {
    console.warn(
      `[ImageCache] 获取失败，回退: ${originalUrl.substring(0, 50)}...`,
      err,
    );
  }

  // 失败时返回原始 URL
  tauriCoverCache.set(originalUrl, originalUrl);
  return originalUrl;
}

// ============================================================
// 音乐缓存
// ============================================================

/**
 * 替换 streaming URL 为本地代理路径
 */
function proxyStreamingUrl(url) {
  if (url.includes("streaming.dizzylab.net")) {
    return url.replace("https://streaming.dizzylab.net", "/streaming");
  }
  return url;
}

export async function getCachedMusicUrl(url) {
  if (!url) return "";

  // 浏览器环境：直接代理 streaming 请求
  if (!isTauri) return proxyStreamingUrl(url);

  // 获取缓存文件路径
  async function useCachedFile(filePath) {
    const port = await tauriInvoke("get_audio_server_port");
    if (port > 0) {
      // Linux: 通过本地 HTTP 服务器流式播放
      const filename = filePath.replace(/\\/g, "/").split("/").pop();
      return `http://127.0.0.1:${port}/${filename}`;
    }
    // Windows/macOS: 通过 asset protocol 直接播放
    return window.__TAURI_INTERNALS__.convertFileSrc(filePath);
  }

  try {
    // 尝试从缓存读取
    const cachedPath = await tauriInvoke("load_music_cache", { url });
    if (cachedPath) {
      console.log(`[MusicCache] 使用缓存: ${url.substring(0, 60)}...`);
      return useCachedFile(cachedPath);
    }

    // 无缓存时下载并缓存
    console.log(`[MusicCache] 下载并缓存: ${url.substring(0, 60)}...`);
    const savedPath = await tauriInvoke("save_music_cache", { url });
    if (savedPath) {
      return useCachedFile(savedPath);
    }
  } catch (err) {
    console.warn(`[MusicCache] 缓存失败，回退到 streaming:`, err);
  }

  return proxyStreamingUrl(url);
}

// ============================================================
// MP3 时长解析
// ============================================================

/**
 * 通过 Rust 后端解析 mp3 文件头部获取时长
 */
export async function getMp3Duration(url) {
  if (!url) return null;

  if (isTauri) {
    try {
      const duration = await tauriInvoke("get_mp3_duration", { url });
      if (duration > 0) {
        console.log(`[Mp3Duration] ${duration}s`);
        return Math.round(duration);
      }
      return null;
    } catch (err) {
      console.warn("[Mp3Duration] 获取失败:", err);
      return null;
    }
  }

  // 浏览器环境：通过 AudioContext 解码
  try {
    const response = await fetch(url, {
      headers: { Range: "bytes=0-524288" },
    });
    if (!response.ok) return null;
    const blob = await response.blob();
    const audioContext = new (
      window.AudioContext || window.webkitAudioContext
    )();
    const arrayBuffer = await blob.arrayBuffer();
    const audioBuffer = await audioContext.decodeAudioData(arrayBuffer);
    const duration = audioBuffer.duration;
    audioContext.close();
    return Math.round(duration);
  } catch {
    return null;
  }
}

// ============================================================
// 用户配置
// ============================================================

export async function saveUserConfig(key, value) {
  return userConfigStorage.save(key, value);
}

export async function loadUserConfig(key) {
  const val = await userConfigStorage.load(key);
  return val || "";
}

// ============================================================
// JSON 数据缓存
// ============================================================

export async function saveCache(key, data) {
  return cacheStorage.save(key, data);
}

export async function loadCache(key) {
  return cacheStorage.load(key, CACHE_MAX_AGE_SECS);
}

export async function clearCache(key) {
  return cacheStorage.delete(key);
}

// ============================================================
// 播放列表
// ============================================================

export async function savePlaylist(playlist) {
  return playlistStorage.save("dizzy_playlist", playlist);
}

export async function loadPlaylist() {
  const data = await playlistStorage.load("dizzy_playlist");
  return data || [];
}

// ============================================================
// API 请求（GET）
// ============================================================

/**
 * 获取 token（优先从 user/config.json，回退到 localStorage）
 */
async function getTokenForApi() {
  if (isTauri) {
    try {
      const configToken = await tauriInvoke("load_user_config", {
        key: "token",
      });
      if (configToken) return configToken;
    } catch {
      // 忽略，回退到 localStorage
    }
  }
  return localStorage.getItem("dizzytoken") || "";
}

/**
 * 构建查询字符串（过滤空值）
 */
function buildQueryString(params, extraParams = {}) {
  const query = new URLSearchParams();
  for (const [key, value] of Object.entries(extraParams)) {
    if (value !== undefined && value !== null && value !== "") {
      query.set(key, value);
    }
  }
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined && value !== null && value !== "") {
      query.set(key, value);
    }
  }
  return query.toString();
}

/**
 * 通过 Tauri invoke 调用 Rust 后端代理 GET 请求
 */
async function tauriApiGet(endpoint, params = {}) {
  // 获取 token
  let token = "";
  try {
    const { getToken } = await import("../stores/user.js");
    token = getToken() || localStorage.getItem("dizzytoken") || "";
  } catch {
    token = localStorage.getItem("dizzytoken") || "";
  }

  const query = buildQueryString(params);
  console.log(
    `[API] tauriApiGet: ${endpoint}`,
    params,
    "token:",
    token ? "yes" : "no",
  );

  const result = await tauriInvoke("proxy_api_get", {
    endpoint,
    params: query,
    token,
  });

  if (result.status >= 400) {
    throw new Error(`API Error: ${result.status}`);
  }
  return result.body;
}

/**
 * 浏览器环境使用 fetch 发起 GET 请求
 */
async function fetchApiGet(endpoint, params = {}) {
  const API_BASE = getProxyBase();
  const token = localStorage.getItem("dizzytoken") || "";

  const query = buildQueryString(params, token ? { token } : {});
  const url = `${API_BASE}${endpoint}/?${query}`;

  console.log(
    `[API] fetchApiGet: ${endpoint}`,
    params,
    "token:",
    token ? "yes" : "no",
  );

  const response = await fetch(url, {
    headers: { Referer: "https://www.dizzylab.net" },
  });

  if (!response.ok) {
    throw new Error(`API Error: ${response.status}`);
  }

  const data = await response.json();
  console.log(`[API] fetchApiGet success: ${endpoint}`);
  return data;
}

/**
 * 自动选择 GET 请求方式
 * 优先使用 Tauri invoke，失败时回退到 fetch
 */
async function apiGet(endpoint, params = {}) {
  if (isTauri) {
    try {
      return await tauriApiGet(endpoint, params);
    } catch (err) {
      console.warn(`[API] Tauri invoke 失败，回退到 fetch: ${endpoint}`, err);
    }
  }
  return fetchApiGet(endpoint, params);
}

// ============================================================
// API 请求（POST）
// ============================================================

/**
 * 通用 POST 请求（Tauri invoke / fetch 回退）
 */
async function apiPost(endpoint, body, contentType, label) {
  if (isTauri) {
    const result = await tauriInvoke("proxy_api_post", {
      endpoint,
      body,
      contentType,
    });
    console.log(`[API] ${label} 响应:`, result);
    if (result.status >= 400) {
      throw new Error(`${label}失败 (${result.status}): ${result.body}`);
    }
    return JSON.parse(result.body);
  }

  const url = `${getProxyBase()}${endpoint}/`;
  const response = await fetch(url, {
    method: "POST",
    headers: {
      "Content-Type": contentType,
      Referer: "https://www.dizzylab.net",
    },
    body,
  });

  if (!response.ok) {
    const errText = await response.text().catch(() => "");
    throw new Error(`${label}失败 (${response.status}): ${errText}`);
  }

  const data = await response.json();
  console.log(`[API] ${label} 响应:`, data);
  return data;
}

// ============================================================
// 登录
// ============================================================

export async function login(username, password) {
  const body = JSON.stringify({ username, password });
  return apiPost("auth/login", body, "application/json", "登录");
}

// ============================================================
// 兑换码
// ============================================================

async function redeemApi(endpoint, code, label) {
  const token = await getTokenForApi();
  const body = new URLSearchParams();
  body.set("token", token);
  body.set("code", code);
  return apiPost(
    endpoint,
    body.toString(),
    "application/x-www-form-urlencoded",
    label,
  );
}

export async function redeemCode(code) {
  return redeemApi("redeem", code, "验证兑换码");
}

export async function confirmRedeem(code) {
  return redeemApi("comfirm_redeem", code, "确认兑换");
}

// ============================================================
// 免费唱片解锁
// ============================================================

export async function unlockFreeDisc(discId) {
  const token = await getTokenForApi();
  const body = new URLSearchParams();
  body.set("token", token);
  body.set("discid", discId);
  return apiPost(
    "unlockafreedisc",
    body.toString(),
    "application/x-www-form-urlencoded",
  );
}

// ============================================================
// 点赞 / 取消点赞 (+2dB)
// ============================================================

export async function toggleLikeDisc(discId) {
  const token = await getTokenForApi();
  const body = new URLSearchParams();
  body.set("token", token);
  body.set("discid", discId);
  return apiPost(
    "ilikethisornot",
    body.toString(),
    "application/x-www-form-urlencoded",
    "点赞",
  );
}

// ============================================================
// GET API 端点（所有页面数据请求）
// ============================================================

export async function getDiscs({ l = 0, r = 6, sort = "ad", type = "" } = {}) {
  return apiGet("getdiscs", { l, r, sort, type });
}

export async function getLabels({ l = 0, r = 6 } = {}) {
  return apiGet("getlabels", { l, r });
}

export async function getSomeCover({ l = 0, r = 6 } = {}) {
  return apiGet("getsomecover", { l, r });
}

export async function getDiscInfo(discid) {
  return apiGet("getthisdicsinfo", { discid });
}

export async function getDiscBuyers(discid, { l = 0, r = 20 } = {}) {
  return apiGet("getdiscbuyers", { discid, l, r });
}

export async function getDiscComments(discid, { l = 0, r = 20 } = {}) {
  return apiGet("getdisccommit", { discid, l, r });
}

export async function getLabelDiscs(labelid, { l = 0, r = 20 } = {}) {
  return apiGet("getlabeldiscs", { labelid, l, r });
}

export async function getTrackDownloadUrl(discid, trackid, packtype) {
  const params = { discid, trackid };
  if (packtype) params.packtype = packtype;
  return apiGet("gettrackdownloadurl", params);
}

export async function getTags({ l = 0, r = 9, tag = "" } = {}) {
  return apiGet("tags", { l, r, tag });
}

export async function search(keyword, { l = 0, r = 9 } = {}) {
  return apiGet("search", { keyword, l, r });
}

export async function getMyInfo() {
  return apiGet("getmyinfo");
}

export async function getMyDiscs({ l = 0, r = 9, sort = "ad" } = {}) {
  return apiGet("getmydisc", { l, r, sort });
}

export async function getMyLikes({ l = 0, r = 9, sort = "ad" } = {}) {
  return apiGet("getmylike", { l, r, sort });
}

export async function getMyFollowing({ l = 0, r = 9, sort = "ad" } = {}) {
  return apiGet("getmyfollowing", { l, r, sort });
}

export async function getOtherUserInfo(uid, { l = 0, r = 9 } = {}) {
  return apiGet("getotheruserinfo", { uid, l, r });
}

export async function getLabelInfo(labelid, { l = 0, r = 9 } = {}) {
  return apiGet("gethislabelinfo", { labelid, l, r });
}

// ============================================================
// 播放列表操作（跨组件共享）
// ============================================================

/**
 * 将专辑/作品添加到播放列表（通过全局事件）
 * @param {Object} item - 专辑/作品对象，需包含 id, title, cover 等字段
 * @param {string} [artist] - 艺术家名称，不传则使用 item.label 或 "未知艺术家"
 */
export function addDiscToPlaylist(item, artist) {
  if (!item || !item.id) return;
  const placeholderTrack = {
    id: item.id,
    discid: item.id,
    title: item.title || item.name,
    authers: artist || item.label || "未知艺术家",
    album: item.title || item.name,
    coverurl: item.cover,
    url: "",
  };
  window.dispatchEvent(
    new CustomEvent("add-to-playlist", {
      detail: {
        songs: [placeholderTrack],
        discid: item.id,
        album: item.title,
        coverurl: item.cover,
      },
    }),
  );
}

// ============================================================
// 下载功能
// ============================================================

/**
 * 获取专辑页面的 HTML
 */
export async function fetchDiscPageHtml(discId, csrfToken, sessionId) {
  if (isTauri) {
    return tauriInvoke("fetch_disc_page_html", {
      discId,
      csrfToken: csrfToken || "",
      sessionId: sessionId || "",
    });
  }
  const url = `https://www.dizzylab.net/d/${discId}/`;
  const headers = { Referer: "https://www.dizzylab.net" };
  const cookie = buildCookieHeader(csrfToken, sessionId);
  if (cookie) headers["Cookie"] = cookie;
  const response = await fetch(url, { headers });
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  return response.text();
}

/**
 * 从专辑页面 HTML 中解析下载链接
 * @param {string} html - 专辑页面的 HTML
 * @returns {Promise<Array<{label: string, url: string}>>} 下载链接列表
 */
export async function parseDownloadLinks(html) {
  if (isTauri) {
    return tauriInvoke("parse_download_links", { html });
  }
  const links = [];

  // 优先解析下拉菜单中的多格式下载链接
  const dropdownMatch = html.match(
    /<div[^>]*class="[^>]*dropdown-menu[^>]*"[^>]*>([\s\S]*?)<\/div>\s*</i,
  );
  if (dropdownMatch) {
    const dropdownHtml = dropdownMatch[1];
    const itemRegex =
      /<a[^>]*class="[^>]*dropdown-item[^>]*"[^>]*href="([^"]*)"[^>]*>([\s\S]*?)<\/a>/gi;
    let match;
    while ((match = itemRegex.exec(dropdownHtml)) !== null) {
      const href = match[1];
      if (href.startsWith("/albums/download/")) {
        const label = match[2].replace(/<[^>]*>/g, "").trim();
        links.push({
          label,
          url: `https://www.dizzylab.net${href}`,
        });
      }
    }
  }

  // 没有下拉菜单时，尝试解析单个下载链接（download_gift）
  if (links.length === 0) {
    const singleRegex =
      /<a[^>]*href="(\/albums\/download_gift\/[^"]*)"[^>]*>([\s\S]*?)<\/a>/gi;
    let match;
    while ((match = singleRegex.exec(html)) !== null) {
      const href = match[1];
      links.push({
        label: "下载商品",
        url: `https://www.dizzylab.net${href}`,
      });
    }
  }
  return links;
}

/**
 * 获取系统默认下载目录（~/Downloads/DizzyPlay）
 */
export async function getDefaultDownloadDir() {
  if (isTauri) {
    try {
      return await tauriInvoke("get_default_download_dir");
    } catch (e) {
      console.warn("[API] 获取默认下载目录失败:", e);
    }
  }
  return "";
}

export async function downloadFile(
  url,
  savePath,
  csrfToken = "",
  sessionId = "",
) {
  if (isTauri) {
    return tauriInvoke("download_file", {
      url,
      savePath,
      csrfToken,
      sessionId,
    });
  }
  const headers = { Referer: "https://www.dizzylab.net" };
  const cookie = buildCookieHeader(csrfToken, sessionId);
  if (cookie) headers["Cookie"] = cookie;

  const response = await fetch(url, { headers });
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  const blob = await response.blob();
  const blobUrl = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = blobUrl;
  a.download = url.split("/").pop().split("?")[0] || "download";
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(blobUrl);
  return url;
}
