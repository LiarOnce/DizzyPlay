// 换算时间
export function formatDuration(seconds) {
  if (!seconds || seconds <= 0) return "--:--";
  const m = Math.floor(seconds / 60);
  const s = Math.floor(seconds % 60);
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

// 换算大小
export function formatSize(bytes) {
  if (!bytes || bytes <= 0) return "0 B";
  if (bytes < 1024 * 1024) {
    return (bytes / 1024).toFixed(1) + " KB";
  }
  const mb = bytes / (1024 * 1024);
  if (bytes < 1024 * 1024 * 1024) {
    return mb.toFixed(1) + " MB";
  }
  return (bytes / (1024 * 1024 * 1024)).toFixed(2) + " GB";
}

export function formatSizeMB(bytes) {
  if (!bytes || bytes <= 0) return "0 MB";
  return (bytes / (1024 * 1024)).toFixed(2) + " MB";
}

export const isTauri =
  typeof window !== "undefined" &&
  window.__TAURI_INTERNALS__ &&
  typeof window.__TAURI_INTERNALS__.invoke === "function";

export function buildCookieHeader(csrfToken, sessionId) {
  const cookies = [];
  if (csrfToken) cookies.push(`csrftoken=${csrfToken}`);
  if (sessionId) cookies.push(`sessionid=${sessionId}`);
  return cookies.length > 0 ? cookies.join("; ") : "";
}

export async function getAuthCredentials() {
  const { loadUserConfig } = await import("../services/api.js");
  let csrfToken = "";
  let sessionId = "";
  if (isTauri) {
    try { const v = await loadUserConfig("csrfToken"); if (v) csrfToken = v; } catch {}
    try { const v = await loadUserConfig("sessionid"); if (v) sessionId = v; } catch {}
  }
  if (!csrfToken) csrfToken = localStorage.getItem("csrfToken") || "";
  if (!sessionId) sessionId = localStorage.getItem("sessionid") || "";
  return { csrfToken, sessionId };
}
