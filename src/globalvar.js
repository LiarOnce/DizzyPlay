// 全局获取 2000 以内的数据并缓存（即 r = 2000）
export const globalOffsets = 2000;

// JSON 数据缓存有效期（1天）
export const CACHE_MAX_AGE_SECS = 86400;

// 图片缓存有效期（7天）
export const IMAGE_CACHE_MAX_AGE_SECS = 7 * 24 * 3600;

// 请求头
export const DIZZYLAB_REFERER = "https://www.dizzylab.net";
export const DIZZYLAB_USER_AGENT =
  "Mozilla/5.0 (X11; Linux x86_64; rv:150.0) Gecko/20100101 Firefox/150.0";
