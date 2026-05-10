<script setup>
import { ref, watch } from "vue";
import { useRouter } from "vue-router";
import { VideoPlay, StarFilled, Loading } from "@element-plus/icons-vue";
import {
  getDiscs,
  getSomeCover,
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

defineOptions({ name: "HomePage" });

const router = useRouter();

const digitalAlbums = ref([]);
const loading = ref(true);
const error = ref("");
const banners = ref([]);

const {
  paginatedItems: paginatedAlbums,
  currentPage,
  totalPages,
  pageSize,
  handlePageChange,
  resetPage,
} = usePagination(digitalAlbums);
const { coverCache, cacheVisibleCovers, getCover } = useCoverCache("album");

async function cacheBannerCovers(bannerList) {
  if (!bannerList || bannerList.length === 0) return;
  const tasks = [];
  for (let i = 0; i < bannerList.length; i++) {
    const cover = bannerList[i].cover || bannerList[i];
    if (cover) {
      const key = `banner_${i}`;
      if (coverCache[key]) continue;
      tasks.push(
        getCachedCoverUrl(cover)
          .then((url) => {
            coverCache[key] = url;
          })
          .catch(() => {}),
      );
    }
  }
  if (tasks.length > 0) {
    await Promise.allSettled(tasks);
    console.log(`[HomePage] 轮播图封面缓存完成: ${tasks.length} 张`);
  }
}

watch(paginatedAlbums, (newItems) => {
  if (newItems?.length > 0) {
    cacheVisibleCovers(newItems, { logLabel: "HomePage" });
  }
});

async function loadHomeData() {
  loading.value = true;
  error.value = "";
  resetPage();

  const cachedData = await loadCache("homepage_discs");
  const cachedBanners = await loadCache("homepage_banners");
  if (cachedData) digitalAlbums.value = cachedData;
  if (cachedBanners) banners.value = cachedBanners;
  if (cachedData && cachedBanners) {
    await cacheBannerCovers(cachedBanners);
    loading.value = false;
    return;
  }

  try {
    const [discsDataResult, coverDataResult] = await Promise.allSettled([
      getDiscs({ l: 0, r: globalOffsets, sort: "ad" }),
      getSomeCover({ l: 0, r: 6 }),
    ]);

    if (
      discsDataResult.status === "fulfilled" &&
      discsDataResult.value?.discs
    ) {
      digitalAlbums.value = discsDataResult.value.discs;
      saveCache("homepage_discs", digitalAlbums.value);
    } else if (!cachedData) {
      digitalAlbums.value = [];
    }

    if (
      coverDataResult.status === "fulfilled" &&
      coverDataResult.value?.covers
    ) {
      banners.value = coverDataResult.value.covers;
      saveCache("homepage_banners", banners.value);
    }
  } catch (err) {
    console.error("[HomePage] 加载首页数据失败:", err);
    if (!cachedData) digitalAlbums.value = [];
    if (!cachedBanners) banners.value = [];
    error.value = "";
  } finally {
    await cacheBannerCovers(banners.value);
    loading.value = false;
  }
}

function getAlbumCover(album) {
  return getCover(album);
}

function getBannerCover(cover, index) {
  return coverCache[`banner_${index}`] || getCoverUrl(cover.cover || cover);
}

function viewAlbum(album) {
  router.push(`/album/${album.id}`);
}

useAppRefresh(async () => {
  await clearCache("homepage_discs");
  await clearCache("homepage_banners");
  loadHomeData();
});

loadHomeData();
</script>

<template>
  <div class="home-page">
    <!-- ===== 加载状态 ===== -->
    <div v-if="loading" class="loading-state">
      <el-icon class="loading-icon" :size="32"><Loading /></el-icon>
      <span>加载中...</span>
    </div>

    <!-- ===== 错误状态 ===== -->
    <div v-else-if="error" class="error-state">
      <el-result icon="error" :title="error" sub-title="请检查网络后重试">
        <template #extra>
          <el-button type="primary" @click="loadHomeData">重新加载</el-button>
        </template>
      </el-result>
    </div>

    <!-- ===== 正常内容 ===== -->
    <template v-else>
      <!-- ===== 轮播图区域 ===== -->
      <section v-if="banners.length > 0" class="section carousel-section">
        <el-carousel
          :interval="5000"
          height="280px"
          indicator-position="outside"
          class="main-carousel"
        >
          <el-carousel-item v-for="(cover, index) in banners" :key="index">
            <div class="carousel-card">
              <img
                :src="getBannerCover(cover, index)"
                :alt="'Banner ' + index"
              />
            </div>
          </el-carousel-item>
        </el-carousel>
      </section>

      <!-- ===== 数字专辑区域 ===== -->
      <section class="section">
        <div class="section-header">
          <h2 class="section-title">数字专辑</h2>
        </div>
        <div v-if="digitalAlbums.length === 0" class="empty-state">
          <el-empty description="暂无数据" />
        </div>
        <template v-else>
          <div class="playlist-grid">
            <el-card
              v-for="album in paginatedAlbums"
              :key="album.id"
              :body-style="{ padding: '0' }"
              shadow="hover"
              class="playlist-card"
            >
              <div class="playlist-cover" @click="viewAlbum(album)">
                <img :src="getAlbumCover(album)" :alt="album.title" />
                <div class="playlist-overlay"></div>
                <div class="playlist-plays">
                  <el-icon><StarFilled /></el-icon>
                  <span>+{{ album.likes * 2 || 0 }}dB</span>
                </div>
              </div>
              <div class="playlist-info">
                <span class="playlist-name" @click="viewAlbum(album)">{{
                  album.title
                }}</span>
                <span class="playlist-label">{{ album.label }}</span>
                <!-- onsell=false：仅可兑换 -->
                <span
                  v-if="album.onsell === false"
                  class="playlist-price is-redeem-only"
                >
                  仅可兑换
                </span>
                <!-- 显示价格 -->
                <span
                  v-else-if="album.price !== undefined"
                  class="playlist-price"
                  :class="{ 'is-free': album.price === 0 }"
                >
                  <template v-if="album.price === 0">免费</template>
                  <template v-else>¥{{ album.price }}</template>
                </span>
              </div>
            </el-card>
          </div>

          <!-- 分页器 -->
          <div class="pagination-wrapper" v-if="totalPages > 1">
            <el-pagination
              v-model:current-page="currentPage"
              :page-size="pageSize"
              :total="digitalAlbums.length"
              layout="prev, pager, next"
              background
              hide-on-single-page
              @current-change="handlePageChange"
            />
          </div>
        </template>
      </section>
    </template>
  </div>
</template>

<style scoped>
.home-page {
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

/* ===== 轮播图 ===== */
.carousel-section {
  margin-bottom: 32px;
}

.main-carousel {
  border-radius: 12px;
  overflow: hidden;
}

.carousel-card {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--el-fill-color-light);
}

.carousel-card img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

/* ===== 分区标题 ===== */
.section {
  margin-bottom: 32px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.section-title {
  font-size: 22px;
  font-weight: 700;
  margin: 0;
  color: var(--el-text-color-primary);
}

/* ===== 专辑网格 ===== */
.playlist-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 20px;
}

.playlist-card {
  cursor: pointer;
  border-radius: 10px;
  overflow: hidden;
  transition:
    transform 0.2s ease,
    box-shadow 0.2s ease;
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color-lighter);
}

.playlist-card:hover {
  transform: translateY(-4px);
}

.playlist-cover {
  position: relative;
  width: 100%;
  aspect-ratio: 1;
  overflow: hidden;
  background: var(--el-fill-color-light);
}

.playlist-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.3s ease;
}

.playlist-card:hover .playlist-cover img {
  transform: scale(1.08);
}

.playlist-overlay {
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

.playlist-card:hover .playlist-overlay {
  opacity: 1;
}

.playlist-plays {
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

.playlist-card:hover .playlist-plays {
  opacity: 1;
}

.playlist-info {
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

.playlist-card:hover .add-to-playlist-btn {
  opacity: 1;
}

.playlist-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.playlist-label {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.playlist-price {
  font-size: 13px;
  font-weight: 600;
  color: var(--el-color-danger);
  margin-top: 2px;
}

.playlist-price.is-free {
  color: var(--el-color-success);
}

.playlist-price.is-redeem-only {
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
