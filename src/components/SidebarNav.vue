<script setup>
import { ref, computed } from "vue";
import { useRoute } from "vue-router";
import {
  HomeFilled,
  List,
  UserFilled,
  ShoppingCart,
  StarFilled,
  User,
  Collection,
  Setting,
  Tickets,
  ArrowLeftBold,
  Download,
  Headset,
  Box,
  OfficeBuilding,
} from "@element-plus/icons-vue";

const route = useRoute();

const isCollapsed = ref(false);

const menuItems = [
  {
    title: "Dizzylab",
    children: [
      { path: "/", name: "数字专辑", icon: HomeFilled },
      { path: "/ep", name: "单曲 EP", icon: Headset },
      { path: "/dig", name: "下载商品", icon: Box },
      { path: "/label", name: "社团", icon: OfficeBuilding },
    ],
  },
  {
    title: "我的音乐",
    children: [
      { path: "/purchased", name: "已购", icon: ShoppingCart },
      { path: "/favorites", name: "+2dB", icon: StarFilled },
      { path: "/following", name: "关注", icon: User },
      { path: "/downloads", name: "已下载", icon: Download },
    ],
  },
  {
    title: "其他",
    children: [
      { path: "/redeem", name: "兑换", icon: Tickets },
      { path: "/settings", name: "软件设置", icon: Setting },
    ],
  },
];

const activeMenu = computed(() => {
  return route.path || "/";
});

function toggleCollapse() {
  isCollapsed.value = !isCollapsed.value;
}
</script>

<template>
  <el-aside
    :width="
      isCollapsed ? 'var(--sidebar-collapsed-width)' : 'var(--sidebar-width)'
    "
    class="sidebar-nav"
  >
    <div class="sidebar-inner">
      <el-menu
        :default-active="activeMenu"
        :collapse="isCollapsed"
        :collapse-transition="false"
        router
        class="sidebar-menu"
      >
        <template v-for="group in menuItems" :key="group.title">
          <div v-if="!isCollapsed" class="menu-group-label">
            {{ group.title }}
          </div>
          <el-menu-item
            v-for="item in group.children"
            :key="item.path"
            :index="item.path"
          >
            <el-icon><component :is="item.icon" /></el-icon>
            <template #title>
              <span>{{ item.name }}</span>
            </template>
          </el-menu-item>
        </template>
      </el-menu>

      <div class="sidebar-footer">
        <div class="collapse-btn" @click="toggleCollapse">
          <el-icon :class="{ rotated: isCollapsed }">
            <ArrowLeftBold />
          </el-icon>
        </div>
      </div>
    </div>
  </el-aside>
</template>

<style scoped>
.sidebar-nav {
  background: var(--bg-sidebar);
  border-right: 1px solid var(--border-color);
  transition: width var(--transition-normal);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  position: relative;
  z-index: 50;
}

.sidebar-inner {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding-top: 8px;
}

.sidebar-menu {
  flex: 1;
  border-right: none;
  background: transparent;
  overflow-y: auto;
  overflow-x: hidden;
}

.sidebar-menu:not(.el-menu--collapse) {
  width: var(--sidebar-width);
}

.menu-group-label {
  padding: 16px 20px 6px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-tertiary);
}

.el-menu-item {
  height: 44px;
  line-height: 44px;
  margin: 2px 8px;
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  transition: all var(--transition-fast);
}

.el-menu-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.el-menu-item.is-active {
  background: var(--color-primary-light);
  color: var(--color-primary);
  font-weight: 600;
}

.el-menu-item.is-active .el-icon {
  color: var(--color-primary);
}

.el-menu-item .el-icon {
  font-size: 18px;
  margin-right: 8px;
}

/* 折叠模式：覆盖 Element Plus 默认样式使图标居中 */
:deep(.el-menu--collapse) {
  width: 100%;
  --el-menu-base-level-padding: 12px !important;
}

:deep(.el-menu--collapse) .el-menu-item {
  margin: 4px auto !important;
  width: 44px !important;
  height: 44px !important;
  line-height: 44px !important;
  padding: 0 !important;
  justify-content: center !important;
  border-radius: var(--radius-sm);
}

:deep(.el-menu--collapse) .el-menu-item .el-icon {
  margin-right: 0 !important;
  font-size: 20px;
  width: 20px;
  height: 20px;
  display: inline-flex !important;
  align-items: center !important;
  justify-content: center !important;
  vertical-align: middle;
}

.sidebar-footer {
  padding: 12px;
  border-top: 1px solid var(--border-color);
  display: flex;
  justify-content: center;
}

.collapse-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  cursor: pointer;
  color: var(--text-tertiary);
  transition: all var(--transition-fast);
}

.collapse-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.collapse-btn .el-icon {
  transition: transform var(--transition-normal);
}

.collapse-btn .el-icon.rotated {
  transform: rotate(180deg);
}

/* 滚动条样式 */
.sidebar-menu::-webkit-scrollbar {
  width: 4px;
}

.sidebar-menu::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 2px;
}

.sidebar-menu::-webkit-scrollbar-track {
  background: transparent;
}
</style>
