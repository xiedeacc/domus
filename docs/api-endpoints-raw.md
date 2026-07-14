### API keys (6)

| Method | Path | OperationId |
|---|---|---|
| `GET` | `/api-keys` | getApiKeys |
| `POST` | `/api-keys` | createApiKey |
| `GET` | `/api-keys/me` | getMyApiKey |
| `DELETE` | `/api-keys/{id}` | deleteApiKey |
| `GET` | `/api-keys/{id}` | getApiKey |
| `PUT` | `/api-keys/{id}` | updateApiKey [DEPRECATED] |

### Activities (4)

| Method | Path | OperationId |
|---|---|---|
| `GET` | `/activities` | getActivities |
| `POST` | `/activities` | createActivity |
| `GET` | `/activities/statistics` | getActivityStatistics |
| `DELETE` | `/activities/{id}` | deleteActivity |

### Albums (13)

| Method | Path | OperationId |
|---|---|---|
| `GET` | `/albums` | getAllAlbums |
| `POST` | `/albums` | createAlbum |
| `PUT` | `/albums/assets` | addAssetsToAlbums |
| `GET` | `/albums/statistics` | getAlbumStatistics |
| `DELETE` | `/albums/{id}` | deleteAlbum |
| `GET` | `/albums/{id}` | getAlbumInfo |
| `PATCH` | `/albums/{id}` | updateAlbumInfo |
| `DELETE` | `/albums/{id}/assets` | removeAssetFromAlbum |
| `PUT` | `/albums/{id}/assets` | addAssetsToAlbum |
| `GET` | `/albums/{id}/map-markers` | getAlbumMapMarkers |
| `DELETE` | `/albums/{id}/user/{userId}` | removeUserFromAlbum |
| `PUT` | `/albums/{id}/user/{userId}` | updateAlbumUser |
| `PUT` | `/albums/{id}/users` | addUsersToAlbum |

### Assets (26)

| Method | Path | OperationId |
|---|---|---|
| `DELETE` | `/assets` | deleteAssets |
| `POST` | `/assets` | uploadAsset |
| `PUT` | `/assets` | updateAssets [DEPRECATED] |
| `POST` | `/assets/bulk-upload-check` | checkBulkUpload |
| `PUT` | `/assets/copy` | copyAsset |
| `POST` | `/assets/jobs` | runAssetJobs |
| `DELETE` | `/assets/metadata` | deleteBulkAssetMetadata |
| `PUT` | `/assets/metadata` | updateBulkAssetMetadata |
| `GET` | `/assets/statistics` | getAssetStatistics |
| `GET` | `/assets/{id}` | getAssetInfo |
| `PUT` | `/assets/{id}` | updateAsset [DEPRECATED] |
| `DELETE` | `/assets/{id}/edits` | removeAssetEdits |
| `GET` | `/assets/{id}/edits` | getAssetEdits |
| `PUT` | `/assets/{id}/edits` | editAsset |
| `GET` | `/assets/{id}/metadata` | getAssetMetadata |
| `PUT` | `/assets/{id}/metadata` | updateAssetMetadata |
| `DELETE` | `/assets/{id}/metadata/{key}` | deleteAssetMetadata |
| `GET` | `/assets/{id}/metadata/{key}` | getAssetMetadataByKey |
| `GET` | `/assets/{id}/ocr` | getAssetOcr |
| `GET` | `/assets/{id}/original` | downloadAsset |
| `GET` | `/assets/{id}/thumbnail` | viewAsset |
| `GET` | `/assets/{id}/video/playback` | playAssetVideo |
| `GET` | `/assets/{id}/video/stream/main.m3u8` | getMainPlaylist |
| `DELETE` | `/assets/{id}/video/stream/{sessionId}` | endSession |
| `GET` | `/assets/{id}/video/stream/{sessionId}/{variantIndex}/playlist.m3u8` | getMediaPlaylist |
| `GET` | `/assets/{id}/video/stream/{sessionId}/{variantIndex}/{filename}` | getSegment |

### Authentication (17)

| Method | Path | OperationId |
|---|---|---|
| `POST` | `/auth/admin-sign-up` | signUpAdmin |
| `POST` | `/auth/change-password` | changePassword |
| `POST` | `/auth/login` | login |
| `POST` | `/auth/logout` | logout |
| `DELETE` | `/auth/pin-code` | resetPinCode |
| `POST` | `/auth/pin-code` | setupPinCode |
| `PUT` | `/auth/pin-code` | changePinCode |
| `POST` | `/auth/session/lock` | lockAuthSession |
| `POST` | `/auth/session/unlock` | unlockAuthSession |
| `GET` | `/auth/status` | getAuthStatus |
| `POST` | `/auth/validateToken` | validateAccessToken |
| `POST` | `/oauth/authorize` | startOAuth |
| `POST` | `/oauth/backchannel-logout` | logoutOAuth |
| `POST` | `/oauth/callback` | finishOAuth |
| `POST` | `/oauth/link` | linkOAuthAccount |
| `GET` | `/oauth/mobile-redirect` | redirectOAuthToMobile |
| `POST` | `/oauth/unlink` | unlinkOAuthAccount |

### Authentication (admin) (1)

| Method | Path | OperationId |
|---|---|---|
| `POST` | `/admin/auth/unlink-all` | unlinkAllOAuthAccountsAdmin |

### Database Backups (admin) (5)

| Method | Path | OperationId |
|---|---|---|
| `DELETE` | `/admin/database-backups` | deleteDatabaseBackup |
| `GET` | `/admin/database-backups` | listDatabaseBackups |
| `POST` | `/admin/database-backups/start-restore` | startDatabaseRestoreFlow |
| `POST` | `/admin/database-backups/upload` | uploadDatabaseBackup |
| `GET` | `/admin/database-backups/{filename}` | downloadDatabaseBackup |

### Download (2)

| Method | Path | OperationId |
|---|---|---|
| `POST` | `/download/archive` | downloadArchive |
| `POST` | `/download/info` | getDownloadInfo |

### Duplicates (4)

| Method | Path | OperationId |
|---|---|---|
| `DELETE` | `/duplicates` | deleteDuplicates |
| `GET` | `/duplicates` | getAssetDuplicates |
| `POST` | `/duplicates/resolve` | resolveDuplicates |
| `DELETE` | `/duplicates/{id}` | deleteDuplicate |

### Faces (4)

| Method | Path | OperationId |
|---|---|---|
| `GET` | `/faces` | getFaces |
| `POST` | `/faces` | createFace |
| `DELETE` | `/faces/{id}` | deleteFace |
| `PUT` | `/faces/{id}` | reassignFacesById |

### Jobs (3)

| Method | Path | OperationId |
|---|---|---|
| `GET` | `/jobs` | getQueuesLegacy [DEPRECATED] |
| `POST` | `/jobs` | createJob |
| `PUT` | `/jobs/{name}` | runQueueCommandLegacy [DEPRECATED] |

### Libraries (8)

| Method | Path | OperationId |
|---|---|---|
| `GET` | `/libraries` | getAllLibraries |
| `POST` | `/libraries` | createLibrary |
| `DELETE` | `/libraries/{id}` | deleteLibrary |
| `GET` | `/libraries/{id}` | getLibrary |
| `PUT` | `/libraries/{id}` | updateLibrary [DEPRECATED] |
| `POST` | `/libraries/{id}/scan` | scanLibrary |
| `GET` | `/libraries/{id}/statistics` | getLibraryStatistics |
| `POST` | `/libraries/{id}/validate` | validate |

### Maintenance (admin) (9)

| Method | Path | OperationId |
|---|---|---|
| `GET` | `/admin/integrity/report` | getIntegrityReport |
| `DELETE` | `/admin/integrity/report/{id}` | deleteIntegrityReport |
| `GET` | `/admin/integrity/report/{id}/file` | getIntegrityReportFile |
| `GET` | `/admin/integrity/report/{type}/csv` | getIntegrityReportCsv |
| `GET` | `/admin/integrity/summary` | getIntegrityReportSummary |
| `POST` | `/admin/maintenance` | setMaintenanceMode |
| `GET` | `/admin/maintenance/detect-install` | detectPriorInstall |
| `POST` | `/admin/maintenance/login` | maintenanceLogin |
| `GET` | `/admin/maintenance/status` | getMaintenanceStatus |

### Map (2)

| Method | Path | OperationId |
|---|---|---|
| `GET` | `/map/markers` | getMapMarkers |
| `GET` | `/map/reverse-geocode` | reverseGeocode |

### Memories (8)

| Method | Path | OperationId |
|---|---|---|
| `GET` | `/memories` | searchMemories |
| `POST` | `/memories` | createMemory |
| `GET` | `/memories/statistics` | memoriesStatistics |
| `DELETE` | `/memories/{id}` | deleteMemory |
| `GET` | `/memories/{id}` | getMemory |
| `PUT` | `/memories/{id}` | updateMemory [DEPRECATED] |
| `DELETE` | `/memories/{id}/assets` | removeMemoryAssets |
| `PUT` | `/memories/{id}/assets` | addMemoryAssets |

### Notifications (6)

| Method | Path | OperationId |
|---|---|---|
| `DELETE` | `/notifications` | deleteNotifications |
| `GET` | `/notifications` | getNotifications |
| `PUT` | `/notifications` | updateNotifications |
| `DELETE` | `/notifications/{id}` | deleteNotification |
| `GET` | `/notifications/{id}` | getNotification |
| `PUT` | `/notifications/{id}` | updateNotification |

### Notifications (admin) (3)

| Method | Path | OperationId |
|---|---|---|
| `POST` | `/admin/notifications` | createNotification |
| `POST` | `/admin/notifications/templates/{name}` | getNotificationTemplateAdmin |
| `POST` | `/admin/notifications/test-email` | sendTestEmailAdmin |

### Partners (5)

| Method | Path | OperationId |
|---|---|---|
| `GET` | `/partners` | getPartners |
| `POST` | `/partners` | createPartner |
| `DELETE` | `/partners/{id}` | removePartner |
| `POST` | `/partners/{id}` | createPartnerDeprecated [DEPRECATED] |
| `PUT` | `/partners/{id}` | updatePartner |

### People (11)

| Method | Path | OperationId |
|---|---|---|
| `DELETE` | `/people` | deletePeople |
| `GET` | `/people` | getAllPeople |
| `POST` | `/people` | createPerson |
| `PUT` | `/people` | updatePeople |
| `DELETE` | `/people/{id}` | deletePerson |
| `GET` | `/people/{id}` | getPerson |
| `PUT` | `/people/{id}` | updatePerson [DEPRECATED] |
| `POST` | `/people/{id}/merge` | mergePerson |
| `PUT` | `/people/{id}/reassign` | reassignFaces |
| `GET` | `/people/{id}/statistics` | getPersonStatistics |
| `GET` | `/people/{id}/thumbnail` | getPersonThumbnail |

### Plugins (4)

| Method | Path | OperationId |
|---|---|---|
| `GET` | `/plugins` | searchPlugins |
| `GET` | `/plugins/methods` | searchPluginMethods |
| `GET` | `/plugins/templates` | searchPluginTemplates |
| `GET` | `/plugins/{id}` | getPlugin |

### Queues (5)

| Method | Path | OperationId |
|---|---|---|
| `GET` | `/queues` | getQueues |
| `GET` | `/queues/{name}` | getQueue |
| `PUT` | `/queues/{name}` | updateQueue |
| `DELETE` | `/queues/{name}/jobs` | emptyQueue |
| `GET` | `/queues/{name}/jobs` | getQueueJobs |

### Search (10)

| Method | Path | OperationId |
|---|---|---|
| `GET` | `/search/cities` | getAssetsByCity |
| `GET` | `/search/explore` | getExploreData |
| `POST` | `/search/large-assets` | searchLargeAssets |
| `POST` | `/search/metadata` | searchAssets |
| `GET` | `/search/person` | searchPerson |
| `GET` | `/search/places` | searchPlaces |
| `POST` | `/search/random` | searchRandom |
| `POST` | `/search/smart` | searchSmart |
| `POST` | `/search/statistics` | searchAssetStatistics |
| `GET` | `/search/suggestions` | getSearchSuggestions |

### Server (14)

| Method | Path | OperationId |
|---|---|---|
| `GET` | `/server/about` | getAboutInfo |
| `GET` | `/server/apk-links` | getApkLinks |
| `GET` | `/server/config` | getServerConfig |
| `GET` | `/server/features` | getServerFeatures |
| `DELETE` | `/server/license` | deleteServerLicense |
| `GET` | `/server/license` | getServerLicense |
| `PUT` | `/server/license` | setServerLicense |
| `GET` | `/server/media-types` | getSupportedMediaTypes |
| `GET` | `/server/ping` | pingServer |
| `GET` | `/server/statistics` | getServerStatistics |
| `GET` | `/server/storage` | getStorage |
| `GET` | `/server/version` | getServerVersion |
| `GET` | `/server/version-check` | getVersionCheck |
| `GET` | `/server/version-history` | getVersionHistory |

### Sessions (6)

| Method | Path | OperationId |
|---|---|---|
| `DELETE` | `/sessions` | deleteAllSessions |
| `GET` | `/sessions` | getSessions |
| `POST` | `/sessions` | createSession |
| `DELETE` | `/sessions/{id}` | deleteSession |
| `PUT` | `/sessions/{id}` | updateSession [DEPRECATED] |
| `POST` | `/sessions/{id}/lock` | lockSession |

### Shared links (9)

| Method | Path | OperationId |
|---|---|---|
| `GET` | `/shared-links` | getAllSharedLinks |
| `POST` | `/shared-links` | createSharedLink |
| `POST` | `/shared-links/login` | sharedLinkLogin |
| `GET` | `/shared-links/me` | getMySharedLink |
| `DELETE` | `/shared-links/{id}` | removeSharedLink |
| `GET` | `/shared-links/{id}` | getSharedLinkById |
| `PATCH` | `/shared-links/{id}` | updateSharedLink |
| `DELETE` | `/shared-links/{id}/assets` | removeSharedLinkAssets |
| `PUT` | `/shared-links/{id}/assets` | addSharedLinkAssets |

### Stacks (7)

| Method | Path | OperationId |
|---|---|---|
| `DELETE` | `/stacks` | deleteStacks |
| `GET` | `/stacks` | searchStacks |
| `POST` | `/stacks` | createStack |
| `DELETE` | `/stacks/{id}` | deleteStack |
| `GET` | `/stacks/{id}` | getStack |
| `PUT` | `/stacks/{id}` | updateStack [DEPRECATED] |
| `DELETE` | `/stacks/{id}/assets/{assetId}` | removeAssetFromStack |

### Sync (4)

| Method | Path | OperationId |
|---|---|---|
| `DELETE` | `/sync/ack` | deleteSyncAck |
| `GET` | `/sync/ack` | getSyncAck |
| `POST` | `/sync/ack` | sendSyncAck |
| `POST` | `/sync/stream` | getSyncStream |

### System config (4)

| Method | Path | OperationId |
|---|---|---|
| `GET` | `/system-config` | getConfig |
| `PUT` | `/system-config` | updateConfig |
| `GET` | `/system-config/defaults` | getConfigDefaults |
| `GET` | `/system-config/storage-template-options` | getStorageTemplateOptions |

### System metadata (4)

| Method | Path | OperationId |
|---|---|---|
| `GET` | `/system-metadata/admin-onboarding` | getAdminOnboarding |
| `POST` | `/system-metadata/admin-onboarding` | updateAdminOnboarding |
| `GET` | `/system-metadata/reverse-geocoding-state` | getReverseGeocodingState |
| `GET` | `/system-metadata/version-check-state` | getVersionCheckState |

### Tags (9)

| Method | Path | OperationId |
|---|---|---|
| `GET` | `/tags` | getAllTags |
| `POST` | `/tags` | createTag |
| `PUT` | `/tags` | upsertTags |
| `PUT` | `/tags/assets` | bulkTagAssets |
| `DELETE` | `/tags/{id}` | deleteTag |
| `GET` | `/tags/{id}` | getTagById |
| `PUT` | `/tags/{id}` | updateTag [DEPRECATED] |
| `DELETE` | `/tags/{id}/assets` | untagAssets |
| `PUT` | `/tags/{id}/assets` | tagAssets |

### Timeline (2)

| Method | Path | OperationId |
|---|---|---|
| `GET` | `/timeline/bucket` | getTimeBucket |
| `GET` | `/timeline/buckets` | getTimeBuckets |

### Trash (3)

| Method | Path | OperationId |
|---|---|---|
| `POST` | `/trash/empty` | emptyTrash |
| `POST` | `/trash/restore` | restoreTrash |
| `POST` | `/trash/restore/assets` | restoreAssets |

### Users (16)

| Method | Path | OperationId |
|---|---|---|
| `GET` | `/users` | searchUsers |
| `GET` | `/users/me` | getMyUser |
| `PUT` | `/users/me` | updateMyUser [DEPRECATED] |
| `GET` | `/users/me/calendar-heatmap` | getMyCalendarHeatmap |
| `DELETE` | `/users/me/license` | deleteUserLicense |
| `GET` | `/users/me/license` | getUserLicense |
| `PUT` | `/users/me/license` | setUserLicense |
| `DELETE` | `/users/me/onboarding` | deleteUserOnboarding |
| `GET` | `/users/me/onboarding` | getUserOnboarding |
| `PUT` | `/users/me/onboarding` | setUserOnboarding |
| `GET` | `/users/me/preferences` | getMyPreferences |
| `PUT` | `/users/me/preferences` | updateMyPreferences [DEPRECATED] |
| `DELETE` | `/users/profile-image` | deleteProfileImage |
| `POST` | `/users/profile-image` | createProfileImage |
| `GET` | `/users/{id}` | getUser |
| `GET` | `/users/{id}/profile-image` | getProfileImage |

### Users (admin) (11)

| Method | Path | OperationId |
|---|---|---|
| `GET` | `/admin/users` | searchUsersAdmin |
| `POST` | `/admin/users` | createUserAdmin |
| `DELETE` | `/admin/users/{id}` | deleteUserAdmin |
| `GET` | `/admin/users/{id}` | getUserAdmin |
| `PUT` | `/admin/users/{id}` | updateUserAdmin [DEPRECATED] |
| `GET` | `/admin/users/{id}/calendar-heatmap` | getUserCalendarHeatmapAdmin |
| `GET` | `/admin/users/{id}/preferences` | getUserPreferencesAdmin |
| `PUT` | `/admin/users/{id}/preferences` | updateUserPreferencesAdmin [DEPRECATED] |
| `POST` | `/admin/users/{id}/restore` | restoreUserAdmin |
| `GET` | `/admin/users/{id}/sessions` | getUserSessionsAdmin |
| `GET` | `/admin/users/{id}/statistics` | getUserStatisticsAdmin |

### Views (2)

| Method | Path | OperationId |
|---|---|---|
| `GET` | `/view/folder` | getAssetsByOriginalPath |
| `GET` | `/view/folder/unique-paths` | getUniqueOriginalPaths |

### Workflows (7)

| Method | Path | OperationId |
|---|---|---|
| `GET` | `/workflows` | searchWorkflows |
| `POST` | `/workflows` | createWorkflow |
| `GET` | `/workflows/triggers` | getWorkflowTriggers |
| `DELETE` | `/workflows/{id}` | deleteWorkflow |
| `GET` | `/workflows/{id}` | getWorkflow |
| `PUT` | `/workflows/{id}` | updateWorkflow [DEPRECATED] |
| `GET` | `/workflows/{id}/share` | getWorkflowForShare |
