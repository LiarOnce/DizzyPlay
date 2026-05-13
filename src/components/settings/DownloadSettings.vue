<script setup>
import { ref, onMounted } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useSetting } from "../../utils/settings.js";
import { isTauri } from "../../utils/format.js";

const {
  value: downloadPath,
  load: loadDownloadPath,
  save: saveDownloadPath,
} = useSetting("downloadPath", { defaultValue: "" });

const {
  value: autoExtract,
  load: loadAutoExtract,
  save: saveAutoExtract,
} = useSetting("autoExtract", {
  defaultValue: false,
  fromStorage: (v) => v === "true",
  toStorage: (v) => (v ? "true" : "false"),
});

async function selectDownloadPath() {
  if (!isTauri) return;
  const selected = await open({
    directory: true,
    multiple: false,
    defaultPath: downloadPath.value || undefined,
  });
  if (selected) {
    downloadPath.value = selected;
    saveDownloadPath();
  }
}

function onDownloadPathChange() {
  saveDownloadPath();
}

function onAutoExtractChange(val) {
  autoExtract.value = val;
  saveAutoExtract();
}

onMounted(async () => {
  await Promise.all([loadDownloadPath(), loadAutoExtract()]);
});
</script>

<template>
  <div class="tab-content">
    <h3>下载设置</h3>
    <div class="settings-list">
      <div class="setting-item">
        <div class="setting-info">
          <span class="setting-title">下载保存路径</span>
          <span class="setting-desc">下载的专辑文件将保存到此目录</span>
        </div>
        <div class="setting-control path-control">
          <el-input
            v-model="downloadPath"
            :readonly="isTauri"
            placeholder="默认：程序所在目录/downloads"
            size="small"
            style="width: 280px"
            @change="onDownloadPathChange"
          />
          <el-button
            v-if="isTauri"
            size="small"
            @click="selectDownloadPath"
          >
            浏览
          </el-button>
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
</template>

<style scoped>
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

.path-control {
  display: flex;
  align-items: center;
  gap: 8px;
}
</style>
