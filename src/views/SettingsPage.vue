<script setup>
import { ref, onMounted, onUnmounted, computed } from "vue";
import {
  Setting,
  DataAnalysis,
  InfoFilled,
  Loading,
  Delete,
  Download,
  WarningFilled,
} from "@element-plus/icons-vue";
import { saveUserConfig, loadUserConfig } from "../services/api.js";
import { isTauri, formatSizeMB as formatSize } from "../utils/format.js";
import aboutHtml from "/src/assets/about.html?raw";

// ===== 通用设置 =====
const use320kbps = ref(false);
const csrfToken = ref("");
const sessionId = ref("");
const configLoaded = ref(false);

async function loadConfig() {
  if (!isTauri) {
    // 非 Tauri 环境使用 localStorage
    const val = localStorage.getItem("settings_use320kbps");
    use320kbps.value = val === "true";
    csrfToken.value = localStorage.getItem("csrfToken") || "";
    sessionId.value = localStorage.getItem("sessionid") || "";
    configLoaded.value = true;
    return;
  }
  try {
    const val = await loadUserConfig("use320kbps");
    use320kbps.value = val === "true";
  } catch (e) {
    console.warn("[Settings] 加载配置失败:", e);
  }
  try {
    const val = await loadUserConfig("csrfToken");
    csrfToken.value = val || "";
  } catch (e) {
    console.warn("[Settings] 加载 CSRF Token 失败:", e);
  }
  try {
    const val = await loadUserConfig("sessionid");
    sessionId.value = val || "";
  } catch (e) {
    console.warn("[Settings] 加载 Session ID 失败:", e);
  }
  configLoaded.value = true;
}

async function saveConfig() {
  if (!isTauri) {
    localStorage.setItem(
      "settings_use320kbps",
      use320kbps.value ? "true" : "false",
    );
    localStorage.setItem("csrfToken", csrfToken.value);
    localStorage.setItem("sessionid", sessionId.value);
    return;
  }
  try {
    await saveUserConfig("use320kbps", use320kbps.value ? "true" : "false");
    console.log("[Settings] 已保存配置: use320kbps =", use320kbps.value);
  } catch (e) {
    console.warn("[Settings] 保存配置失败:", e);
  }
  try {
    await saveUserConfig("csrfToken", csrfToken.value);
    console.log("[Settings] 已保存 CSRF Token");
  } catch (e) {
    console.warn("[Settings] 保存 CSRF Token 失败:", e);
  }
  try {
    await saveUserConfig("sessionid", sessionId.value);
    console.log("[Settings] 已保存 Session ID");
  } catch (e) {
    console.warn("[Settings] 保存 Session ID 失败:", e);
  }
}

function onUse320kbpsChange(val) {
  use320kbps.value = val;
  saveConfig();
  // 触发全局事件，通知其他组件 320Kbps 设置已变更
  window.dispatchEvent(
    new CustomEvent("settings-changed", {
      detail: { key: "use320kbps", value: val },
    }),
  );
}

function onCsrfTokenChange() {
  saveConfig();
}

function onSessionIdChange() {
  saveConfig();
}

// ===== 下载设置 =====
const downloadPath = ref("");
const autoExtract = ref(false);

async function loadDownloadPath() {
  if (!isTauri) {
    downloadPath.value = localStorage.getItem("downloadPath") || "";
    return;
  }
  try {
    const val = await loadUserConfig("downloadPath");
    downloadPath.value = val || "";
  } catch (e) {
    console.warn("[Settings] 加载下载路径失败:", e);
  }
}

async function saveDownloadPath() {
  if (!isTauri) {
    localStorage.setItem("downloadPath", downloadPath.value);
    return;
  }
  try {
    await saveUserConfig("downloadPath", downloadPath.value);
    console.log("[Settings] 已保存下载路径:", downloadPath.value);
  } catch (e) {
    console.warn("[Settings] 保存下载路径失败:", e);
  }
}

function onDownloadPathChange() {
  saveDownloadPath();
}

// ===== 解压设置 =====

async function loadAutoExtract() {
  if (!isTauri) {
    autoExtract.value = localStorage.getItem("autoExtract") === "true";
    return;
  }
  try {
    const val = await loadUserConfig("autoExtract");
    autoExtract.value = val === "true";
  } catch (e) {
    console.warn("[Settings] 加载自动解压设置失败:", e);
  }
}

async function saveAutoExtract() {
  if (!isTauri) {
    localStorage.setItem("autoExtract", autoExtract.value ? "true" : "false");
    return;
  }
  try {
    await saveUserConfig("autoExtract", autoExtract.value ? "true" : "false");
    console.log("[Settings] 已保存自动解压设置:", autoExtract.value);
  } catch (e) {
    console.warn("[Settings] 保存自动解压设置失败:", e);
  }
}

function onAutoExtractChange(val) {
  autoExtract.value = val;
  saveAutoExtract();
}

// ===== 数据管理 =====
const datasSize = ref(0);
const imagesSize = ref(0);
const musicSize = ref(0);
const sizesLoading = ref(false);
const clearingType = ref(""); // 正在清除的类型

async function getDirSize(dirType) {
  if (!isTauri) return 0;
  try {
    const { invoke } = window.__TAURI_INTERNALS__;
    const bytes = await invoke("get_cache_dir_size", { dirType });
    return bytes;
  } catch (e) {
    console.warn(`[Settings] 获取 ${dirType} 大小失败:`, e);
    return 0;
  }
}

async function loadSizes() {
  sizesLoading.value = true;
  try {
    const [datas, images, music] = await Promise.all([
      getDirSize("datas"),
      getDirSize("images"),
      getDirSize("music"),
    ]);
    datasSize.value = datas;
    imagesSize.value = images;
    musicSize.value = music;
  } catch (e) {
    console.warn("[Settings] 加载缓存大小失败:", e);
  } finally {
    sizesLoading.value = false;
  }
}

async function clearDir(dirType) {
  if (!isTauri) return;
  clearingType.value = dirType;
  try {
    const { invoke } = window.__TAURI_INTERNALS__;
    const result = await invoke("clear_cache_dir", { dirType });
    console.log("[Settings] 清除结果:", result);
    // 重新加载大小
    await loadSizes();
  } catch (e) {
    console.warn(`[Settings] 清除 ${dirType} 失败:`, e);
  } finally {
    clearingType.value = false;
  }
}

async function clearAll() {
  if (!isTauri) return;
  clearingType.value = "all";
  try {
    const { invoke } = window.__TAURI_INTERNALS__;
    const result = await invoke("clear_cache_dir", { dirType: "all" });
    console.log("[Settings] 一键清除结果:", result);
    await loadSizes();
  } catch (e) {
    console.warn("[Settings] 一键清除失败:", e);
  } finally {
    clearingType.value = false;
  }
}

const totalSize = computed(() => {
  return datasSize.value + imagesSize.value + musicSize.value;
});

// ===== 关于页面 =====
const aboutContent = ref("");

function loadAboutMd() {
  aboutContent.value = aboutHtml || "";
}

// ===== 生命周期 =====
onMounted(() => {
  loadConfig();
  loadSizes();
  loadAboutMd();
  loadDownloadPath();
  loadAutoExtract();
});
</script>

<template>
  <div class="settings-page">
    <div class="settings-header">
      <h2>软件设置</h2>
    </div>

    <el-tabs tab-position="left" class="settings-tabs">
      <!-- 通用设置 -->
      <el-tab-pane>
        <template #label>
          <span class="tab-label">
            <el-icon><Setting /></el-icon>
            <span>通用</span>
          </span>
        </template>

        <div class="tab-content">
          <h3>通用设置</h3>
          <div class="settings-list">
            <div class="setting-item" v-if="configLoaded">
              <div class="setting-info">
                <span class="setting-title">使用 320Kbps 在线播放 (Beta)</span>
                <span class="setting-desc"
                  >这会消耗更多空间和流量，目前需要清空播放列表并重新添加才能生效</span
                >
              </div>
              <div class="setting-control">
                <el-switch v-model="use320kbps" @change="onUse320kbpsChange" />
              </div>
            </div>

            <div v-if="!configLoaded" class="setting-item">
              <el-icon class="is-loading"><Loading /></el-icon>
              <span>加载配置中...</span>
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- 认证设置 -->
      <el-tab-pane>
        <template #label>
          <span class="tab-label">
            <el-icon><Setting /></el-icon>
            <span>认证</span>
          </span>
        </template>

        <div class="tab-content">
          <h3>认证设置</h3>
          <p>
            由于缺少 API 的原因，如果需要通过 DizzyPlay 下载已购商品，请登录
            <a href="https://www.dizzylab.net/albums/login/" target="_blank"
              >Dizzylab 网页版</a
            >
            后使用<code>Ctrl+Shift+I</code>或<code>F12</code>快捷键，选择存储-Cookie并根据以下设置项的介绍填写。
          </p>
          <br />
          <div class="settings-list">
            <div class="setting-item" v-if="configLoaded">
              <div class="setting-info">
                <span class="setting-title">CSRF Token</span>
                <span class="setting-desc"
                  >用于需要 CSRF 验证的请求 (Cookie: csrftoken)</span
                >
              </div>
              <div class="setting-control">
                <el-input
                  v-model="csrfToken"
                  placeholder="请输入 CSRF Token"
                  size="small"
                  style="width: 320px"
                  @change="onCsrfTokenChange"
                />
              </div>
            </div>

            <div class="setting-item" v-if="configLoaded">
              <div class="setting-info">
                <span class="setting-title">Session ID</span>
                <span class="setting-desc"
                  >用于下载认证的会话标识 (Cookie: sessionid)</span
                >
              </div>
              <div class="setting-control">
                <el-input
                  v-model="sessionId"
                  placeholder="请输入 Session ID"
                  size="small"
                  style="width: 320px"
                  @change="onSessionIdChange"
                />
              </div>
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- 下载设置 -->
      <el-tab-pane>
        <template #label>
          <span class="tab-label">
            <el-icon><Download /></el-icon>
            <span>下载</span>
          </span>
        </template>

        <div class="tab-content">
          <h3>下载设置</h3>
          <div class="settings-list">
            <div class="setting-item">
              <div class="setting-info">
                <span class="setting-title">下载保存路径</span>
                <span class="setting-desc">下载的专辑文件将保存到此目录</span>
              </div>
              <div class="setting-control">
                <el-input
                  v-model="downloadPath"
                  placeholder="默认：程序所在目录/downloads"
                  size="small"
                  style="width: 280px"
                  @change="onDownloadPathChange"
                />
              </div>
            </div>

            <div class="setting-item">
              <div class="setting-info">
                <span class="setting-title">下载后自动解压</span>
                <span class="setting-desc">
                  下载完成后自动解压 zip 压缩包并删除原文件
                </span>
              </div>
              <div class="setting-control">
                <el-switch v-model="autoExtract" @change="onAutoExtractChange" />
              </div>
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- 数据管理 -->
      <el-tab-pane>
        <template #label>
          <span class="tab-label">
            <el-icon><DataAnalysis /></el-icon>
            <span>数据</span>
          </span>
        </template>

        <div class="tab-content">
          <h3>缓存管理</h3>
          <div class="settings-list">
            <div v-if="sizesLoading" class="loading-hint">
              <el-icon class="is-loading"><Loading /></el-icon>
              <span>计算缓存大小中...</span>
            </div>

            <template v-else>
              <div class="cache-item">
                <div class="cache-info">
                  <span class="cache-title">数据缓存 (datas)</span>
                  <span class="cache-size">{{ formatSize(datasSize) }}</span>
                </div>
                <el-button
                  type="danger"
                  size="small"
                  :icon="Delete"
                  :loading="clearingType === 'datas'"
                  @click="clearDir('datas')"
                >
                  删除
                </el-button>
              </div>

              <div class="cache-item">
                <div class="cache-info">
                  <span class="cache-title">图片缓存 (images)</span>
                  <span class="cache-size">{{ formatSize(imagesSize) }}</span>
                </div>
                <el-button
                  type="danger"
                  size="small"
                  :icon="Delete"
                  :loading="clearingType === 'images'"
                  @click="clearDir('images')"
                >
                  删除
                </el-button>
              </div>

              <div class="cache-item">
                <div class="cache-info">
                  <span class="cache-title">音乐缓存 (music)</span>
                  <span class="cache-size">{{ formatSize(musicSize) }}</span>
                </div>
                <el-button
                  type="danger"
                  size="small"
                  :icon="Delete"
                  :loading="clearingType === 'music'"
                  @click="clearDir('music')"
                >
                  删除
                </el-button>
              </div>

              <el-divider />

              <div class="cache-total">
                <div class="cache-info">
                  <span class="cache-title">总计</span>
                  <span class="cache-size total">{{
                    formatSize(totalSize)
                  }}</span>
                </div>
                <el-button
                  type="danger"
                  size="default"
                  :icon="Delete"
                  :loading="clearingType === 'all'"
                  @click="clearAll"
                >
                  一键删除全部缓存
                </el-button>
              </div>
            </template>
          </div>
        </div>
      </el-tab-pane>

      <!-- 关于 -->
      <el-tab-pane>
        <template #label>
          <span class="tab-label">
            <el-icon><InfoFilled /></el-icon>
            <span>关于</span>
          </span>
        </template>

        <div class="tab-content">
          <div class="about-content" v-html="aboutContent"></div>
        </div>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<style scoped>
.settings-page {
  padding: 24px;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.settings-header {
  margin-bottom: 20px;
}

.settings-header h2 {
  margin: 0;
  font-size: 22px;
  font-weight: 600;
  color: var(--text-primary, #e0e0e0);
}

.settings-tabs {
  flex: 1;
  overflow: hidden;
}

.settings-tabs :deep(.el-tabs__content) {
  overflow: hidden;
  height: 100%;
}

.settings-tabs :deep(.el-tab-pane) {
  height: 100%;
  overflow: hidden;
}

.settings-tabs :deep(.el-tabs__item) {
  height: 48px;
  line-height: 48px;
  font-size: 14px;
}

.tab-label {
  display: flex;
  align-items: center;
  gap: 8px;
}

.tab-content {
  padding: 0 16px;
  height: 100%;
  overflow-y: auto;
}

.tab-content h3 {
  margin: 0 0 20px 0;
  font-size: 18px;
  font-weight: 500;
  color: var(--text-primary, #e0e0e0);
}

.settings-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px;
  background: var(--bg-secondary, #1e1e2e);
  border-radius: 8px;
  border: 1px solid var(--border-color, #2d2d3d);
}

.setting-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.setting-title {
  font-size: 15px;
  font-weight: 500;
  color: var(--text-primary, #e0e0e0);
}

.setting-desc {
  font-size: 12px;
  color: var(--text-secondary, #888);
}

.setting-control {
  flex-shrink: 0;
}

.cache-item,
.cache-total {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px;
  background: var(--bg-secondary, #1e1e2e);
  border-radius: 8px;
  border: 1px solid var(--border-color, #2d2d3d);
}

.cache-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.cache-title {
  font-size: 14px;
  color: var(--text-primary, #e0e0e0);
}

.cache-size {
  font-size: 13px;
  color: var(--text-secondary, #888);
  font-family: monospace;
}

.cache-size.total {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-primary, #409eff);
}

.loading-hint {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 24px;
  color: var(--text-secondary, #888);
  font-size: 14px;
}

.about-content {
  padding: 16px;
  background: var(--bg-secondary, #1e1e2e);
  border-radius: 8px;
  border: 1px solid var(--border-color, #2d2d3d);
  min-height: 200px;
  line-height: 1.8;
}

.about-content :deep(h1) {
  color: var(--text-primary, #e0e0e0);
  margin-bottom: 8px;
}
</style>
