# 业务模块模板

档案库底层在 `src-tauri/src/db`：建库、开库、密钥、进程级连接。  
业务表不要写进 `db/`，而是和 `db` 同一层级，在 crate 根下自立模块。

参考实现：`[src-tauri/src/db_demo](../src-tauri/src/db_demo)`。  
前端对照：`[src/bridge/demo.ts](../src/bridge/demo.ts)`。

后面加手记、计划、书库时，复制 `db_demo` 再改名，不要另起一套分层。

## 和 `db` 的边界


|      | `db`                             | 业务模块（如 `db_demo`）                  |
| ---- | -------------------------------- | ---------------------------------- |
| 职责   | 加密库文件、会话、`with_conn`             | 一张（或一组）业务表                         |
| 连接   | 持有 `DbState`                     | 不 `Connection::open`               |
| 对应文件 | `state.rs` + 看门人 `repository.rs` | `service.rs` + SQL `repository.rs` |
| 放哪   | `src-tauri/src/db/`              | 与 `db` 平级，例如 `journal/`            |


`db_demo` 没有自己的 State。进程里只有一份 `DbState`。

## 目录

```
src-tauri/src/<feature>/
  mod.rs          可见性 + migrate
  commands.rs     Tauri IPC，只调 service
  service.rs      校验、with_conn、测试
  repository.rs   SQL，只收 &Connection
  dto.rs          给前端的结构
  error.rs        本模块错误
  constants.rs    表名、schema 版本键

src/bridge/<feature>.ts   与 command 一一对应
```

`mod.rs` 可见性按这个抄：

```rust
pub mod commands;
pub mod dto;
pub mod error;
mod constants;
mod repository;
mod service;
```

对外只暴露 IPC、DTO、错误。`commands` 通过 `super::service` 调用即可，`service` 不必 `pub`。

## 分层

```
前端 bridge
    ↓ invoke("demo_list")
commands     State<'_, DbState>  →  service
service      校验；db.with_conn(...)
repository   &Connection 上的 SQL
```


| 层          | 入参                           | 做什么                     | 不做什么                     |
| ---------- | ---------------------------- | ----------------------- | ------------------------ |
| commands   | Tauri `State`、IPC 的 `String` | 转成 `&str`，调 service     | 写 SQL、做业务校验              |
| service    | `&DbState`                   | trim / 空标题等；`with_conn` | `Connection::open`、拼 SQL |
| repository | `&Connection`                | 建表、CRUD、行映射             | 碰 `DbState`、自己开连接        |


跨表写入（写 A 再写 B）必须在 **同一次** `with_conn` 里调多个 repository，必要时 `conn.unchecked_transaction()`。不要每个 repository 自己再走一遍 `with_conn`。

## Schema 与迁移

建表不挂在每次 CRUD 上。建库或解锁成功、连接已就绪后，`DbState` 按注册顺序跑各模块的 `migrate`。

1. `repository::ensure_schema`：读 `db_meta` 里本模块版本，按 `if current < N` 执行迁移，最后写回版本。
2. `mod.rs` 的 `migrate` 把它转成 `DbError`，供 `DbState` 调用：

```rust
pub(crate) fn migrate(conn: &Connection) -> Result<(), DbError> {
  repository::ensure_schema(conn).map_err(|_| DbError::Internal)
}
```

1. 在 `lib.rs` 注册（后续模块往 Vec 里追加）：

```rust
app.manage(db::state::DbState::new(app_data_dir).with_migrators(vec![
  db_demo::migrate,
  // journal::migrate,
]));
```

约定：

- 版本键与探针键 `ok` 分开，形如 `demo.schema_version`。
- 升 `SCHEMA_VERSION` 时必须加对应的 `if current < N { ALTER ... }`，不要只改数字。
- `migrate` 失败：建库会关掉连接并删掉半成品；解锁则不会进入已解锁状态。
- `db` 不 `use` 任何业务模块；谁有表谁提供 `migrate`。



## 错误与 DTO

业务错误实现 `From<DbError>`，这样 `with_conn` 才能把「未解锁」转过来：

- `DbError::Locked` → 本模块的 `Locked`
- 其余库错误 → `Internal`（不要把密钥/IO 细节漏给前端）

`thiserror` 的 `#[error("...")]` 就是给前端看的文案。再 `Serialize` 成字符串。

DTO 只 `Serialize`，字段 `#[serde(rename_all = "camelCase")]`，与 `src/bridge` 的类型对齐。Command 参数继续用 `String`；入参字段变多再加 `Deserialize` 结构。

主键用 `crate::utils::id::new_uuid_v4`，时间戳用 `crate::utils::time::now_unix_ms`。

## 前端

`src/bridge/<feature>.ts` 与 command 同名、一对一。不要把业务 invoke 写进 `bridge/db.ts`。


| Rust command    | TS               |
| --------------- | ---------------- |
| `demo_list`     | `demoList()`     |
| `demo_get`      | `demoGet(id)`    |
| 返回 `created_at` | 类型里写 `createdAt` |


在 `src/bridge/index.ts` 里 `export *`。

## 测试

写在 `service.rs` 的 `#[cfg(test)]`，打在业务 API 上（含校验、锁库），不要只测 SQL。

```rust
let db = DbState::new(dir.path().to_path_buf()).with_migrators(vec![crate::db_demo::migrate]);
db.create("test-password-123").expect("create db");
```

测试里也要注册 `migrate`，否则 `create` 不会建业务表。

## 新模块清单

假设模块名 `journal`：

1. 复制 `src-tauri/src/db_demo` 为 `src-tauri/src/journal`，替换表名、前缀、错误、DTO。
2. `src-tauri/src/lib.rs`：`mod journal;`
3. 同一文件 `with_migrators` 追加 `journal::migrate`
4. `invoke_handler` 注册 `journal::commands::*`
5. 新增 `src/bridge/journal.ts`，并在 `src/bridge/index.ts` 导出
6. 在 `journal/service.rs` 留一条 CRUD + 锁库测试



## 不要做的

- 不要在 `db/` 或 `db/repository.rs` 里加业务表
- 不要给业务模块起名为 `state.rs`（那是 `DbState` 的位子）
- 不要 `Connection::open`；读写一律 `DbState::with_conn`
- 不要让 `repository` / `constants` 变 `pub`（否则会绕过 service 校验）
- 不要在热路径上 `CREATE TABLE`
- 不要只 bump 版本号而不写 `ALTER`
- 不要再拆 mapper / usecase；Tauri 桌面这三层够了

