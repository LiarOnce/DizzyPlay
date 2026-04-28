<script setup>
import { ref } from "vue";
import { useRoute } from "vue-router";
import TopNavbar from "./components/TopNavbar.vue";
import SidebarNav from "./components/SidebarNav.vue";
import PlayerBar from "./components/PlayerBar.vue";

const route = useRoute();
const isDark = ref(false);

function handleToggleTheme(val) {
  isDark.value = val;
}
</script>

<template>
  <div class="app-container" :class="{ dark: isDark }">
    <el-container class="layout-container">
      <!-- 顶部导航栏 -->
      <TopNavbar />

      <el-container class="middle-container">
        <!-- 左侧侧边栏 -->
        <SidebarNav />

        <!-- 主页 -->
        <el-main class="main-content">
          <router-view v-slot="{ Component }">
            <keep-alive :include="['HomePage', 'DiscList']">
              <component :is="Component" />
            </keep-alive>
          </router-view>
        </el-main>
      </el-container>

      <!-- 底部播放控制栏 -->
      <PlayerBar />
    </el-container>
  </div>
</template>

<style>
@import "./styles/variables.css";
@import "./styles/custom.css";

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html,
body {
  height: 100%;
  overflow: hidden;
  user-select: none;
  -webkit-user-select: none;
}

body {
  font-family:
    -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue",
    Arial, "Noto Sans", sans-serif, "Apple Color Emoji", "Segoe UI Emoji",
    "Segoe UI Symbol", "Noto Color Emoji";
  font-size: 14px;
  line-height: 1.5;
  color: var(--text-primary);
  background: var(--bg-primary);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  transition:
    background var(--transition-normal),
    color var(--transition-normal);
}

#app {
  height: 100%;
}

/* 全局滚动条样式 */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 3px;
}

::-webkit-scrollbar-thumb:hover {
  background: var(--text-tertiary);
}

/* 选中文本颜色 */
::selection {
  background: var(--color-primary-light);
  color: var(--color-primary);
}
</style>

<style scoped>
.app-container {
  height: 100vh;
  background: var(--bg-primary);
  transition: background var(--transition-normal);
}

.layout-container {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.middle-container {
  flex: 1;
  overflow: hidden;
  display: flex;
}

.main-content {
  flex: 1;
  padding: 0;
  overflow-y: auto;
  background: var(--bg-secondary);
  transition: background var(--transition-normal);
}
</style>
