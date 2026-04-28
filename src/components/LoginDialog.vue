<script setup>
import { ref } from "vue";
import { User, Lock, Key } from "@element-plus/icons-vue";
import { login } from "../stores/user.js";

const emit = defineEmits(["login-success"]);

const dialogVisible = ref(false);
const username = ref("");
const password = ref("");
const loading = ref(false);
const errorMsg = ref("");

function open() {
  dialogVisible.value = true;
  username.value = "";
  password.value = "";
  errorMsg.value = "";
}

function close() {
  dialogVisible.value = false;
}

async function handleLogin() {
  if (!username.value.trim() || !password.value.trim()) {
    errorMsg.value = "请输入用户名和密码";
    return;
  }

  loading.value = true;
  errorMsg.value = "";

  try {
    await login(username.value.trim(), password.value);
    console.log("[LoginDialog] 登录成功");
    dialogVisible.value = false;
    emit("login-success");
  } catch (err) {
    console.error("[LoginDialog] 登录失败:", err);
    errorMsg.value = err.message || "登录失败，请检查用户名和密码";
  } finally {
    loading.value = false;
  }
}

defineExpose({ open, close });
</script>

<template>
  <el-dialog
    v-model="dialogVisible"
    title="登录"
    width="380px"
    :close-on-click-modal="false"
    :close-on-press-escape="true"
    align-center
    class="login-dialog"
  >
    <div class="login-form">
      <el-form
        :model="{ username, password }"
        label-position="top"
        @keyup.enter="handleLogin"
      >
        <el-form-item label="用户名">
          <el-input
            v-model="username"
            placeholder="请输入用户名"
            :prefix-icon="User"
            clearable
            size="large"
          />
        </el-form-item>

        <el-form-item label="密码">
          <el-input
            v-model="password"
            type="password"
            placeholder="请输入密码"
            :prefix-icon="Lock"
            show-password
            size="large"
          />
        </el-form-item>

        <div v-if="errorMsg" class="error-message">
          <el-alert
            :title="errorMsg"
            type="error"
            show-icon
            :closable="false"
          />
        </div>

        <el-form-item>
          <el-button
            type="primary"
            size="large"
            :loading="loading"
            :icon="Key"
            class="login-btn"
            @click="handleLogin"
          >
            {{ loading ? "登录中..." : "登录" }}
          </el-button>
        </el-form-item>
      </el-form>
    </div>
  </el-dialog>
</template>

<style scoped>
.login-dialog {
  :deep(.el-dialog__body) {
    padding-top: 12px;
  }
}

.login-form {
  padding: 8px 0;
}

.login-btn {
  width: 100%;
  margin-top: 8px;
}

.error-message {
  margin-bottom: 16px;
}
</style>
