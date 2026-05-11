<script setup>
import { ref, onMounted, onUnmounted, computed } from "vue";
import {
  Download,
  Delete,
  FolderOpened,
  VideoPause,
  VideoPlay,
  Close,
  Loading,
  Check
} from "@element-plus/icons-vue";
import { loadUserConfig } from "../services/api.js";
import {
  downloadManager,
  DownloadStatus,
} from "../services/downloadManager.js";
import { formatSize, isTauri } from "../utils/format.js";

// ===== 响应式任务列表（从全局管理器同步） =====
const downloadTasks = ref([]);

/**
 * 从全局管理器同步任务列表
 */
function syncTasks() {
  downloadTasks.value = downloadManager.getTasks();
}

// ===== 操作代理到全局管理器 =====

function startDownloadTask(taskId) {
  downloadManager.startTask(taskId);
}

function pauseDownloadTask(taskId) {
  downloadManager.pauseTask(taskId);
}

function resumeDownloadTask(taskId) {
  downloadManager.resumeTask(taskId);
}

function cancelDownloadTask(taskId) {
  downloadManager.cancelTask(taskId);
}

function removeDownloadTask(taskId) {
  downloadManager.removeTask(taskId);
}

function clearFinishedTasks() {
  downloadManager.clearFinished();
}

/**
 * 打开下载文件夹
 */
async function openDownloadFolder() {
  if (!isTauri) return;
  try {
    let savePath = "";
    try {
      const path = await loadUserConfig("downloadPath");
      if (path) savePath = path;
    } catch (e) {
      // 忽略
    }
    if (!savePath) {
      savePath = localStorage.getItem("downloadPath") || "downloads";
    }
    const { invoke } = window.__TAURI_INTERNALS__;
    await invoke("open_download_folder", { path: savePath });
  } catch (e) {
    console.warn("[DownloadPage] 打开下载文件夹失败:", e);
  }
}

// ===== 计算属性 =====
const activeTaskCount = computed(() => downloadManager.activeCount);
const completedTaskCount = computed(() => downloadManager.completedCount);

const statusText = computed(() => {
  const map = {
    [DownloadStatus.PENDING]: "等待中",
    [DownloadStatus.DOWNLOADING]: "下载中",
    [DownloadStatus.PAUSED]: "已暂停",
    [DownloadStatus.COMPLETED]: "已完成",
    [DownloadStatus.FAILED]: "下载失败",
    [DownloadStatus.CANCELLED]: "已取消",
    [DownloadStatus.EXTRACTING]: "解压中",
  };
  return (s) => map[s] || s;
});

const statusClass = computed(() => {
  const map = {
    [DownloadStatus.PENDING]: "status-pending",
    [DownloadStatus.DOWNLOADING]: "status-downloading",
    [DownloadStatus.PAUSED]: "status-paused",
    [DownloadStatus.COMPLETED]: "status-completed",
    [DownloadStatus.FAILED]: "status-failed",
    [DownloadStatus.CANCELLED]: "status-cancelled",
    [DownloadStatus.EXTRACTING]: "status-extracting",
  };
  return (s) => map[s] || "";
});

// ===== 生命周期 =====
let unsubUpdate = null;

onMounted(() => {
  // 初始同步
  syncTasks();
  // 注册更新回调，当全局管理器中的任务变化时自动同步
  unsubUpdate = downloadManager.onUpdate(() => {
    syncTasks();
  });
});

onUnmounted(() => {
  if (unsubUpdate) {
    unsubUpdate();
    unsubUpdate = null;
  }
  // 注意：不销毁 downloadManager，让下载在后台继续运行
});
</script>

<template>
  <div class="download-page">
    <div class="page-header">
      <h2>下载管理</h2>
      <div class="header-actions">
        <span class="task-summary">
          活跃 {{ activeTaskCount }} / 已完成 {{ completedTaskCount }}
        </span>
        <el-button
          size="small"
          :icon="FolderOpened"
          @click="openDownloadFolder"
          v-if="isTauri"
        >
          打开下载文件夹
        </el-button>
        <el-button
          size="small"
          :icon="Delete"
          @click="clearFinishedTasks"
          :disabled="downloadTasks.length === 0"
        >
          清除已完成
        </el-button>
      </div>
    </div>

    <!-- 下载列表 -->
    <div class="download-list" v-if="downloadTasks.length > 0">
      <div
        v-for="task in downloadTasks"
        :key="task.id"
        class="download-item"
        :class="statusClass(task.status)"
      >
        <div class="item-info">
          <div class="item-title">{{ task.albumTitle || task.label }}</div>
          <div class="item-meta">
            <span class="item-status" :class="statusClass(task.status)">
              {{ statusText(task.status) }}
            </span>
            <span
              v-if="task.albumTitle && task.label !== task.albumTitle"
              class="item-album"
            >
              {{ task.label }}
            </span>
            <span v-if="task.error" class="item-error">
              {{ task.error }}
            </span>
          </div>

          <!-- 进度条 -->
          <div
            v-if="
              task.status === DownloadStatus.DOWNLOADING ||
              task.status === DownloadStatus.PAUSED ||
              task.status === DownloadStatus.PENDING ||
              task.status === DownloadStatus.EXTRACTING
            "
            class="progress-wrapper"
          >
            <el-progress
              v-if="task.status !== DownloadStatus.EXTRACTING"
              :percentage="task.progress"
              :status="task.status === DownloadStatus.PAUSED ? 'warning' : ''"
              :stroke-width="6"
              :width="160"
              style="width: 100%; max-width: 300px"
            />
            <span class="progress-text" v-if="task.status === DownloadStatus.EXTRACTING">
              <el-icon class="is-loading"><Loading /></el-icon> 解压中...
            </span>
            <span class="progress-text" v-else-if="task.totalBytes > 0">
              {{ formatSize(task.downloadedBytes) }} /
              {{ formatSize(task.totalBytes) }}
            </span>
            <span
              class="progress-text"
              v-else-if="task.status === DownloadStatus.DOWNLOADING"
            >
              下载中...
            </span>
          </div>
        </div>

        <div class="item-actions">
          <!-- 下载中 → 暂停 -->
          <el-button
            v-if="task.status === DownloadStatus.DOWNLOADING"
            size="small"
            :icon="VideoPause"
            circle
            @click="pauseDownloadTask(task.id)"
          />
          <!-- 已暂停/失败 → 继续 -->
          <el-button
            v-else-if="
              task.status === DownloadStatus.PAUSED ||
              task.status === DownloadStatus.FAILED
            "
            size="small"
            :icon="VideoPlay"
            circle
            @click="resumeDownloadTask(task.id)"
          />
          <!-- 等待中 → 取消 -->
          <el-button
            v-else-if="task.status === DownloadStatus.PENDING"
            size="small"
            :icon="Close"
            circle
            @click="cancelDownloadTask(task.id)"
          />
          <!-- 解压中 → loading -->
          <el-button
            v-else-if="task.status === DownloadStatus.EXTRACTING"
            size="small"
            :icon="Loading"
            circle
            disabled
          />
          <!-- 已完成 -->
          <el-button
            v-else-if="task.status === DownloadStatus.COMPLETED"
            size="small"
            :icon="Check"
            circle
            disabled
          />

          <!-- 删除 -->
          <el-button
            size="small"
            :icon="Delete"
            circle
            @click="removeDownloadTask(task.id)"
          />
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div class="empty-state" v-else>
      <el-empty description="暂无下载任务" />
    </div>
  </div>
</template>

<style scoped>
.download-page {
  padding: 24px;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.page-header h2 {
  margin: 0;
  font-size: 22px;
  font-weight: 600;
  color: var(--text-primary, #e0e0e0);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.task-summary {
  font-size: 13px;
  color: var(--text-secondary, #888);
}

.download-list {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
  overflow-y: auto;
}

.download-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px;
  background: var(--bg-secondary, #1e1e2e);
  border-radius: 8px;
  border: 1px solid var(--border-color, #2d2d3d);
  transition: border-color 0.2s;
}

.download-item.status-completed {
  border-color: var(--color-success, #67c23a);
  opacity: 0.8;
}

.download-item.status-failed {
  border-color: var(--color-danger, #f56c6c);
}

.download-item.status-downloading {
  border-color: var(--color-primary, #409eff);
}

.download-item.status-extracting {
  border-color: var(--color-warning, #e6a23c);
}

.item-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.item-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary, #e0e0e0);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 12px;
}

.item-status {
  padding: 1px 8px;
  border-radius: 4px;
  font-size: 11px;
}

.item-status.status-pending {
  background: rgba(255, 255, 255, 0.1);
  color: #aaa;
}

.item-status.status-downloading {
  background: rgba(64, 158, 255, 0.15);
  color: #409eff;
}

.item-status.status-paused {
  background: rgba(230, 162, 60, 0.15);
  color: #e6a23c;
}

.item-status.status-completed {
  background: rgba(103, 194, 58, 0.15);
  color: #67c23a;
}

.item-status.status-failed {
  background: rgba(245, 108, 108, 0.15);
  color: #f56c6c;
}

.item-status.status-cancelled {
  background: rgba(255, 255, 255, 0.05);
  color: #666;
}

.item-status.status-extracting {
  background: rgba(230, 162, 60, 0.15);
  color: #e6a23c;
}

.item-album {
  color: var(--text-secondary, #888);
}

.item-error {
  color: #f56c6c;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 300px;
}

.progress-wrapper {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 6px;
}

.progress-text {
  font-size: 11px;
  color: var(--text-secondary, #888);
  white-space: nowrap;
  min-width: 90px;
}

.item-actions {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
  margin-left: 12px;
}

.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
