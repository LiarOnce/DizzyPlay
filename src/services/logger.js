/**
 * 日志 - 将 console.log/warn/error/info 输出到 user/DizzyPlay.log
 *        在浏览器环境下不会有任何行为
 */

import { isTauri } from "../utils/format.js";

/**
 * 通过 Tauri invoke 调用 Rust 后端的 append_log 命令
 * @param {string} level - 日志级别 (INFO, WARN, ERROR, DEBUG)
 * @param {string} message - 日志消息
 */
async function writeLog(level, message) {
  if (!isTauri) return;
  try {
    await window.__TAURI_INTERNALS__.invoke("append_log", {
      level: level,
      message: String(message),
    });
  } catch (err) {
    // 如果写日志本身失败，使用原生 console.error 输出
    console.error(`[Logger] 写入日志失败: ${err}`);
  }
}

/**
 * 将多个参数拼接为字符串
 */
function joinArgs(...args) {
  return args
    .map((arg) => {
      if (arg === null) return "null";
      if (arg === undefined) return "undefined";
      if (typeof arg === "object") {
        try {
          return JSON.stringify(arg);
        } catch {
          return String(arg);
        }
      }
      return String(arg);
    })
    .join(" ");
}

// 保存原始 console 方法
const consoleLogNative = {
  log: console.log,
  warn: console.warn,
  error: console.error,
  info: console.info,
  debug: console.debug,
};

/**
 * 初始化日志模块
 * 仅在 Tauri 环境中生效
 */
/**
 * 备份上一次运行的日志文件
 */
async function rotateLog() {
  if (!isTauri) return;
  try {
    await window.__TAURI_INTERNALS__.invoke("rotate_log");
  } catch (err) {
    consoleLogNative.error(`[Logger] 备份日志失败: ${err}`);
  }
}

export function initLogger() {
  if (!isTauri) {
    console.log("[Logger] 非 Tauri 环境");
    return;
  }

  // 先备份上一次运行的日志，再初始化新的日志记录
  rotateLog();

  console.log = function (...args) {
    const message = joinArgs(...args);
    consoleLogNative.log.apply(console, args);
    writeLog("INFO", message);
  };

  console.warn = function (...args) {
    const message = joinArgs(...args);
    consoleLogNative.warn.apply(console, args);
    writeLog("WARN", message);
  };

  console.error = function (...args) {
    const message = joinArgs(...args);
    consoleLogNative.error.apply(console, args);
    writeLog("ERROR", message);
  };

  console.info = function (...args) {
    const message = joinArgs(...args);
    consoleLogNative.info.apply(console, args);
    writeLog("INFO", message);
  };

  console.debug = function (...args) {
    const message = joinArgs(...args);
    consoleLogNative.debug.apply(console, args);
    writeLog("DEBUG", message);
  };

  console.log("[Logger] 日志模块已初始化，日志将输出到 user/DizzyPlay.log");
}

/**
 * 手动记录日志
 * @param {string} level - 日志级别
 * @param  {...any} args - 日志内容
 */
export async function log(level, ...args) {
  const message = joinArgs(...args);
  await writeLog(level, message);
}

export default { initLogger, log };
