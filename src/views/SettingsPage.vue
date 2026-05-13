<script setup>
import {
  Setting,
  DataAnalysis,
  InfoFilled,
  Download,
} from "@element-plus/icons-vue";
import CacheManager from "../components/settings/CacheManager.vue";
import GeneralSettings from "../components/settings/GeneralSettings.vue";
import AuthSettings from "../components/settings/AuthSettings.vue";
import DownloadSettings from "../components/settings/DownloadSettings.vue";
import AboutPage from "../components/settings/AboutPage.vue";
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
        <GeneralSettings />
      </el-tab-pane>

      <!-- 认证设置 -->
      <el-tab-pane>
        <template #label>
          <span class="tab-label">
            <el-icon><Setting /></el-icon>
            <span>认证</span>
          </span>
        </template>
        <AuthSettings />
      </el-tab-pane>

      <!-- 下载设置 -->
      <el-tab-pane>
        <template #label>
          <span class="tab-label">
            <el-icon><Download /></el-icon>
            <span>下载</span>
          </span>
        </template>
        <DownloadSettings />
      </el-tab-pane>

      <!-- 数据管理 -->
      <el-tab-pane>
        <template #label>
          <span class="tab-label">
            <el-icon><DataAnalysis /></el-icon>
            <span>数据</span>
          </span>
        </template>

        <CacheManager />
      </el-tab-pane>

      <!-- 关于 -->
      <el-tab-pane>
        <template #label>
          <span class="tab-label">
            <el-icon><InfoFilled /></el-icon>
            <span>关于</span>
          </span>
        </template>

        <AboutPage />
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


</style>
