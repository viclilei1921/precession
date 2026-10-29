pub const UPLOAD_TABLE: &str = "upload";

pub const SCHEMA_VERSION_KEY: &str = "upload.schema_version";
pub const SCHEMA_VERSION: i64 = 1;

pub const KIND_IMAGE: &str = "image";
pub const KIND_VIDEO: &str = "video";
pub const KIND_BOOK: &str = "book";
pub const KIND_FILE: &str = "file";

/// 允许的文件类型，也是上传目录下的子目录名
pub const KINDS: [&str; 4] = [KIND_IMAGE, KIND_VIDEO, KIND_BOOK, KIND_FILE];

/// 上传目录名，相对数据根目录，和数据库目录 `db` 平级
pub const UPLOAD_DIR: &str = "upload";

/// 导入中的临时目录名，相对上传目录
pub const TMP_DIR: &str = ".tmp";

/// 加解密已有文件时留在原文件旁边的后缀。清理时不碰
pub const SIDE_SUFFIXES: [&str; 3] = ["part", "bak", "aside"];

/// 扩展名最长字符数，超出时不保留扩展名
pub const EXT_MAX: usize = 10;
