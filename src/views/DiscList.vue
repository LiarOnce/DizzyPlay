<script setup>
import { ref, onMounted, onUnmounted, computed, watch, reactive } from "vue";
import { useRoute, useRouter } from "vue-router";

defineOptions({ name: "DiscList" });
import { VideoPlay, Headset, Loading } from "@element-plus/icons-vue";
import {
  getDiscs,
  getMyDiscs,
  getMyLikes,
  getMyFollowing,
  getLabels,
  getTags,
  getCoverUrl,
  getCachedCoverUrl,
  saveCache,
  loadCache,
  clearCache,
} from "../services/api.js";

import { globalOffsets } from "../globalvar.js";

const route = useRoute();
const router = useRouter();

const discs = ref([]);
const loading = ref(true);
const error = ref("");
const pageTitle = ref("");

// 图片缓存映射
const coverCache = reactive({});

// 用于去重，避免同一张图片被重复发起缓存请求
const pendingCovers = new Set();

// ===== 分页状态 =====
const pageSize = 20;
const currentPage = ref(1);

const paginatedDiscs = computed(() => {
  const start = (currentPage.value - 1) * pageSize;
  return discs.value.slice(start, start + pageSize);
});

const totalPages = computed(() => {
  return Math.ceil(discs.value.length / pageSize) || 1;
});

function handlePageChange(page) {
  currentPage.value = page;
  window.scrollTo({ top: 0, behavior: "smooth" });
}

// 监听分页变化，自动缓存新页面的封面
watch(
  paginatedDiscs,
  (newItems) => {
    if (newItems && newItems.length > 0) {
      cacheVisibleCovers(newItems);
    }
  },
  { immediate: false },
);

// 根据路由路径确定要加载的数据
const pageConfig = computed(() => {
  const path = route.path;
  switch (path) {
    case "/playlists":
      return {
        title: "全部专辑",
        fetcher: () => getDiscs({ l: 0, r: globalOffsets, sort: "ad" }),
      };
    case "/ep":
      return {
        title: "单曲 EP",
        fetcher: () => getDiscs({ l: 0, r: globalOffsets, type: "ep" }),
      };
    case "/dig":
      return {
        title: "下载商品",
        fetcher: () => getDiscs({ l: 0, r: globalOffsets, type: "dig" }),
      };
    case "/label":
      return {
        title: "社团",
        fetcher: () => getLabels({ l: 0, r: globalOffsets }),
      };
    case "/purchased":
      return {
        title: "已购音乐",
        fetcher: () => getMyDiscs({ l: 0, r: globalOffsets }),
      };
    case "/favorites":
      return {
        title: "我的收藏",
        fetcher: () => getMyLikes({ l: 0, r: globalOffsets }),
      };
    case "/following":
      return {
        title: "我的关注",
        fetcher: () => getMyFollowing({ l: 0, r: globalOffsets }),
      };
    default:
      return {
        title: "浏览",
        fetcher: () => getDiscs({ l: 0, r: globalOffsets }),
      };
  }
});

/**
 * 仅缓存当前分页可见的封面
 */
async function cacheVisibleCovers(items) {
  if (!items || items.length === 0) return;
  const tasks = [];
  for (const item of items) {
    const cover = item.cover || item.labelcover;
    if (cover) {
      const key = `item_${item.id || item.labelid || item.uid}`;
      // 如果已有缓存或正在请求中，跳过
      if (coverCache[key] || pendingCovers.has(key)) continue;
      pendingCovers.add(key);
      tasks.push(
        getCachedCoverUrl(cover)
          .then((url) => {
            coverCache[key] = url;
            pendingCovers.delete(key);
          })
          .catch(() => {
            pendingCovers.delete(key);
          }),
      );
    }
  }
  if (tasks.length > 0) {
    await Promise.allSettled(tasks);
    console.log(`[DiscList] 当前分页封面缓存完成: ${tasks.length} 张`);
  }
}

function getItemCover(item) {
  const key = `item_${item.id || item.labelid || item.uid}`;
  return coverCache[key] || getCoverUrl(item.cover || item.labelcover);
}

async function loadData() {
  loading.value = true;
  error.value = "";
  pageTitle.value = pageConfig.value.title;
  const cacheKey = `discs_${route.path}`;

  // 尝试从缓存加载
  const cachedData = await loadCache(cacheKey);
  if (cachedData) {
    discs.value = cachedData;
    loading.value = false;
    console.log(`[DiscList] 使用缓存数据: ${route.path}`, discs.value.length);
    return;
  }

  try {
    const data = await pageConfig.value.fetcher();
    if (data?.discs) {
      discs.value = data.discs;
    } else if (data?.labels) {
      discs.value = data.labels;
    } else if (data?.following) {
      discs.value = data.following;
    } else {
      discs.value = [];
    }
    // 保存到缓存
    saveCache(cacheKey, discs.value);
  } catch (err) {
    console.error("加载数据失败:", err);
    error.value = "加载失败，请检查网络连接";
  } finally {
    loading.value = false;
  }
}

function viewItem(item) {
  if (route.path === "/label") {
    if (item.labelid) {
      router.push(`/label/${item.labelid}`);
    }
  } else if (item.id) {
    router.push(`/album/${item.id}`);
  }
}

// 需要缓存的列表页路径
const listPaths = [
  "/playlists",
  "/ep",
  "/dig",
  "/label",
  "/purchased",
  "/favorites",
  "/following",
];

// 监听全局刷新事件：清除缓存并重新加载
let refreshHandler = null;

onMounted(() => {
  currentPage.value = 1;
  loadData();
  // 注册全局刷新事件监听
  refreshHandler = async () => {
    console.log(`[DiscList] 收到刷新事件，清除缓存并重新加载: ${route.path}`);
    const cacheKey = `discs_${route.path}`;
    await clearCache(cacheKey);
    currentPage.value = 1;
    loadData();
  };
  window.addEventListener("app-refresh", refreshHandler);
});

onUnmounted(() => {
  if (refreshHandler) {
    window.removeEventListener("app-refresh", refreshHandler);
    refreshHandler = null;
  }
});

// 监听路由变化，进入列表页时重新加载数据
// 从详情页返回时，使用缓存数据（不清除缓存），保持分页
watch(
  () => route.path,
  (newPath, oldPath) => {
    if (!newPath) return;
    // 进入列表页时重新加载（包括从非列表页跳转过来）
    if (listPaths.includes(newPath)) {
      currentPage.value = 1;
      loadData();
    }
  },
);
</script>

<template>
  <div class="disc-list-page">
    <!-- 加载状态 -->
    <div v-if="loading" class="loading-state">
      <el-icon class="loading-icon" :size="32"><Loading /></el-icon>
      <span>加载中...</span>
    </div>

    <!-- 错误状态 -->
    <div v-else-if="error" class="error-state">
      <el-result icon="error" :title="error" sub-title="请检查后重试">
        <template #extra>
          <el-button type="primary" @click="loadData">重新加载</el-button>
        </template>
      </el-result>
    </div>

    <!-- 正常内容 -->
    <template v-else>
      <div class="page-header">
        <h1 class="page-title">{{ pageTitle }}</h1>
      </div>

      <div v-if="discs.length === 0" class="empty-state">
        <el-empty description="暂无数据" />
      </div>

      <div v-else class="disc-grid">
        <el-card
          v-for="item in paginatedDiscs"
          :key="item.id || item.labelid || item.uid"
          :body-style="{ padding: '0' }"
          shadow="hover"
          class="disc-card"
        >
          <div class="disc-cover" @click="viewItem(item)">
            <img :src="getItemCover(item)" :alt="item.title || item.label" />
            <div class="disc-overlay"></div>
            <div class="disc-plays" v-if="item.likes !== undefined">
              <el-icon><Headset /></el-icon>
              <span>+{{ item.likes * 2 }}dB</span>
            </div>
          </div>
          <div class="disc-info">
            <span class="disc-name" @click="viewItem(item)">{{
              item.title || item.label || item.name
            }}</span>
            <span class="disc-label" v-if="item.label">{{ item.label }}</span>
            <!-- onsell=false：仅可兑换 -->
            <span
              v-if="item.onsell === false"
              class="disc-price is-redeem-only"
            >
              仅可兑换
            </span>
            <!-- 显示价格 -->
            <span
              v-if="
                item.price !== undefined &&
                item.onsell !== false &&
                route.path !== '/purchased' &&
                route.path !== '/favorites'
              "
              class="disc-price"
              :class="{ 'is-free': item.price === 0 }"
            >
              <template v-if="item.price === 0">免费</template>
              <template v-else>¥{{ item.price }}</template>
            </span>
          </div>
        </el-card>
      </div>

      <!-- 分页器 -->
      <div class="pagination-wrapper" v-if="totalPages > 1">
        <el-pagination
          v-model:current-page="currentPage"
          :page-size="pageSize"
          :total="discs.length"
          layout="prev, pager, next"
          background
          hide-on-single-page
          @current-change="handlePageChange"
        />
      </div>
    </template>
  </div>
</template>

<style scoped>
.disc-list-page {
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

/* ===== 页面标题 ===== */
.page-header {
  margin-bottom: 24px;
}

.page-title {
  font-size: 24px;
  font-weight: 700;
  margin: 0;
  color: var(--el-text-color-primary);
}

/* ===== 专辑网格 ===== */
.disc-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 20px;
}

.disc-card {
  cursor: pointer;
  border-radius: 10px;
  overflow: hidden;
  transition:
    transform 0.2s ease,
    box-shadow 0.2s ease;
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color-lighter);
}

.disc-card:hover {
  transform: translateY(-4px);
}

.disc-cover {
  position: relative;
  width: 100%;
  aspect-ratio: 1;
  overflow: hidden;
  background: var(--el-fill-color-light);
}

.disc-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.3s ease;
}

.disc-card:hover .disc-cover img {
  transform: scale(1.08);
}

.disc-overlay {
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

.disc-card:hover .disc-overlay {
  opacity: 1;
}

.disc-plays {
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

.disc-card:hover .disc-plays {
  opacity: 1;
}

.disc-info {
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.add-to-playlist-btn {
  margin-top: 6px;
  opacity: 0;
  transition: opacity 0.2s ease;
  align-self: flex-start;
}

.disc-card:hover .add-to-playlist-btn {
  opacity: 1;
}

.disc-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.disc-label {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.disc-price {
  font-size: 13px;
  font-weight: 600;
  color: var(--el-color-danger);
  margin-top: 2px;
}

.disc-price.is-free {
  color: var(--el-color-success);
}

.disc-price.is-redeem-only {
  color: var(--el-color-info);
  font-size: 12px;
}

/* ===== 空状态 ===== */
.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 200px;
}

/* ===== 分页器 ===== */
.pagination-wrapper {
  display: flex;
  justify-content: center;
  margin-top: 32px;
  padding: 16px 0;
}
</style>
