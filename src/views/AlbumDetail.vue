<script setup>
import { ref, onMounted, onUnmounted, computed, reactive, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import {
  VideoPlay,
  VideoPause,
  Star,
  StarFilled,
  OfficeBuilding,
  Share,
  ArrowLeft,
  Loading,
  Document,
} from "@element-plus/icons-vue";
import {
  getDiscInfo,
  getDiscComments,
  getCoverUrl,
  getCachedCoverUrl,
  saveCache,
  loadCache,
  getMp3Duration,
  getTrackDownloadUrl,
  loadUserConfig,
  fetchDiscPageHtml,
  parseDownloadLinks,
  downloadFile,
  unlockFreeDisc,
  toggleLikeDisc,
} from "../services/api.js";
import { downloadManager } from "../services/downloadManager.js";
import MetadataExporter from "../components/dialogs/MetadataExporter.vue";
import {
  formatDuration,
  isTauri,
  getAuthCredentials,
} from "../utils/format.js";
import { globalOffsets } from "../globalvar.js";

const route = useRoute();
const router = useRouter();

const album = ref(null);
const tracks = ref([]);
const comments = ref([]);
const loading = ref(true);
const error = ref("");
const isLiked = ref(false);

// 320Kbps 播放设置
const use320kbps = ref(false);

// ===== 下载功能 =====
const downloadLinks = ref([]);
const downloadLoading = ref(false);
const downloadDialogVisible = ref(false);
const redeeming = ref(false);

const downloadingIndex = ref(-1); // 正在下载的链接索引
const downloadSavePath = ref("");

// 图片缓存映射
const coverCache = reactive({});

const discId = computed(() => route.params.id);

/**
 * 通过 Rust 后端解析 mp3 文件的 Xing/Info 或 VBRI 头部来获取时长
 * 绕过 CORS 限制，无需下载整个文件
 */
async function fetchTrackDuration(mp3Url) {
  if (!mp3Url) return null;
  console.log(`[AlbumDetail] 获取曲目时长: ${mp3Url.substring(0, 60)}...`);
  // 使用 getMp3Duration 从 mp3 头部解析时长
  const duration = await getMp3Duration(mp3Url);
  if (duration && duration > 0) {
    console.log(`[AlbumDetail] 通过 MP3 头部解析到时长: ${duration}s`);
    return duration;
  }
  // 解析失败时回退到 Audio 对象方式
  console.log(`[AlbumDetail] MP3 头部解析失败，回退到 Audio 方式`);
  const audioDuration = await getDurationViaAudio(mp3Url);
  console.log(`[AlbumDetail] Audio 方式获取到时长: ${audioDuration}s`);
  return audioDuration;
}

/**
 * 使用 HTML5 Audio 对象获取时长
 * 创建一个临时的 Audio 元素来获取元数据
 */
function getDurationViaAudio(mp3Url) {
  return new Promise((resolve) => {
    const audio = new Audio();
    // 设置超时，防止无限等待
    const timeout = setTimeout(() => {
      audio.src = "";
      resolve(null);
    }, 10000);

    audio.addEventListener("loadedmetadata", () => {
      clearTimeout(timeout);
      const duration = audio.duration;
      audio.src = "";
      resolve(Math.round(duration));
    });

    audio.addEventListener("error", () => {
      clearTimeout(timeout);
      audio.src = "";
      resolve(null);
    });

    audio.preload = "metadata";
    audio.src = mp3Url;
    audio.load();
  });
}

async function loadAlbumDetail() {
  loading.value = true;
  error.value = "";
  const cacheKey = `album_detail_${discId.value}`;

  try {
    // 尝试从缓存加载专辑详情
    const cachedData = await loadCache(cacheKey);
    if (cachedData) {
      console.log(`[AlbumDetail] 使用缓存数据: ${discId.value}`);
      album.value = cachedData.disc || cachedData;
      isLiked.value = album.value.ilikeit || false;
      tracks.value = cachedData.tracks || cachedData.songs || [];
      // 缓存中已有时长信息，直接使用
      if (tracks.value.length > 0 && tracks.value[0]._duration) {
        console.log("[AlbumDetail] 缓存中已有曲目时长");
      } else {
        // 缓存中没有时长，异步获取
        fetchAllTrackDurations();
      }
    } else {
      // 从 API 获取
      const data = await getDiscInfo(discId.value);
      if (data) {
        album.value = data.disc || data;
        isLiked.value = album.value.ilikeit || false;
        tracks.value = data.tracks || data.songs || [];
        // 保存到缓存
        saveCache(cacheKey, data);
        console.log("[AlbumDetail] 已保存专辑详情到缓存");
        // 异步获取曲目时长
        fetchAllTrackDurations();
      }
    }

    // 预加载专辑封面
    if (album.value?.cover) {
      const url = await getCachedCoverUrl(album.value.cover);
      coverCache.album = url;
    }

    // 加载评论
    const commentData = await getDiscComments(discId.value, { l: 0, r: globalOffsets });
    comments.value = commentData?.commit || [];
    // 预加载评论头像
    for (const comment of comments.value) {
      if (comment.cover) {
        const key = `comment_${comment.id}`;
        getCachedCoverUrl(comment.cover).then((url) => {
          coverCache[key] = url;
        });
      }
    }
  } catch (err) {
    console.error("加载专辑详情失败:", err);
    error.value = "加载失败，请检查网络连接";
  } finally {
    loading.value = false;
  }
}

/**
 * 遍历所有曲目，异步获取每首曲目的时长
 * 获取到时长后更新 tracks 并保存到缓存
 */
async function fetchAllTrackDurations() {
  const trackList = tracks.value;
  if (!trackList || trackList.length === 0) return;

  let hasUpdate = false;
  const promises = trackList.map(async (track) => {
    // 跳过已有时长的曲目
    if (track._duration) return;
    // 跳过没有 URL 的曲目
    if (!track.url) return;
    const duration = await fetchTrackDuration(track.url);
    if (duration && duration > 0) {
      track._duration = duration;
      hasUpdate = true;
    }
  });

  await Promise.allSettled(promises);

  // 强制触发响应式更新
  if (hasUpdate) {
    tracks.value = [...tracks.value];
    // 更新缓存中的时长信息
    const cacheKey = `album_detail_${discId.value}`;
    const cachedData = await loadCache(cacheKey);
    if (cachedData) {
      const dataToCache = cachedData;
      const trackArray = dataToCache.tracks || dataToCache.songs || [];
      trackList.forEach((track, index) => {
        if (track._duration && trackArray[index]) {
          trackArray[index]._duration = track._duration;
        }
      });
      saveCache(cacheKey, dataToCache);
      console.log("[AlbumDetail] 已更新缓存中的曲目时长");
    }
  }
}

function getAlbumCover() {
  return (
    coverCache.album || (album.value ? getCoverUrl(album.value.cover) : "")
  );
}

function getCommentAvatar(comment) {
  return coverCache[`comment_${comment.id}`] || getCoverUrl(comment.cover);
}

/**
 * 获取曲目的播放 URL，如果启用 320Kbps 则通过 API 获取高音质链接
 * API 返回的数据结构: { track: { trackid, ext, url: "https://.../full_320/...mp3" } }
 */
async function getTrackPlayUrl(track) {
  if (!use320kbps.value || !track?.id || !album.value?.id) {
    return track.url || "";
  }
  try {
    console.log(
      `[AlbumDetail] 获取 320Kbps 链接: discid=${album.value.id}, trackid=${track.id}`,
    );
    const data = await getTrackDownloadUrl(album.value.id, track.id, "320");
    console.log("[AlbumDetail] getTrackDownloadUrl 返回:", data);
    // API 返回 { track: { url: "..." } }
    if (data?.track?.url) {
      console.log(
        `[AlbumDetail] 获取到 320Kbps 链接: ${data.track.url.substring(0, 60)}...`,
      );
      return data.track.url;
    }
    console.warn("[AlbumDetail] 获取 320Kbps 链接失败，使用原始 URL");
    return track.url || "";
  } catch (err) {
    console.warn("[AlbumDetail] 获取 320Kbps 链接出错，使用原始 URL:", err);
    return track.url || "";
  }
}

function playTrack(track) {
  console.log("[AlbumDetail] 播放曲目:", track?.title || track?.name);
  if (!track) return;
  // 异步获取 320Kbps 链接（如果需要）
  getTrackPlayUrl(track).then((playUrl) => {
    const trackToPlay = { ...track, url: playUrl || track.url };
    // 通过全局事件将曲目发送到 PlayerBar
    window.dispatchEvent(
      new CustomEvent("add-to-playlist", {
        detail: {
          songs: [trackToPlay],
          discid: album.value?.id,
          album: album.value?.title,
          coverurl: album.value?.cover,
        },
      }),
    );
  });
}

/**
 * 将专辑所有曲目添加到播放列表
 */
async function addAllToPlaylist() {
  if (!tracks.value || tracks.value.length === 0) return;
  console.log(
    `[AlbumDetail] 添加全部曲目到播放列表: ${tracks.value.length} 首`,
  );

  let songsToAdd = tracks.value;
  if (use320kbps.value) {
    // 异步获取所有曲目的 320Kbps 链接
    console.log("[AlbumDetail] 批量获取 320Kbps 链接...");
    const updatedTracks = await Promise.all(
      tracks.value.map(async (track) => {
        const playUrl = await getTrackPlayUrl(track);
        return { ...track, url: playUrl || track.url };
      }),
    );
    songsToAdd = updatedTracks;
  }

  window.dispatchEvent(
    new CustomEvent("add-to-playlist", {
      detail: {
        songs: songsToAdd,
        discid: album.value?.id,
        album: album.value?.title,
        coverurl: album.value?.cover,
      },
    }),
  );
}

async function toggleLike() {
  if (!album.value?.id) return;
  try {
    const result = await toggleLikeDisc(album.value.id);
    if (result && typeof result.ilikeit === "boolean") {
      isLiked.value = result.ilikeit;
      album.value.likes = result.likes ?? album.value.likes;
      // ElMessage.success(isLiked.value ? "+2dB 成功" : "已取消");
      // 更新缓存
      const cacheKey = `album_detail_${discId.value}`;
      const cachedData = await loadCache(cacheKey);
      if (cachedData) {
        const disc = cachedData.disc || cachedData;
        disc.ilikeit = result.ilikeit;
        disc.likes = result.likes ?? disc.likes;
        await saveCache(cacheKey, cachedData);
      }
    }
  } catch (err) {
    console.error("[AlbumDetail] +2dB 操作失败:", err);
    ElMessage.error("失败，请稍后重试");
  }
}

/**
 * 兑换免费唱片
 */
async function redeemFreeDisc() {
  if (!album.value?.id || redeeming.value) return;
  redeeming.value = true;
  try {
    const result = await unlockFreeDisc(album.value.id);
    console.log("[AlbumDetail] 免费唱片兑换结果:", result);
    // 兑换成功后，更新本地状态
    album.value.ihavethis = true;
    // 清除专辑详情缓存，下次加载时获取最新状态
    const cacheKey = `album_detail_${discId.value}`;
    try {
      const { clearCache } = await import("../services/api.js");
      await clearCache(cacheKey);
    } catch (e) {
      // 忽略
    }
    ElMessage.success("兑换成功！");
  } catch (err) {
    console.error("[AlbumDetail] 兑换免费唱片失败:", err);
    ElMessage.error("兑换失败，请稍后重试");
  } finally {
    redeeming.value = false;
  }
}

function goBack() {
  router.back();
}

function goToLabel() {
  if (album.value?.labelid) {
    router.push(`/label/${album.value.labelid}`);
  }
}

function goToUser(userId) {
  if (userId) {
    router.push(`/user/${userId}`);
  }
}

// ===== 下载功能 =====

/**
 * 获取下载链接并显示下载对话框
 */
async function showDownloadDialog() {
  if (!album.value?.id) return;
  downloadLoading.value = true;
  downloadDialogVisible.value = true;
  try {
    if (isTauri) {
      try {
        const path = await loadUserConfig("downloadPath");
        if (path) downloadSavePath.value = path;
      } catch (e) {
        console.warn("[AlbumDetail] 加载下载路径失败:", e);
      }
    }
    const { csrfToken, sessionId } = await getAuthCredentials();
    const html = await fetchDiscPageHtml(album.value.id, csrfToken, sessionId);
    const links = await parseDownloadLinks(html);
    downloadLinks.value = links;
    console.log(`[AlbumDetail] 获取到 ${links.length} 个下载链接`);
  } catch (err) {
    console.error("[AlbumDetail] 获取下载链接失败:", err);
    downloadLinks.value = [];
  } finally {
    downloadLoading.value = false;
  }
}

/**
 * 下载指定格式的文件
 * 通过全局 DownloadManager 添加下载任务（支持后台下载）
 */
function startDownload(index) {
  const link = downloadLinks.value[index];
  if (!link) return;

  // 使用全局下载管理器（组件卸载后下载仍在后台继续）
  downloadManager.addTask(
    link.url,
    link.label,
    album.value?.id || "",
    album.value?.title || "",
  );

  // 关闭下载对话框
  downloadDialogVisible.value = false;
}

/**
 * 检测 Tauri 环境
 */
import { ElMessage } from "element-plus";

/**
 * 通过外部浏览器打开链接
 */
function openInBrowser(url) {
  if (isTauri) {
    window.__TAURI_INTERNALS__.invoke("plugin:opener|open_url", { url });
  } else {
    window.open(url, "_blank");
  }
}

let refreshHandler = null;

let settingsHandler = null;

onMounted(async () => {
  loadAlbumDetail();
  // 加载 320Kbps 设置
  try {
    const val = await loadUserConfig("use320kbps");
    use320kbps.value = val === "true";
    console.log("[AlbumDetail] 320Kbps 设置:", use320kbps.value);
  } catch (e) {
    console.warn("[AlbumDetail] 加载 320Kbps 设置失败:", e);
  }

  refreshHandler = async () => {
    console.log("[AlbumDetail] 收到刷新事件，清除缓存并重新加载");
    const cacheKey = `album_detail_${discId.value}`;
    try {
      const { clearCache } = await import("../services/api.js");
      await clearCache(cacheKey);
    } catch (e) {
      console.log("[AlbumDetail] 清除缓存失败，直接重新加载");
    }
    loadAlbumDetail();
  };
  window.addEventListener("app-refresh", refreshHandler);

  // 监听设置变更事件
  settingsHandler = (event) => {
    if (event.detail?.key === "use320kbps") {
      use320kbps.value = event.detail.value;
      console.log("[AlbumDetail] 320Kbps 设置已更新:", use320kbps.value);
    }
  };
  window.addEventListener("settings-changed", settingsHandler);
});

// 监听路由参数变化，从 /album/1 跳转到 /album/2 时重新加载
watch(
  () => route.params.id,
  (newId, oldId) => {
    if (newId && newId !== oldId) {
      loadAlbumDetail();
    }
  },
);

onUnmounted(() => {
  if (refreshHandler) {
    window.removeEventListener("app-refresh", refreshHandler);
    refreshHandler = null;
  }
  if (settingsHandler) {
    window.removeEventListener("settings-changed", settingsHandler);
    settingsHandler = null;
  }
});
</script>

<template>
  <div class="album-detail">
    <!-- 加载状态 -->
    <div v-if="loading" class="loading-state">
      <el-icon class="loading-icon" :size="32"><Loading /></el-icon>
      <span>加载中...</span>
    </div>

    <!-- 错误状态 -->
    <div v-else-if="error" class="error-state">
      <el-result icon="error" :title="error" sub-title="请检查后重试">
        <template #extra>
          <el-button type="primary" @click="loadAlbumDetail"
            >重新加载</el-button
          >
        </template>
      </el-result>
    </div>

    <!-- 正常内容 -->
    <template v-else-if="album">
      <!-- 返回按钮 -->
      <div class="back-bar">
        <el-button text :icon="ArrowLeft" @click="goBack">返回</el-button>
      </div>

      <!-- 专辑头部信息 -->
      <div class="album-header">
        <div class="album-cover">
          <img :src="getAlbumCover()" :alt="album.title" />
        </div>
        <div class="album-info">
          <h1 class="album-title">{{ album.title }}</h1>
          <p
            class="album-label"
            :class="{ clickable: album.labelid }"
            @click="goToLabel"
          >
            <el-icon><OfficeBuilding /></el-icon>
            {{ album.label || "未知厂牌" }}
          </p>
          <div class="album-meta">
            <span
              v-if="album.price !== undefined && album.onsell !== false"
              class="album-price"
            >
              <template v-if="album.price === 0">免费</template>
              <template v-else>¥{{ album.price }}</template>
            </span>
            <span class="album-likes"> +{{ album.likes * 2 || 0 }}dB </span>
            <span v-if="album.ishires" class="album-hires">Hi-Res</span>
          </div>
          <div class="album-tags" v-if="album.tags && album.tags.length">
            <el-tag
              v-for="tag in album.tags"
              :key="tag"
              size="small"
              class="album-tag"
            >
              {{ tag }}
            </el-tag>
          </div>

          <div class="album-actions">
            <!-- 已拥有：添加到播放列表（下载商品除外） -->
            <el-button
              v-if="album.ihavethis && !album.onlyhavegift"
              type="primary"
              :icon="VideoPlay"
              round
              @click="addAllToPlaylist"
            >
              添加到播放列表
            </el-button>
            <!-- 未拥有：onsell=false 显示仅可兑换（不可点击） -->
            <el-button
              v-else-if="
                album.onsell === false && album.onlyhavegift && !album.ihavethis
              "
              type="info"
              round
              disabled
            >
              仅可兑换
            </el-button>
            <!-- 未拥有：免费商品显示兑换 -->
            <el-button
              v-else-if="album.price === 0 && !album.onlyhavegift"
              type="warning"
              round
              :loading="redeeming"
              @click="redeemFreeDisc"
            >
              兑换
            </el-button>
            <!-- 未拥有：跳转到购买页面 -->
            <el-button
              v-else-if="!album.ihavethis"
              type="danger"
              round
              @click="openInBrowser('https://www.dizzylab.net/d/' + album.id)"
            >
              购买
            </el-button>
            <el-button
              :icon="isLiked ? StarFilled : Star"
              :type="isLiked ? 'primary' : 'default'"
              round
              @click="toggleLike"
            >
              +2dB
            </el-button>
            <!-- 已拥有：显示下载按钮 -->
            <el-button
              v-if="album.ihavethis"
              type="success"
              round
              @click="showDownloadDialog"
            >
              下载
            </el-button>
            <!-- 已拥有：输出元数据 -->
            <MetadataExporter
              v-if="album.ihavethis && !album.onlyhavegift"
              :disc-id="album.id"
              :album-title="album.title"
            />
          </div>
        </div>
      </div>

      <!-- 曲目列表（下载商品不显示曲目列表） -->
      <div v-if="!album.onlyhavegift" class="section">
        <h2 class="section-title">曲目列表</h2>
        <div v-if="tracks.length === 0" class="empty-state">
          <el-empty description="暂无曲目信息" />
        </div>
        <div v-else class="track-list">
          <div
            v-for="(track, index) in tracks"
            :key="track.id || index"
            class="track-item"
            @dblclick="playTrack(track)"
          >
            <span class="track-index">{{
              String(index + 1).padStart(2, "0")
            }}</span>
            <div class="track-info">
              <span class="track-name">{{ track.name || track.title }}</span>
              <span class="track-artist">{{
                track.authers || album.label
              }}</span>
            </div>
            <span class="track-duration">{{
              formatDuration(track._duration)
            }}</span>
          </div>
        </div>
      </div>

      <div class="section">
        <h2 class="section-title">介绍</h2>
        <div
            class="album-descriptions"
            v-if="album.disc_description || album.disc_description_2"
          >
            <p v-if="album.disc_description" class="album-description">
              {{ album.disc_description }}
            </p>
            <p v-if="album.disc_description_2" class="album-description">
              {{ album.disc_description_2 }}
            </p>
          </div>
      </div>

      <!-- 评论区域 -->
      <div class="section">
        <h2 class="section-title">评论 ({{ comments.length }})</h2>
        <div v-if="comments.length === 0" class="empty-state">
          <el-empty description="暂无评论" />
        </div>
          <div v-else class="comment-list">
          <div
            v-for="comment in comments"
            :key="comment.id"
            class="comment-item"
          >
            <el-avatar :size="36" :src="getCommentAvatar(comment)" class="comment-avatar" @click="goToUser(comment.id)" />
            <div class="comment-body">
              <span class="comment-author" @click="goToUser(comment.id)">{{ comment.name }}</span>
              <p class="comment-content">{{ comment.commit }}</p>
            </div>
          </div>
        </div>
      </div>
    </template>

    <!-- 下载对话框 -->
    <el-dialog
      v-model="downloadDialogVisible"
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
  </div>
</template>

<style scoped>
.album-detail {
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

/* ===== 专辑头部 ===== */
.album-header {
  display: flex;
  gap: 32px;
  margin-bottom: 32px;
}

.album-cover {
  flex-shrink: 0;
  width: 280px;
  height: 280px;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
}

.album-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.album-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.album-title {
  font-size: 28px;
  font-weight: 700;
  margin: 0;
  color: var(--el-text-color-primary);
}

.album-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 15px;
  color: var(--el-text-color-secondary);
  margin: 0;
}

.album-label.clickable {
  cursor: pointer;
  transition: color 0.2s ease;
}

.album-label.clickable:hover {
  color: var(--el-color-primary);
}

.album-meta {
  display: flex;
  align-items: center;
  gap: 16px;
  font-size: 14px;
  color: var(--el-text-color-secondary);
}

.album-price {
  font-size: 20px;
  font-weight: 700;
  color: var(--el-color-danger);
}

.album-hires {
  background: linear-gradient(135deg, #f6d365, #fda085);
  color: #fff;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 600;
}

.album-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.album-tag {
  border-radius: 12px;
}

.album-descriptions {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 4px;
}

.album-description {
  margin: 0;
  font-size: 13px;
  color: var(--el-text-color-regular);
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
  user-select: text;
  -webkit-user-select: text; /* For Safari and Webkit2GTK */
}

.album-actions {
  display: flex;
  gap: 12px;
  margin-top: 8px;
}

/* ===== 分区标题 ===== */
.section {
  margin-bottom: 32px;
}

.section-title {
  font-size: 20px;
  font-weight: 700;
  margin: 0 0 16px 0;
  color: var(--el-text-color-primary);
}

/* ===== 曲目列表 ===== */
.track-list {
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 8px;
  overflow: hidden;
}

.track-item {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  cursor: pointer;
  transition: background 0.2s ease;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.track-item:last-child {
  border-bottom: none;
}

.track-item:hover {
  background: var(--el-fill-color-light);
}

.track-index {
  width: 32px;
  font-size: 14px;
  color: var(--el-text-color-secondary);
  font-weight: 600;
}

.track-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.track-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.track-artist {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.track-duration {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

/* ===== 评论列表 ===== */
.comment-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.comment-item {
  display: flex;
  gap: 12px;
  padding: 12px;
  border-radius: 8px;
  background: var(--el-fill-color-blank);
  border: 1px solid var(--el-border-color-lighter);
  cursor: pointer;
  transition: background 0.2s ease;
}

.comment-item:hover {
  background: var(--el-fill-color-light);
}

.comment-avatar {
  flex-shrink: 0;
}

.comment-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.comment-author {
  font-size: 13px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.comment-content {
  font-size: 14px;
  color: var(--el-text-color-regular);
  margin: 0;
  line-height: 1.5;
  user-select: text;
  -webkit-user-select: text; /* For Safari and Webkit2GTK */
}

/* ===== 空状态 ===== */
.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100px;
}

/* ===== 下载对话框 ===== */
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
