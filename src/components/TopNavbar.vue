<script setup>
import { ref, computed, onMounted, watch } from "vue";
import { useRouter } from "vue-router";
import {
  Search,
  User,
  Setting,
  SwitchButton,
  Moon,
  Sunny,
  Minus,
  FullScreen,
  Close,
  Refresh,
} from "@element-plus/icons-vue";
import { useUser, initToken, fetchUserInfo, logout } from "../stores/user.js";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { getCoverUrl, getCachedCoverUrl } from "../services/api.js";
import LoginDialog from "./dialogs/LoginDialog.vue";

const router = useRouter();
const searchQuery = ref("");
const isDark = ref(false);
const isMaximized = ref(false);

const { userInfo, token } = useUser();

const emit = defineEmits(["toggle-theme"]);

const userAvatar = ref("");

// 登录弹窗引用
const loginDialogRef = ref(null);

// 异步加载用户头像（带缓存）
async function loadAvatar() {
  if (userInfo.avatar) {
    const cachedUrl = await getCachedCoverUrl(userInfo.avatar);
    userAvatar.value = cachedUrl;
  }
}

// 监听 userInfo.avatar 变化
watch(
  () => userInfo.avatar,
  (newAvatar) => {
    if (newAvatar) {
      loadAvatar();
    }
  },
);

function handleSearch() {
  if (searchQuery.value.trim()) {
    router.push(`/search?q=${encodeURIComponent(searchQuery.value.trim())}`);
  }
}

function openLoginDialog() {
  if (loginDialogRef.value) {
    loginDialogRef.value.open();
  }
}

async function handleLoginSuccess() {
  console.log("[TopNavbar] 登录成功，刷新用户信息");
  await loadAvatar();
}

async function handleLogout() {
  await logout();
  userAvatar.value = "";
}

function toggleTheme() {
  isDark.value = !isDark.value;
  document.documentElement.classList.toggle("dark", isDark.value);
  emit("toggle-theme", isDark.value);
}

// 刷新按钮：清除缓存并重新加载当前页面
const isRefreshing = ref(false);

function handleRefresh() {
  if (isRefreshing.value) return;
  isRefreshing.value = true;
  // 触发全局刷新事件，各页面监听此事件来清除缓存并重新加载
  window.dispatchEvent(new CustomEvent("app-refresh"));
  console.log("[TopNavbar] 触发全局刷新事件");
  // 1 秒后重置刷新状态
  setTimeout(() => {
    isRefreshing.value = false;
  }, 1000);
}

// ===== 窗口控制函数 =====
async function minimizeWindow() {
  try {
    await getCurrentWindow().minimize();
  } catch (e) {
    console.warn("[Window] minimize failed:", e);
  }
}

async function toggleMaximize() {
  try {
    const maximized = await getCurrentWindow().isMaximized();
    if (maximized) {
      await getCurrentWindow().unmaximize();
      isMaximized.value = false;
    } else {
      await getCurrentWindow().maximize();
      isMaximized.value = true;
    }
  } catch (e) {
    console.warn("[Window] toggleMaximize failed:", e);
  }
}

async function closeWindow() {
  try {
    await getCurrentWindow().close();
  } catch (e) {
    console.warn("[Window] close failed:", e);
  }
}

onMounted(async () => {
  console.log("[TopNavbar] onMounted - 开始初始化");
  try {
    // 初始化 token（从 user/config.json 或 localStorage）
    await initToken();
    console.log(
      "[TopNavbar] token 初始化完成:",
      token.value ? "有 token" : "无 token",
    );
    // 如果有 token，尝试获取用户信息
    if (token.value) {
      console.log("[TopNavbar] 开始获取用户信息...");
      await fetchUserInfo();
      console.log("[TopNavbar] 用户信息获取完成");
      // 加载头像
      await loadAvatar();
    }
  } catch (err) {
    console.error("[TopNavbar] 初始化失败:", err);
  }
  console.log("[TopNavbar] onMounted 完成");
});
</script>

<template>
  <el-header class="top-navbar">
    <!-- 拖拽区域（左侧） -->
    <div class="drag-region" data-tauri-drag-region>
      <div class="navbar-left">
        <div class="logo" @click="router.push('/')">
          <div class="logo-icon" />
          <span class="logo-text">DizzyPlay</span>
        </div>
      </div>

      <div class="navbar-center">
        <div class="search-wrapper">
          <el-input
            v-model="searchQuery"
            placeholder="搜索专辑、标签..."
            :prefix-icon="Search"
            clearable
            class="search-input"
            @keyup.enter="handleSearch"
          />
        </div>
      </div>

      <div class="navbar-right">
        <!-- 刷新按钮 -->
        <el-button
          :icon="Refresh"
          :loading="isRefreshing"
          circle
          size="small"
          class="refresh-btn"
          title="刷新数据"
          @click="handleRefresh"
        />

        <el-button
          :icon="isDark ? Sunny : Moon"
          :type="isDark ? 'warning' : 'default'"
          circle
          size="small"
          class="theme-toggle-btn"
          @click="toggleTheme"
        />

        <el-dropdown
          v-if="userInfo.isLoggedIn"
          trigger="click"
          placement="bottom-end"
        >
          <el-avatar :size="36" :src="userAvatar" class="user-avatar" />
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item :icon="User">
                {{ userInfo.name || "用户" }}
              </el-dropdown-item>
              <el-dropdown-item :icon="Setting">账号设置</el-dropdown-item>
              <el-dropdown-item
                divided
                :icon="SwitchButton"
                @click="handleLogout"
              >
                退出登录
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>

        <el-button
          v-else
          type="primary"
          size="small"
          round
          @click="openLoginDialog"
        >
          登录
        </el-button>

        <!-- 窗口控制按钮 -->
        <div class="window-controls">
          <el-button
            circle
            class="win-btn win-btn-minimize"
            @click="minimizeWindow"
            title="minimize"
          >
            <el-icon><Minus /></el-icon>
          </el-button>
          <el-button
            circle
            class="win-btn win-btn-maximize"
            @click="toggleMaximize"
            title="maximize"
          >
            <el-icon><FullScreen /></el-icon>
          </el-button>
          <el-button
            circle
            class="win-btn win-btn-close"
            @click="closeWindow"
            title="close"
          >
            <el-icon><Close /></el-icon>
          </el-button>
        </div>
      </div>
    </div>
  </el-header>

  <!-- 登录弹窗 -->
  <LoginDialog ref="loginDialogRef" @login-success="handleLoginSuccess" />
</template>

<style scoped>
.top-navbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: var(--header-height);
  background: var(--bg-header);
  border-bottom: 1px solid var(--border-color);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  position: relative;
  z-index: 100;
  transition:
    background var(--transition-normal),
    border-color var(--transition-normal);
}

.navbar-left {
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.logo {
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
}

.logo-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform var(--transition-fast);
  background-image: url(/logo.png);
  background-size: cover;
}

.logo:hover .logo-icon {
  transform: scale(1.1);
}

.logo-text {
  font-size: 20px;
  font-weight: 700;
  background: var(--color-primary-gradient);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.navbar-center {
  flex: 1;
  display: flex;
  justify-content: center;
  padding: 0 40px;
  max-width: 600px;
  margin: 0 auto;
}

.search-wrapper {
  width: 100%;
  max-width: 480px;
}

.search-input {
  --el-input-border-radius: 20px;
}

.search-input :deep(.el-input__wrapper) {
  background: var(--bg-secondary);
  border: 1px solid transparent;
  box-shadow: none;
  transition: all var(--transition-fast);
}

.search-input :deep(.el-input__wrapper:hover),
.search-input :deep(.el-input__wrapper.is-focus) {
  background: var(--bg-primary);
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px var(--color-primary-light);
}

.search-input :deep(.el-input__inner) {
  font-size: 14px;
  color: var(--text-primary);
}

.search-input :deep(.el-input__inner::placeholder) {
  color: var(--text-tertiary);
}

.navbar-right {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
}

.theme-toggle-btn,
.refresh-btn {
  --el-button-size: 36px;
  border: 1px solid var(--border-color);
  background: var(--bg-secondary);
  color: var(--text-secondary);
  transition: all var(--transition-fast);
}

.theme-toggle-btn:hover,
.refresh-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
  background: var(--color-primary-light);
}

.user-avatar {
  cursor: pointer;
  border: 2px solid transparent;
  transition: border-color var(--transition-fast);
}

.user-avatar:hover {
  border-color: var(--color-primary);
}
/* ===== 窗口拖拽区域 ===== */
.drag-region {
  display: flex;
  align-items: center;
  flex: 1;
  min-width: 0;
  height: 100%;
}

/* ===== 窗口控制按钮 ===== */
.window-controls {
  display: flex;
  align-items: center;
  margin-left: 8px;
  height: 36px;
}

.win-btn {
  width: 36px;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
  font-size: 20px;
  outline: none;
  padding: 0;
}

.win-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.win-btn-close:hover {
  background: #e81123;
  color: #ffffff;
}
</style>
