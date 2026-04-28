import { reactive, ref } from "vue";
import {
  getMyInfo,
  login as apiLogin,
  saveUserConfig,
  loadUserConfig,
} from "../services/api.js";

// Token 管理
const token = ref("");

// 用户信息
const userInfo = reactive({
  id: null,
  name: "",
  avatar: "",
  email: "",
  isLoggedIn: false,
});

/**
 * 设置 token（同时保存到 user/config.json）
 */
export async function setToken(newToken) {
  token.value = newToken;
  localStorage.setItem("dizzytoken", newToken);
  // 保存到 user/config.json
  await saveUserConfig("token", newToken);
}

/**
 * 获取 token
 */
export function getToken() {
  return token.value;
}

/**
 * 初始化 token（从 user/config.json 或 localStorage 加载）
 */
export async function initToken() {
  // 优先从 user/config.json 加载
  const configToken = await loadUserConfig("token");
  if (configToken) {
    token.value = configToken;
    localStorage.setItem("dizzytoken", configToken);
    console.log("[User] 从 user/config.json 加载 token");
    return;
  }
  // 回退到 localStorage
  const localToken = localStorage.getItem("dizzytoken");
  if (localToken) {
    token.value = localToken;
    console.log("[User] 从 localStorage 加载 token");
    return;
  }
  console.log("[User] 未找到 token，用户未登录");
}

/**
 * 登录
 * @param {string} username - 用户名
 * @param {string} password - 密码
 * @returns {Promise<object>} 登录响应数据
 */
export async function login(username, password) {
  const data = await apiLogin(username, password);
  if (data && data.token) {
    await setToken(data.token);
    // 获取用户信息
    await fetchUserInfo();
    return data;
  }
  throw new Error("登录响应中未包含 token");
}

/**
 * 获取用户信息
 */
export async function fetchUserInfo() {
  try {
    const data = await getMyInfo();
    if (data && data.user) {
      userInfo.id = data.user.uid;
      userInfo.name = data.user.username || "";
      userInfo.avatar = data.user.cover || "";
      userInfo.email = data.user.mail || "";
      userInfo.isLoggedIn = true;
    }
    return data;
  } catch (err) {
    console.error("获取用户信息失败:", err);
    userInfo.isLoggedIn = false;
    return null;
  }
}

/**
 * 登出
 */
export async function logout() {
  token.value = "";
  localStorage.removeItem("dizzytoken");
  // 清除 user/config.json 中的 token
  await saveUserConfig("token", "");
  userInfo.id = null;
  userInfo.name = "";
  userInfo.avatar = "";
  userInfo.email = "";
  userInfo.isLoggedIn = false;
}

export function useUser() {
  return {
    token,
    userInfo,
    setToken,
    getToken,
    initToken,
    fetchUserInfo,
    login,
    logout,
  };
}
