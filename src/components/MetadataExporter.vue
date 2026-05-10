<script setup>
import { ref, computed } from "vue";
import { Loading, Document } from "@element-plus/icons-vue";
import { ElMessage } from "element-plus";
import { getDiscInfo } from "../services/api.js";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

const props = defineProps({
  discId: {
    type: String,
    required: true,
  },
  albumTitle: {
    type: String,
    default: "",
  },
});

const dialogVisible = ref(false);
const loading = ref(false);
const tracks = ref([]);
const releaseYear = ref("");
const selectedFormat = ref("eac");

// 输出格式选项
const formatOptions = [
  { value: "eac", label: "Exact Audio Copy 剪贴板" },
  { value: "mp3tag", label: "Mp3Tag CSV" },
  { value: "kid3", label: "Kid3 CSV" },
];

// 各格式的模板文本
const formatTemplates = {
  eac: "tracks.title - tracks.authors",
  mp3tag: "%title%;%artist%;%album%;%track%;%year%",
  kid3: '"?%{track}([^\\r\\n\\t"]*)"?\\t"?%{title}([^\\r\\n\\t"]*)"?\\t"?%{artist}([^\\r\\n\\t"]*)"?\\t"?%{album}([^\\r\\n\\t"]*)"?\\t"?%{year}([^\\r\\n\\t"]*)"',
};

/**
 * 获取曲目信息
 */
async function openDialog() {
  dialogVisible.value = true;
  loading.value = true;
  tracks.value = [];
  releaseYear.value = "";
  selectedFormat.value = "eac";

  try {
    const data = await getDiscInfo(props.discId);
    const trackList = data?.tracks || data?.songs || [];
    tracks.value = trackList;

    // 从 release_date 提取年份
    if (data?.disc?.release_date) {
      const match = String(data.disc.release_date).match(/^\d{4}/);
      if (match) releaseYear.value = match[0];
    } else if (data?.release_date) {
      const match = String(data.release_date).match(/^\d{4}/);
      if (match) releaseYear.value = match[0];
    }

    console.log(
      `[MetadataExporter] 获取到 ${trackList.length} 首曲目, 年份: ${releaseYear.value}`,
    );
  } catch (err) {
    console.error("[MetadataExporter] 获取曲目信息失败:", err);
    ElMessage.error("获取曲目信息失败");
  } finally {
    loading.value = false;
  }
}

function getTrackFields(track) {
  return {
    title: track.title || track.name || "",
    authors: track.authers || "",
    trackId: track.id || "",
    album: props.albumTitle,
    year: releaseYear.value,
  };
}

/**
 * Exact Audio Copy 剪贴板
 * tracks.title - tracks.authors
 */
function generateEacClipboard() {
  return tracks.value
    .map((track) => {
      const { title, authors } = getTrackFields(track);
      return `${title} - ${authors}`;
    })
    .join("\n");
}

/**
 * Mp3Tag CSV 格式
 * tracks.title;tracks.authors;title;tracks.id;YYYY
 * 分隔符为 `;`
 */

function generateMp3TagCsv() {
  return tracks.value
    .map((track) => {
      const { title, authors, album, trackId, year } = getTrackFields(track);
      return `${title};${authors};${album};${trackId};${year}`;
    })
    .join("\n");
}

/**
 * Kid3 CSV 格式
 * "tracks.id"\t"tracks.title"\t"tracks.authors"\t"title"\t"YYYY"
 * 使用 `"` 包裹字段，`\t`(制表符)分隔
 */
function generateKid3Csv() {
  return tracks.value
    .map((track) => {
      const { title, authors, album, trackId, year } = getTrackFields(track);
      return `"${trackId}"\t"${title}"\t"${authors}"\t"${album}"\t"${year}"`;
    })
    .join("\n");
}

/**
 * 根据当前选中的格式生成内容
 */
const currentContent = computed(() => {
  switch (selectedFormat.value) {
    case "eac":
      return generateEacClipboard();
    case "mp3tag":
      return generateMp3TagCsv();
    case "kid3":
      return generateKid3Csv();
    default:
      return "";
  }
});

/**
 * 复制文本到剪贴板
 * @param {string} text - 要复制的内容
 */
async function copyToClipboard(text) {
  if (!text) return;
  try {
    await writeText(text);
    ElMessage.success("已复制到剪贴板");
  } catch (err) {
    console.error("[MetadataExporter] 复制失败:", err);
  }
}
</script>

<template>
  <!-- 触发按钮 -->
  <el-button type="primary" :icon="Document" round @click="openDialog">
    输出元数据
  </el-button>

  <!-- 元数据导出对话框 -->
  <el-dialog
    v-model="dialogVisible"
    :title="'输出元数据 - ' + albumTitle"
    width="620px"
    :close-on-click-modal="false"
  >
    <div v-if="loading" class="metadata-loading">
      <el-icon class="is-loading" :size="24"><Loading /></el-icon>
      <span>正在获取曲目信息...</span>
    </div>
    <template v-else-if="tracks.length === 0">
      <el-empty description="暂无曲目数据" />
    </template>
    <template v-else>
      <!-- 格式选择器 -->
      <div class="format-selector">
        <span class="format-label">输出格式：</span>
        <el-select v-model="selectedFormat" style="width: 280px">
          <el-option
            v-for="opt in formatOptions"
            :key="opt.value"
            :label="opt.label"
            :value="opt.value"
          />
        </el-select>
        <el-button type="primary" @click="copyToClipboard(currentContent)"
          >复制元数据</el-button
        >
        <el-button
          type="primary"
          @click="copyToClipboard(formatTemplates[selectedFormat])"
          >复制格式串</el-button
        >
      </div>

      <!-- 格式模板说明 -->
      <div class="metadata-preview">
        <div class="metadata-preview-header">格式</div>
        <pre class="metadata-preview-content">{{
          formatTemplates[selectedFormat]
        }}</pre>
      </div>

      <!-- 预览区域 -->
      <div class="metadata-preview">
        <div class="metadata-preview-header">预览</div>
        <pre class="metadata-preview-content">{{ currentContent }}</pre>
      </div>
    </template>
  </el-dialog>
</template>

<style scoped>
.metadata-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 40px;
  color: var(--el-text-color-secondary);
}

.format-selector {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}

.format-label {
  font-size: 14px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  white-space: nowrap;
}

.metadata-preview {
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 8px;
  overflow: hidden;
}

.metadata-preview-header {
  font-size: 12px;
  font-weight: 600;
  color: var(--el-text-color-secondary);
  padding: 8px 12px;
  background: var(--el-fill-color-light);
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.metadata-preview-content {
  margin: 0;
  padding: 12px;
  font-size: 12px;
  font-family: monospace;
  line-height: 1.6;
  color: var(--el-text-color-regular);
  background: var(--el-bg-color);
  max-height: 300px;
  overflow-y: auto;
  white-space: pre-wrap;
  word-break: break-all;
}
</style>
