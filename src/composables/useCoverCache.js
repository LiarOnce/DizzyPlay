import { reactive } from "vue";
import { getCachedCoverUrl, getCoverUrl } from "../services/api.js";

// 专辑封面的缓存
function resolveField(item, field) {
  return typeof field === "function" ? field(item) : item[field];
}

export function useCoverCache(prefix) {
  const coverCache = reactive({});
  const pendingCovers = new Set();

  async function cacheVisibleCovers(items, opts = {}) {
    const { coverField = "cover", idField = "id", logLabel = "" } = opts;
    if (!items || items.length === 0) return;
    const tasks = [];
    for (const item of items) {
      const cover = resolveField(item, coverField);
      if (cover) {
        const key = `${prefix}_${resolveField(item, idField) ?? ""}`;
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
      if (logLabel)
        console.log(`[${logLabel}] 封面缓存完成: ${tasks.length} 张`);
    }
  }

  function getCover(item, opts = {}) {
    const { coverField = "cover", idField = "id" } = opts;
    const key = `${prefix}_${resolveField(item, idField) ?? ""}`;
    return coverCache[key] || getCoverUrl(resolveField(item, coverField));
  }

  return { coverCache, cacheVisibleCovers, getCover };
}
