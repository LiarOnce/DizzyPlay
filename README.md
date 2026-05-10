# DizzyPlay

<img src="./app-icon.png" width="200" height="200" />

[![License](https://img.shields.io/badge/License-LGPL--3.0-blue.svg)](LICENSE)

第三方的开源 Dizzylab 桌面客户端，基于 Tauri + Vue 3。

本项目不是专业的音乐播放器，仅仅只是用于学习交流用途的非官方客户端实现，请不要将其作为平台专用的流媒体播放器。

## 目前已实现的功能

| 原生功能                                     | 通过 API 实现的功能                            | 没有 API<br />通过解析 HTML 实现的功能     | 没有 API<br />无法实现的功能                                                       | 因客户端性质<br />不考虑实现的功能 |
| -------------------------------------------- | ---------------------------------------------- | ------------------------------------------ | ---------------------------------------------------------------------------------- | ---------------------------------- |
| 播放控制<br />（播放、暂停、上一首、下一首） | 商品展示:<br />（数字专辑、单曲 EP、下载商品） | 下载已购作品<br />（需额外填写 Cookie 值） | 标签为 `PRO` 和 `STAFF` 的用户，且由 Dizzylab 提供的额外功能<br />（例如发布商品） | 购物车（功能来自官方 APP）         |
| 播放列表                                     | 社团展示                                       | 用户信息展示（repo、关注、+2dB）           | 搜索社团和用户                                                                     | 通过本客户端支付购买作品           |
| 数据缓存（数据、图片、音乐）                 | 用户信息展示（已购）                           |                                            | 首页轮播                                                                           |                                    |
| 下载管理                                     | 登录 Dizzylab 账号                             |                                            | 限时优惠                                                                           |                                    |
| 深色主题                                     | 搜索作品                                       |                                            | 站内信/收件箱                                                                      |                                    |
|                                              | CD key 兑换                                    |                                            |                                                                                    |                                    |

## 获取源码并编译测试

### 前置要求

- [Node.js](https://nodejs.org/) >= 18
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/) (edition 2021)

### 编译运行

```bash
# 克隆仓库
git clone https://github.com/LiarOnce/DizzyPlay
cd DizzyPlay

# 安装前端依赖
pnpm install

# 开发模式（热重载）
pnpm tauri dev

# 生产构建
pnpm tauri build --no-bundle
```

## 参考

- [Ovler-Young/DizzySync](https://github.com/Ovler-Young/DizzySync)
- [enkerewpo/dizzylab-redeem-autoprint](https://github.com/enkerewpo/dizzylab-redeem-autoprint)

## 许可证

[LGPL-3.0](LICENSE)
