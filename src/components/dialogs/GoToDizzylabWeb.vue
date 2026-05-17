<script setup>
import { computed } from "vue";
import { isTauri } from "../../utils/format.js";

const props = defineProps({
  modelValue: {
    type: Boolean,
    default: false,
  },
  albumUrl: {
    type: String,
    default: "",
  },
  albumTitle: {
    type: String,
    default: "",
  },
});

const emit = defineEmits(["update:modelValue", "purchase-complete"]);

const dialogVisible = computed({
  get: () => props.modelValue,
  set: (val) => emit("update:modelValue", val),
});

function openInBrowser() {
  if (isTauri) {
    window.__TAURI_INTERNALS__.invoke("plugin:opener|open_url", {
      url: props.albumUrl,
    });
  } else {
    window.open(props.albumUrl, "_blank");
  }
}

function onPurchaseComplete() {
  emit("purchase-complete");
  dialogVisible.value = false;
}

function onCancel() {
  dialogVisible.value = false;
}
</script>

<template>
  <el-dialog
    v-model="dialogVisible"
    title="前往网页端购买"
    width="420px"
    :close-on-click-modal="false"
  >
    <div class="purchase-dialog-content">
      <p>由于 DizzyPlay 没有也不会提供任何支付功能，购买商品和追加 BOOST 请前往 Dizzylab 网页端完成。</p>
      <el-button type="primary" @click="openInBrowser">
        打开购买页面
      </el-button>
    </div>
    <template #footer>
        <el-button type="success" @click="onPurchaseComplete">已完成购买</el-button>
        <el-button @click="onCancel">取消</el-button>
    </template>
  </el-dialog>
</template>

<style scoped>
.purchase-dialog-content {
  text-align: center;
  padding: 20px 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.purchase-dialog-content p {
  margin: 0;
  font-size: 14px;
  color: var(--el-text-color-regular);
  line-height: 1.6;
}
</style>
