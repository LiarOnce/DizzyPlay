import { loadUserConfig } from "./api.js";
import { listen } from "@tauri-apps/api/event";
import { isTauri, buildCookieHeader, getAuthCredentials } from "../utils/format.js";

// ===== 下载状态常量 =====
export const DownloadStatus = {
  PENDING: "pending",
  DOWNLOADING: "downloading",
  PAUSED: "paused",
  COMPLETED: "completed",
  FAILED: "failed",
  CANCELLED: "cancelled",
  EXTRACTING: "extracting",
};

// ===== 存储键 =====
const STORAGE_KEY = "downloadTasks";

class DownloadManager {
  constructor() {
    /** @type {Array<Object>} 下载任务列表 */
    this.tasks = [];
    /** @type {Set<string>} 正在下载的任务 ID 集合 */
    this.activeDownloads = new Set();
    /** @type {Map<string, Function>} 进度事件取消函数 */
    this.unlistenMap = new Map();
    /** @type {Set<Function>} 任务更新回调 */
    this._listeners = new Set();
    /** @type {boolean} 是否已初始化 */
    this._initialized = false;
  }

  /**
   * 初始化：加载持久化任务、注册全局事件监听
   */
  init() {
    if (this._initialized) return;
    this._initialized = true;

    this._loadTasks();

    // 监听来自 AlbumDetail 等组件的添加任务事件
    window.addEventListener("add-download-task", (event) => {
      this.addTask(
        event.detail.url,
        event.detail.label,
        event.detail.discId,
        event.detail.albumTitle,
      );
    });

    console.log("[DownloadManager] 初始化完成，当前任务数:", this.tasks.length);
  }

  /**
   * 销毁：清理事件监听
   */
  destroy() {
    window.removeEventListener("add-download-task", this._boundAddHandler);
    this._listeners.clear();
  }

  /**
   * 注册任务更新回调
   * @param {Function} callback
   * @returns {Function} 取消注册的函数
   */
  onUpdate(callback) {
    this._listeners.add(callback);
    return () => this._listeners.delete(callback);
  }

  /**
   * 通知所有监听者
   */
  _notify() {
    for (const cb of this._listeners) {
      try {
        cb(this.tasks);
      } catch (e) {
        console.warn("[DownloadManager] 通知回调出错:", e);
      }
    }
  }

  // ===== 持久化 =====

  _loadTasks() {
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved) {
        this.tasks = JSON.parse(saved);
        // 恢复时，将 DOWNLOADING 状态重置为 PENDING（进程已中断）
        for (const task of this.tasks) {
          if (task.status === DownloadStatus.DOWNLOADING) {
            task.status = DownloadStatus.PENDING;
          }
        }
      }
    } catch (e) {
      console.warn("[DownloadManager] 加载任务失败:", e);
    }
  }

  _saveTasks() {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(this.tasks));
    } catch (e) {
      console.warn("[DownloadManager] 保存任务失败:", e);
    }
  }

  // ===== 任务管理 =====

  /**
   * 获取所有任务
   */
  getTasks() {
    return [...this.tasks];
  }

  /**
   * 根据 ID 查找任务
   */
  getTask(taskId) {
    return this.tasks.find((t) => t.id === taskId);
  }

  /**
   * 添加下载任务
   */
  addTask(url, label, discId, albumTitle) {
    if (!url) return null;

    // 检查是否已存在相同 URL 的任务
    const existing = this.tasks.find((t) => t.url === url);
    if (existing) {
      if (existing.status === DownloadStatus.PENDING) {
        this.startTask(existing.id);
      }
      return existing;
    }

    const task = {
      id: Date.now().toString(36) + Math.random().toString(36).slice(2, 6),
      url,
      label: label || "未知文件",
      discId: discId || "",
      albumTitle: albumTitle || "",
      status: DownloadStatus.PENDING,
      progress: 0,
      totalBytes: 0,
      downloadedBytes: 0,
      savePath: "",
      saveFilePath: "",
      extractedPath: "",
      createdAt: Date.now(),
      completedAt: null,
      error: "",
    };

    this.tasks.unshift(task);
    this._saveTasks();
    this._notify();

    // 自动开始下载
    this.startTask(task.id);

    return task;
  }

  /**
   * 开始下载任务
   */
  async startTask(taskId) {
    const task = this.tasks.find((t) => t.id === taskId);
    if (!task) return;

    // 清除旧的错误状态
    task.error = "";
    task.progress = 0;

    // 如果是暂停后恢复，保留已下载字节数作为续传偏移量
    // 如果是失败后重试或全新开始，重置下载计数
    if (task.status !== DownloadStatus.PAUSED) {
      task.downloadedBytes = 0;
      task.totalBytes = 0;
    }

    // 获取下载保存路径
    let savePath = "";
    if (isTauri) {
      try {
        const path = await loadUserConfig("downloadPath");
        if (path) savePath = path;
      } catch (e) {
        // 忽略
      }
    }
    if (!savePath) {
      savePath = localStorage.getItem("downloadPath") || "";
    }
    if (!savePath) {
      try {
        const { getDefaultDownloadDir } = await import("./api.js");
        const defaultDir = await getDefaultDownloadDir();
        if (defaultDir) savePath = defaultDir;
      } catch (e) {
        // 忽略
      }
    }
    if (!savePath) {
      savePath = "downloads";
    }

    // 获取认证凭据
    const { csrfToken, sessionId } = await this._getAuthCredentials();

    task.status = DownloadStatus.DOWNLOADING;
    task.savePath = savePath;
    this.activeDownloads.add(taskId);
    this._saveTasks();
    this._notify();

    if (isTauri) {
      await this._tauriDownload(task, csrfToken, sessionId);
    } else {
      await this._browserDownload(task, csrfToken, sessionId);
    }

    this.activeDownloads.delete(taskId);
    this._saveTasks();
    this._notify();
  }

  /**
   * Tauri 环境下载
   */
  async _tauriDownload(task, csrfToken, sessionId) {
    const { invoke } = window.__TAURI_INTERNALS__;

    // 监听下载进度事件
    let unlisten = null;
    try {
      unlisten = await listen("download-progress", (event) => {
        const data = event.payload;
        const t = this.tasks.find((t) => t.id === task.id);
        if (t) {
          t.downloadedBytes = data.downloaded || 0;
          t.totalBytes = data.total || 0;
          if (data.percent > 0) {
            t.progress = data.percent;
          } else if (data.total > 0) {
            t.progress = Math.round((data.downloaded / data.total) * 100);
          }
          this._notify();
        }
      });
    } catch (e) {
      console.warn("[DownloadManager] 监听进度事件失败:", e);
    }

    try {
      const result = await invoke("download_file", {
        url: task.url,
        savePath: task.savePath,
        csrfToken: csrfToken,
        sessionId: sessionId,
        taskId: task.id,
        offset: task.downloadedBytes || 0,
      });

      task.saveFilePath = result;
      task.progress = 100;
      task.completedAt = Date.now();
      console.log("[DownloadManager] 下载完成:", result);

      // 检查是否需要自动解压
      const autoExtract = await this._getAutoExtract();
      if (autoExtract) {
        try {
          await this._extractArchive(task);
          task.status = DownloadStatus.COMPLETED;
        } catch (extractErr) {
          task.status = DownloadStatus.FAILED;
          task.error = String(extractErr);
          task.completedAt = null;
        }
      } else {
        task.status = DownloadStatus.COMPLETED;
      }
    } catch (err) {
      const errMsg = String(err);
      console.error("[DownloadManager] 下载失败:", errMsg);

      // 检测暂停信号：Rust 端返回 "下载已暂停" 表示用户请求暂停
      if (errMsg.includes("下载已暂停")) {
        task.status = DownloadStatus.PAUSED;
        // 保留已下载的字节数，用于后续续传
        console.log(
          "[DownloadManager] 任务已暂停:",
          task.id,
          "已下载:",
          task.downloadedBytes,
          "bytes",
        );
      } else {
        task.status = DownloadStatus.FAILED;
        task.error = errMsg;
      }
    } finally {
      if (unlisten) {
        try {
          unlisten();
        } catch (e) {
          // 忽略
        }
      }
    }
  }

  /**
   * 浏览器下载
   */
  async _browserDownload(task, csrfToken, sessionId) {
    try {
      const headers = { Referer: "https://www.dizzylab.net" };
      const cookie = buildCookieHeader(csrfToken, sessionId);
      if (cookie) headers["Cookie"] = cookie;

      const response = await fetch(task.url, { headers });
      if (!response.ok) throw new Error(`HTTP ${response.status}`);

      const contentLength = response.headers.get("content-length");
      const total = contentLength ? parseInt(contentLength, 10) : 0;
      const reader = response.body.getReader();
      const chunks = [];
      let downloaded = 0;

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        chunks.push(value);
        downloaded += value.length;
        if (total > 0) {
          task.progress = Math.round((downloaded / total) * 100);
          task.downloadedBytes = downloaded;
          task.totalBytes = total;
          this._notify();
        }
      }

      const blob = new Blob(chunks);
      const blobUrl = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = blobUrl;
      a.download = task.label || "download";
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(blobUrl);

      task.status = DownloadStatus.COMPLETED;
      task.progress = 100;
      task.completedAt = Date.now();
    } catch (err) {
      console.error("[DownloadManager] 下载失败:", err);
      task.status = DownloadStatus.FAILED;
      task.error = String(err);
    }
  }

  /**
   * 暂停下载任务
   */
  async pauseTask(taskId) {
    const task = this.tasks.find((t) => t.id === taskId);
    if (!task || task.status !== DownloadStatus.DOWNLOADING) return;

    // Tauri 环境：调用 Rust 取消命令，触发下载循环退出
    if (isTauri) {
      try {
        const { invoke } = window.__TAURI_INTERNALS__;
        await invoke("cancel_download", { taskId: task.id });
        console.log("[DownloadManager] 已发送暂停信号:", task.id);
      } catch (e) {
        console.warn("[DownloadManager] 发送暂停信号失败:", e);
      }
    }

    // 标记为暂停（_tauriDownload 的 catch 中也会设置 PAUSED，
    // 但这里先标记确保 UI 即时响应）
    task.status = DownloadStatus.PAUSED;
    this.activeDownloads.delete(taskId);
    this._saveTasks();
    this._notify();
  }

  /**
   * 继续下载任务（重试失败或继续暂停的）
   */
  resumeTask(taskId) {
    const task = this.tasks.find((t) => t.id === taskId);
    if (
      !task ||
      (task.status !== DownloadStatus.PAUSED &&
        task.status !== DownloadStatus.FAILED)
    )
      return;
    this.startTask(taskId);
  }

  /**
   * 取消下载任务
   */
  async cancelTask(taskId) {
    const task = this.tasks.find((t) => t.id === taskId);
    if (!task) return;

    // 如果正在下载，先发送取消信号给 Rust
    if (task.status === DownloadStatus.DOWNLOADING && isTauri) {
      try {
        const { invoke } = window.__TAURI_INTERNALS__;
        await invoke("cancel_download", { taskId: task.id });
      } catch (e) {
        console.warn("[DownloadManager] 发送取消信号失败:", e);
      }
    }

    task.status = DownloadStatus.CANCELLED;
    this.activeDownloads.delete(taskId);
    this._saveTasks();
    this._notify();
  }

  /**
   * 删除下载任务
   */
  removeTask(taskId) {
    const idx = this.tasks.findIndex((t) => t.id === taskId);
    if (idx === -1) return;
    this.activeDownloads.delete(taskId);
    this.tasks.splice(idx, 1);
    this._saveTasks();
    this._notify();
  }

  /**
   * 清除所有已完成/失败/已取消的任务
   */
  clearFinished() {
    this.tasks = this.tasks.filter(
      (t) =>
        t.status === DownloadStatus.DOWNLOADING ||
        t.status === DownloadStatus.PENDING ||
        t.status === DownloadStatus.PAUSED,
    );
    this._saveTasks();
    this._notify();
  }

  /**
   * 获取自动解压设置
   */
  async _getAutoExtract() {
    if (isTauri) {
      try {
        const val = await loadUserConfig("autoExtract");
        return val === "true";
      } catch (e) {
        // 忽略
      }
    }
    return localStorage.getItem("autoExtract") === "true";
  }

  /**
   * 解压已下载的压缩包
   */
  async _extractArchive(task) {
    if (!isTauri || !task.saveFilePath) return;

    task.status = DownloadStatus.EXTRACTING;
    this._notify();

    const { invoke } = window.__TAURI_INTERNALS__;
    const extractedPath = await invoke("extract_archive", {
      archivePath: task.saveFilePath,
      deleteAfter: true, // 解压后直接删除压缩包
    });

    task.extractedPath = extractedPath;
    console.log("[DownloadManager] 解压完成:", extractedPath);
  }

  /**
   * 获取认证凭据
   */
  async _getAuthCredentials() {
    return getAuthCredentials();
  }

  // ===== 统计 =====

  get activeCount() {
    return this.tasks.filter(
      (t) =>
        t.status === DownloadStatus.DOWNLOADING ||
        t.status === DownloadStatus.PENDING ||
        t.status === DownloadStatus.PAUSED,
    ).length;
  }

  get completedCount() {
    return this.tasks.filter((t) => t.status === DownloadStatus.COMPLETED)
      .length;
  }
}

// ===== 导出单例 =====
export const downloadManager = new DownloadManager();

// 自动初始化
if (typeof window !== "undefined") {
  downloadManager.init();
}
