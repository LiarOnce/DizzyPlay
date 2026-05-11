<script setup>
import { ref, onMounted, computed } from "vue";
import { Loading, Delete } from "@element-plus/icons-vue";
import { isTauri, formatSizeMB as formatSize } from "../utils/format.js";

const datasSize = ref(0);
const imagesSize = ref(0);
const musicSize = ref(0);
const sizesLoading = ref(false);
const clearingType = ref("");

async function getDirSize(dirType) {
  if (!isTauri) return 0;
  try {
    const { invoke } = window.__TAURI_INTERNALS__;
    const bytes = await invoke("get_cache_dir_size", { dirType });
    return bytes;
  } catch (e) {
    console.warn(`[CacheManager] 获取 ${dirType} 大小失败:`, e);
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
    console.warn("[CacheManager] 加载缓存大小失败:", e);
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
    console.log("[CacheManager] 清除结果:", result);
    await loadSizes();
  } catch (e) {
    console.warn(`[CacheManager] 清除 ${dirType} 失败:`, e);
  } finally {
    clearingType.value = "";
  }
}

async function clearAll() {
  if (!isTauri) return;
  clearingType.value = "all";
  try {
    const { invoke } = window.__TAURI_INTERNALS__;
    const result = await invoke("clear_cache_dir", { dirType: "all" });
    console.log("[CacheManager] 一键清除结果:", result);
    await loadSizes();
  } catch (e) {
    console.warn("[CacheManager] 一键清除失败:", e);
  } finally {
    clearingType.value = "";
  }
}

const totalSize = computed(() => {
  return datasSize.value + imagesSize.value + musicSize.value;
});

onMounted(() => {
  loadSizes();
});
</script>

<template>
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
            <span class="cache-size total">{{ formatSize(totalSize) }}</span>
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
</template>

<style scoped>
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
</style>
