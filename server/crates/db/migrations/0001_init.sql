-- Domus initial schema.
-- Table and column names mirror Immich 3.x (singular table names, camelCase
-- columns) so tooling and, eventually, existing databases stay compatible.
-- This is the skeleton subset: core identity, assets, albums, sharing and
-- sync. ML-related tables (smart_search, face_search, asset_face, ocr_*) are
-- intentionally omitted.

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- === users & auth =========================================================

CREATE TABLE "user" (
    "id"                   uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    "email"                varchar NOT NULL UNIQUE,
    "password"             varchar NOT NULL DEFAULT '',
    "name"                 varchar NOT NULL DEFAULT '',
    "isAdmin"              boolean NOT NULL DEFAULT false,
    "avatarColor"          varchar,
    "profileImagePath"     varchar NOT NULL DEFAULT '',
    "storageLabel"         varchar,
    "oauthId"              varchar NOT NULL DEFAULT '',
    "quotaSizeInBytes"     bigint,
    "quotaUsageInBytes"    bigint NOT NULL DEFAULT 0,
    "shouldChangePassword" boolean NOT NULL DEFAULT true,
    "createdAt"            timestamptz NOT NULL DEFAULT now(),
    "updatedAt"            timestamptz NOT NULL DEFAULT now(),
    "deletedAt"            timestamptz,
    "profileChangedAt"     timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE "session" (
    "id"         uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    "token"      bytea NOT NULL,                          -- sha256 digest of bearer token
    "userId"     uuid NOT NULL REFERENCES "user" ("id") ON DELETE CASCADE,
    "deviceType" varchar NOT NULL DEFAULT '',
    "deviceOS"   varchar NOT NULL DEFAULT '',
    "expiresAt"  timestamptz,
    "createdAt"  timestamptz NOT NULL DEFAULT now(),
    "updatedAt"  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX "IDX_session_token" ON "session" ("token");

CREATE TABLE "api_key" (
    "id"          uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    "name"        varchar NOT NULL,
    "key"         bytea NOT NULL,                         -- sha256 digest of raw key
    "userId"      uuid NOT NULL REFERENCES "user" ("id") ON DELETE CASCADE,
    "permissions" varchar[] NOT NULL DEFAULT '{}',
    "createdAt"   timestamptz NOT NULL DEFAULT now(),
    "updatedAt"   timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX "IDX_api_key_key" ON "api_key" ("key");

-- === libraries & assets ===================================================

CREATE TABLE "library" (
    "id"                uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    "ownerId"           uuid NOT NULL REFERENCES "user" ("id") ON DELETE CASCADE,
    "name"              varchar NOT NULL,
    "importPaths"       varchar[] NOT NULL DEFAULT '{}',
    "exclusionPatterns" varchar[] NOT NULL DEFAULT '{}',
    "createdAt"         timestamptz NOT NULL DEFAULT now(),
    "updatedAt"         timestamptz NOT NULL DEFAULT now(),
    "refreshedAt"       timestamptz
);

CREATE TABLE "stack" (
    "id"             uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    "ownerId"        uuid NOT NULL REFERENCES "user" ("id") ON DELETE CASCADE,
    "primaryAssetId" uuid NOT NULL
);

CREATE TABLE "asset" (
    "id"               uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    "ownerId"          uuid NOT NULL REFERENCES "user" ("id") ON DELETE CASCADE,
    "libraryId"        uuid REFERENCES "library" ("id") ON DELETE CASCADE,
    "deviceAssetId"    varchar NOT NULL DEFAULT '',
    "deviceId"         varchar NOT NULL DEFAULT '',
    "type"             varchar NOT NULL,                 -- IMAGE | VIDEO | AUDIO | OTHER
    "originalPath"     varchar NOT NULL,
    "originalFileName" varchar NOT NULL,
    "checksum"         bytea NOT NULL,                   -- sha1
    "checksumAlgorithm" varchar NOT NULL DEFAULT 'sha1',
    "visibility"       varchar NOT NULL DEFAULT 'timeline',
    "isFavorite"       boolean NOT NULL DEFAULT false,
    "isOffline"        boolean NOT NULL DEFAULT false,
    "isExternal"       boolean NOT NULL DEFAULT false,
    "livePhotoVideoId" uuid REFERENCES "asset" ("id") ON DELETE SET NULL,
    "stackId"          uuid REFERENCES "stack" ("id") ON DELETE SET NULL,
    "duration"         integer,
    "thumbhash"        bytea,
    "fileCreatedAt"    timestamptz NOT NULL,
    "fileModifiedAt"   timestamptz NOT NULL,
    "localDateTime"    timestamptz NOT NULL,
    "createdAt"        timestamptz NOT NULL DEFAULT now(),
    "updatedAt"        timestamptz NOT NULL DEFAULT now(),
    "deletedAt"        timestamptz
);
CREATE UNIQUE INDEX "UQ_asset_owner_checksum" ON "asset" ("ownerId", "checksum") WHERE "libraryId" IS NULL;
CREATE INDEX "IDX_asset_owner_localDateTime" ON "asset" ("ownerId", "localDateTime");

CREATE TABLE "asset_exif" (
    "assetId"          uuid PRIMARY KEY REFERENCES "asset" ("id") ON DELETE CASCADE,
    "make"             varchar,
    "model"            varchar,
    "exifImageWidth"   integer,
    "exifImageHeight"  integer,
    "fileSizeInByte"   bigint,
    "orientation"      varchar,
    "dateTimeOriginal" timestamptz,
    "modifyDate"       timestamptz,
    "timeZone"         varchar,
    "latitude"         double precision,
    "longitude"        double precision,
    "city"             varchar,
    "state"            varchar,
    "country"          varchar,
    "description"      text NOT NULL DEFAULT '',
    "fNumber"          double precision,
    "focalLength"      double precision,
    "iso"              integer,
    "exposureTime"     varchar,
    "lensModel"        varchar,
    "projectionType"   varchar,
    "rating"           integer,
    "fps"              double precision
);

-- Generated derivative files (preview/thumbnail/encoded video paths).
CREATE TABLE "asset_file" (
    "id"        uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    "assetId"   uuid NOT NULL REFERENCES "asset" ("id") ON DELETE CASCADE,
    "type"      varchar NOT NULL,                        -- preview | thumbnail | fullsize
    "path"      varchar NOT NULL,
    "createdAt" timestamptz NOT NULL DEFAULT now(),
    "updatedAt" timestamptz NOT NULL DEFAULT now(),
    UNIQUE ("assetId", "type")
);

-- Pipeline progress per asset (drives re-runs and admin visibility).
CREATE TABLE "asset_job_status" (
    "assetId"           uuid PRIMARY KEY REFERENCES "asset" ("id") ON DELETE CASCADE,
    "metadataExtractedAt" timestamptz,
    "previewAt"         timestamptz,
    "thumbnailAt"       timestamptz
);

-- === albums & sharing =====================================================

CREATE TABLE "album" (
    "id"                     uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    "ownerId"                uuid NOT NULL REFERENCES "user" ("id") ON DELETE CASCADE,
    "albumName"              varchar NOT NULL DEFAULT '',
    "description"            text NOT NULL DEFAULT '',
    "albumThumbnailAssetId"  uuid REFERENCES "asset" ("id") ON DELETE SET NULL,
    "isActivityEnabled"      boolean NOT NULL DEFAULT true,
    "order"                  varchar NOT NULL DEFAULT 'desc',
    "createdAt"              timestamptz NOT NULL DEFAULT now(),
    "updatedAt"              timestamptz NOT NULL DEFAULT now(),
    "deletedAt"              timestamptz
);

CREATE TABLE "album_asset" (
    "albumId"   uuid NOT NULL REFERENCES "album" ("id") ON DELETE CASCADE,
    "assetId"   uuid NOT NULL REFERENCES "asset" ("id") ON DELETE CASCADE,
    "createdAt" timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY ("albumId", "assetId")
);

CREATE TABLE "album_user" (
    "albumId" uuid NOT NULL REFERENCES "album" ("id") ON DELETE CASCADE,
    "userId"  uuid NOT NULL REFERENCES "user" ("id") ON DELETE CASCADE,
    "role"    varchar NOT NULL DEFAULT 'editor',         -- editor | viewer
    PRIMARY KEY ("albumId", "userId")
);

CREATE TABLE "shared_link" (
    "id"            uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    "userId"        uuid NOT NULL REFERENCES "user" ("id") ON DELETE CASCADE,
    "key"           bytea NOT NULL UNIQUE,
    "slug"          varchar UNIQUE,
    "type"          varchar NOT NULL,                    -- ALBUM | INDIVIDUAL
    "albumId"       uuid REFERENCES "album" ("id") ON DELETE CASCADE,
    "description"   varchar,
    "password"      varchar,
    "allowUpload"   boolean NOT NULL DEFAULT false,
    "allowDownload" boolean NOT NULL DEFAULT true,
    "showExif"      boolean NOT NULL DEFAULT true,
    "expiresAt"     timestamptz,
    "createdAt"     timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE "shared_link_asset" (
    "sharedLinkId" uuid NOT NULL REFERENCES "shared_link" ("id") ON DELETE CASCADE,
    "assetId"      uuid NOT NULL REFERENCES "asset" ("id") ON DELETE CASCADE,
    PRIMARY KEY ("sharedLinkId", "assetId")
);

CREATE TABLE "partner" (
    "sharedById"   uuid NOT NULL REFERENCES "user" ("id") ON DELETE CASCADE,
    "sharedWithId" uuid NOT NULL REFERENCES "user" ("id") ON DELETE CASCADE,
    "inTimeline"   boolean NOT NULL DEFAULT false,
    "createdAt"    timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY ("sharedById", "sharedWithId")
);

CREATE TABLE "activity" (
    "id"        uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    "userId"    uuid NOT NULL REFERENCES "user" ("id") ON DELETE CASCADE,
    "albumId"   uuid NOT NULL REFERENCES "album" ("id") ON DELETE CASCADE,
    "assetId"   uuid REFERENCES "asset" ("id") ON DELETE CASCADE,
    "comment"   text,
    "isLiked"   boolean NOT NULL DEFAULT false,
    "createdAt" timestamptz NOT NULL DEFAULT now(),
    "updatedAt" timestamptz NOT NULL DEFAULT now()
);

-- === organisation =========================================================

CREATE TABLE "tag" (
    "id"        uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    "userId"    uuid NOT NULL REFERENCES "user" ("id") ON DELETE CASCADE,
    "value"     varchar NOT NULL,                        -- full path "a/b/c"
    "color"     varchar,
    "parentId"  uuid REFERENCES "tag" ("id") ON DELETE CASCADE,
    "createdAt" timestamptz NOT NULL DEFAULT now(),
    "updatedAt" timestamptz NOT NULL DEFAULT now(),
    UNIQUE ("userId", "value")
);

CREATE TABLE "tag_asset" (
    "tagId"   uuid NOT NULL REFERENCES "tag" ("id") ON DELETE CASCADE,
    "assetId" uuid NOT NULL REFERENCES "asset" ("id") ON DELETE CASCADE,
    PRIMARY KEY ("tagId", "assetId")
);

CREATE TABLE "memory" (
    "id"        uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    "ownerId"   uuid NOT NULL REFERENCES "user" ("id") ON DELETE CASCADE,
    "type"      varchar NOT NULL DEFAULT 'on_this_day',
    "data"      jsonb NOT NULL,
    "memoryAt"  timestamptz NOT NULL,
    "isSaved"   boolean NOT NULL DEFAULT false,
    "seenAt"    timestamptz,
    "createdAt" timestamptz NOT NULL DEFAULT now(),
    "updatedAt" timestamptz NOT NULL DEFAULT now(),
    "deletedAt" timestamptz
);

CREATE TABLE "memory_asset" (
    "memoryId" uuid NOT NULL REFERENCES "memory" ("id") ON DELETE CASCADE,
    "assetId"  uuid NOT NULL REFERENCES "asset" ("id") ON DELETE CASCADE,
    PRIMARY KEY ("memoryId", "assetId")
);

CREATE TABLE "notification" (
    "id"          uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    "userId"      uuid NOT NULL REFERENCES "user" ("id") ON DELETE CASCADE,
    "level"       varchar NOT NULL DEFAULT 'info',
    "type"        varchar NOT NULL,
    "title"       varchar NOT NULL,
    "description" text,
    "data"        jsonb,
    "readAt"      timestamptz,
    "createdAt"   timestamptz NOT NULL DEFAULT now()
);

-- === sync & system ========================================================

CREATE TABLE "session_sync_checkpoint" (
    "sessionId" uuid NOT NULL REFERENCES "session" ("id") ON DELETE CASCADE,
    "type"      varchar NOT NULL,                        -- SyncEntityType
    "ack"       varchar NOT NULL,
    "createdAt" timestamptz NOT NULL DEFAULT now(),
    "updatedAt" timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY ("sessionId", "type")
);

-- Audit tables reconstruct deletes for the sync stream.
CREATE TABLE "asset_audit" (
    "id"        uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    "assetId"   uuid NOT NULL,
    "ownerId"   uuid NOT NULL,
    "deletedAt" timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX "IDX_asset_audit_deletedAt" ON "asset_audit" ("deletedAt");

CREATE TABLE "system_metadata" (
    "key"   varchar PRIMARY KEY,
    "value" jsonb NOT NULL
);

CREATE TABLE "version_history" (
    "id"        uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    "createdAt" timestamptz NOT NULL DEFAULT now(),
    "version"   varchar NOT NULL
);

-- === job queue (Domus-specific; Immich keeps this in Redis/BullMQ) ========

CREATE TABLE "job" (
    "id"          uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    "queue"       varchar NOT NULL,
    "name"        varchar NOT NULL,
    "payload"     jsonb NOT NULL DEFAULT '{}',
    "status"      varchar NOT NULL DEFAULT 'waiting',    -- waiting|active|completed|failed|delayed|paused
    "attempts"    integer NOT NULL DEFAULT 0,
    "maxAttempts" integer NOT NULL DEFAULT 3,
    "error"       text,
    "runAt"       timestamptz NOT NULL DEFAULT now(),
    "createdAt"   timestamptz NOT NULL DEFAULT now(),
    "updatedAt"   timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX "IDX_job_claim" ON "job" ("queue", "status", "runAt");
