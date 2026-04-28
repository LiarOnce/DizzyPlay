<script setup>
import { ref } from "vue";
import { useRouter } from "vue-router";
import {
  redeemCode,
  confirmRedeem,
  getCoverUrl,
  getCachedCoverUrl,
} from "../services/api.js";
import { ElMessageBox, ElMessage } from "element-plus";
import { Tickets } from "@element-plus/icons-vue";

const router = useRouter();

const codeInput = ref("");
const verifying = ref(false);

/**
 * 验证兑换码
 * 注：以下部分在福建THO03-白鹭风茗期间
 *    花了我 365 元购买实体专辑并索要了 6 个兑换码并配合先前未使用的用于测试
 *    感谢以下社团在我购买专辑时附赠的 CD key 用于协助测试（排名不分先后）：
 *    - 善谐国度（Sociable State）
 *    - 幻想诗篇（Xanadu Canto）
 *    - 疯帽子茶会（Mad hatter tea）
 *    - 静止世界（Static World）
 */
async function handleVerify() {
  const code = codeInput.value.trim();
  if (!code) return;

  verifying.value = true;
  try {
    const data = await redeemCode(code);
    console.log("[Redeem] 验证结果:", data);

    const res = String(data?.res || "");
    const disc = data?.disc || null;

    if (res === "4" && disc) {
      // 未兑换，弹窗显示唱片信息并询问是否确认兑换
      await showConfirmDialog(disc, code);
    } else if (res === "1" && !disc) {
      await ElMessageBox.alert("该兑换码已被使用过", "提示", {
        confirmButtonText: "知道了",
        type: "warning",
        center: true,
      });
    } else {
      await ElMessageBox.alert("兑换码无效，请检查后重试", "验证失败", {
        confirmButtonText: "知道了",
        type: "error",
        center: true,
      });
    }
  } catch (err) {
    if (err === "cancel" || err?.toString().includes("cancel")) {
      return;
    }
    console.error("[Redeem] 验证出错:", err);
    await ElMessageBox.alert(`验证失败: ${err.message || "网络错误"}`, "错误", {
      confirmButtonText: "知道了",
      type: "error",
      center: true,
    });
  } finally {
    verifying.value = false;
  }
}

/**
 * 显示确认兑换弹窗，使用 beforeClose 控制异步关闭
 */
async function showConfirmDialog(disc, code) {
  // 先异步获取缓存的封面（Tauri 环境中会通过 proxy_image 获取 data: URI）
  const coverUrl = disc?.cover
    ? await getCachedCoverUrl(disc.cover).catch(() => getCoverUrl(disc.cover))
    : "";

  return new Promise((resolve) => {
    ElMessageBox.confirm(
      `<div style="text-align:center;padding:8px 0;">
        <img src="${coverUrl || getCoverUrl(disc.cover)}" style="width:120px;height:120px;border-radius:8px;object-fit:cover;margin-bottom:12px;" onerror="this.style.display='none'" />
        <p style="font-size:16px;font-weight:600;margin:8px 0 4px;color:var(--el-text-color-primary)">${disc.title}</p>
        <p style="font-size:13px;color:var(--el-text-color-secondary);margin:0">ID: ${disc.id}</p>
      </div>`,
      "兑换码有效",
      {
        confirmButtonText: "确认兑换",
        cancelButtonText: "取消",
        dangerouslyUseHTMLString: true,
        center: true,
        roundButton: true,
        beforeClose: async (action, instance, done) => {
          if (action === "confirm") {
            instance.confirmButtonLoading = true;
            instance.confirmButtonText = "兑换中...";
            try {
              const result = await doConfirm(code);
              // 先关闭确认弹窗
              done();
              // 再根据结果弹出提示
              if (result.success && result.disc) {
                showSuccessDialog(result.disc);
              } else if (result.message) {
                ElMessageBox.alert(result.message, "提示", {
                  confirmButtonText: "知道了",
                  type: result.success ? "success" : "warning",
                  center: true,
                });
              }
            } catch (err) {
              instance.confirmButtonLoading = false;
              instance.confirmButtonText = "确认兑换";
              ElMessageBox.alert(
                `兑换失败: ${err.message || "网络错误"}`,
                "错误",
                { confirmButtonText: "知道了", type: "error", center: true },
              );
            }
          } else {
            done();
          }
        },
      },
    )
      .then(resolve)
      .catch(() => {
        resolve();
      });
  });
}

/**
 * 执行确认兑换请求，返回结果对象
 */
async function doConfirm(code) {
  const data = await confirmRedeem(code);
  console.log("[Redeem] 确认结果:", data);

  const res = String(data?.res || "");
  const resultDisc = data?.disc || null;

  if (res === "1" && resultDisc) {
    return { success: true, disc: resultDisc, message: "兑换成功！" };
  } else if (res === "1" && !resultDisc) {
    return { success: false, disc: null, message: "该兑换码已被使用过" };
  } else {
    return {
      success: false,
      disc: null,
      message: "兑换失败，请检查兑换码后重试",
    };
  }
}

/**
 * 显示兑换成功弹窗
 */
async function showSuccessDialog(disc) {
  // 先异步获取缓存的封面（Tauri 环境中会通过 proxy_image 获取 data: URI）
  const coverUrl = disc?.cover
    ? await getCachedCoverUrl(disc.cover).catch(() => getCoverUrl(disc.cover))
    : "";

  ElMessageBox.alert(
    `<div style="text-align:center;padding:8px 0;">
      <img src="${coverUrl || getCoverUrl(disc.cover)}" style="width:100px;height:100px;border-radius:8px;object-fit:cover;margin-bottom:12px;" onerror="this.style.display='none'" />
      <p style="font-size:16px;font-weight:600;margin:8px 0 4px;color:var(--el-text-color-primary)">${disc.title}</p>
      <p style="font-size:13px;color:var(--el-text-color-secondary);margin:0">ID: ${disc.id}</p>
    </div>`,
    "兑换成功！",
    {
      confirmButtonText: "查看唱片",
      dangerouslyUseHTMLString: true,
      center: true,
      roundButton: true,
      callback: (action) => {
        if (action === "confirm" && disc.id) {
          router.push(`/album/${disc.id}`);
        }
      },
    },
  );
}

/**
 * 重置输入
 */
function handleReset() {
  codeInput.value = "";
}
</script>

<template>
  <div class="redeem-page">
    <div class="page-header">
      <h2>兑换码</h2>
      <p class="page-desc">输入兑换码，兑换数字专辑或单曲 EP</p>
    </div>

    <div class="redeem-card">
      <el-input
        v-model="codeInput"
        placeholder="请输入兑换码"
        size="large"
        clearable
        class="code-input"
        @keyup.enter="handleVerify"
      />
      <el-button
        type="primary"
        size="large"
        :loading="verifying"
        :disabled="!codeInput.trim()"
        class="verify-btn"
        @click="handleVerify"
      >
        {{ verifying ? "验证中..." : "验证兑换码" }}
      </el-button>
    </div>
  </div>
</template>

<style scoped>
.redeem-page {
  max-width: 600px;
  margin: 0 auto;
  padding: 24px;
}

.page-header {
  text-align: center;
  margin-bottom: 32px;
}

.page-header h2 {
  margin: 0 0 8px 0;
  font-size: 24px;
  color: var(--el-text-color-primary);
}

.page-desc {
  margin: 0;
  color: var(--el-text-color-secondary);
  font-size: 14px;
}

.redeem-card {
  background: var(--el-bg-color-overlay);
  border-radius: 12px;
  padding: 32px;
  text-align: center;
  box-shadow: var(--el-box-shadow-light);
}

.card-icon {
  margin-bottom: 16px;
}

.redeem-card h3 {
  margin: 0 0 20px 0;
  font-size: 18px;
  color: var(--el-text-color-primary);
}

.code-input {
  max-width: 400px;
  margin: 0 auto 16px;
}

.verify-btn {
  width: 200px;
}
</style>
