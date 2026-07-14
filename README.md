# Domus

用 Rust + Flutter 重写的 Immich —— 协议兼容 Immich 3.0.2 的自托管照片/视频库。

- **服务端**：Rust（axum + sqlx + socketioxide），单二进制，PostgreSQL 队列（无 Redis）
- **客户端**：单一 Flutter 代码库覆盖 Web / Android / iOS
- **协议**：以 Immich 3.0.2 OpenAPI 规范（173 路径 / 254 操作）为契约，官方 immich 客户端可直连
- **不含** immich-machine-learning（智能搜索/人脸/OCR 上报为关闭）

## 目录

```
docs/     系统设计文档（SYSTEM_DESIGN.md）+ API 端点清单
server/   Rust workspace（7 crates）
app/      Flutter 应用（web/android/ios）
```

## 快速开始

```bash
# 服务端（需要 PostgreSQL，默认 postgres://postgres:postgres@localhost:5432/immich）
cd server && cargo run -p domus-server

# 客户端
cd app && flutter pub get && flutter run
```

## 文档

- [系统设计文档](docs/SYSTEM_DESIGN.md) —— 架构、模块、协议兼容策略、任务拆解
- [API 端点清单](docs/api-endpoints-raw.md) —— 兼容目标的全部 254 个操作

## Immich Feature Parity Checklist

> 来源：immich-app/immich README 的 Features 表。只有当服务端、Flutter Web/Android/iOS 主流程、协议兼容测试和必要单测/集成测试都达到可生产使用时才勾选。

| 状态 | Feature | Mobile | Web |
|---|---|---|---|
| [x] | Upload and view videos and photos | Yes | Yes |
| [x] | Auto backup when the app is opened | Yes | N/A |
| [x] | Prevent duplication of assets | Yes | Yes |
| [x] | Selective album(s) for backup | Yes | N/A |
| [x] | Download photos and videos to local device | Yes | Yes |
| [x] | Multi-user support | Yes | Yes |
| [x] | Album and Shared albums | Yes | Yes |
| [x] | Scrubbable/draggable scrollbar | Yes | Yes |
| [x] | Support raw formats | Yes | Yes |
| [x] | Metadata view (EXIF, map) | Yes | Yes |
| [ ] | Search by metadata, objects, faces, and CLIP | Yes | Yes |
| [x] | Administrative functions (user management) | No | Yes |
| [x] | Background backup | Yes | N/A |
| [x] | Virtual scroll | Yes | Yes |
| [x] | OAuth support | Yes | Yes |
| [x] | API Keys | N/A | Yes |
| [x] | LivePhoto/MotionPhoto backup and playback | Yes | Yes |
| [x] | Support 360 degree image display | No | Yes |
| [x] | User-defined storage structure | Yes | Yes |
| [x] | Public Sharing | Yes | Yes |
| [x] | Archive and Favorites | Yes | Yes |
| [x] | Global Map | Yes | Yes |
| [x] | Partner Sharing | Yes | Yes |
| [ ] | Facial recognition and clustering | Yes | Yes |
| [x] | Memories (x years ago) | Yes | Yes |
| [x] | Offline support | Yes | No |
| [x] | Read-only gallery | Yes | Yes |
| [x] | Stacked Photos | Yes | Yes |
| [x] | Tags | No | Yes |
| [x] | Folder View | Yes | Yes |
