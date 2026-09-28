# 术语

界面上的中文和代码里的标识按这张表对齐。新增字段时先补一行，再写代码。

## 模块

| 中文 | 代码 | 说明 |
| --- | --- | --- |
| 今天 | `today` | 只读时间轴，不单独建表 |
| 计划 | `plan` | |
| 手记 | `journal` | 日记正文可以再用单独密码封起来 |
| 成长 | `growth` | |
| 书库 | `library` | |
| 工具箱 | `toolbox` | 界面还没有 |
| 回顾 | `review` | 和今天共用 `timeline` |
| 设置 | `settings` | |
| 成员 | `member` | |
| 标签 | `tag` | |
| 地点 | `place` | |
| 影像 | `media` | 图片和视频的元数据，文件本体不进库。`encrypted` 表示文件是否已加密 |
| 任务 | `task` | 耗时的文件处理。密码只在入队时输入，队列不入库 |
| 时间轴 | `timeline` | 只读聚合，没有自己的表 |

## 实体

读取用实体名，写入用 `XxxInput`。列表需要更短的卡片时再用 `XxxSummary`，现在还没有。

| 中文 | 读取 | 写入 |
| --- | --- | --- |
| 计划 | `Plan` | `PlanInput` |
| 计划步骤 | `PlanStep` | `PlanStepInput` |
| 手记 | `JournalEntry` | `JournalEntryInput` |
| 手记流转 | `JournalLink` | 命令参数 |
| 手记引用书摘 | `JournalCitation` | 命令参数 |
| 成长 | `GrowthEntry` | `GrowthEntryInput` |
| 书 | `Book` | `BookInput` |
| 书摘 / 笔记 | `BookNote` | `BookNoteInput` |
| 成员 | `Member` | `MemberInput` |
| 标签 | `Tag` | 命令参数 `name` |
| 地点 | `Place` | 命令参数 `name` |
| 影像 | `Media` | `MediaInput` |
| 任务 | `Task` | `TaskInput` |
| 时间轴条目 | `TimelineItem` | 无 |

类型字段一律叫 `kind`，不用 `type`。

## kind 与状态

| 实体 | 字段 | 取值 | 中文 |
| --- | --- | --- | --- |
| 计划 | `status` | `inbox` / `scheduled` / `done` | 收集箱 / 已安排 / 已完成 |
| 手记 | `kind` | `diary` / `spark` / `writing` | 日记 / 灵感 / 写作 |
| 手记流转 | `kind` | `spark_to_writing` / `writing_to_diary` | 灵感到写作 / 写作到日记 |
| 成长 | `kind` | `milestone` / `moment` | 里程碑 / 瞬间 |
| 书 | `status` | `want` / `reading` / `finished` | 想读 / 在读 / 读完 |
| 书摘或笔记 | `kind` | `excerpt` / `note` | 书摘 / 笔记 |
| 影像 | `kind` | `image` / `video` | 图片 / 视频 |
| 归属 | `owner` | `plan` / `journal_entry` / `growth_entry` / `book` / `book_note` | 见 `owner.rs` |

时间轴的 `kinds` 过滤用：`plan`、`diary`、`spark`、`writing`、`milestone`、`moment`、`excerpt`、`note`。

## 字段

| 中文 | 字段 | 说明 |
| --- | --- | --- |
| 已锁定 | `locked` | 记录级锁定。和会话里的「锁定档案」不是一回事 |
| 高亮 | `highlight` | |
| 档案密码 | 命令参数 `password` | 建库、解锁、启用设备槽都用这个词 |
| 设备解锁 | `deviceUnlock` | `device.wrap` 是否存在 |
| 用户锁定 | `userLocked` | `user.lock` 是否存在 |
| 成员 | `member_id` | 成长记录必填。引用表里是 `member_ids` |
| 发生时间 | `occurred_at` | 手记、成长、书摘 |
| 安排时间 | `scheduled_at` | 计划。时间轴上没有它时用 `created_at` |
| 完成时间 | `completed_at` | 计划 |
| 读完时间 | `finished_at` | 书。不要把书的状态写成 `done` |

## 裁定

这些词在设计稿里撞过，代码里只保留一种说法：

- 记录上的锁统一叫「已锁定」，字段 `locked`。把档案锁上叫「锁定」，命令 `db_lock`。
- 打开档案的密码统一叫「档案密码」。至少 8 位。
- 用本机系统密钥库免密打开，统一叫「设备解锁」。不叫设备槽解锁以外的别名混进界面。
- 全局快速入口统一叫「记一笔」。它只选择类型并跳转，不在浮层里写正文。
- 图片和视频统一叫「影像」，模块是 `media`。
- 书里的摘录和批注是同一张 `book_note`，用 `kind` 区分书摘和笔记。
- 「规划 / 思考 / 成长 / 认知」只作为年度之书的章节名，不作为模块、表或 `kind`。
