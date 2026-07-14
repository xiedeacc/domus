# Immich Test Porting

Source tree: `/root/src/software/immich`, branch `deploy_3.0.2`.

The goal is to keep Domus wire-compatible with native Immich while translating
Immich's TypeScript/Dart tests into Rust/Flutter tests. ML/AI tests are
intentionally excluded for now.

## Inventory

Current non-ML Immich test inventory:

| Area | Source tests | Domus target |
|---|---:|---|
| `server/src/controllers` | 26 | Rust API route/integration tests |
| `server/src/services` | 45 | Rust domain service tests |
| `server/src/utils` | 9 | Rust unit tests |
| `server/src/dtos` | 2 | Rust DTO/validation tests |
| `server/src/repositories` | 6 | Rust repository/database tests |
| `server/src/cores` | 1 | Rust storage tests |
| `server/src/maintenance` | 1 | Rust job/maintenance tests |
| `server/src/validation.spec.ts` | 1 | Rust API validation tests |
| `mobile/test` + `mobile/integration_test` | 9+ | Flutter unit/integration tests |
| `machine-learning` | skipped | Out of scope |

## Ported In This Batch

| Immich source | Domus test coverage |
|---|---|
| `server/src/utils/mime-types.spec.ts` | `server/crates/domain/src/services/asset_media.rs` extension-to-asset-type compatibility |
| `server/src/dtos/user.dto.spec.ts` | `server/crates/domain/src/services/user.rs` email validation compatibility |
| OAuth service/controller behavior | `server/crates/api/src/routes/oauth.rs` OAuth profile/id-token parsing |
| System config defaults | `server/crates/domain/src/services/system_config.rs` OAuth and storage-template default shape |
| Mobile backup settings behavior | `app/test/backup_settings_test.dart` persisted auto-backup and selected-album settings |
| `server/src/utils/misc.spec.ts` | `server/crates/common/src/utils.rs` deep key listing, deep unset, glob-to-SQL pattern |
| `server/src/utils/request.spec.ts` | `server/crates/common/src/utils.rs` Immich mobile user-agent version parsing |
| `server/src/cores/storage.core.spec.ts` | `server/crates/media/src/storage.rs` media-root path containment checks |
| `server/src/dtos/album-response.dto.spec.ts` | `server/crates/api/src/dto.rs` album `startDate`/`endDate` mapping |
| DTO camelCase contract | `server/crates/api/src/dto.rs` album/user/login response field casing |
| `server/src/utils/date.spec.ts` | `server/crates/common/src/utils.rs` date and datetime string formatting |
| `server/src/utils/duplicate.spec.ts` | `server/crates/common/src/utils.rs` duplicate keep-candidate selection |
| `server/src/controllers/auth.controller.spec.ts` | `server/crates/api/src/routes/auth.rs` email normalization and auth cookie contract |
| Auth credential carriers | `server/crates/api/src/extractors.rs` bearer/cookie/API-key/legacy/shared-link extraction |
| `server/src/controllers/api-key.controller.spec.ts` | `server/crates/api/src/routes/api_keys.rs` API key default name and response casing |
| `server/src/controllers/server.controller.spec.ts` | `server/crates/api/src/routes/server.rs` authenticated license placeholder, ping/media-types shape |
| `server/src/services/server.service.spec.ts` | `server/crates/domain/src/services/server.rs` version/features shape with ML features disabled |
| `server/src/services/version.service.spec.ts` | `server/crates/domain/src/services/server.rs` version response shape |
| `server/src/controllers/tag.controller.spec.ts` | `server/crates/api/src/routes/tags.rs` tag response shape, optional field omission, null `parentId` input |
| `server/src/services/tag.service.spec.ts` | `server/crates/domain/src/services/tag.rs` hierarchical tag upsert and slash normalization |
| `server/src/controllers/partner.controller.spec.ts` | `server/crates/domain/src/services/partner.rs` required `direction` enum parsing |
| `server/src/services/partner.service.spec.ts` | `server/crates/api/src/dto.rs` partner response maps to shared user + `inTimeline`; duplicate/missing partner checks in service |
| `server/src/controllers/shared-link.controller.spec.ts` | `server/crates/api/src/routes/shared_links.rs` nullable `expiresAt` create payload and shared-link mutation routes |
| `server/src/services/shared-link.service.spec.ts` | `server/crates/domain/src/services/shared_link.rs` create validation, showMetadata/download coupling, base64url key decoding |
| `server/src/dtos/shared-link.dto.ts` | `server/crates/api/src/dto.rs` shared-link response key/type/showMetadata field compatibility |
| `server/src/services/stack.service.spec.ts` | `server/crates/domain/src/services/stack.rs` min-2 create validation; `server/crates/api/src/dto.rs` primary-first response shape |
| `server/src/dtos/stack.dto.ts` | `server/crates/api/src/dto.rs` stack response omits non-Immich `ownerId` |
| `server/src/controllers/memory.controller.spec.ts` | `server/crates/domain/src/services/memory.rs` on-this-day data and non-empty update validation |
| `server/src/dtos/memory.dto.ts` | `server/crates/api/src/dto.rs` memory response optional `deletedAt` and type/data shape |
| `server/src/controllers/timeline.controller.spec.ts` | `server/crates/api/src/routes/timeline.rs` bbox query parsing and invalid bbox errors |
| `server/src/services/timeline.service.spec.ts` | `server/crates/domain/src/services/timeline.rs` withPartners incompatible filter validation |
| `server/src/services/trash.service.spec.ts` | `server/crates/api/src/routes/trash.rs` trash response `{count}` shape; restore-assets returns affected count |
| `server/src/controllers/search.controller.spec.ts` | `server/crates/domain/src/services/search.rs` metadata/random filter validation and required suggestion `type` enum |
| `server/src/services/search.service.spec.ts` | `server/crates/db/src/repositories/search.rs` non-ML suggestion kinds for country/state/city/camera/lens |
| `server/src/services/map.service.spec.ts` | `server/crates/api/src/routes/map.rs` reverse-geocode coordinate validation/array response; `server/crates/api/src/dto.rs` marker response shape |
| `server/src/utils/editor.spec.ts` | `server/crates/common/src/utils.rs` non-ML bounding-box overlap geometry |

## Rules

- Prefer exact Immich API/DTO/database names and casing.
- Keep Domus-specific behavior only when it is required by the Rust/Flutter
  architecture or explicitly better for this rewrite.
- Keep ML/AI/OCR/face/CLIP tests pending until those modules are in scope.
