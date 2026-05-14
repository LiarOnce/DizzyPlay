<script setup>
import { ref, watch, computed } from "vue";
import { Loading } from "@element-plus/icons-vue";
import {
  loadUserConfig,
  fetchDiscPageHtml,
  parseDownloadLinks,
} from "../../services/api.js";
import { downloadManager } from "../../services/downloadManager.js";
import { isTauri, getAuthCredentials } from "../../utils/format.js";

const props = defineProps({
  modelValue: {
    type: Boolean,
    default: false,
  },
  albumId: {
    type: String,
    required: true,
  },
  albumTitle: {
    type: String,
    default: "",
  },
});

const emit = defineEmits(["update:modelValue"]);

const downloadLoading = ref(false);
const downloadLinks = ref([]);
const downloadingIndex = ref(-1);
const downloadSavePath = ref("");

const dialogVisible = computed({
  get: () => props.modelValue,
  set: (val) => emit("update:modelValue", val),
});

watch(
  () => props.modelValue,
  async (visible) => {
    if (visible) {
      await loadDownloadLinks();
    } else {
      downloadLinks.value = [];
      downloadingIndex.value = -1;
    }
  },
);

async function loadDownloadLinks() {
  if (!props.albumId) return;
  downloadLoading.value = true;
  downloadLinks.value = [];
  try {
    if (isTauri) {
      try {
        const path = await loadUserConfig("downloadPath");
        if (path) downloadSavePath.value = path;
      } catch (e) {
        console.warn("[DownloadDialog] 加载下载路径失败:", e);
      }
    }
    const { csrfToken, sessionId } = await getAuthCredentials();
    const html = await fetchDiscPageHtml(props.albumId, csrfToken, sessionId);
    const links = await parseDownloadLinks(html);
    downloadLinks.value = links;
    console.log(`[DownloadDialog] 获取到 ${links.length} 个下载链接`);
  } catch (err) {
    console.error("[DownloadDialog] 获取下载链接失败:", err);
    downloadLinks.value = [];
  } finally {
    downloadLoading.value = false;
  }
}

function startDownload(index) {
  const link = downloadLinks.value[index];
  if (!link) return;

  downloadingIndex.value = index;

  downloadManager.addTask(
    link.url,
    link.label,
    props.albumId || "",
    props.albumTitle || "",
  );

  dialogVisible.value = false;
}
</script>

<template>
  <el-dialog
    v-model="dialogVisible"
    title="选择下载格式"
    width="420px"
    :close-on-click-modal="false"
  >
    <div v-if="downloadLoading" class="download-loading">
      <el-icon class="is-loading" :size="24"><Loading /></el-icon>
      <span>正在获取下载链接...</span>
    </div>
    <div v-else-if="downloadLinks.length === 0" class="download-empty">
      <el-empty description="暂无可用下载链接，请检查 CSRF Token 和 SessionID" />
    </div>
    <div v-else class="download-list">
      <div
        v-for="(link, index) in downloadLinks"
        :key="index"
        class="download-item"
      >
        <div class="download-info">
          <span class="download-label">{{ link.label }}</span>
        </div>
        <el-button
          type="primary"
          size="small"
          :loading="downloadingIndex === index"
          :disabled="downloadingIndex >= 0"
          @click="startDownload(index)"
        >
          下载
        </el-button>
      </div>
    </div>
  </el-dialog>
</template>

<style scoped>
.download-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 40px;
  color: var(--el-text-color-secondary);
}

.download-empty {
  padding: 20px;
}

.download-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.download-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: var(--el-fill-color-light);
  border-radius: 8px;
  border: 1px solid var(--el-border-color-lighter);
}

.download-info {
  flex: 1;
}

.download-label {
  font-size: 14px;
  color: var(--el-text-color-primary);
}
</style>
