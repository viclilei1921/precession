/// 数据库目录（相对应用数据目录）
pub const DB_DIR: &str = "db";

/// SQLCipher 库文件
pub const DB_SQLITE: &str = "data.sqlite";

/// DEK 包裹头（与 sqlite 成对）
pub const DB_HEADER: &str = "key.header.json";

/// 写头文件时的临时文件
pub const DB_HEADER_TMP: &str = "key.header.json.tmp";

/// 探针键
pub const PROBE_KEY: &str = "ok";

/// 探针值
pub const PROBE_VALUE: &str = "precession.db.ready";
