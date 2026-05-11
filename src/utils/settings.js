import { ref } from "vue";
import { loadUserConfig, saveUserConfig } from "../services/api.js";
import { isTauri } from "./format.js";

/**
 * 通用设置函数
 *
 * @param {string} key - Tauri 配置键名
 * @param {Object} [options]
 * @param {*}      [options.defaultValue=""] - 默认值
 * @param {string} [options.localStorageKey] - 浏览器模式下的 localStorage 键名（默认同 key）
 * @param {Function} [options.fromStorage = (val) => val] - 从存储字符串反序列化
 * @param {Function} [options.toStorage = (val) => String(val)] - 序列化为存储字符串
 */
export function useSetting(key, options = {}) {
  const {
    defaultValue = "",
    localStorageKey,
    fromStorage = (val) => val,
    toStorage = (val) => String(val),
  } = options;

  const storageKey = localStorageKey ?? key;
  const value = ref(defaultValue);
  let _loaded = false;

  async function load() {
    if (isTauri) {
      try {
        const val = await loadUserConfig(key);
        value.value = val != null ? fromStorage(val) : defaultValue;
      } catch {
        value.value = defaultValue;
      }
    } else {
      const stored = localStorage.getItem(storageKey);
      value.value = stored != null ? fromStorage(stored) : defaultValue;
    }
    _loaded = true;
    return value.value;
  }

  async function save() {
    const strValue = toStorage(value.value);
    if (isTauri) {
      await saveUserConfig(key, strValue);
    } else {
      localStorage.setItem(storageKey, strValue);
    }
    return strValue;
  }

  return { value, load, save, loaded: () => _loaded };
}
