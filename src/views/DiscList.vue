<script setup>
import { ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { VideoPlay, StarFilled, Loading } from "@element-plus/icons-vue";
import {
  getDiscs,
  getMyDiscs,
  getMyLikes,
  getMyFollowing,
  getLabels,
  getCoverUrl,
  saveCache,
  loadCache,
  clearCache,
} from "../services/api.js";
import { globalOffsets } from "../globalvar.js";
import { usePagination } from "../composables/usePagination.js";
import { useCoverCache } from "../composables/useCoverCache.js";
import { useAppRefresh } from "../composables/useAppRefresh.js";

defineOptions({ name: "DiscList" });

const route = useRoute();
const router = useRouter();

const discs = ref([]);
const loading = ref(true);
const error = ref("");
const pageTitle = ref("");

const {
  paginatedItems: paginatedDiscs,
  currentPage,
  totalPages,
  pageSize,
  handlePageChange,
  resetPage,
} = usePagination(discs);
const { cacheVisibleCovers, getCover } = useCoverCache("item");

function getItemCover(item) {
  return getCover(item, {
    coverField: (i) => i.cover || i.labelcover,
    idField: (i) => i.id || i.labelid || i.uid,
  });
}

watch(paginatedDiscs, (newItems) => {
  if (newItems?.length > 0) {
    cacheVisibleCovers(newItems, {
      coverField: (i) => i.cover || i.labelcover,
      idField: (i) => i.id || i.labelid || i.uid,
      logLabel: "DiscList",
    });
  }
});

const pageConfig = {
  "/allalbum": {
    title: "全部专辑",
    fetcher: () => getDiscs({ l: 0, r: globalOffsets, sort: "ad" }),
  },
  "/ep": {
    title: "单曲 EP",
    fetcher: () => getDiscs({ l: 0, r: globalOffsets, type: "ep" }),
  },
  "/dig": {
    title: "下载商品",
    fetcher: () => getDiscs({ l: 0, r: globalOffsets, type: "dig" }),
  },
  "/label": {
    title: "社团",
    fetcher: () => getLabels({ l: 0, r: globalOffsets }),
  },
  "/purchased": {
    title: "已购内容",
    fetcher: () => getMyDiscs({ l: 0, r: globalOffsets }),
  },
  "/favorites": {
    title: "+2dB",
    fetcher: () => getMyLikes({ l: 0, r: globalOffsets }),
  },
  "/following": {
    title: "我的关注",
    fetcher: () => getMyFollowing({ l: 0, r: globalOffsets }),
  },
};

const listPaths = Object.keys(pageConfig);

async function loadData() {
  loading.value = true;
  error.value = "";
  const cfg = pageConfig[route.path] || pageConfig["/allalbum"];
  pageTitle.value = cfg.title;
  const cacheKey = `discs_${route.path}`;

  const cachedData = await loadCache(cacheKey);
  if (cachedData) {
    discs.value = cachedData;
    loading.value = false;
    return;
  }

  try {
    const data = await cfg.fetcher();
    discs.value = data?.discs || data?.labels || data?.following || [];
    saveCache(cacheKey, discs.value);
  } catch (err) {
    console.error("加载数据失败:", err);
    error.value = "加载失败，请检查网络连接";
  } finally {
    loading.value = false;
  }
}

function viewItem(item) {
  if (route.path === "/following") {
    // 关注列表全是社团，id 实际为 labelid
    if (item.id) router.push(`/label/${item.id}`);
  } else if (route.path === "/label") {
    // 社团列表
    if (item.id) router.push(`/label/${item.id}`);
    else if (item.labelid) router.push(`/label/${item.labelid}`);
  } else {
    // 专辑类列表（/allalbum, /ep, /dig, /purchased, /favorites）
    if (item.id) router.push(`/album/${item.id}`);
    else if (item.labelid) router.push(`/label/${item.labelid}`);
    else if (item.uid) router.push(`/user/${item.uid}`);
  }
}

useAppRefresh(async () => {
  const cacheKey = `discs_${route.path}`;
  await clearCache(cacheKey);
  resetPage();
  loadData();
});

watch(
  () => route.path,
  (newPath) => {
    if (newPath && listPaths.includes(newPath)) {
      resetPage();
      loadData();
    }
  },
);

loadData();
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
              <el-icon><StarFilled /></el-icon>
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
