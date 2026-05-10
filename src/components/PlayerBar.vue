<script setup>
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import defaultAlbumCover from "../assets/images/default-album.png";
import {
  VideoPause,
  CaretRight,
  Back,
  Right,
  Rank,
  Refresh,
  Mute,
  MuteNotification,
  List,
  Delete,
  Close,
} from "@element-plus/icons-vue";
import {
  loadPlaylist,
  savePlaylist,
  getCoverUrl,
  getCachedCoverUrl,
  getCachedMusicUrl,
} from "../services/api.js";
import { formatDuration } from "../utils/format.js";

// ===== 播放状态 =====
const isPlaying = ref(false);
const currentTime = ref(0);
const duration = ref(0);
const volume = ref(60);
const isMuted = ref(false);
const playMode = ref("loop"); // loop, one, shuffle
const showPlaylist = ref(false);
const currentIndex = ref(-1); // 当前播放的曲目在 playlist 中的索引

// ===== 当前歌曲 =====
const currentSong = ref({
  name: "",
  artist: "",
  album: "",
  cover: "",
  duration: "",
});

// ===== 播放列表 =====
const playlist = ref([]);

// ===== HTML5 Audio 实例 =====
let audioElement = null;

// ===== 计算属性 =====
const progressPercent = computed(() => {
  if (duration.value === 0) return 0;
  return (currentTime.value / duration.value) * 100;
});

const formattedCurrentTime = computed(() => {
  const min = Math.floor(currentTime.value / 60);
  const sec = Math.floor(currentTime.value % 60);
  return `${min}:${sec.toString().padStart(2, "0")}`;
});

const formattedDuration = computed(() => {
  if (!duration.value || duration.value <= 0) return "--:--";
  const min = Math.floor(duration.value / 60);
  const sec = Math.floor(duration.value % 60);
  return `${min}:${sec.toString().padStart(2, "0")}`;
});

const volumeIcon = computed(() => {
  if (isMuted.value || volume.value === 0) return Mute;
  return MuteNotification;
});

const playModeIcon = computed(() => {
  switch (playMode.value) {
    case "loop":
      return Refresh;
    case "one":
      return Refresh;
    case "shuffle":
      return Rank;
    default:
      return Refresh;
  }
});

const playModeTitle = computed(() => {
  switch (playMode.value) {
    case "loop":
      return "循环播放";
    case "one":
      return "单曲循环";
    case "shuffle":
      return "随机播放";
    default:
      return "循环播放";
  }
});

// 当前是否有歌曲在播放
const hasCurrentSong = computed(() => {
  return currentIndex.value >= 0 && currentIndex.value < playlist.value.length;
});

// ===== 音频控制 =====

/**
 * 初始化 Audio 元素并绑定事件
 */
function initAudio() {
  if (audioElement) {
    audioElement.pause();
    audioElement.src = "";
    audioElement = null;
  }

  audioElement = new Audio();

  audioElement.addEventListener("timeupdate", () => {
    currentTime.value = audioElement.currentTime;
  });

  audioElement.addEventListener("loadedmetadata", () => {
    duration.value = audioElement.duration || 0;
  });

  audioElement.addEventListener("ended", () => {
    isPlaying.value = false;
    // 自动播放下一首
    onTrackEnded();
  });

  audioElement.addEventListener("error", (e) => {
    console.warn("[Player] 音频播放错误:", e);
    isPlaying.value = false;
  });

  // 恢复音量设置
  audioElement.volume = isMuted.value ? 0 : volume.value / 100;
}

/**
 * 播放指定索引的曲目
 */
async function playTrack(index) {
  if (index < 0 || index >= playlist.value.length) return;

  currentIndex.value = index;
  const song = playlist.value[index];

  // 先设置基本信息
  const coverUrl = song.coverurl || song.cover || "";
  currentSong.value = {
    name: song.title || song.name || "未知曲目",
    artist: song.authers || song.artist || "未知艺术家",
    album: song.album || "",
    cover: getCoverUrl(coverUrl),
    duration: song.duration || "",
  };

  // 异步获取缓存的封面（Tauri 环境中会通过 proxy_image 获取 data: URI）
  if (coverUrl) {
    getCachedCoverUrl(coverUrl).then((url) => {
      currentSong.value = { ...currentSong.value, cover: url };
    });
  }

  if (!audioElement) {
    initAudio();
  }

  // 获取音频 URL
  const audioUrl = song.url || song.audioUrl;
  if (!audioUrl) {
    console.warn("[Player] 曲目没有音频 URL，跳过播放");
    // 没有 URL 时不要设置 audioElement.src，避免 NotSupportedError
    isPlaying.value = false;
    return;
  }

  // 使用 getCachedMusicUrl 获取缓存后的音频 URL（Tauri 环境会缓存到本地）
  let playUrl;
  try {
    playUrl = await getCachedMusicUrl(audioUrl);
  } catch (err) {
    console.warn("[Player] 获取缓存音乐 URL 失败，使用原始 URL:", err);
    // 回退：手动替换 streaming 域名
    playUrl = audioUrl.includes("streaming.dizzylab.net")
      ? audioUrl.replace("https://streaming.dizzylab.net", "/streaming")
      : audioUrl;
  }

  console.log(`[Player] 播放: ${currentSong.value.name}`);

  audioElement.src = playUrl;
  audioElement.currentTime = 0;
  audioElement
    .play()
    .then(() => {
      isPlaying.value = true;
    })
    .catch((err) => {
      console.warn("[Player] 播放失败:", err);
      isPlaying.value = false;
    });
}

/**
 * 曲目播放结束后的处理
 */
function onTrackEnded() {
  if (playMode.value === "one") {
    // 单曲循环：重新播放当前曲目
    playTrack(currentIndex.value);
  } else if (playMode.value === "shuffle") {
    // 随机播放：随机选择下一首
    if (playlist.value.length > 0) {
      let nextIndex;
      do {
        nextIndex = Math.floor(Math.random() * playlist.value.length);
      } while (nextIndex === currentIndex.value && playlist.value.length > 1);
      playTrack(nextIndex);
    }
  } else {
    // 循环播放：顺序播放下一首
    nextSong();
  }
}

// ===== 播放控制方法 =====

function togglePlay() {
  if (!hasCurrentSong.value) {
    // 没有当前歌曲，尝试播放列表第一首
    if (playlist.value.length > 0) {
      playTrack(0);
    }
    return;
  }

  if (!audioElement) {
    initAudio();
  }

  if (isPlaying.value) {
    audioElement.pause();
    isPlaying.value = false;
  } else {
    audioElement
      .play()
      .then(() => {
        isPlaying.value = true;
      })
      .catch((err) => {
        console.warn("[Player] 恢复播放失败:", err);
      });
  }
}

function prevSong() {
  if (playlist.value.length === 0) return;

  let prevIndex;
  if (playMode.value === "shuffle") {
    // 随机模式：随机选择
    do {
      prevIndex = Math.floor(Math.random() * playlist.value.length);
    } while (prevIndex === currentIndex.value && playlist.value.length > 1);
  } else {
    // 顺序模式：上一首
    prevIndex = currentIndex.value - 1;
    if (prevIndex < 0) {
      prevIndex = playlist.value.length - 1; // 循环到末尾
    }
  }
  playTrack(prevIndex);
}

function nextSong() {
  if (playlist.value.length === 0) return;

  let nextIndex;
  if (playMode.value === "shuffle") {
    // 随机模式：随机选择
    do {
      nextIndex = Math.floor(Math.random() * playlist.value.length);
    } while (nextIndex === currentIndex.value && playlist.value.length > 1);
  } else {
    // 顺序模式：下一首
    nextIndex = currentIndex.value + 1;
    if (nextIndex >= playlist.value.length) {
      nextIndex = 0; // 循环到开头
    }
  }
  playTrack(nextIndex);
}

function toggleMute() {
  isMuted.value = !isMuted.value;
  if (audioElement) {
    audioElement.muted = isMuted.value;
  }
}

function cyclePlayMode() {
  const modes = ["loop", "one", "shuffle"];
  const currentIndex = modes.indexOf(playMode.value);
  playMode.value = modes[(currentIndex + 1) % modes.length];
}

function togglePlaylist() {
  showPlaylist.value = !showPlaylist.value;
}

function onProgressChange(val) {
  if (!audioElement || !duration.value) return;
  const newTime = (val / 100) * duration.value;
  audioElement.currentTime = newTime;
  currentTime.value = newTime;
}

function onVolumeChange(val) {
  volume.value = val;
  if (audioElement) {
    audioElement.volume = isMuted.value ? 0 : val / 100;
  }
  isMuted.value = false;
}

/**
 * 点击播放列表中的曲目
 */
function playFromList(index) {
  if (index === currentIndex.value && isPlaying.value) {
    // 点击正在播放的曲目，不做操作
    return;
  }
  playTrack(index);
}

/**
 * 从播放列表中删除曲目
 */
function removeFromList(index, event) {
  event.stopPropagation();
  if (index < 0 || index >= playlist.value.length) return;

  const removedSong = playlist.value[index];
  playlist.value.splice(index, 1);

  // 如果删除的是当前播放的曲目
  if (index === currentIndex.value) {
    if (playlist.value.length > 0) {
      // 播放下一首（或上一首）
      const nextIndex = Math.min(index, playlist.value.length - 1);
      playTrack(nextIndex);
    } else {
      // 列表为空，停止播放
      stopPlayback();
    }
  } else if (index < currentIndex.value) {
    // 删除的曲目在当前播放之前，调整索引
    currentIndex.value--;
  }

  // 保存播放列表
  savePlaylist(playlist.value);
}

/**
 * 停止播放
 */
function stopPlayback() {
  if (audioElement) {
    audioElement.pause();
    audioElement.src = "";
  }
  isPlaying.value = false;
  currentIndex.value = -1;
  currentSong.value = {
    name: "",
    artist: "",
    album: "",
    cover: "",
    duration: "",
  };
  currentTime.value = 0;
  duration.value = 0;
}

/**
 * 清空播放列表
 */
function clearPlaylist() {
  stopPlayback();
  playlist.value = [];
  savePlaylist(playlist.value);
}

/**
 * 暴露方法给父组件调用
 */
function addToPlaylist(songs) {
  if (!songs) return;
  const items = Array.isArray(songs) ? songs : [songs];
  let addedCount = 0;

  for (const item of items) {
    // 防止重复：检查是否已存在相同 id 和 discid 的曲目
    const exists = playlist.value.some(
      (s) => s.id === item.id && s.discid === item.discid,
    );
    if (!exists) {
      playlist.value.push({
        id: item.id,
        discid: item.discid,
        title: item.title,
        authers: item.authers,
        album: item.album,
        coverurl: item.coverurl,
        url: item.url,
        duration: item._duration || item.duration || "",
      });
      addedCount++;
    }
  }

  if (addedCount > 0) {
    savePlaylist(playlist.value);
    console.log(`[Player] 已添加 ${addedCount} 首曲目到播放列表`);
  }

  return addedCount;
}

// ===== 生命周期 =====

onMounted(async () => {
  // 加载播放列表
  const savedPlaylist = await loadPlaylist();
  if (savedPlaylist && savedPlaylist.length > 0) {
    playlist.value = savedPlaylist;
    console.log(`[Player] 已加载播放列表: ${playlist.value.length} 首`);
  }

  // 初始化 Audio
  initAudio();

  // 监听全局"添加到播放列表"事件
  window.addEventListener("add-to-playlist", handleAddToPlaylistEvent);
});

onUnmounted(() => {
  if (audioElement) {
    audioElement.pause();
    audioElement.src = "";
    audioElement = null;
  }
  window.removeEventListener("add-to-playlist", handleAddToPlaylistEvent);
});

/**
 * 处理全局"添加到播放列表"事件
 */
function handleAddToPlaylistEvent(event) {
  const { songs, discid, album, coverurl } = event.detail || {};
  if (!songs) return;

  const items = Array.isArray(songs) ? songs : [songs];
  const tracksToAdd = items.map((track) => ({
    ...track,
    // 如果单个曲目没有 discid/album/coverurl，从事件 detail 继承
    discid: track.discid || discid,
    album: track.album || album,
    coverurl: track.coverurl || coverurl,
  }));

  const count = addToPlaylist(tracksToAdd);
  if (count > 0) {
    // 如果当前没有播放，且添加的曲目有 URL，自动开始播放
    if (!hasCurrentSong.value) {
      const firstNewIndex = playlist.value.length - count;
      const firstNew = playlist.value[firstNewIndex];
      if (firstNew && firstNew.url) {
        playTrack(firstNewIndex);
      }
    }
    // 显示播放列表
    showPlaylist.value = true;
  }
}

// 监听播放列表变化，自动保存
watch(
  () => playlist.value.length,
  () => {
    // 自动保存由 addToPlaylist 和 removeFromList 触发
  },
);

// 暴露方法给父组件
defineExpose({
  addToPlaylist,
  playTrack,
});
</script>

<template>
  <el-footer class="player-bar">
    <div class="player-inner">
      <!-- 左侧：歌曲信息 -->
      <div class="player-left">
        <div class="song-cover">
          <img
            :src="currentSong.cover || defaultAlbumCover"
            :alt="currentSong.name"
          />
          <div class="cover-overlay" :class="{ playing: isPlaying }">
            <span class="equalizer" v-if="isPlaying">
              <span></span><span></span><span></span><span></span>
            </span>
          </div>
        </div>
        <div class="song-meta">
          <div class="song-name-wrapper">
            <span class="song-name" :title="currentSong.name">
              {{ currentSong.name || "未在播放" }}
            </span>
          </div>
          <span class="song-artist" :title="currentSong.artist">
            {{ currentSong.artist || "" }}
          </span>
        </div>
      </div>

      <!-- 中间：播放控制 -->
      <div class="player-center">
        <div class="controls-top">
          <el-tooltip :content="playModeTitle" placement="top">
            <el-button
              :icon="playModeIcon"
              circle
              size="small"
              class="control-btn"
              :class="{ active: playMode !== 'loop' }"
              @click="cyclePlayMode"
            />
          </el-tooltip>

          <el-button
            :icon="Back"
            circle
            size="small"
            class="control-btn"
            :disabled="!hasCurrentSong"
            @click="prevSong"
          />

          <el-button
            :icon="isPlaying ? VideoPause : CaretRight"
            circle
            size="large"
            class="play-btn"
            @click="togglePlay"
          />

          <el-button
            :icon="Right"
            circle
            size="small"
            class="control-btn"
            :disabled="!hasCurrentSong"
            @click="nextSong"
          />
        </div>

        <div class="progress-area">
          <span class="time current">{{ formattedCurrentTime }}</span>
          <el-slider
            :model-value="progressPercent"
            :show-tooltip="false"
            :step="0.1"
            size="small"
            class="progress-slider"
            :disabled="!hasCurrentSong"
            @input="onProgressChange"
          />
          <span class="time total">{{ formattedDuration }}</span>
        </div>
      </div>

      <!-- 右侧：音量 & 列表 -->
      <div class="player-right">
        <el-tooltip content="播放列表" placement="top">
          <el-button
            :icon="List"
            circle
            size="small"
            class="control-btn"
            :class="{ active: showPlaylist }"
            @click="togglePlaylist"
          />
        </el-tooltip>

        <div class="volume-control">
          <el-tooltip :content="isMuted ? '取消静音' : '静音'" placement="top">
            <el-button
              :icon="volumeIcon"
              circle
              size="small"
              class="control-btn"
              @click="toggleMute"
            />
          </el-tooltip>
          <el-slider
            :model-value="isMuted ? 0 : volume"
            :show-tooltip="false"
            :max="100"
            size="small"
            class="volume-slider"
            @input="onVolumeChange"
          />
        </div>
      </div>
    </div>

    <!-- 播放列表抽屉 -->
    <Transition name="playlist-slide">
      <div v-if="showPlaylist" class="playlist-drawer">
        <div class="playlist-header">
          <h3>播放列表 ({{ playlist.length }})</h3>
          <div class="playlist-header-actions">
            <el-button
              text
              size="small"
              :disabled="playlist.length === 0"
              @click="clearPlaylist"
            >
              清空
            </el-button>
            <el-button text size="small" @click="showPlaylist = false">
              收起
            </el-button>
          </div>
        </div>
        <div class="playlist-songs">
          <div
            v-for="(song, index) in playlist"
            :key="`${song.discid}_${song.id}`"
            class="playlist-item"
            :class="{ active: index === currentIndex }"
            @click="playFromList(index)"
          >
            <span class="playlist-index">{{
              String(index + 1).padStart(2, "0")
            }}</span>
            <div class="playlist-song-info">
              <span class="playlist-song-name">{{
                song.title || song.name
              }}</span>
              <span class="playlist-song-artist">{{
                song.authers || song.artist
              }}</span>
            </div>
            <span class="playlist-duration">{{
              song._duration
                ? formatDuration(song._duration)
                : song.duration || "--:--"
            }}</span>
            <el-button
              text
              size="small"
              :icon="Delete"
              class="remove-btn"
              @click="removeFromList(index, $event)"
            />
          </div>
          <div v-if="playlist.length === 0" class="playlist-empty">
            播放列表为空
          </div>
        </div>
      </div>
    </Transition>
  </el-footer>
</template>

<script>
export default {
  name: "PlayerBar",
};
</script>

<style scoped>
.player-bar {
  height: var(--player-height);
  background: var(--bg-player);
  border-top: 1px solid var(--border-color);
  padding: 0;
  position: relative;
  z-index: 200;
  transition:
    background var(--transition-normal),
    border-color var(--transition-normal);
}

.player-inner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 100%;
  padding: 0 24px;
  gap: 24px;
}

/* ===== 左侧：歌曲信息 ===== */
.player-left {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 0 0 280px;
  min-width: 0;
}

.song-cover {
  position: relative;
  width: 56px;
  height: 56px;
  border-radius: var(--radius-sm);
  overflow: hidden;
  flex-shrink: 0;
}

.song-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.cover-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity var(--transition-fast);
}

.cover-overlay.playing {
  opacity: 1;
}

/* 均衡器动画 */
.equalizer {
  display: flex;
  align-items: flex-end;
  gap: 2px;
  height: 20px;
}

.equalizer span {
  width: 3px;
  background: white;
  border-radius: 1px;
  animation: equalizer 0.8s ease-in-out infinite alternate;
}

.equalizer span:nth-child(1) {
  height: 8px;
  animation-delay: 0s;
}
.equalizer span:nth-child(2) {
  height: 16px;
  animation-delay: 0.2s;
}
.equalizer span:nth-child(3) {
  height: 12px;
  animation-delay: 0.4s;
}
.equalizer span:nth-child(4) {
  height: 18px;
  animation-delay: 0.6s;
}

@keyframes equalizer {
  0% {
    height: 4px;
  }
  100% {
    height: 20px;
  }
}

.song-meta {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.song-name-wrapper {
  display: flex;
  align-items: center;
  gap: 6px;
}

.song-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 160px;
}

.song-artist {
  font-size: 12px;
  color: var(--text-tertiary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* ===== 中间：播放控制 ===== */
.player-center {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  max-width: 600px;
}

.controls-top {
  display: flex;
  align-items: center;
  gap: 8px;
}

.control-btn {
  --el-button-size: 32px;
  border: 1px solid var(--border-light);
  font-size: 22px;
  background: transparent;
  color: var(--text-secondary);
  transition: all var(--transition-fast);
}

.control-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.control-btn.active {
  color: var(--color-primary);
}

.control-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.play-btn {
  --el-button-size: 40px;
  background: var(--color-primary);
  color: white;
  border: none;
  font-size: 30px;
  transition: all var(--transition-fast);
}

.play-btn:hover {
  background: var(--color-primary-hover);
  transform: scale(1.05);
}

.progress-area {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
}

.time {
  font-size: 11px;
  color: var(--text-tertiary);
  font-variant-numeric: tabular-nums;
  min-width: 36px;
}

.time.current {
  text-align: right;
}

.time.total {
  text-align: left;
}

.progress-slider {
  flex: 1;
}

.progress-slider :deep(.el-slider__runway) {
  height: 4px;
  background: var(--border-color);
}

.progress-slider :deep(.el-slider__bar) {
  height: 4px;
  background: var(--color-primary);
}

.progress-slider :deep(.el-slider__button) {
  width: 12px;
  height: 12px;
  border: 2px solid var(--color-primary);
  background: white;
  transition: transform var(--transition-fast);
}

.progress-slider :deep(.el-slider__button:hover) {
  transform: scale(1.2);
}

/* ===== 右侧：音量 & 列表 ===== */
.player-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 0 0 280px;
  justify-content: flex-end;
}

.volume-control {
  display: flex;
  align-items: center;
  gap: 6px;
}

.volume-slider {
  width: 100px;
}

.volume-slider :deep(.el-slider__runway) {
  height: 4px;
  background: var(--border-color);
}

.volume-slider :deep(.el-slider__bar) {
  height: 4px;
  background: var(--color-primary);
}

.volume-slider :deep(.el-slider__button) {
  width: 12px;
  height: 12px;
  border: 2px solid var(--color-primary);
  background: white;
}

/* ===== 播放列表抽屉 ===== */
.playlist-drawer {
  position: absolute;
  bottom: 100%;
  right: 24px;
  width: 400px;
  max-height: 450px;
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md) var(--radius-md) 0 0;
  box-shadow: var(--shadow-lg);
  overflow: hidden;
  z-index: 300;
}

.playlist-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  border-bottom: 1px solid var(--border-color);
}

.playlist-header h3 {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
}

.playlist-header-actions {
  display: flex;
  gap: 4px;
}

.playlist-songs {
  max-height: 390px;
  overflow-y: auto;
  padding: 4px 0;
}

.playlist-item {
  display: flex;
  align-items: center;
  padding: 8px 20px;
  gap: 10px;
  cursor: pointer;
  transition: background var(--transition-fast);
}

.playlist-item:hover {
  background: var(--bg-hover);
}

.playlist-item.active {
  background: var(--color-primary-light);
}

.playlist-item.active .playlist-song-name {
  color: var(--color-primary);
}

.playlist-index {
  width: 24px;
  font-size: 12px;
  color: var(--text-tertiary);
  font-weight: 500;
  text-align: center;
  flex-shrink: 0;
}

.playlist-song-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.playlist-song-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.playlist-song-artist {
  font-size: 11px;
  color: var(--text-tertiary);
}

.playlist-duration {
  font-size: 12px;
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.remove-btn {
  flex-shrink: 0;
  opacity: 0;
  transition: opacity var(--transition-fast);
  color: var(--text-tertiary);
}

.playlist-item:hover .remove-btn {
  opacity: 1;
}

.remove-btn:hover {
  color: var(--color-danger);
}

.playlist-empty {
  padding: 40px 20px;
  text-align: center;
  color: var(--text-tertiary);
  font-size: 13px;
}

/* 播放列表动画 */
.playlist-slide-enter-active,
.playlist-slide-leave-active {
  transition: all var(--transition-normal);
}

.playlist-slide-enter-from,
.playlist-slide-leave-to {
  transform: translateY(10px);
  opacity: 0;
}

/* 滚动条 */
.playlist-songs::-webkit-scrollbar {
  width: 4px;
}

.playlist-songs::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 2px;
}

/* ===== 响应式 ===== */
@media (max-width: 768px) {
  .player-inner {
    padding: 0 12px;
    gap: 12px;
  }

  .player-left {
    flex: 0 0 auto;
  }

  .player-right {
    flex: 0 0 auto;
  }

  .volume-slider {
    width: 60px;
  }

  .playlist-drawer {
    width: 300px;
    right: 12px;
  }
}
</style>
