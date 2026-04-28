<script setup>
import { ref, onMounted, onUnmounted, computed, reactive, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import {
  ArrowLeft,
  VideoPlay,
  Headset,
  Loading,
  UserFilled,
} from "@element-plus/icons-vue";
import {
  getOtherUserInfo,
  getCoverUrl,
  getCachedCoverUrl,
  saveCache,
  loadCache,
} from "../services/api.js";
import {
  fetchUserPageHtml,
  parseUserPageDiscs,
} from "../services/htmlParser.js";
import { globalOffsets } from "../globalvar.js";

defineOptions({ name: "UserDetail" });

const route = useRoute();
const router = useRouter();

const loading = ref(true);
const error = ref("");
const userInfo = ref(null);

// Tab 数据
const activeTab = ref("purchased");
const tabDiscs = reactive({
  purchased: [],
  review: [],
  following: [],
  likes: [],
});
const tabLoading = reactive({
  purchased: false,
  review: false,
  following: false,
  likes: false,
});
const tabLoaded = reactive({
  purchased: false,
  review: false,
  following: false,
  likes: false,
});

// 图片缓存映射
const coverCache = reactive({});
const pendingCovers = new Set();

// 用户头像缓存
const userAvatarUrl = ref("");

// ===== 分页状态 =====
const pageSize = 20;
const currentPage = ref(1);

const currentDiscs = computed(() => {
  return tabDiscs[activeTab.value] || [];
});

const paginatedDiscs = computed(() => {
  const start = (currentPage.value - 1) * pageSize;
  return currentDiscs.value.slice(start, start + pageSize);
});

const totalPages = computed(() => {
  return Math.ceil(currentDiscs.value.length / pageSize) || 1;
});

function handlePageChange(page) {
  currentPage.value = page;
  window.scrollTo({ top: 0, behavior: "smooth" });
}

/**
 * 缓存当前分页可见的封面
 */
async function cacheVisibleCovers(items) {
  if (!items || items.length === 0) return;
  const tasks = [];
  for (const item of items) {
    const cover = item.cover;
    if (cover) {
      const key = `disc_${item.id}`;
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
  }
}

function getDiscCover(item) {
  const key = `disc_${item.id}`;
  return coverCache[key] || getCoverUrl(item.cover);
}

/**
 * 查看详情（专辑或社团）
 */
function viewDisc(item) {
  if (!item.id) return;
  if (activeTab.value === "following") {
    // API 接口不支持发送 string，无法获得 labelid
    // 对此将不对用户的关注页面提供跳转
    //router.push(`/label/${item.id}`);
    return;
  } else {
    router.push(`/album/${item.id}`);
  }
}

/**
 * 返回上一页
 */
function goBack() {
  router.back();
}

/**
 * 从已购缓存恢复数据
 */
async function restorePurchasedFromCache(uid) {
  if (tabLoaded.purchased) return;
  const purchasedCache = await loadCache(`user_purchased_${uid}`);
  if (purchasedCache && purchasedCache.length > 0) {
    tabDiscs.purchased = purchasedCache;
    tabLoaded.purchased = true;
  }
}

/**
 * 缓存用户头像
 */
function cacheUserAvatar(cover) {
  if (cover) {
    getCachedCoverUrl(cover).then((url) => {
      userAvatarUrl.value = url;
    });
  }
}

/**
 * 加载用户基本信息
 */
async function loadUserDetail() {
  const uid = route.params.id;
  if (!uid) {
    error.value = "缺少用户 ID";
    loading.value = false;
    return;
  }

  error.value = "";
  const cacheKey = `user_detail_${uid}`;

  // 尝试从缓存加载
  const cachedData = await loadCache(cacheKey);
  if (cachedData) {
    // 如果已有用户数据（组件复用场景），直接恢复
    if (userInfo.value) {
      await restorePurchasedFromCache(uid);
      return;
    }

    loading.value = true;
    userInfo.value = cachedData.user;
    cacheUserAvatar(userInfo.value?.cover);
    await restorePurchasedFromCache(uid);
    loading.value = false;
    console.log(`[UserDetail] 使用缓存数据: ${uid}`);
    return;
  }

  loading.value = true;

  try {
    const data = await getOtherUserInfo(uid, { l: 0, r: globalOffsets });
    console.log(`[UserDetail] 获取到数据:`, data);
    userInfo.value = data.user;
    cacheUserAvatar(userInfo.value?.cover);
    // 从同一响应中提取已购列表并缓存
    if (data.discs && data.discs.length > 0) {
      tabDiscs.purchased = data.discs;
      tabLoaded.purchased = true;
      saveCache(`user_purchased_${uid}`, data.discs);
    }
    // 保存用户信息到缓存
    saveCache(cacheKey, data);
  } catch (err) {
    console.error("加载用户详情失败:", err);
    error.value = "加载失败，请检查网络连接";
  } finally {
    loading.value = false;
  }
}

/**
 * 通过 HTML 解析加载 tab 数据
 */
async function loadHtmlTab(tabType) {
  const uid = route.params.id;
  if (tabLoaded[tabType] || tabLoading[tabType]) return;
  tabLoading[tabType] = true;

  const cacheKey = `user_${tabType}_${uid}`;
  const cached = await loadCache(cacheKey);
  if (cached) {
    tabDiscs[tabType] = cached;
    tabLoaded[tabType] = true;
    tabLoading[tabType] = false;
    return;
  }

  try {
    const html = await fetchUserPageHtml(uid, tabType);
    const discs = parseUserPageDiscs(html);
    tabDiscs[tabType] = discs;
    saveCache(cacheKey, discs);
  } catch (err) {
    console.error(`[UserDetail] 加载 ${tabType} 失败:`, err);
  } finally {
    tabLoaded[tabType] = true;
    tabLoading[tabType] = false;
  }
}

/**
 * Tab 切换处理
 */
function handleTabChange(tab) {
  currentPage.value = 1;
  if (tab === "purchased") {
    // 已购数据已在 loadUserDetail 中通过 API 获取并缓存，只需从缓存恢复
    restorePurchasedFromCache(route.params.id);
  } else {
    loadHtmlTab(tab);
  }
}

// 监听分页变化，自动缓存新页面的封面
watch(
  paginatedDiscs,
  (newItems) => {
    if (newItems && newItems.length > 0) {
      cacheVisibleCovers(newItems);
    }
  },
  { immediate: true },
);

let refreshHandler = null;

onMounted(() => {
  loadUserDetail();
  refreshHandler = async () => {
    console.log("[UserDetail] 收到刷新事件，清除所有缓存并重新加载");
    const uid = route.params.id;
    // 清除所有用户相关的 JSON 缓存
    try {
      const { clearCache } = await import("../services/api.js");
      const cacheKeys = [
        `user_detail_${uid}`,
        `user_purchased_${uid}`,
        `user_review_${uid}`,
        `user_following_${uid}`,
        `user_likes_${uid}`,
      ];
      for (const key of cacheKeys) {
        await clearCache(key);
      }
    } catch (e) {
      console.log("[UserDetail] 清除缓存失败，重新加载");
    }
    // 重置所有 tab 的加载状态
    tabLoaded.purchased = false;
    tabLoaded.review = false;
    tabLoaded.following = false;
    tabLoaded.likes = false;
    tabDiscs.purchased = [];
    tabDiscs.review = [];
    tabDiscs.following = [];
    tabDiscs.likes = [];
    // 重置分页
    currentPage.value = 1;
    // 重新加载用户信息
    loadUserDetail().then(() => {
      if (activeTab.value !== "purchased") {
        loadHtmlTab(activeTab.value);
      }
    });
  };
  window.addEventListener("app-refresh", refreshHandler);
});

// 监听路由参数变化
watch(
  () => route.params.id,
  (newId, oldId) => {
    if (newId && newId !== oldId) {
      // 重置所有 tab 加载状态
      tabLoaded.purchased = false;
      tabLoaded.review = false;
      tabLoaded.following = false;
      tabLoaded.likes = false;
      tabDiscs.purchased = [];
      tabDiscs.review = [];
      tabDiscs.following = [];
      tabDiscs.likes = [];
      currentPage.value = 1;
      activeTab.value = "purchased";
      loadUserDetail();
    }
  },
);

onUnmounted(() => {
  if (refreshHandler) {
    window.removeEventListener("app-refresh", refreshHandler);
    refreshHandler = null;
  }
});
</script>

<template>
  <div class="user-detail">
    <!-- 加载状态 -->
    <div v-if="loading" class="loading-state">
      <el-icon class="loading-icon" :size="32"><Loading /></el-icon>
      <span>加载中...</span>
    </div>

    <!-- 错误状态 -->
    <div v-else-if="error" class="error-state">
      <el-result icon="error" :title="error" sub-title="请检查后重试">
        <template #extra>
          <el-button type="primary" @click="loadUserDetail">重新加载</el-button>
        </template>
      </el-result>
    </div>

    <!-- 正常内容 -->
    <template v-else-if="userInfo">
      <!-- 返回按钮 -->
      <div class="back-bar">
        <el-button text :icon="ArrowLeft" @click="goBack">返回</el-button>
      </div>

      <!-- 用户头部信息 -->
      <div class="user-header">
        <div class="user-avatar">
          <el-avatar
            :size="120"
            :src="userAvatarUrl || getCoverUrl(userInfo.cover)"
          >
            <el-icon :size="48"><UserFilled /></el-icon>
          </el-avatar>
        </div>
        <div class="user-info">
          <h1 class="user-name">{{ userInfo.username }}</h1>
          <div class="user-meta">
            <span class="user-uid">UID: {{ userInfo.uid }}</span>
            <span class="user-disc-count">
              <el-icon><Headset /></el-icon>
              {{ userInfo.allcount || 0 }} 张作品
            </span>
            <span class="user-group" v-if="userInfo.user_group">
              {{ userInfo.user_group }}
            </span>
          </div>
          <p class="user-desp" v-if="userInfo.desp">
            {{ userInfo.desp }}
          </p>
        </div>
      </div>

      <!-- Tab 导航 -->
      <div class="tab-section">
        <el-tabs
          v-model="activeTab"
          type="card"
          class="user-tabs"
          @tab-change="handleTabChange"
        >
          <el-tab-pane label="已购" name="purchased">
            <template #label>
              <span class="tab-label">
                <i class="fa fa-database"></i> 已购
                <span class="tab-count" v-if="tabDiscs.purchased.length"
                  >({{ tabDiscs.purchased.length }})</span
                >
              </span>
            </template>
          </el-tab-pane>
          <el-tab-pane label="repo" name="review">
            <template #label>
              <span class="tab-label">
                <i class="fa fa-pencil-square-o"></i> repo
                <span class="tab-count" v-if="tabDiscs.review.length"
                  >({{ tabDiscs.review.length }})</span
                >
              </span>
            </template>
          </el-tab-pane>
          <el-tab-pane label="关注" name="following">
            <template #label>
              <span class="tab-label">
                <i class="fa fa-user"></i> 关注
                <span class="tab-count" v-if="tabDiscs.following.length"
                  >({{ tabDiscs.following.length }})</span
                >
              </span>
            </template>
          </el-tab-pane>
          <el-tab-pane label="+2dB" name="likes">
            <template #label>
              <span class="tab-label">
                <i class="fa fa-heart"></i> +2dB
                <span class="tab-count" v-if="tabDiscs.likes.length"
                  >({{ tabDiscs.likes.length }})</span
                >
              </span>
            </template>
          </el-tab-pane>
        </el-tabs>

        <!-- Tab 内容 -->
        <div class="tab-content">
          <!-- 加载中 -->
          <div v-if="tabLoading[activeTab]" class="loading-state small">
            <el-icon class="loading-icon" :size="24"><Loading /></el-icon>
            <span>加载中...</span>
          </div>

          <!-- 空状态 -->
          <div v-else-if="currentDiscs.length === 0" class="empty-state">
            <el-empty description="暂无数据" />
          </div>

          <!-- 唱片网格 -->
          <div v-else class="disc-grid">
            <el-card
              v-for="item in paginatedDiscs"
              :key="item.id"
              :body-style="{ padding: '0' }"
              shadow="hover"
              class="disc-card"
            >
              <div class="disc-cover" @click="viewDisc(item)">
                <img :src="getDiscCover(item)" :alt="item.title" />
                <div class="disc-overlay"></div>
              </div>
              <div class="disc-info">
                <span class="disc-name" @click="viewDisc(item)">{{
                  item.title
                }}</span>
                <span
                  class="disc-label"
                  v-if="item.label && activeTab !== 'following'"
                  >{{ item.label }}</span
                >
              </div>
            </el-card>
          </div>

          <!-- 分页器 -->
          <div class="pagination-wrapper" v-if="totalPages > 1">
            <el-pagination
              v-model:current-page="currentPage"
              :page-size="pageSize"
              :total="currentDiscs.length"
              layout="prev, pager, next"
              background
              hide-on-single-page
              @current-change="handlePageChange"
            />
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.user-detail {
  padding: 24px;
  max-width: 1000px;
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

.loading-state.small {
  min-height: 200px;
  gap: 12px;
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

/* ===== 返回按钮 ===== */
.back-bar {
  margin-bottom: 16px;
}

/* ===== 用户头部 ===== */
.user-header {
  display: flex;
  gap: 32px;
  margin-bottom: 32px;
  padding: 24px;
  background: var(--el-bg-color);
  border-radius: 12px;
  border: 1px solid var(--el-border-color-lighter);
}

.user-avatar {
  flex-shrink: 0;
}

.user-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.user-name {
  font-size: 28px;
  font-weight: 700;
  margin: 0;
  color: var(--el-text-color-primary);
}

.user-meta {
  display: flex;
  align-items: center;
  gap: 16px;
  font-size: 14px;
  color: var(--el-text-color-secondary);
}

.user-meta .el-icon {
  margin-right: 4px;
}

.user-group {
  padding: 2px 10px;
  border-radius: 10px;
  font-size: 12px;
  background: var(--el-color-primary-light-9);
  color: var(--el-color-primary);
}

.user-desp {
  font-size: 14px;
  line-height: 1.6;
  color: var(--el-text-color-regular);
  margin: 0;
  white-space: pre-wrap;
}

/* ===== Tab 区域 ===== */
.tab-section {
  margin-bottom: 32px;
}

.user-tabs {
  margin-bottom: 24px;
}

.user-tabs :deep(.el-tabs__item) {
  font-size: 14px;
}

.tab-label {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.tab-label .fa {
  font-size: 14px;
}

.tab-count {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-left: 2px;
}

.tab-content {
  min-height: 200px;
}

/* ===== 作品网格 ===== */
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

.disc-info {
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
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

.add-to-playlist-btn {
  margin-top: 6px;
  opacity: 0;
  transition: opacity 0.2s ease;
  align-self: flex-start;
}

.disc-card:hover .add-to-playlist-btn {
  opacity: 1;
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
