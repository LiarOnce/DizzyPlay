<script setup>
import { ref, onMounted } from "vue";
import { useSetting } from "../../utils/settings.js";

const {
  value: csrfToken,
  load: loadCsrfToken,
  save: saveCsrfToken,
} = useSetting("csrfToken", { defaultValue: "" });

const {
  value: sessionId,
  load: loadSessionId,
  save: saveSessionId,
} = useSetting("sessionid", { defaultValue: "" });

const configLoaded = ref(false);

function onCsrfTokenChange() {
  saveCsrfToken();
}

function onSessionIdChange() {
  saveSessionId();
}

onMounted(async () => {
  await Promise.all([loadCsrfToken(), loadSessionId()]);
  configLoaded.value = true;
});
</script>

<template>
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
