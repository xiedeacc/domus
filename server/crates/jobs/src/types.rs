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
    #[serde(rename = "smartSearch")]
    SmartSearch,
    #[serde(rename = "faceDetection")]
    FaceDetection,
    #[serde(rename = "facialRecognition")]
    FacialRecognition,
    #[serde(rename = "ocr")]
    Ocr,
    #[serde(rename = "workflow")]
    Workflow,
    #[serde(rename = "integrityCheck")]
    IntegrityCheck,
    #[serde(rename = "editor")]
    Editor,
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
            SmartSearch,
            FaceDetection,
            FacialRecognition,
            Ocr,
            Workflow,
            IntegrityCheck,
            Editor,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            QueueName::BackgroundTask => "backgroundTask",
            QueueName::MetadataExtraction => "metadataExtraction",
            QueueName::ThumbnailGeneration => "thumbnailGeneration",
            QueueName::VideoConversion => "videoConversion",
            QueueName::StorageTemplateMigration => "storageTemplateMigration",
            QueueName::Migration => "migration",
            QueueName::Search => "search",
            QueueName::Sidecar => "sidecar",
            QueueName::Library => "library",
            QueueName::Notifications => "notifications",
            QueueName::BackupDatabase => "backupDatabase",
            QueueName::DuplicateDetection => "duplicateDetection",
            QueueName::SmartSearch => "smartSearch",
            QueueName::FaceDetection => "faceDetection",
            QueueName::FacialRecognition => "facialRecognition",
            QueueName::Ocr => "ocr",
            QueueName::Workflow => "workflow",
            QueueName::IntegrityCheck => "integrityCheck",
            QueueName::Editor => "editor",
        }
    }

    /// Default worker concurrency, mirroring Immich's job settings.
    pub fn default_concurrency(&self) -> usize {
        match self {
            QueueName::MetadataExtraction => 5,
            QueueName::ThumbnailGeneration => 3,
            QueueName::VideoConversion => 1,
            QueueName::StorageTemplateMigration => 5,
            QueueName::FacialRecognition
            | QueueName::DuplicateDetection
            | QueueName::FaceDetection
            | QueueName::SmartSearch
            | QueueName::Ocr => 1,
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
    #[serde(rename = "SmartSearch")]
    SmartSearch,
    #[serde(rename = "SmartSearchQueueAll")]
    SmartSearchQueueAll,
    #[serde(rename = "FaceDetection")]
    FaceDetection,
    #[serde(rename = "FaceDetectionQueueAll")]
    FaceDetectionQueueAll,
    #[serde(rename = "FacialRecognition")]
    FacialRecognition,
    #[serde(rename = "FacialRecognitionQueueAll")]
    FacialRecognitionQueueAll,
    #[serde(rename = "Ocr")]
    Ocr,
    #[serde(rename = "OcrQueueAll")]
    OcrQueueAll,
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
            GeneratePreview | GenerateThumbnail | GenerateThumbhash => {
                QueueName::ThumbnailGeneration
            }
            VideoConversion => QueueName::VideoConversion,
            StorageTemplateMigration => QueueName::StorageTemplateMigration,
            LibraryScan | LibrarySyncFiles => QueueName::Library,
            DuplicateDetection => QueueName::DuplicateDetection,
            SmartSearch | SmartSearchQueueAll => QueueName::SmartSearch,
            FaceDetection | FaceDetectionQueueAll => QueueName::FaceDetection,
            FacialRecognition | FacialRecognitionQueueAll => QueueName::FacialRecognition,
            Ocr | OcrQueueAll => QueueName::Ocr,
            SendMail => QueueName::Notifications,
            DatabaseBackup => QueueName::BackupDatabase,
            _ => QueueName::BackgroundTask,
        }
    }
}
