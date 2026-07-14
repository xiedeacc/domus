//! Queue and job identifiers. String values mirror Immich's enums because
//! they surface in the /jobs and /queues admin APIs.
//! NOTE: verify against server/src/enum.ts when wiring the real handlers.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QueueName {
    #[serde(rename = "backgroundTask")]
    BackgroundTask,
    #[serde(rename = "metadataExtraction")]
    MetadataExtraction,
    #[serde(rename = "thumbnailGeneration")]
    ThumbnailGeneration,
    #[serde(rename = "videoConversion")]
    VideoConversion,
    #[serde(rename = "storageTemplateMigration")]
    StorageTemplateMigration,
    #[serde(rename = "migration")]
    Migration,
    #[serde(rename = "search")]
    Search,
    #[serde(rename = "sidecar")]
    Sidecar,
    #[serde(rename = "library")]
    Library,
    #[serde(rename = "notifications")]
    Notifications,
    #[serde(rename = "backupDatabase")]
    BackupDatabase,
    #[serde(rename = "duplicateDetection")]
    DuplicateDetection,
    // ML-backed queues (smartSearch, faceDetection, facialRecognition, ocr)
    // are intentionally absent: Domus does not ship the ML service. Their
    // admin-API entries report as disabled.
}

impl QueueName {
    pub fn all() -> &'static [QueueName] {
        use QueueName::*;
        &[
            BackgroundTask,
            MetadataExtraction,
            ThumbnailGeneration,
            VideoConversion,
            StorageTemplateMigration,
            Migration,
            Search,
            Sidecar,
            Library,
            Notifications,
            BackupDatabase,
            DuplicateDetection,
        ]
    }

    /// Default worker concurrency, mirroring Immich's job settings.
    pub fn default_concurrency(&self) -> usize {
        match self {
            QueueName::MetadataExtraction => 5,
            QueueName::ThumbnailGeneration => 3,
            QueueName::VideoConversion => 1,
            QueueName::StorageTemplateMigration => 5,
            _ => 2,
        }
    }
}

/// Individual job kinds routed to a queue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobName {
    // metadata
    #[serde(rename = "MetadataExtraction")]
    MetadataExtraction,
    #[serde(rename = "SidecarDiscovery")]
    SidecarDiscovery,
    // thumbnails
    #[serde(rename = "GeneratePreview")]
    GeneratePreview,
    #[serde(rename = "GenerateThumbnail")]
    GenerateThumbnail,
    #[serde(rename = "GenerateThumbhash")]
    GenerateThumbhash,
    // video
    #[serde(rename = "VideoConversion")]
    VideoConversion,
    // housekeeping
    #[serde(rename = "AssetDeletion")]
    AssetDeletion,
    #[serde(rename = "FileDeletion")]
    FileDeletion,
    #[serde(rename = "StorageTemplateMigration")]
    StorageTemplateMigration,
    #[serde(rename = "PersonCleanup")]
    PersonCleanup,
    #[serde(rename = "UserDeletion")]
    UserDeletion,
    #[serde(rename = "TrashEmpty")]
    TrashEmpty,
    // library scanning
    #[serde(rename = "LibraryScan")]
    LibraryScan,
    #[serde(rename = "LibrarySyncFiles")]
    LibrarySyncFiles,
    // misc
    #[serde(rename = "MemoriesCreate")]
    MemoriesCreate,
    #[serde(rename = "DuplicateDetection")]
    DuplicateDetection,
    #[serde(rename = "SendMail")]
    SendMail,
    #[serde(rename = "DatabaseBackup")]
    DatabaseBackup,
}

impl JobName {
    pub fn queue(&self) -> QueueName {
        use JobName::*;
        match self {
            MetadataExtraction => QueueName::MetadataExtraction,
            SidecarDiscovery => QueueName::Sidecar,
            GeneratePreview | GenerateThumbnail | GenerateThumbhash => QueueName::ThumbnailGeneration,
            VideoConversion => QueueName::VideoConversion,
            StorageTemplateMigration => QueueName::StorageTemplateMigration,
            LibraryScan | LibrarySyncFiles => QueueName::Library,
            DuplicateDetection => QueueName::DuplicateDetection,
            SendMail => QueueName::Notifications,
            DatabaseBackup => QueueName::BackupDatabase,
            _ => QueueName::BackgroundTask,
        }
    }
}
