pub const MEMBER_TABLE: &str = "member";
pub const TAG_TABLE: &str = "tag";
pub const PLACE_TABLE: &str = "place";

/// 本模块写在 db_meta 里的版本键（与探针键 `ok` 分开）
pub const SCHEMA_VERSION_KEY: &str = "catalog.schema_version";

/// 当前 schema 版本
pub const SCHEMA_VERSION: i64 = 1;
