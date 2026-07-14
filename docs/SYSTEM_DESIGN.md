# Domus 系统设计文档

> 用 Rust + Flutter 重写 Immich（基于 deploy_3.0.2 分支），协议与原生 Immich 完全兼容。
>
> 版本：v0.1 · 日期：2026-07-14 · 参考源码：`/root/src/software/immich`（分支 `deploy_3.0.2`）

---

## 目录

1. [项目概述与目标](#1-项目概述与目标)
2. [总体架构](#2-总体架构)
3. [技术选型](#3-技术选型)
4. [协议兼容性策略](#4-协议兼容性策略)
5. [后端模块划分（Rust Workspace）](#5-后端模块划分rust-workspace)
6. [API 兼容矩阵与优先级](#6-api-兼容矩阵与优先级)
7. [数据库设计](#7-数据库设计)
8. [后台任务队列设计](#8-后台任务队列设计)
9. [媒体处理管线](#9-媒体处理管线)
10. [认证与授权](#10-认证与授权)
11. [增量同步协议（Sync）](#11-增量同步协议sync)
12. [WebSocket 实时通道](#12-websocket-实时通道)
13. [Flutter 客户端架构](#13-flutter-客户端架构)
14. [部署架构](#14-部署架构)
15. [非功能性需求](#15-非功能性需求)
16. [任务拆解与里程碑](#16-任务拆解与里程碑)
17. [风险与缓解](#17-风险与缓解)
18. [附录](#18-附录)

---

## 1. 项目概述与目标

### 1.1 背景

Immich 是自托管照片/视频库方案，官方实现为 TypeScript/NestJS 服务端 + SvelteKit Web + Flutter 移动端 + Python ML 服务。Domus 项目用 **Rust 重写服务端**、用 **单一 Flutter 代码库覆盖 Web / Android / iOS 三端**，目标是更低的资源占用、更简单的部署（单二进制、可去 Redis 化）与统一的客户端技术栈。

### 1.2 目标

| # | 目标 | 说明 |
|---|------|------|
| G1 | **协议兼容** | 以 Immich 3.0.2 的 OpenAPI 规范（173 路径 / 254 操作 / 376 Schema）为契约；官方 immich 移动端 App、immich CLI、第三方工具可直接连接 Domus |
| G2 | Rust 服务端 | axum + sqlx + tokio，单二进制同时承载 API 与后台 worker |
| G3 | Flutter 三端 | 一套代码构建 Web、Android、iOS |
| G4 | 不含 ML | 不实现 immich-machine-learning；智能搜索/人脸识别/OCR/重复检测（依赖向量）在 `/server/features` 中上报为关闭 |
| G5 | 数据可迁移 | 数据库表名/列名、磁盘目录布局与 Immich 对齐，保留"直接挂接存量 Immich 数据"的可能性 |

### 1.3 非目标（明确不做）

- immich-machine-learning 模块及其依赖的功能：CLIP 智能搜索、人脸检测/识别、OCR、基于向量的重复检测；
- 插件系统（`/plugins`）与工作流（`/workflows`）——3.x 新增的扩展机制，首期只保留路由占位；
- 图片编辑器（`/assets/:id/edits`）、实时转码 HLS（`/assets/:id/video/hls`）首期占位；
- 与 Immich 内部实现细节（BullMQ 队列结构、Redis 协议）的兼容——**兼容边界是对客户端的 HTTP/WebSocket 协议**，服务端内部实现自由。

---

## 2. 总体架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        Flutter 客户端 (app/)                      │
│   Web (CanvasKit)      Android              iOS                  │
│   ├── presentation  页面/组件（Material 3，自适应导航）              │
│   ├── application   Riverpod Provider（状态）                     │
│   ├── data          Repository（REST/sync 客户端、本地 drift 库）   │
│   └── platform      photo_manager / 后台上传（仅移动端）            │
└──────────────┬──────────────────────────────────────────────────┘
               │ HTTPS  /api/*  (REST, Immich OpenAPI 3.0.2 契约)
               │ WSS    /api/socket.io  (socket.io 协议)
┌──────────────▼──────────────────────────────────────────────────┐
│                    domus-server (单二进制, Rust)                  │
│                                                                  │
│  ┌───────────── api worker ─────────────┐  ┌── microservices ──┐ │
│  │ domus-api                            │  │ domus-jobs        │ │
│  │  · axum Router（/api 前缀）           │  │  · WorkerPool     │ │
│  │  · 认证 extractor（4 种载体）          │  │  · 每队列 N 并发   │ │
│  │  · DTO（camelCase 序列化）            │  │  · 媒体管线 handler│ │
│  │  · socketioxide（事件推送）           │  │                   │ │
│  └──────────────┬───────────────────────┘  └────────┬──────────┘ │
│                 │            domus-domain            │            │
│                 │   （业务服务层：授权/编排/事务边界）    │            │
│                 └──────────────┬──────────┬──────────┘            │
│                          domus-db    domus-media                 │
│                    （sqlx 仓储 + 迁移） （exiftool/ffmpeg/缩略图）    │
└───────────────────────┬──────────────────────┬───────────────────┘
                        │                      │
              ┌─────────▼────────┐   ┌─────────▼─────────┐
              │   PostgreSQL     │   │   媒体目录 /data    │
              │  · 业务表(仿immich)│   │  upload/ library/  │
              │  · job 队列表     │   │  thumbs/ profile/  │
              │  · 审计/同步检查点 │   │  encoded-video/    │
              └──────────────────┘   │  backups/          │
                                     └────────────────────┘
```

要点：

1. **单二进制、双角色**。与 Immich 的 api/microservices 双 worker 模型对应，Domus 用同一个二进制的两组任务：`workers.api`（HTTP + WebSocket）与 `workers.microservices`（后台 job 轮询）。通过环境变量（兼容 `IMMICH_WORKERS_INCLUDE` 语义）可拆成两个进程水平扩展。
2. **去 Redis 化**。Immich 用 BullMQ/Redis 做队列；队列不属于客户端协议，Domus 改用 PostgreSQL `FOR UPDATE SKIP LOCKED` 队列（详见 §8），部署少一个组件。
3. **PostgreSQL 不需要向量扩展**。Immich 的官方镜像带 VectorChord/pgvecto.rs，仅服务于 ML 功能；Domus 用原生 PostgreSQL 14+ 即可。

---

## 3. 技术选型

### 3.1 服务端（Rust）

| 领域 | 选型 | 理由 |
|------|------|------|
| 异步运行时 | tokio | 事实标准；多线程调度覆盖 IO 密集（API）+ CPU 密集（媒体处理走 spawn_blocking/子进程） |
| Web 框架 | **axum 0.8** | tower 生态、类型安全 extractor、multipart/WS 支持好；路由语法 `{id}` 与 OpenAPI 路径天然对应 |
| ORM/DB | **sqlx 0.8**（PostgreSQL，运行时查询） | 无重 ORM 开销；`migrate!` 内嵌迁移；不用编译期宏检查（避免构建依赖活库） |
| socket.io | **socketioxide 0.17** | 直接讲 socket.io/engine.io v4 协议，兼容 immich 客户端（`path=/api/socket.io`，`transports=['websocket']`） |
| 密码哈希 | bcrypt | 与 Immich 相同算法 ⇒ 存量用户密码哈希可直接迁移 |
| 会话/API key 哈希 | SHA-256（hex） | 与 Immich 相同：DB 只存哈希 |
| 资产校验和 | SHA-1 | Immich 的资产身份标识（去重、`x-immich-checksum`） |
| 配置 | figment（env） | 同时接受 `DOMUS_*` 与 `IMMICH_*`/`DB_*` 环境变量，兼容存量部署 |
| 日志/追踪 | tracing + tracing-subscriber | 结构化日志，后续接 OTLP |
| 图像处理 | 见 §9：优先 libvips CLI/绑定，退化 `image` crate | Immich 用 sharp(libvips)，格式覆盖广（HEIC/RAW） |
| 视频 | ffmpeg/ffprobe 子进程 | 与 Immich 一致 |
| EXIF | exiftool 子进程（`-json -n`） | Immich 用 exiftool-vendored，标签覆盖等价 |

### 3.2 客户端（Flutter）

| 领域 | 选型 | 理由（对照 immich mobile 现状） |
|------|------|------|
| 状态管理 | **flutter_riverpod 3** | immich mobile 同为 riverpod，模式可迁移 |
| 路由 | **go_router**（immich 用 auto_route） | go_router 官方维护、web URL 策略好（三端统一诉求下优于 auto_route） |
| HTTP | dio | 拦截器（token 注入、重试）、上传进度、web 兼容 |
| 本地库 | **drift（SQLite）**，web 端用 drift 的 wasm/IndexedDB 后端 | immich mobile 已从 Isar 迁到 drift；三端可用 |
| API client | 首期手写轻量 Repository；后续用 openapi-generator 从 immich 规范生成 dart client 作为唯一数据源 | immich mobile 的 `mobile/openapi` 即生成产物 |
| 设备相册 | photo_manager（仅 Android/iOS） | immich mobile 同款 |
| 后台上传 | background_downloader（移动端）；web 端仅前台手动上传 | immich mobile 同款 |
| 地图 | maplibre_gl（后期） | immich mobile 同款，样式 URL 由 `/server/config` 下发 |

### 3.3 三端差异处理原则

- 纯 Dart 逻辑（模型、API client、状态）全平台共享；
- 平台能力（相册扫描、后台任务、本地通知）收敛到 `platform/` 目录，经接口抽象 + 条件导入（`dart.library.html` / `dart.library.io`）隔离；
- **Web 端明确不做**：设备相册自动备份、后台同步（浏览器无此能力），对应 UI 隐藏。

---

## 4. 协议兼容性策略

### 4.1 契约源

唯一契约：`immich/open-api/immich-openapi-specs.json`（3.0.2，OpenAPI 3.0.0）。统计：**173 路径、254 操作、376 Schema**。全部端点清单见 [api-endpoints-raw.md](./api-endpoints-raw.md)。

### 4.2 兼容规则

1. **路径与方法完全一致**，统一挂在 `/api` 前缀下（immich-server 对客户端暴露的就是 `/api/*`；缩略图等媒体流亦同）。
2. **DTO 字段名/大小写/类型逐字段一致**（camelCase；时间为 ISO-8601 毫秒 UTC，如 `2026-07-14T08:00:00.000Z`；UUID 字符串）。
3. **错误信封一致**：`{ "message": ..., "error": "Bad Request", "statusCode": 400 }`。
4. **认证载体一致**（见 §10）：cookie `immich_access_token`、`Authorization: Bearer`、`x-api-key`、`x-immich-session-token`、共享链接 `?key=`/`?slug=`。
5. **状态码语义一致**：如上传去重返回 200+`status:duplicate`、新建 201+`status:created`；批量删除 204。
6. **未实现端点返回 501**（带标准错误信封），绝不 404 —— 客户端可以显式失败而非路由错乱。
7. **版本上报**：`/server/version` 返回 `{major:3, minor:0, patch:2}`，`/server/features` 按真实能力上报（ML 相关一律 `false`），官方 App 据此隐藏智能搜索/人脸入口，实现优雅降级。
8. **弃用端点**（spec 中 17 个 Deprecated 操作，如 `/sync/full-sync`、`/sync/delta-sync`、`/people` 旧接口）：保留路由，按优先级实现或 501。

### 4.3 兼容性验证

- **基线**：immich 仓库自带 e2e 测试（`immich/e2e`，vitest + supertest，直接打 HTTP），改造其 base URL 指向 Domus，作为协议回归套件；
- **真机验证**：官方 immich mobile App（3.0.x）连接 Domus 走通：登录 → 时间线 → 查看大图 → 上传 → 相册 → 删除；
- **CI**：对每个已实现端点做请求/响应 JSON Schema 校验（从 OpenAPI 规范生成校验器）。

---

## 5. 后端模块划分（Rust Workspace）

代码位于 `server/`，7 个 crate 的单向依赖链：

```
domus-server (bin)
 ├─→ domus-api      HTTP/WS 层：路由、DTO、extractor、错误映射、socket.io
 │     └─→ domus-domain   业务层：每个功能域一个 Service（授权决策、编排）
 │           ├─→ domus-db      数据层：sqlx 仓储 + 迁移（SQL 只出现在这里）
 │           ├─→ domus-jobs    队列：PgJobQueue + WorkerPool + job handler
 │           │     └─→ domus-media
 │           └─→ domus-media   媒体：exif/缩略图/转码/存储路径（外部工具封装）
 └─→ domus-common   基础：Config、Error、协议枚举
```

| crate | 职责 | 关键类型（骨架已建） |
|-------|------|---------------------|
| `domus-common` | 配置加载（DOMUS_*/IMMICH_* env）、统一错误、跨层枚举 | `Config`、`Error`（→HTTP 状态码）、`AssetType`、`AssetVisibility` |
| `domus-db` | PG 连接池、`migrations/`、每聚合一个 Repository | `Repositories`（18 个仓储的束）、`entities::*` |
| `domus-domain` | 19 个 Service；授权、事务边界、跨仓储编排 | `Services`、`AuthService`、`AssetMediaService`、`SyncService`… |
| `domus-jobs` | PG 队列（SKIP LOCKED）、worker 池、媒体管线 handler | `PgJobQueue`、`WorkerPool`、`QueueName`、`JobName` |
| `domus-media` | exiftool/ffmpeg/缩略图封装、磁盘布局 | `StorageCore`、`exif::extract`、`thumbnail::generate`、`transcode::*` |
| `domus-api` | axum Router（30 个路由模块，254 操作全部挂载）、DTO、认证 extractor、socket.io | `build_router`、`Auth`/`AdminAuth`、`ApiError` |
| `domus-server` | 入口：装配、按 worker 角色启动 | `main` |

**分层纪律**：HTTP 形状只在 api；SQL 只在 db；业务规则只在 domain；api 不直接触 db。

---

## 6. API 兼容矩阵与优先级

254 个操作按实现优先级分级（P0 = 官方 App 主链路必需；P1 = 常用；P2 = 低频/管理；P3 = 明确降级）：

| Tag | 操作数 | 优先级 | 说明 |
|-----|-------:|--------|------|
| Authentication | 17 | **P0** | login/logout/validate/admin-sign-up/change-password；pin-code、session lock P1 |
| Server | 14 | **P0** | ping/version/features/config/about/storage/media-types（骨架已实现） |
| Assets | 26 | **P0** | 上传、original/thumbnail/playback 流、批量更新/删除、exist/bulk-upload-check、statistics |
| Timeline | 2 | **P0** | buckets + bucket（列式响应） |
| Sync | 4 | **P0** | stream/ack；官方 App 主同步链路（v1 full/delta-sync 为 Deprecated，P2） |
| Users | 16 | **P0** | me/preferences/profile-image；license P2 |
| Albums | 13 | **P1** | CRUD、资产增删、成员共享、统计 |
| Memories | 8 | P1 | 列表/查看/保存 + 每日生成 job |
| Search | 10 | P1 | metadata/random/cities/places/explore/suggestions（SQL 实现）；`/search/smart` 501（P3） |
| Shared links | 9 | P1 | 公开分享页链路 |
| Partners | 5 | P1 | 伙伴共享 + 时间线合并 |
| Trash | 3 | P1 | restore/empty |
| Download | 2 | P1 | zip 打包下载（流式） |
| Sessions / API keys | 6+6 | P1 | 设备管理、第三方接入 |
| Tags / Stacks | 9+7 | P1 | 组织功能 |
| Activities / Notifications | 4+6(+3 admin) | P2 | 社交/通知 |
| Libraries | 8 | P2 | 外部库扫描 |
| Jobs / Queues | 3+5 | P2 | 管理端队列控制（骨架已有聚合状态） |
| System config / metadata | 4+4 | P2 | 管理端配置 |
| Users (admin) | 11 | P2 | 用户管理 |
| Map | 2 | P2 | 地图标记 + 样式反代 |
| Views | 2 | P2 | 文件夹视图 |
| Database backups / Maintenance / Integrity (admin) | 5+9 | P2 | 运维 |
| OAuth | (含在 Auth) | P2 | authorize/callback/link/unlink（PKCE） |
| Duplicates | 4 | **P3** | 依赖向量嵌入；列表返回空集 |
| Faces / People | 4+11 | **P3** | 数据模型与只读接口保留；识别永不产生数据 |
| Plugins / Workflows | 4+7 | **P3** | 3.x 扩展机制，占位 501 |
| Deprecated | 17 | P2/P3 | 逐个评估：官方 App 仍在用的（full-sync/delta-sync 旧版）→P1 |

**骨架现状**：254 操作已全部挂载路由；其中 server(7)/auth(4)/users(5)/albums(6)/timeline(2)/sync(3)/assets(3) 共约 30 个操作有真实（或到 NotImplemented 边界的）实现路径，其余 501。

---

## 7. 数据库设计

### 7.1 对齐策略

**表名/列名与 Immich 3.x 完全一致**（单数表名 + camelCase 列，如 `"user"`、`asset."ownerId"`），目的：

- 存量 Immich 库"原地挂接"成为可能（G5）；
- e2e/工具对 DB 的假设不破坏；
- 减少双向映射心智负担（Rust 侧 entity 用 snake_case 字段 + SQL 列别名）。

迁移管理：`sqlx::migrate!` 内嵌 `server/crates/db/migrations/*.sql`，启动时自动执行（与 immich 行为一致）。

### 7.2 表清单（Immich 3.0.2 全集 → Domus 取舍）

**核心业务表（M1-M3 实现，首个迁移已建骨架）**：

| 域 | 表 | 说明 |
|----|----|----|
| 用户 | `user`、`user_metadata`、`session`、`api_key` | 密码 bcrypt；session.token 存 SHA-256 |
| 资产 | `asset`、`asset_exif`、`asset_file`、`asset_job_status`、`asset_metadata` | `asset_file` 存 preview/thumbnail 等派生文件路径；checksum SHA-1 bytea |
| 相册 | `album`、`album_asset`、`album_user` | |
| 分享 | `shared_link`、`shared_link_asset`、`partner`、`activity` | |
| 组织 | `tag`、`tag_asset`、`tag_closure`、`stack`、`memory`、`memory_asset` | tag_closure 为闭包表（层级查询） |
| 库 | `library` | 外部目录导入 |
| 系统 | `system_metadata`、`notification`、`version_history`、`move_history` | move_history 支撑存储模板迁移回滚 |
| 同步 | `session_sync_checkpoint` + 15 张 `*_audit` 审计表 | 见 §11；audit 由触发器在删除时写入 |
| 地理 | `geodata_places`、`natural_earth_countries` | 反向地理编码（导入 GeoNames 数据集） |

**不建的表（ML/扩展）**：`smart_search`、`face_search`、`person`、`asset_face`、`asset_ocr`、`ocr_search`、`plugin`、`plugin_method`、`workflow`、`workflow_step`、`asset_edit`、`video_stream`、`integrity_report` 及对应 audit 表。`person` 例外——People API 返回空列表需要表存在与否皆可，首期不建，Service 返回空集。

**Domus 自有表**：`job`（PG 队列，见 §8）。Immich 将队列放 Redis，这是内部实现差异，不影响协议。

### 7.3 关键设计点

- **软删除**：`asset.deletedAt` = 回收站；到期由 `TrashEmpty` job 物理删除；
- **触发器**：`updatedAt` 自动更新触发器 + 删除审计触发器（immich 用 `pg_trigger_depth()=0` 防递归，Domus 迁移中复刻）；
- **时间线索引**：`(ownerId, localDateTime)` 复合索引支撑 bucket 聚合；bucket 键为 `date_trunc('month', localDateTime)`；
- **去重唯一索引**：`(ownerId, checksum)` 部分唯一索引（`WHERE libraryId IS NULL`）；
- 全文/模糊搜索：`asset_exif` 上 `pg_trgm` GIN 索引（城市/相机/文件名 LIKE 搜索），替代 ML 智能搜索的兜底。

---

## 8. 后台任务队列设计

### 8.1 决策：PostgreSQL 队列（放弃 Redis/BullMQ）

| 方案 | 评估 |
|------|------|
| Redis + BullMQ 兼容 | ✗ 需要实现 BullMQ 的 Lua 语义；队列非协议边界，兼容无收益；多一个部署组件 |
| **PG `FOR UPDATE SKIP LOCKED`** | ✓ 事务性入队（与业务写同事务）、少组件、吞吐足够（媒体管线瓶颈在 CPU/IO 而非队列）；`LISTEN/NOTIFY` 做低延迟唤醒 |

`job` 表结构见迁移文件；worker 领取：

```sql
UPDATE job SET status='active', "updatedAt"=now(), attempts=attempts+1
WHERE id = (SELECT id FROM job
            WHERE queue=$1 AND status='waiting' AND "runAt"<=now()
            ORDER BY "createdAt" LIMIT 1
            FOR UPDATE SKIP LOCKED)
RETURNING id, name, payload, attempts;
```

失败重试：指数退避（`runAt = now() + interval '30s' * 2^attempts`），超过 `maxAttempts` 置 `failed`。`/jobs`、`/queues` 管理 API 的 `jobCounts`（waiting/active/completed/failed/delayed/paused）由聚合查询提供，保持 Immich 响应形状。

### 8.2 队列清单（对照 Immich `QueueName`）

| 队列（wire 名） | Domus | 并发 | 内容 |
|----------------|-------|-----:|------|
| `metadataExtraction` | ✓ | 5 | exiftool 解析、地理编码、fan-out 下游 |
| `thumbnailGeneration` | ✓ | 3 | preview/thumbnail/thumbhash |
| `videoConversion` | ✓ | 1 | ffmpeg 转码 |
| `storageTemplateMigration` | ✓ | 5 | 存储模板迁移 |
| `migration` | ✓ | 5 | 派生文件路径迁移 |
| `backgroundTask` | ✓ | 2 | 删除、清理等杂项 |
| `search` | ✓ | 2 | （仅元数据索引维护） |
| `sidecar` | ✓ | 2 | XMP sidecar 发现/同步 |
| `library` | ✓ | 2 | 外部库扫描 |
| `notifications` | ✓ | 2 | 邮件/通知 |
| `backupDatabase` | ✓ | 1 | pg_dump 定时备份 |
| `duplicateDetection` | 保留占位 | – | 无嵌入向量，永远空跑 |
| `smartSearch`/`faceDetection`/`facialRecognition`/`ocr` | ✗ 不建 | – | ML，`/queues` 中上报为禁用 |
| `workflow`/`integrityCheck`/`editor` | ✗ 不建 | – | 3.x 扩展，超出首期范围 |

### 8.3 定时任务

tokio 定时器驱动（替代 immich 的 cron 装饰器）：夜间回忆生成（`MemoriesCreate`）、回收站到期清理、过期 session 清理、版本检查、库定时扫描、数据库备份。间隔可经 system config 调整。

---

## 9. 媒体处理管线

### 9.1 上传→就绪流程（与 Immich 逐步对齐）

```
POST /assets (multipart)
  │ 1. 流式落盘到临时文件，同时算 SHA-1
  │    （若带 x-immich-checksum 头，先查重可短路，避免收body）
  │ 2. 查 (ownerId, checksum) 唯一索引
  │      命中 → 200 {id, status:"duplicate"}（不落盘）
  │ 3. move 到 upload/<userId>/<xx>/<yy>/<assetId>.<ext>
  │ 4. INSERT asset 行（visibility/favorite/livePhotoVideoId…）
  │ 5. enqueue MetadataExtraction        → 201 {id, status:"created"}
  ▼
[metadataExtraction] exiftool -json -n
  ├─ 写 asset_exif（尺寸/时间/GPS/相机/镜头/评分…）
  ├─ localDateTime = 拍摄时间 + 时区推导（immich 语义）
  ├─ 反向地理编码（geodata_places 最近邻）→ city/state/country
  ├─ live photo 配对（iOS：MotionPhoto / ContentIdentifier）
  └─ enqueue GeneratePreview
[thumbnailGeneration]
  ├─ preview  : JPEG ≤1440px（EXIF 方向归正）→ thumbs/.../<id>-preview.jpeg
  ├─ thumbnail: WEBP ≤250px                → thumbs/.../<id>-thumbnail.webp
  ├─ thumbhash: 从 thumbnail 计算 → asset.thumbhash
  ├─ 写 asset_file 行、asset_job_status 时间戳
  ├─ WS 推送 on_upload_success(AssetResponseDto)
  └─ 视频 → enqueue VideoConversion
[videoConversion] ffprobe 探测 → 按策略 ffmpeg 转码
  └─ encoded-video/.../<id>.mp4（h264/aac 默认，可配 hevc/vp9/av1、CRF、硬件加速）
[storageTemplateMigration]（若启用模板）
  └─ upload/ → library/<storageLabel|userId>/<模板路径>；写 move_history
```

### 9.2 磁盘布局（与 Immich 相同）

```
<MEDIA_LOCATION>/
├── upload/<userId>/<xx>/<yy>/<uuid>.<ext>       上传原件（模板未启用时的常驻位置）
├── library/<storageLabel|userId>/<template>     存储模板输出（人类可读路径）
├── thumbs/<userId>/<xx>/<yy>/<uuid>-{preview.jpeg,thumbnail.webp}
├── encoded-video/<userId>/<xx>/<yy>/<uuid>.mp4
├── profile/<userId>/<uuid>.<ext>
└── backups/                                     pg_dump 产物
```

`<xx>/<yy>` 为 UUID 前 4 个 hex 的两级扇出目录。**校验点**：目录结构与命名保持与 Immich 一致，使存量磁盘数据可被 Domus 直接服务（G5）。

### 9.3 外部工具依赖

| 工具 | 用途 | 打包 |
|------|------|------|
| exiftool | 元数据提取（RAW/HEIC/视频全覆盖） | Docker 镜像内置 |
| ffmpeg/ffprobe | 探测、转码、视频海报帧 | 内置（可挂载硬件加速设备） |
| libvips（`vipsthumbnail`/FFI） | 缩略图（HEIC/RAW 经 libheif/libraw） | 内置；纯 Rust `image` crate 作为常见格式 fallback |

---

## 10. 认证与授权

### 10.1 凭据载体（wire 兼容，源自 immich `enum.ts`）

| 载体 | 形式 | 用户 |
|------|------|------|
| Cookie | `immich_access_token`（HttpOnly）+ `immich_auth_type` + `immich_is_authenticated` | Web |
| Bearer | `Authorization: Bearer <token>` | 移动端/CLI |
| Header | `x-immich-user-token`、`x-immich-session-token`（旧） | 兼容 |
| API Key | `x-api-key: <key>` | 第三方/CLI |
| 共享链接 | `?key=<hex>`／`?slug=`；header `x-immich-share-key`/`x-immich-share-slug` | 匿名访客 |

### 10.2 机制

- **登录**：bcrypt 校验 → 生成 32 字节随机 token（base64url）→ DB 存 SHA-256(hex) → `LoginResponseDto{accessToken,…}` + Set-Cookie（201）；
- **会话**：`session` 表一行=一设备；`/sessions` 可枚举/吊销；被吊销时 WS 推 `on_session_delete`；
- **API key**：创建时明文只返回一次，存 SHA-256；带 `permissions[]`（immich 3.x 细粒度权限枚举），中间件按 route 声明校验；
- **共享链接**：`key` 为随机 bytea（hex 呈现）；可带密码、过期时间、allowUpload/Download/showExif；命中后 AuthContext 降级为受限身份，仅可访问链接资产集；
- **管理员**：`AdminAuth` extractor（`isAdmin`）；首个注册用户即 admin（`/auth/admin-sign-up`）；
- **OAuth（P2）**：Authorization Code + PKCE，`/oauth/authorize`→`/oauth/callback`，自动建号/关联由 system config 控制。

### 10.3 授权模型

资产可见性：owner ∪ partner（inTimeline）∪ 相册成员 ∪ 共享链接。规则收敛在 domain 层 `access` 检查（对照 immich 的 AccessRepository：`checkOwnerAccess/checkAlbumAccess/checkPartnerAccess/checkSharedLinkAccess`），仓储查询一律带 owner 约束兜底。

---

## 11. 增量同步协议（Sync）

官方移动端主链路（v2 sync）。**这是兼容性最难也最关键的部分。**

### 11.1 端点

| 端点 | 语义 |
|------|------|
| `POST /sync/stream` | 请求体 `{types:[SyncRequestType…], reset?}`；响应 `application/jsonlines+json` 流，每行 `{type: SyncEntityType, data: {...}, ack: "..."}` |
| `POST /sync/ack` | `{acks:[...]}` 持久化检查点（按 session × 实体类型 upsert 到 `session_sync_checkpoint`） |
| `GET /sync/ack` | 列出当前 session 的检查点 |
| `DELETE /sync/ack` | 删除指定类型检查点（触发全量重拉） |
| `POST /sync/full-sync`、`/sync/delta-sync` | v1 旧协议（Deprecated），旧版 App 使用 |

### 11.2 机制

- **检查点即游标**：ack 编码"实体流内单调递增位置"（immich 基于 `updateId`/updatedAt 序）；服务端每类实体按序产出：先补历史（backfill 类型），再跟增量；
- **删除事件**来自 `*_audit` 审计表（触发器在 DELETE 时记录 id + 时间），使离线客户端能追平删除；
- **26 种 SyncRequestType**（AlbumsV2、AssetsV2、AssetExifsV1、PartnerAssetsV2、MemoriesV1、AuthUsersV1…，完整清单见骨架 `sync.rs`），Domus 全部接受；OCR/edits 等无数据类型产出空流即可——协议合法；
- 流式响应用 axum `Body::from_stream`，分批查询（每批 ~1000 行）背压推送。

### 11.3 实现阶段

M1 先实现 `stream` 返回空流 + ack 存取（官方 App 可登录并空库同步不报错）；M3 起按 AssetsV2 → AssetExifsV1 → AlbumsV2/AlbumAssets → Partners → Memories 顺序点亮。

---

## 12. WebSocket 实时通道

- 协议：**socket.io（engine.io v4）**，path `/api/socket.io`，`transports:['websocket']`（immich 明确只用 websocket transport）；
- 握手认证：复用 §10 全部载体（cookie/bearer/api-key）；成功后加入 `user:<id>` 房间；
- 服务端→客户端事件（与 immich `ClientEventMap` 一致）：

  `on_upload_success(AssetResponseDto)`、`on_asset_trash(ids)`、`on_asset_delete(id)`、`on_asset_restore(ids)`、`on_asset_update(dto)`、`on_asset_hidden(id)`、`on_asset_stack_update`、`on_user_delete(id)`、`on_server_version`、`on_config_update`、`on_new_release`、`on_notification(dto)`、`on_session_delete(id)`、`AssetUploadReadyV2{asset,exif}`、`AppRestartV1`；（`on_person_thumbnail` 不会发生——无人脸管线）
- 事件源：domain 层发领域事件 → api 层订阅并 emit（骨架里 `websocket::emit_to_user`）。

---

## 13. Flutter 客户端架构

### 13.1 目录结构（feature-first，骨架已建）

```
app/lib/
├── main.dart / app.dart          入口、MaterialApp.router
├── core/
│   ├── api/api_client.dart       dio 封装：baseUrl=<server>/api、Bearer 注入、媒体 URL 构造
│   ├── router/app_router.dart    go_router：登录守卫 + StatefulShellRoute 四分支
│   ├── storage/app_settings.dart 服务器地址/token 持久化（→ 后续 drift）
│   └── theme/app_theme.dart      Material 3 深浅色
├── models/                        Immich DTO 映射（Asset/Album/User/TimeBucket）
└── features/<feature>/
    ├── data/        Repository（REST 调用）
    ├── application/ Riverpod Provider（状态/用例）
    ├── presentation/ 页面
    └── widgets/     组件
```

已建 feature：`auth`（登录全链路可用）、`shell`（自适应导航：<600dp 底栏，≥600dp 侧栏）、`timeline`（bucket 懒加载网格 + 列式解包）、`albums`（列表/详情）、`asset_viewer`（预览+缩放）、`search`/`memories`/`backup`/`settings`（占位）。

### 13.2 数据流与离线（完整版设计）

```
UI ← watch ─ Provider ← Stream ─ drift(SQLite) ←─ SyncEngine ← /sync/stream
                                      ▲                            │ acks
                                      └── UploadEngine ─────→ POST /assets
```

- **本地库为唯一 UI 数据源**（immich mobile 同构）：时间线/相册从 drift 查询渲染，网络层只负责把服务端状态同步进本地库 —— 移动端离线可用；
- **SyncEngine**：前台启动即拉 `/sync/stream`，按行 upsert 本地表，批量 ack；WS 事件触发即时增量拉取；
- **UploadEngine（仅移动端）**：photo_manager 枚举设备相册 → 与服务端 `bulk-upload-check`（checksum）对账 → background_downloader 队列上传（Wi-Fi/充电策略）→ iOS BGTask / Android WorkManager 背景续传；
- **Web 特化**：drift → wasm(IndexedDB) 后端只做轻缓存；备份页隐藏自动备份，只留手动上传；大图直接流式 `<img>`/video 元素（`Authorization` 经 cookie 由服务器同源下发，或走 blob fetch）。

### 13.3 页面清单（完整版）

登录/服务器选择、时间线（滚动条月份刮擦、多选操作栏）、大图查看器（缩放/滑动/视频/实况照片/EXIF 抽屉/收藏/删除）、相册（列表/详情/成员/评论点赞）、搜索（元数据过滤器+建议）、回忆（卡片轮播）、地图、分享链接管理与公开页、伙伴、回收站、归档、标签、Stack、备份中心、设置（含管理员：用户/队列/系统配置/存储模板）。

---

## 14. 部署架构

### 14.1 docker-compose（目标形态）

```yaml
services:
  domus-server:
    image: domus/server            # 多阶段构建：cargo → debian-slim + exiftool/ffmpeg/libvips
    volumes: [ "${UPLOAD_LOCATION}:/data" ]
    ports: [ "2283:2283" ]
    env_file: .env                  # 兼容 immich 的 .env（DB_*, UPLOAD_LOCATION…）
    depends_on: [ database ]
  # Flutter web 产物直接内嵌进 domus-server 静态服务，无独立 web 容器
  database:
    image: postgres:16              # 无需向量扩展镜像
    volumes: [ "${DB_DATA_LOCATION}:/var/lib/postgresql/data" ]
```

对比 immich：少 `immich-machine-learning`、少 `redis`、PG 用官方镜像。端口保持 2283。

### 14.2 从 Immich 迁移路径

1. **蓝绿并行**（推荐首选）：Domus 指向 Immich 的 PG 库副本 + 同一媒体目录（只读验证）→ 校验读路径 → 切写；
2. 表结构对齐使 schema 差异收敛为"缺 ML 表/多 job 表"，迁移脚本只增不改；
3. 密码（bcrypt）、session（SHA-256）、磁盘布局均兼容，用户无感。

---

## 15. 非功能性需求

| 维度 | 目标 |
|------|------|
| 性能 | 时间线 bucket 查询 P95 < 100ms @ 50 万资产；缩略图首字节 < 30ms（sendfile/Range）；上传吞吐受磁盘限制而非服务 |
| 内存 | 空载 RSS < 50MB（对比 immich node ~400MB+） |
| 并发媒体处理 | 队列并发可配；转码默认 1 并发防止 CPU 打满 API |
| 可观测性 | tracing 结构化日志；`/api/server/ping` 健康检查；Prometheus exporter（后期，对齐 immich 8081/8082 telemetry 口） |
| 安全 | token 只存哈希；上传路径穿越防护；共享链接密码 bcrypt；CORS 默认放开（同 immich）可收紧 |
| 测试 | 单测（domain）+ sqlx 集成测（testcontainers）+ immich e2e 改造回归 |

---

## 16. 任务拆解与里程碑

### M0 —— 设计与骨架 ✅（本次交付）

- [x] 系统设计文档（本文档）+ 全量端点清单附录
- [x] Rust workspace 7 crate；254 操作路由全挂载；`cargo check` 通过
- [x] 认证纵向切片（login/logout/sign-up/validate + session 仓储 + bcrypt/SHA-256）
- [x] `/server/*` 7 端点真实实现；错误信封；socket.io 层挂载
- [x] 初始迁移（immich 对齐的核心表 + job 表）
- [x] Flutter 骨架：登录/时间线/相册/查看器/搜索/回忆/备份/设置 + 自适应导航；`flutter analyze` 0 issue

### M1 —— 资产纵向切片（可看图）（~2 周）

1. multipart 上传落盘 + SHA-1 去重 + `x-immich-checksum` 短路
2. `AssetRepository` 完整 CRUD；exiftool 集成；`asset_exif` 写入
3. 缩略图管线（libvips preview/thumbnail + thumbhash）
4. original/thumbnail/playback 流式下载（Range）
5. timeline buckets/bucket 真实查询（列式响应）
6. worker 池 + PG 队列可用（enqueue/claim/complete/fail/counts）
7. e2e：assets/timeline 子集打通
8. Flutter：时间线接真数据、查看器 preview、手动上传

### M2 —— 相册与组织（~2 周）

album CRUD/成员/资产、tag、stack、memory 生成 job、trash、favorites/archive 批量操作、`/users` 全量、profile image；Flutter 对应页面。

### M3 —— 同步与实时（~3 周）

审计触发器、checkpoint 游标、AssetsV2/AssetExifsV1/AlbumsV2… 流实现；socket.io 握手认证 + 全事件表；官方 immich App 连 Domus 走通主链路（**关键验收**）；Flutter drift 本地库 + SyncEngine。

### M4 —— 分享与协作（~2 周）

shared link（含公开页匿名鉴权）、partner、activity、notification、download zip；Flutter 分享页。

### M5 —— 视频与媒体增强（~2 周）

ffmpeg 转码策略（分辨率/CRF/硬件加速配置）、live photo 配对、sidecar、存储模板迁移 + move_history、地理编码数据导入。

### M6 —— 管理面（~2 周）

system config 全量（含 config file 模式）、admin users、jobs/queues 控制、database backup、maintenance/integrity 子集、OAuth。

### M7 —— 移动端备份与三端打磨（~3 周）

photo_manager 对账备份、background_downloader 后台上传、iOS/Android 原生任务、地图页、性能优化（时间线虚拟化/滚动刮擦）、i18n。

### M8 —— 迁移与发布（~2 周）

immich e2e 全量回归、蓝绿迁移演练、Docker 镜像/CI/CD、文档、负载测试。

> 关键依赖链：M1 → M3 → M7（同步是移动端体验的地基）；M2/M4/M5/M6 可与 M3 部分并行。

---

## 17. 风险与缓解

| # | 风险 | 影响 | 缓解 |
|---|------|------|------|
| R1 | 官方 App 对未文档化行为的依赖（响应字段顺序无关，但空值/缺省语义有关） | 高 | 以 immich e2e 为回归基线；抓包对比真实 immich 与 Domus 响应 diff |
| R2 | socket.io 协议细节（ack、二进制、房间语义）与 socketioxide 的差异 | 中 | M3 早期用官方 App 真机验证握手；退路：事件全部广播（App 侧按 userId 过滤） |
| R3 | HEIC/RAW 解码覆盖不足 | 中 | libvips(+libheif/libraw) 与 immich 同源；对不支持格式回退"仅存原件 + exif 缩略图" |
| R4 | sync 协议游标语义还原偏差（backfill 顺序、水位） | 高 | 逐实体类型对照 immich `sync.service.ts` 单测；App 端以"全量重置同步"作为兜底 |
| R5 | 无 ML 导致官方 App 某些入口异常 | 低 | `/server/features` 正确上报即被官方 App 优雅隐藏（immich 自身就支持禁用 ML 部署） |
| R6 | Flutter Web 大库性能（万级缩略图滚动） | 中 | 列式 bucket + 虚拟化网格 + CanvasKit；必要时 Web 端降级分页 |
| R7 | PG 队列在超大导入时的争抢 | 低 | SKIP LOCKED 天然分散；批量 enqueue；必要时按队列分表 |

---

## 18. 附录

### 18.1 全量 API 端点清单

见 [api-endpoints-raw.md](./api-endpoints-raw.md)（254 操作，按 tag 分组，标注 Deprecated），从 `immich-openapi-specs.json` 生成。

### 18.2 骨架代码地图

| 路径 | 内容 |
|------|------|
| `server/Cargo.toml` | workspace + 依赖版本单一来源 |
| `server/crates/common/src/{config,error,types}.rs` | 配置/错误/协议枚举 |
| `server/crates/db/migrations/0001_init.sql` | 初始迁移（immich 对齐核心表 + job 表） |
| `server/crates/db/src/repositories/` | 18 个仓储（user/session 有真实 SQL，其余签名 stub） |
| `server/crates/domain/src/services/` | 19 个服务（auth 完整；server 完整；其余到仓储边界） |
| `server/crates/jobs/src/` | 队列/worker/媒体管线 handler 骨架 |
| `server/crates/media/src/` | exif/thumbnail/transcode/storage 封装 |
| `server/crates/api/src/routes/` | 30 个路由模块，254 操作全挂载 |
| `app/lib/` | Flutter 骨架（结构见 §13.1） |

### 18.3 与 Immich 的实现差异一览（协议不受影响）

| 方面 | Immich | Domus |
|------|--------|-------|
| 服务端语言/框架 | TypeScript/NestJS + Kysely | Rust/axum + sqlx |
| 队列 | BullMQ + Redis(valkey) | PostgreSQL SKIP LOCKED |
| PG 镜像 | 定制（VectorChord/pgvecto.rs） | 官方 postgres |
| Web 前端 | SvelteKit（独立产物） | Flutter Web（server 静态托管） |
| ML | Python 服务（CLIP/人脸/OCR） | 无；features 上报关闭 |
| OpenAPI | 服务端生成规范 | 规范作为外部契约输入 |
