/**
 * HTML 页面抓取与解析
 *
 * - https://www.dizzylab.net/u/{uid}/likes/   （+2dB，用户的喜欢列表）
 * - https://www.dizzylab.net/u/{uid}/review/  （repo，用户的评论列表）
 * - https://www.dizzylab.net/u/{uid}/following/（关注，用户的关注列表）
 */

import { isTauri } from "../utils/format.js";
import { DIZZYLAB_REFERER } from "../globalvar.js";

/**
 * 通过 Rust 后端代理获取用户页面的 HTML 内容
 * @param {string|number} uid - 用户 ID
 * @param {string} type - 页面类型：'likes' | 'review' | 'following'
 * @returns {Promise<string>} HTML 字符串
 */
export async function fetchUserPageHtml(uid, type) {
  const url = `https://www.dizzylab.net/u/${uid}/${type}/`;
  console.log(`[HtmlParser] 获取 HTML: ${url}`);

  if (isTauri) {
    try {
      const { invoke } = window.__TAURI_INTERNALS__;
      const html = await invoke("proxy_html_page", { url });
      return html;
    } catch (err) {
      console.error(`[HtmlParser] Tauri 获取 HTML 失败:`, err);
      throw err;
    }
  } else {
    // 非 Tauri 环境：通过 Vite proxy 获取
    try {
      const proxyUrl = `/user-page?url=${encodeURIComponent(url)}`;
      const response = await fetch(proxyUrl, {
        headers: { Referer: DIZZYLAB_REFERER },
      });
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }
      return await response.text();
    } catch (err) {
      console.error(`[HtmlParser] fetch 获取 HTML 失败:`, err);
      throw err;
    }
  }
}

/**
 * 从用户页面的 HTML 中解析出唱片列表
 *
 * 支持两种页面结构（均在 tab-pane#album 区域内）：
 *
 * 1. 唱片列表（likes/review 页面）- 参考 docs/html/Response-userinfo.html：
 *    <div class="tab-pane ..." id="album">
 *      <div class="row">
 *        <div class="card-group ... item">
 *          <div class="card">
 *            <div class="card-body">
 *              <a href="/d/{discId}">
 *                <img data-src="{coverUrl}" ... />
 *              </a>
 *            </div>
 *            <div class="card-footer">
 *              <h4 title="{title}">{title}</h4>
 *              <a href="/l/{labelName}">@ {label}</a>
 *            </div>
 *          </div>
 *        </div>
 *      </div>
 *    </div>
 *
 * 2. 社团列表（following 页面）- 参考 docs/html/Response-userinfo-following.html：
 *    <div class="tab-pane ..." id="album">
 *      <div class="row">
 *        <div class="col-md-2 col-4">
 *          <div class="card">
 *            <div class="card-body">
 *              <a href="/l/{labelName}">
 *                <img data-src="{coverUrl}" ... />
 *              </a>
 *              <p class="text-center">{labelName}</p>
 *            </div>
 *          </div>
 *        </div>
 *      </div>
 *    </div>
 *
 * 解析策略：
 * 1. 先提取 tab-pane#album 内容区域，排除导航栏、页脚等无关区域的干扰
 * 2. 在内容区域内，先尝试匹配唱片条目（/d/{id}），再尝试匹配社团条目（/l/{name}）
 *
 * @param {string} html - 用户页面的 HTML 字符串
 * @returns {Array<{id: string, title: string, cover: string, label: string}>}
 */
export function parseUserPageDiscs(html) {
  const discs = [];

  // ===== 步骤1：提取 tab-pane#album 内容区域 =====
  // 排除导航栏、用户头部、页脚等无关区域的干扰
  const albumPaneMatch = html.match(
    /<div[^>]*class="[^"]*tab-pane[^"]*"[^>]*\bid="album"[^>]*>([\s\S]*?)<\/div>\s*<nav[^>]*>/i,
  );
  if (!albumPaneMatch) {
    console.warn("[HtmlParser] 未找到 tab-pane#album 内容区域");
    return discs;
  }

  const albumContent = albumPaneMatch[1];
  console.log(
    `[HtmlParser] 提取到 tab-pane#album 内容区域: ${albumContent.length} 字符`,
  );

  // ===== 步骤2：在内容区域内匹配唱片条目（/d/{id} 结构，用于 likes/review 页面）=====
  // 通过 card-group 结构定位每个唱片，避免上下文重叠导致数据错乱
  const cardGroupRegex =
    /<div[^>]*class="[^"]*card-group[^"]*"[^>]*>[\s\S]*?<div\s+class="card[^"]*"[^>]*>[\s\S]*?<div\s+class="card-body[^"]*"[^>]*>([\s\S]*?)<\/div>\s*<div\s+class="card-footer[^"]*"[^>]*>([\s\S]*?)<\/div>\s*<\/div>\s*<\/div>/gi;
  let cardMatch;

  while ((cardMatch = cardGroupRegex.exec(albumContent)) !== null) {
    const cardBody = cardMatch[1];
    const cardFooter = cardMatch[2];

    // 从 card-body 提取 discId 和封面
    const linkMatch = cardBody.match(/<a\s+href="\/d\/([^"]+)"[^>]*>/);
    if (!linkMatch) continue;
    const discId = linkMatch[1];

    // 封面 URL: data-src="{coverUrl}"（在 card-body 的 <a> 内）
    const coverMatch = cardBody.match(/data-src="([^"]+)"/);
    const cover = coverMatch ? coverMatch[1] : "";

    // 从 card-footer 提取标题: 优先取 <h4 ... title="{title}">，否则取 <h4> 文本内容
    let title = "";
    const titleAttrMatch = cardFooter.match(/<h4[^>]*\s+title="([^"]+)"/);
    if (titleAttrMatch) {
      title = titleAttrMatch[1];
    } else {
      const titleTextMatch = cardFooter.match(/<h4[^>]*>([^<]+)<\/h4>/);
      if (titleTextMatch) {
        title = titleTextMatch[1].trim();
      }
    }

    // 从 card-footer 提取社团名: @ {labelName}</a>
    const labelMatch = cardFooter.match(/@\s*([^<]+)<\/a>/);
    const label = labelMatch ? labelMatch[1].trim() : "";

    // 去重
    const exists = discs.some((d) => d.id === discId);
    if (!exists) {
      discs.push({ id: discId, title, cover, label });
    }
  }

  // ===== 步骤3：如果在内容区域内没找到唱片条目，尝试匹配社团条目（/l/{name} 结构，用于 following 页面）=====
  if (discs.length === 0) {
    // 通过 href="/l/{labelName}" 定位每个社团
    // 注意：只匹配 card-body 内的 /l/ 链接（有 data-src 图片的），排除导航栏的 /label/ 链接
    const labelLinkRegex =
      /<a\s+href="\/l\/([^"]+)"[^>]*>[\s\S]*?<img[^>]*data-src="([^"]*)"/gi;
    let labelMatch;

    while ((labelMatch = labelLinkRegex.exec(albumContent)) !== null) {
      const labelName = labelMatch[1];
      const cover = labelMatch[2];
      const linkPos = labelMatch.index;

      // 从链接位置向后取一段上下文，提取社团显示名
      const contextEnd = Math.min(albumContent.length, linkPos + 500);
      const context = albumContent.slice(linkPos, contextEnd);

      // 提取社团显示名（在 <p class="text-center"> 中）
      const nameMatch = context.match(
        /<p[^>]*class="[^"]*text-center[^"]*"[^>]*>([^<]+)<\/p>/,
      );
      const displayName = nameMatch
        ? nameMatch[1].trim()
        : decodeURIComponent(labelName);

      // 去重
      const exists = discs.some((d) => d.id === labelName);
      if (!exists) {
        discs.push({
          id: labelName,
          title: displayName,
          cover,
          label: displayName,
        });
      }
    }
  }

  console.log(`[HtmlParser] 从 HTML 解析出 ${discs.length} 张唱片`);
  return discs;
}
