import { onMounted, onUnmounted } from "vue";

// 全局刷新功能
export function useAppRefresh(handler) {
  let refreshHandler = null;

  onMounted(() => {
    refreshHandler = handler;
    window.addEventListener("app-refresh", refreshHandler);
  });

  onUnmounted(() => {
    if (refreshHandler) {
      window.removeEventListener("app-refresh", refreshHandler);
      refreshHandler = null;
    }
  });

  return refreshHandler;
}
