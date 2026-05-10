# Dizzylab API 文档

```markdown
This document is authored with assistance from AI/LLM:

- Model: DeepSeek V4 Pro
- Platform: DeepSeek Platform
- Agent platform: Kilo Code
- Prompt:
  请读取 `index.android.js` 文件，整理一份完整的API文档，其API的地址为 `https://www.dizzylab.net/apis/`，且请排除 `cart` 和 `alipay` 相关的接口，
```

**Base URL:** `https://www.dizzylab.net/`

---

## 认证说明

大部分接口需要 `token` 参数进行身份验证，token 通过登录接口获取。

---

## 一、GET 接口

### 1. getappinfo/ — 获取应用信息

```
GET /apis/getappinfo/?fromver={version}&token={token}
```

| 参数    | 类型   | 说明                        |
| ------- | ------ | --------------------------- |
| fromver | int    | 当前 APP 版本号 (APPVER=12) |
| token   | string | 用户登录 token              |

**返回：**

```json
{
  "appinfo": [],
  "needupdate": false
}
```

---

### 2. getdiscs/ — 获取作品列表 (首页)

```
GET /apis/getdiscs/?l={offset}&r={limit}&sort={sort}&token={token}
```

| 参数  | 类型   | 说明                              |
| ----- | ------ | --------------------------------- |
| l     | int    | 起始偏移量 (page\* loadcount)     |
| r     | int    | 结束偏移量 ((page+1)\* loadcount) |
| sort  | string | 排序方式 (默认 "ad")              |
| token | string | 用户登录 token                    |

**返回：**

```json
{
  "discs": [],
  "canshowmore": true
}
```

---

### 3. getlabels/ — 获取社团列表

```
GET /apis/getlabels/?l={offset}&r={limit}&token={token}
```

| 参数  | 类型   | 说明           |
| ----- | ------ | -------------- |
| l     | int    | 起始偏移量     |
| r     | int    | 结束偏移量     |
| token | string | 用户登录 token |

**返回：**

```json
{
  "labels": [],
  "canshowmore": true
}
```

---

### 4. getmyinfo/ — 获取当前用户信息

```
GET /apis/getmyinfo/?token={token}
```

| 参数  | 类型   | 说明           |
| ----- | ------ | -------------- |
| token | string | 用户登录 token |

**返回：**

```json
{
  "user": {
    "uid": 1,
    "username": "string",
    "cover": "url",
    "user_group": "STAFF" | "PRO" | ""
  }
}
```

---

### 5. getmydisc/ — 获取"已购"作品列表

```
GET /apis/getmydisc/?l={offset}&r={limit}&sort={sort}&token={token}
```

| 参数  | 类型   | 说明                 |
| ----- | ------ | -------------------- |
| l     | int    | 起始偏移量           |
| r     | int    | 结束偏移量           |
| sort  | string | 排序方式 (默认 "ad") |
| token | string | 用户登录 token       |

**返回：**

```json
{
  "discs": [],
  "canshowmore": true
}
```

---

### 6. getmylike/ — 获取"+2dB"点赞作品列表

```
GET /apis/getmylike/?l={offset}&r={limit}&sort={sort}&token={token}
```

| 参数  | 类型   | 说明                 |
| ----- | ------ | -------------------- |
| l     | int    | 起始偏移量           |
| r     | int    | 结束偏移量           |
| sort  | string | 排序方式 (默认 "ad") |
| token | string | 用户登录 token       |

**返回：**

```json
{
  "discs": [],
  "canshowmore": true
}
```

---

### 7. getmyfollowing/ — 获取"关注"的社团列表

```
GET /apis/getmyfollowing/?l={offset}&r={limit}&sort={sort}&token={token}
```

| 参数  | 类型   | 说明                 |
| ----- | ------ | -------------------- |
| l     | int    | 起始偏移量           |
| r     | int    | 结束偏移量           |
| sort  | string | 排序方式 (默认 "ad") |
| token | string | 用户登录 token       |

**返回：**

```json
{
  "discs": [],
  "canshowmore": true
}
```

---

### 8. getotheruserinfo/ — 获取其他用户信息及作品

```
GET /apis/getotheruserinfo/?l={offset}&r={limit}&uid={uid}&token={token}
```

| 参数  | 类型   | 说明               |
| ----- | ------ | ------------------ |
| l     | int    | 起始偏移量         |
| r     | int    | 结束偏移量         |
| uid   | int    | 目标用户 ID        |
| token | string | 当前用户登录 token |

**返回：**

```json
{
  "user": {},
  "discs": [],
  "mylabel": [],
  "canshowmore": true
}
```

---

### 9. getsomecover/ — 获取封面图片 (登录页背景)

不需要 token。

```
GET /apis/getsomecover/?l={offset}&r={limit}
```

| 参数 | 类型 | 说明       |
| ---- | ---- | ---------- |
| l    | int  | 起始偏移量 |
| r    | int  | 结束偏移量 |

**返回：**

```json
{
  "discs": [],
  "canshowmore": true
}
```

---

### 10. getthisdicsinfo/ — 获取作品详情

```
GET /apis/getthisdicsinfo/?discid={discId}&token={token}
```

| 参数   | 类型   | 说明           |
| ------ | ------ | -------------- |
| discid | int    | 作品 ID        |
| token  | string | 用户登录 token |

**返回：**

```json
{
  "tracks": [],
  "ilikeit": false,
  "likes": 0,
  "ihavethis": false,
  "price": 0,
  "onsell": true,
  "ispreselling": false,
  "onlyhavegift": false,
  "labelid": 1,
  "boost": 0,
  "tags": [],
  "label": "string",
  "title": "string",
  "cover": "url",
  "release_date": "string",
  "comment": "string"
}
```

---

### 11. gettrackdownloadurl/ — 获取音轨下载地址

```
GET /apis/gettrackdownloadurl/?discid={discId}&token={token}&trackid={trackId}&packtype={packtype}
```

| 参数     | 类型       | 说明                      |
| -------- | ---------- | ------------------------- |
| discid   | int        | 作品 ID                   |
| token    | string     | 用户登录 token            |
| trackid  | int/string | 音轨 ID                   |
| packtype | string     | 音质类型 ("128" 或 "320") |

**返回：**

```json
{
  "track": {
    "url": "https://..."
  }
}
```

下载时需要带 `Referer: https://www.dizzylab.net` 请求头。

---

### 12. gethislabelinfo/ — 获取社团详情

```
GET /apis/gethislabelinfo/?labelid={labelId}&token={token}&l={offset}&r={limit}
```

| 参数    | 类型   | 说明           |
| ------- | ------ | -------------- |
| labelid | int    | 社团 ID        |
| token   | string | 用户登录 token |
| l       | int    | 起始偏移量     |
| r       | int    | 结束偏移量     |

**返回：**

```json
{
  "disc": [],
  "members": [],
  "following": false,
  "canshowmore": true
}
```

---

### 13. getdiscbuyers/ — 获取作品购买者/支持者列表

```
GET /apis/getdiscbuyers/?l={offset}&r={limit}&discid={discId}&token={token}
```

| 参数   | 类型   | 说明           |
| ------ | ------ | -------------- |
| l      | int    | 起始偏移量     |
| r      | int    | 结束偏移量     |
| discid | int    | 作品 ID        |
| token  | string | 用户登录 token |

**返回：**

```json
{
  "buyers": [],
  "boosters": [],
  "canshowmore": true
}
```

---

### 14. getdisccommit/ — 获取作品评论列表

```
GET /apis/getdisccommit/?l={offset}&r={limit}&discid={discId}&token={token}
```

| 参数   | 类型   | 说明           |
| ------ | ------ | -------------- |
| l      | int    | 起始偏移量     |
| r      | int    | 结束偏移量     |
| discid | int    | 作品 ID        |
| token  | string | 用户登录 token |

**返回：**

```json
{
  "commit": [],
  "canshowmore": true,
  "mycomment": ""
}
```

---

### 15. getlabeldiscs/ — 获取同社团更多作品

```
GET /apis/getlabeldiscs/?l={offset}&r={limit}&labelid={labelId}&token={token}&without={discId}
```

| 参数    | 类型   | 说明                           |
| ------- | ------ | ------------------------------ |
| l       | int    | 起始偏移量                     |
| r       | int    | 结束偏移量                     |
| labelid | int    | 社团 ID                        |
| token   | string | 用户登录 token                 |
| without | int    | 排除的作品 ID (当前查看的作品) |

**返回：**

```json
{
  "discs": [],
  "canshowmore": true
}
```

---

### 16. search/ — 搜索

```
GET /apis/search/?l={offset}&r={limit}&token={token}&keyword={keyword}
```

| 参数    | 类型   | 说明           |
| ------- | ------ | -------------- |
| l       | int    | 起始偏移量     |
| r       | int    | 结束偏移量     |
| token   | string | 用户登录 token |
| keyword | string | 搜索关键词     |

**返回：**

```json
{
  "discs": [],
  "allcount": 0,
  "canshowmore": true
}
```

---

### 17. tags/ — 按标签浏览

```
GET /apis/tags/?l={offset}&r={limit}&token={token}&tag={tag}
```

| 参数  | 类型   | 说明           |
| ----- | ------ | -------------- |
| l     | int    | 起始偏移量     |
| r     | int    | 结束偏移量     |
| token | string | 用户登录 token |
| tag   | string | 标签名称       |

**返回：**

```json
{
  "discs": [],
  "allcount": 0,
  "alltagslist": [],
  "canshowmore": true
}
```

---

### 18. iwanttweeksomething/ — 切换邮件通知

```
GET /apis/iwanttweeksomething/?labelid={labelId}&token={token}&changemail
```

| 参数       | 类型   | 说明             |
| ---------- | ------ | ---------------- |
| labelid    | int    | 社团 ID          |
| token      | string | 用户登录 token   |
| changemail | -      | 无值，仅作为标记 |

**返回：** json

---

## 二、POST 接口

### 19. auth/login/ — 登录

```
POST /apis/auth/login/
Content-Type: application/json
```

**请求体：**

```json
{
  "username": "string",
  "password": "string"
}
```

**返回：**

```json
{
  "token": "xxx"
}
```

登录失败返回空 token，客户端显示"用户名或密码错误"。

---

### 20. ilikethisornot/ — 切换点赞 (+2dB)

```
POST /apis/ilikethisornot/
Content-Type: multipart/form-data
```

**请求体 (FormData)：**

| 字段   | 类型   | 说明           |
| ------ | ------ | -------------- |
| discid | int    | 作品 ID        |
| token  | string | 用户登录 token |

**返回：**

```json
{
  "ilikeit": true,
  "likes": 10
}
```

---

### 21. ifollowthisornot/ — 切换关注社团

```
POST /apis/ifollowthisornot/
Content-Type: multipart/form-data
```

**请求体 (FormData)：**

| 字段    | 类型   | 说明           |
| ------- | ------ | -------------- |
| labelid | int    | 社团 ID        |
| token   | string | 用户登录 token |

**返回：**

```json
{
  "following": true
}
```

---

### 22. postcomment/ — 发表评论

```
POST /apis/postcomment/
Content-Type: multipart/form-data
```

**请求体 (FormData)：**

| 字段    | 类型   | 说明           |
| ------- | ------ | -------------- |
| discid  | int    | 作品 ID        |
| token   | string | 用户登录 token |
| comment | string | 评论内容       |

**返回：**

```json
{
  "return": "ok"
}
```

---

### 23. unlockafreedisc/ — 解锁免费作品

```
POST /apis/unlockafreedisc/
Content-Type: multipart/form-data
```

**请求体 (FormData)：**

| 字段   | 类型   | 说明           |
| ------ | ------ | -------------- |
| discid | int    | 作品 ID        |
| token  | string | 用户登录 token |

请求需带 `Referer: https://www.dizzylab.net` 头。

**返回：**

```json
{
  "state": "ok"
}
```

---

### 24. uploadandchangeuseravatar/ — 上传头像

```
POST /apis/uploadandchangeuseravatar/
Content-Type: multipart/form-data
```

**请求体 (FormData)：**

| 字段   | 类型   | 说明                                                  |
| ------ | ------ | ----------------------------------------------------- |
| token  | string | 用户登录 token                                        |
| avatar | file   | 头像图片文件 (uri, type: "multipart/form-data", name) |

**返回：** json

---

### 25. redeem/ — 验证兑换码

```
POST /apis/redeem/
Content-Type: multipart/form-data
```

**请求体 (FormData)：**

| 字段  | 类型   | 说明           |
| ----- | ------ | -------------- |
| token | string | 用户登录 token |
| code  | string | 兑换码         |

请求需带 `Referer: https://www.dizzylab.net` 头。

**返回：**

```json
{
  "res": "4",
  "disc": {}
}
```

`res` 值说明：

- `"0"` — 未知错误
- `"1"` — 兑换码无效或已使用
- `"2"` — 已拥有该商品
- `"4"` — 验证成功，可以兑换 (返回 disc 信息)
- `"6"` — 需使用网页版兑换

---

### 26. comfirm_redeem/ — 确认兑换

```
POST /apis/comfirm_redeem/
Content-Type: multipart/form-data
```

**请求体 (FormData)：**

| 字段  | 类型   | 说明           |
| ----- | ------ | -------------- |
| token | string | 用户登录 token |
| code  | string | 兑换码         |

请求需带 `Referer: https://www.dizzylab.net` 头。

**返回：**

```json
{
  "res": "1"
}
```

`res` 值说明：

- `"0"` — 未知错误
- `"1"` — 兑换成功
- `"2"` — 兑换失败
- `"6"` — 需使用网页版下载

---

## 三、通用说明

### 分页参数

列表接口统一使用 `l` (lower/offset) 和 `r` (upper/limit) 进行分页：

- `l = page * loadcount`
- `r = (page + 1) * loadcount`
- 返回字段 `canshowmore` 指示是否有更多数据

### 认证

登录成功后获取 token，存储在客户端 (AsyncStorage key: "token")，后续请求作为查询参数携带。

### APP 版本

当前 APP 版本 `APPVER = 12`，仅在 `getappinfo` 接口中使用。

### 下载

音频下载需要 Referer 头：`Referer: https://www.dizzylab.net`
