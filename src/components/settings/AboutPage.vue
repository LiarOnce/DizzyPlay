<script setup>
import logoUrl from "/logo.png";
</script>

<template>
  <div class="tab-content">
    <h1>DizzyPlay</h1>
    <div class="about-content">
      <img :src="logoUrl" width="200" height="200" />

      <p>
        LiarOnce：
        <a href="https://github.com/LiarOnce" target="_blank">GitHub</a>
        <a href="https://www.dizzylab.net/u/5443" target="_blank">Dizzylab</a>
      </p>

      <div>
        第三方开源的 Dizzylab 桌面客户端<br />
        本项目不是专业的音乐播放器，仅仅只是用于学习交流用途的非官方客户端实现，请不要将其作为平台专用的流媒体播放器。<br />

        <h2>免责声明</h2>

        本项目按"现状"提供，不附带任何明示或暗示的保证，包括但不限于对适销性、特定用途适用性及不侵权的保证。<br />
        在任何情况下，提供方均不对因使用或无法使用本软件而产生的任何直接、间接、偶然、特殊、惩戒性或后果性损害赔偿（包括但不限于替代商品或服务的采购、使用中断、数据丢失或利润损失）承担责任，即使提供方已被告知可能发生此类损害。<br />

        本项目没有义务提供与本软件相关的技术支持、维护或更新。使用者应自行承担使用本软件的风险。<br />

        本项目可能包含由第三方提供的代码或库。对于此类第三方组件，使用者应遵守其各自的授权许可条款。提供方不对第三方组件的安全性、可靠性或合法性作出任何陈述或保证。<br />

        本项目是与 Dizzylab 官方无任何关联的第三方客户端实现，若该软件出现任何问题，不要尝试联系 Dizzylab 官方人员，请直接在本项目的 GitHub 仓库反馈。<br />

        本项目是完全免费的开源软件，若你是通过付费方式获得了软件，请立刻退款并向平台举报。对于从非 GitHub Release 渠道获得的软件恕不提供任何技术支持。<br />
      </div>
    </div>
  </div>
</template>

<style scoped>
.tab-content {
  padding: 0 16px;
  height: 100%;
  overflow-y: auto;
}

.about-content {
  padding: 16px;
  background: var(--bg-secondary, #1e1e2e);
  border-radius: 8px;
  border: 1px solid var(--border-color, #2d2d3d);
  min-height: 200px;
  line-height: 1.8;
}

.about-content h1 {
  color: var(--text-primary, #e0e0e0);
  margin-bottom: 8px;
}

.about-content h2 {
  color: var(--text-primary, #e0e0e0);
  margin: 16px 0 8px;
}

.about-content a {
  color: var(--el-color-primary);
  margin-right: 8px;
}

.about-content img {
  display: block;
  margin: 12px 0;
}
</style>
