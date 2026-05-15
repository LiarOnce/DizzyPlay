<script setup>
import { ref, onMounted } from "vue";
import { Loading, FolderOpened } from "@element-plus/icons-vue";
import { ElMessage } from "element-plus";
import { useSetting } from "../../utils/settings.js";
import { isTauri } from "../../utils/format.js";

const {
  value: use320kbps,
  load: load320kbps,
  save: save320kbps,
} = useSetting("use320kbps", {
  defaultValue: false,
  localStorageKey: "settings_use320kbps",
  fromStorage: (v) => v === "true",
  toStorage: (v) => (v ? "true" : "false"),
});

const configLoaded = ref(false);

function onUse320kbpsChange(val) {
  use320kbps.value = val;
  save320kbps();
  window.dispatchEvent(
    new CustomEvent("settings-changed", {
      detail: { key: "use320kbps", value: val },
    }),
  );
}

onMounted(async () => {
  await load320kbps();
  configLoaded.value = true;
});

async function openDataDir() {
  if (!isTauri) {
    ElMessage.info("浏览器模式下不支持此操作");
    return;
  }
  try {
    await window.__TAURI_INTERNALS__.invoke("open_user_data_dir");
  } catch (err) {
    console.error("打开用户数据目录失败:", err);
    ElMessage.error("打开用户数据目录失败");
  }
}
</script>

<template>
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

      <div class="setting-item" v-if="configLoaded">
        <div class="setting-info">
          <span class="setting-title">用户数据目录</span>
          <span class="setting-desc">缓存、配置文件等用户数据的存储位置</span>
        </div>
        <div class="setting-control">
          <el-button size="small" :icon="FolderOpened" @click="openDataDir">
            打开目录
          </el-button>
        </div>
      </div>

      <div v-if="!configLoaded" class="setting-item">
        <el-icon class="is-loading"><Loading /></el-icon>
        <span>加载配置中...</span>
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
</style>
