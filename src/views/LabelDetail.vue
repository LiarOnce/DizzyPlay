<script setup>
import { ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import {
  ArrowLeft,
  VideoPlay,
  Headset,
  StarFilled,
  Loading,
  UserFilled,
} from "@element-plus/icons-vue";
import {
  getLabelInfo,
  getCoverUrl,
  getCachedCoverUrl,
  saveCache,
  loadCache,
  clearCache,
} from "../services/api.js";
import { globalOffsets } from "../globalvar.js";
import { usePagination } from "../composables/usePagination.js";
import { useCoverCache } from "../composables/useCoverCache.js";
import { useAppRefresh } from "../composables/useAppRefresh.js";

const route = useRoute();
const router = useRouter();

const loading = ref(true);
const error = ref("");
const labelInfo = ref(null);
const discs = ref([]);
const labelCoverUrl = ref("");

const {
  paginatedItems: paginatedDiscs,
  currentPage,
  totalPages,
  pageSize,
  handlePageChange,
  resetPage,
} = usePagination(discs);

const { cacheVisibleCovers, getCover: getDiscCover } = useCoverCache("disc");
const { cacheVisibleCovers: cacheMemberCovers, getCover: _getMemberAvatar } =
  useCoverCache("member");

function getMemberAvatar(member) {
  return _getMemberAvatar(member, { coverField: "cover", idField: "id" });
}

watch(paginatedDiscs, (newItems) => {
  if (newItems?.length > 0) cacheVisibleCovers(newItems);
});

// 跳转到用户信息
function viewMember(member) {
  if (member.id) router.push(`/user/${member.id}`);
}

// 跳转到专辑信息
function viewDisc(item) {
  if (item.id) router.push(`/album/${item.id}`);
}

// 返回上一页
function goBack() {
  router.back();
}

async function loadLabelDetail() {
  const labelid = route.params.id;
  if (!labelid) {
    error.value = "缺少社团 ID";
    loading.value = false;
    return;
  }

  loading.value = true;
  error.value = "";
  const cacheKey = `label_detail_${labelid}`;

  const cachedData = await loadCache(cacheKey);
  if (cachedData) {
    labelInfo.value = cachedData;
    discs.value = cachedData.disc || [];
    // 缓存社团封面
    if (labelInfo.value?.cover) {
      getCachedCoverUrl(labelInfo.value.cover).then((url) => {
        labelCoverUrl.value = url;
      });
    }
    loading.value = false;
    console.log(`[LabelDetail] 使用缓存数据: ${labelid}`);
    // 缓存封面
    cacheVisibleCovers(discs.value);
    cacheMemberCovers(cachedData.members || []);
    return;
  }

  try {
    const data = await getLabelInfo(labelid, { l: 0, r: globalOffsets });
    labelInfo.value = data;
    discs.value = data.disc || [];
    // 缓存社团封面
    if (labelInfo.value?.cover) {
      getCachedCoverUrl(labelInfo.value.cover).then((url) => {
        labelCoverUrl.value = url;
      });
    }
    // 保存到缓存
    saveCache(cacheKey, data);
    // 缓存封面
    cacheVisibleCovers(discs.value);
    cacheMemberCovers(data.members || []);
  } catch (err) {
    console.error("加载社团详情失败:", err);
    error.value = "加载失败，请检查网络连接";
  } finally {
    loading.value = false;
  }
}

useAppRefresh(async () => {
  const cacheKey = `label_detail_${route.params.id}`;
  await clearCache(cacheKey).catch(() => {});
  resetPage();
  loadLabelDetail();
});

watch(
  () => route.params.id,
  (newId, oldId) => {
    if (newId && newId !== oldId) {
      resetPage();
      loadLabelDetail();
    }
  },
);

loadLabelDetail();
</script>

<template>
  <div class="label-detail">
    <!-- 加载状态 -->
    <div v-if="loading" class="loading-state">
      <el-icon class="loading-icon" :size="32"><Loading /></el-icon>
      <span>加载中...</span>
    </div>

    <!-- 错误状态 -->
    <div v-else-if="error" class="error-state">
      <el-result icon="error" :title="error" sub-title="请检查后重试">
        <template #extra>
          <el-button type="primary" @click="loadLabelDetail"
            >重新加载</el-button
          >
        </template>
      </el-result>
    </div>

    <!-- 正常内容 -->
    <template v-else-if="labelInfo">
      <!-- 返回按钮 -->
      <div class="back-bar">
        <el-button text :icon="ArrowLeft" @click="goBack">返回</el-button>
      </div>

      <!-- 社团头部信息 -->
      <div class="label-header">
        <div class="label-cover">
          <img
            :src="labelCoverUrl || getCoverUrl(labelInfo.cover)"
            :alt="labelInfo.title"
          />
        </div>
        <div class="label-info">
          <h1 class="label-title">{{ labelInfo.title }}</h1>
          <div class="label-meta">
            <span class="label-disc-count">
              <el-icon><Headset /></el-icon>
              {{ discs.length }} 张作品
            </span>
            <span
              class="label-following"
              v-if="labelInfo.following !== undefined"
            >
              {{ labelInfo.following ? "已关注" : "未关注" }}
            </span>
          </div>
          <p class="label-description" v-if="labelInfo.description">
            {{ labelInfo.description }}
          </p>
          <p class="label-description-2" v-if="labelInfo.description_2">
            {{ labelInfo.description_2 }}
          </p>
        </div>
      </div>

      <!-- 社团成员 -->
      <div
        class="section"
        v-if="labelInfo.members && labelInfo.members.length > 0"
      >
        <h2 class="section-title">社团成员 ({{ labelInfo.members.length }})</h2>
        <div class="members-list">
          <div
            v-for="member in labelInfo.members"
            :key="member.id"
            class="member-item"
            @click="viewMember(member)"
          >
            <el-avatar :size="48" :src="getMemberAvatar(member)">
              <el-icon><UserFilled /></el-icon>
            </el-avatar>
          </div>
        </div>
      </div>

      <!-- 作品列表 -->
      <div class="section">
        <h2 class="section-title">作品列表 ({{ discs.length }})</h2>
        <div v-if="discs.length === 0" class="empty-state">
          <el-empty description="暂无作品" />
        </div>
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
              <div class="disc-plays" v-if="item.likes !== undefined">
                <el-icon><StarFilled /></el-icon>
                <span>+{{ item.likes * 2 }}dB</span>
              </div>
            </div>
            <div class="disc-info">
              <span class="disc-name" @click="viewDisc(item)">{{
                item.title
              }}</span>
              <span class="disc-label" v-if="item.label">{{ item.label }}</span>
              <div class="disc-price" v-if="item.price !== undefined">
                <span v-if="item.price === 0" class="free-tag">免费</span>
                <span v-else class="price-tag">¥{{ item.price }}</span>
              </div>
            </div>
          </el-card>
        </div>
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
.label-detail {
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

/* ===== 社团头部 ===== */
.label-header {
  display: flex;
  gap: 32px;
  margin-bottom: 32px;
  padding: 24px;
  background: var(--el-bg-color);
  border-radius: 12px;
  border: 1px solid var(--el-border-color-lighter);
}

.label-cover {
  flex-shrink: 0;
  width: 200px;
  height: 200px;
  border-radius: 12px;
  overflow: hidden;
  background: var(--el-fill-color-light);
}

.label-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.label-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.label-title {
  font-size: 28px;
  font-weight: 700;
  margin: 0;
  color: var(--el-text-color-primary);
}

.label-meta {
  display: flex;
  align-items: center;
  gap: 16px;
  font-size: 14px;
  color: var(--el-text-color-secondary);
}

.label-meta .el-icon {
  margin-right: 4px;
}

.label-following {
  padding: 2px 10px;
  border-radius: 10px;
  font-size: 12px;
  background: var(--el-color-primary-light-9);
  color: var(--el-color-primary);
}

.label-description,
.label-description-2 {
  font-size: 14px;
  line-height: 1.6;
  color: var(--el-text-color-regular);
  margin: 0;
  white-space: pre-wrap;
}

/* ===== 区块标题 ===== */
.section {
  margin-bottom: 32px;
}

.section-title {
  font-size: 20px;
  font-weight: 600;
  margin: 0 0 16px 0;
  color: var(--el-text-color-primary);
}

/* ===== 社团成员 ===== */
.members-list {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
}

.member-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  transition: transform 0.2s ease;
}

.member-item:hover {
  transform: translateY(-2px);
}

.member-id {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  max-width: 80px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
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
  margin-top: 2px;
}

.free-tag {
  font-size: 12px;
  color: var(--el-color-success);
  font-weight: 500;
}

.price-tag {
  font-size: 12px;
  color: var(--el-color-danger);
  font-weight: 500;
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
