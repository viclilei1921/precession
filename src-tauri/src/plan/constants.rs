/// 计划表
pub const PLAN_TABLE: &str = "plan";
/// 计划步骤表
pub const PLAN_STEP_TABLE: &str = "plan_step";
/// 清单表
pub const PLAN_GROUP_TABLE: &str = "plan_group";
/// 重复规则表
pub const PLAN_REPEAT_TABLE: &str = "plan_repeat";
/// 提醒表
pub const PLAN_REMINDER_TABLE: &str = "plan_reminder";
/// 评论表
pub const PLAN_COMMENT_TABLE: &str = "plan_comment";

/// 计划架构版本键
pub const SCHEMA_VERSION_KEY: &str = "plan.schema_version";
/// 计划架构版本
pub const SCHEMA_VERSION: i64 = 1;

/// 计划状态：草稿
pub const STATUS_DRAFT: &str = "draft";
/// 计划状态：已安排
pub const STATUS_SCHEDULED: &str = "scheduled";
/// 计划状态：已完成
pub const STATUS_DONE: &str = "done";

/// 计划状态列表
pub const STATUSES: &[&str] = &[STATUS_DRAFT, STATUS_SCHEDULED, STATUS_DONE];

/// 重复：每天
pub const REPEAT_DAILY: &str = "daily";
/// 重复：每周
pub const REPEAT_WEEKLY: &str = "weekly";
/// 重复：每月
pub const REPEAT_MONTHLY: &str = "monthly";
/// 重复：每年
pub const REPEAT_YEARLY: &str = "yearly";

/// 重复种类
pub const REPEAT_KINDS: &[&str] = &[REPEAT_DAILY, REPEAT_WEEKLY, REPEAT_MONTHLY, REPEAT_YEARLY];
