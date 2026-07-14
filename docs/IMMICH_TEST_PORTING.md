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

## Rules

- Prefer exact Immich API/DTO/database names and casing.
- Keep Domus-specific behavior only when it is required by the Rust/Flutter
  architecture or explicitly better for this rewrite.
- Keep ML/AI/OCR/face/CLIP tests pending until those modules are in scope.
