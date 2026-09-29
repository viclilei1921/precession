pub const EVENT_TASK: &str = "task-update";
pub const EVENT_QUEUE: &str = "queue-updated";

pub const STATUS_PENDING: &str = "pending";
pub const STATUS_PROCESSING: &str = "processing";
pub const STATUS_COMPLETED: &str = "completed";
pub const STATUS_FAILED: &str = "failed";
pub const STATUS_CANCELED: &str = "canceled";

pub const KIND_ENCRYPT_FILE: &str = "encryptFile";
pub const KIND_DECRYPT_FILE: &str = "decryptFile";
pub const KIND_IMPORT_MEDIA: &str = "importMedia";
pub const KIND_ENCRYPT_MEDIA: &str = "encryptMedia";
pub const KIND_DECRYPT_MEDIA: &str = "decryptMedia";
pub const KIND_CONVERT_VIDEO: &str = "convertVideo";
pub const KIND_CUT_VIDEO: &str = "cutVideo";
pub const KIND_MERGE_VIDEO: &str = "mergeVideo";
pub const KIND_APPEND_VIDEO: &str = "appendVideo";
pub const KIND_CONVERT_AVIF: &str = "convertAvif";
pub const KIND_CONVERT_JXL: &str = "convertJxl";

pub(super) const PROGRESS_STEP: u64 = 1024 * 1024;
