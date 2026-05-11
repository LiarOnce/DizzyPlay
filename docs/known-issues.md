# 常见问题

## 在 Linux 下播放器显示异常（闪烁、错位等）

这是一个来自 Webkit2GTK 的已知问题，与 DizzyPlay 无关，若出现显示异常请尝试更新 Webkit2GTK 版本到 2.52 及以上的版本，该版本经测试可用。

若异常仍然存在，请在程序前添加以下环境变量之一来启动播放器：

```
WEBKIT_DISABLE_COMPOSITING_MODE=1 # 该环境变量将强制使用 Bitmap 合成
# OR
WEBKIT_DISABLE_DMABUF_RENDERER=1 # 该环境变量将完全禁用 DMA-BUF 合成器
```

请注意，使用这些环境变量可能导致播放器页面显示卡顿，但不会影响功能。
