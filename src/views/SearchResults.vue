<script setup>
import { ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { Headset, Loading } from "@element-plus/icons-vue";
import { search, getCoverUrl } from "../services/api.js";
import { globalOffsets } from "../globalvar.js";
import { useCoverCache } from "../composables/useCoverCache.js";
import { useAppRefresh } from "../composables/useAppRefresh.js";

const route = useRoute();
const router = useRouter();

const query = ref("");
const results = ref([]);
const loading = ref(false);
const error = ref("");

const { cacheVisibleCovers, getCover: getResultCover } = useCoverCache("result");

async function doSearch(keyword) {
  if (!keyword?.trim()) { results.value = []; return; }
  loading.value = true;
  error.value = "";
  try {
    const data = await search(keyword, { l: 0, r: globalOffsets });
    results.value = data?.discs || [];
    await cacheVisibleCovers(results.value, { logLabel: "SearchResults" });
  } catch (err) {
    console.error("搜索失败:", err);
    error.value = "搜索失败，请检查网络连接";
  } finally { loading.value = false; }
}

function viewAlbum(album) {
  router.push(`/album/${album.id}`);
}

watch(() => route.query.q, (newQ) => {
  if (newQ) { query.value = newQ; doSearch(newQ); }
});

useAppRefresh(async () => {
  if (query.value?.trim()) doSearch(query.value);
});

if (route.query.q) {
  query.value = route.query.q;
  doSearch(route.query.q);
}
</script>

<template>
  <div class="search-results">
    <!-- 加载状态 -->
    <div v-if="loading" class="loading-state">
      <el-icon class="loading-icon" :size="32"><Loading /></el-icon>
      <span>搜索中...</span>
    </div>

    <!-- 错误状态 -->
    <div v-else-if="error" class="error-state">
      <el-result icon="error" :title="error" sub-title="请检查后重试">
        <template #extra>
          <el-button type="primary" @click="doSearch(query)">重试</el-button>
        </template>
      </el-result>
    </div>

    <!-- 搜索结果 -->
    <template v-else>
      <div v-if="query && results.length === 0 && !loading" class="empty-state">
        <el-empty :description="`未找到与「${query}」相关的结果`" />
      </div>

      <div v-else-if="results.length > 0" class="results-section">
        <div class="results-header">
          <h2 class="section-title">搜索结果 ({{ results.length }})</h2>
        </div>
        <div class="results-grid">
          <el-card
            v-for="album in results"
            :key="album.id"
            :body-style="{ padding: '0' }"
            shadow="hover"
            class="result-card"
            @click="viewAlbum(album)"
          >
            <div class="result-cover">
              <img :src="getResultCover(album)" :alt="album.title" />
              <div class="result-overlay"></div>
              <div class="result-plays">
                <el-icon><Headset /></el-icon>
                <span>+{{ album.likes * 2 || 0 }}dB</span>
              </div>
            </div>
            <div class="result-info">
              <span class="result-name">{{ album.title }}</span>
              <span class="result-label">{{ album.label }}</span>
            </div>
          </el-card>
        </div>
      </div>

      <div v-else-if="!query" class="empty-state">
        <el-empty description="输入关键词开始搜索" />
      </div>
    </template>
  </div>
</template>

<style scoped>
.search-results {
  padding: 24px;
  max-width: 1400px;
  margin: 0 auto;
}

/* ===== 加载状态 ===== */
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 400px;
  gap: 16px;
  color: var(--el-text-color-secondary);
}

.loading-icon {
  animation: rotating 2s linear infinite;
}

@keyframes rotating {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

/* ===== 错误状态 ===== */
.error-state {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 400px;
}

/* ===== 搜索结果 ===== */
.results-section {
  margin-bottom: 32px;
}

.results-header {
  margin-bottom: 20px;
}

.section-title {
  font-size: 22px;
  font-weight: 700;
  margin: 0;
  color: var(--el-text-color-primary);
}

.results-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 20px;
}

.result-card {
  cursor: pointer;
  border-radius: 10px;
  overflow: hidden;
  transition:
    transform 0.2s ease,
    box-shadow 0.2s ease;
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color-lighter);
}

.result-card:hover {
  transform: translateY(-4px);
}

.result-cover {
  position: relative;
  width: 100%;
  aspect-ratio: 1;
  overflow: hidden;
  background: var(--el-fill-color-light);
}

.result-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.3s ease;
}

.result-card:hover .result-cover img {
  transform: scale(1.08);
}

.result-overlay {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: linear-gradient(
    to bottom,
    transparent 60%,
    rgba(0, 0, 0, 0.5) 100%
  );
  opacity: 0;
  transition: opacity 0.3s ease;
}

.result-card:hover .result-overlay {
  opacity: 1;
}

.result-plays {
  position: absolute;
  bottom: 8px;
  right: 10px;
  color: #fff;
  font-size: 12px;
  display: flex;
  align-items: center;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.3s ease;
}

.result-card:hover .result-plays {
  opacity: 1;
}

.result-info {
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.result-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.result-label {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* ===== 空状态 ===== */
.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 200px;
}
</style>
