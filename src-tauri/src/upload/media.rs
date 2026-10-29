//! 本机媒体服务。图片和视频用 `http://127.0.0.1` 上的区间请求播放，加密文件按 ChaCha20 的明文偏移解密，不用先解开整份。

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Manager};
use zeroize::Zeroize;

use super::dto::Upload;
use super::error::UploadError;
use super::storage::resolve;
use crate::chacha::chacha::{CipherReader, FileError, FileKey};
use crate::db::state::DbState;
use crate::utils::id::new_uuid_v4;

/// 单次响应最多读出的明文字节。浏览器会接着要下一段
const CHUNK: u64 = 1024 * 1024;
/// 图片等非视频一次读完的上限
const FULL_MAX: u64 = 40 * 1024 * 1024;

/// 前端拼播放地址用的本机入口
///
/// # 字段
/// - `origin`: `http://127.0.0.1:端口`。绑定失败时为空
/// - `token`: 路径里的口令，避免本机其他页面随便读文件
///
/// # 示例
/// 见 [`MediaState::endpoint`]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaEndpoint {
  /// `http://127.0.0.1:端口`。绑定失败时为空
  pub origin: String,
  /// 路径里的口令
  pub token: String,
}

/// 一次读出的响应
///
/// # 字段
/// - `status`: HTTP 状态码
/// - `mime`: `Content-Type`
/// - `total`: 明文总长度
/// - `start`: 这段明文的起点
/// - `length`: `Content-Length`。`HEAD` 不带正文时仍是这段的长度
/// - `body`: 正文。`HEAD` 时为空
struct MediaBody {
  /// HTTP 状态码
  status: u16,
  /// `Content-Type`
  mime: String,
  /// 明文总长度
  total: u64,
  /// 这段明文的起点
  start: u64,
  /// `Content-Length`
  length: u64,
  /// 正文
  body: Vec<u8>,
}

impl MediaBody {
  /// 不带正文的错误响应
  ///
  /// # 参数
  /// - `status`: HTTP 状态码
  /// - `total`: 已知的明文长度，没有时为 0
  ///
  /// # 返回值
  /// 空正文的响应
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let body = MediaBody::error(404, 0);
  /// ```
  fn error(status: u16, total: u64) -> Self {
    Self { status, mime: "text/plain".to_string(), total, start: 0, length: 0, body: Vec::new() }
  }
}

struct Inner {
  origin: String,
  token: String,
  keys: HashMap<String, FileKey>,
}

/// 播放地址和已解开的文件密钥
///
/// 密钥只留在内存里。锁上档案时清掉。
///
/// # 字段
/// - `inner`: 地址、路径口令和文件密钥
///
/// # 示例
/// 见 [`install`]
#[derive(Clone)]
pub struct MediaState {
  inner: Arc<Mutex<Inner>>,
}

impl MediaState {
  /// 造一个还没监听端口的状态，测试和绑定失败时用
  ///
  /// # 返回值
  /// `origin` 为空的状态
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let media = MediaState::offline();
  /// ```
  pub(crate) fn offline() -> Self {
    Self::with_origin(String::new())
  }

  /// 用已经绑定好的本机地址造状态
  ///
  /// # 参数
  /// - `origin`: `http://127.0.0.1:端口`
  ///
  /// # 返回值
  /// 带新路径口令的状态
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let media = MediaState::with_origin(origin);
  /// ```
  fn with_origin(origin: String) -> Self {
    Self { inner: Arc::new(Mutex::new(Inner { origin, token: new_uuid_v4(), keys: HashMap::new() })) }
  }

  /// 锁中毒时接着用里面的数据
  ///
  /// # 返回值
  /// 内部状态的锁
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let inner = media.lock();
  /// ```
  fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
    self.inner.lock().unwrap_or_else(|err| err.into_inner())
  }

  /// 返回播放地址
  ///
  /// # 返回值
  /// 本机入口。还没绑上端口时 `origin` 为空
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let endpoint = media.endpoint();
  /// ```
  pub(crate) fn endpoint(&self) -> MediaEndpoint {
    let inner = self.lock();
    MediaEndpoint { origin: inner.origin.clone(), token: inner.token.clone() }
  }

  /// 这个文件的密钥是否已经留下
  ///
  /// # 参数
  /// - `id`: 文件 ID
  ///
  /// # 返回值
  /// 已经解开过时为 `true`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let ready = media.has(id);
  /// ```
  fn has(&self, id: &str) -> bool {
    self.lock().keys.contains_key(id)
  }

  /// 取出一份密钥副本，读文件时不再占着锁
  ///
  /// # 参数
  /// - `id`: 文件 ID
  ///
  /// # 返回值
  /// 解开过时返回密钥
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let key = media.key(id);
  /// ```
  fn key(&self, id: &str) -> Option<FileKey> {
    self.lock().keys.get(id).cloned()
  }

  /// 记下这个文件的密钥
  ///
  /// # 参数
  /// - `id`: 文件 ID
  /// - `key`: 从这份密文派生的密钥
  ///
  /// # 返回值
  /// 不返回数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// media.insert(id, key);
  /// ```
  fn insert(&self, id: String, key: FileKey) {
    self.lock().keys.insert(id, key);
  }

  /// 清掉全部密钥。锁上档案时调用
  ///
  /// # 返回值
  /// 不返回数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// media.clear();
  /// ```
  pub(crate) fn clear(&self) {
    self.lock().keys.clear();
  }

  /// 忘掉一个文件的密钥
  ///
  /// # 参数
  /// - `id`: 文件 ID
  ///
  /// # 返回值
  /// 不返回数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// media.forget(id);
  /// ```
  pub(crate) fn forget(&self, id: &str) {
    self.lock().keys.remove(id);
  }
}

/// 在回环地址上监听，并交给应用状态
///
/// 绑定失败时仍放入一个空地址的状态，播放会提示暂时不可用，应用可以继续开。
///
/// # 参数
/// - `app`: 应用句柄，里面已经放了数据库状态
///
/// # 返回值
/// 不返回数据
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// install(app.handle().clone());
/// ```
pub(crate) fn install(app: AppHandle) {
  let listener = match TcpListener::bind((Ipv4Addr::LOCALHOST, 0)) {
    Ok(listener) => listener,
    Err(err) => {
      tauri_plugin_log::log::error!("media bind: {err}");
      app.manage(MediaState::offline());
      return;
    }
  };
  let port = match listener.local_addr() {
    Ok(addr) => addr.port(),
    Err(err) => {
      tauri_plugin_log::log::error!("media addr: {err}");
      app.manage(MediaState::offline());
      return;
    }
  };
  app.manage(MediaState::with_origin(format!("http://127.0.0.1:{port}")));
  let app = app.clone();
  if let Err(err) = thread::Builder::new().name("media".into()).spawn(move || serve(app, listener)) {
    tauri_plugin_log::log::error!("media thread: {err}");
  }
}

/// 接受连接，每条连接自己读请求
///
/// # 参数
/// - `app`: 应用句柄
/// - `listener`: 已经绑定的监听
///
/// # 返回值
/// 不返回数据。监听退出时函数结束
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// serve(app, listener);
/// ```
fn serve(app: AppHandle, listener: TcpListener) {
  for incoming in listener.incoming() {
    let Ok(stream) = incoming else {
      continue;
    };
    let app = app.clone();
    thread::spawn(move || {
      let _ = handle_stream(&app, stream);
    });
  }
}

/// 读一条 HTTP 请求并写回一段明文
///
/// # 参数
/// - `app`: 应用句柄
/// - `stream`: 这条连接
///
/// # 返回值
/// 读写完成时返回空成功
///
/// # 错误
/// 套接字读写失败时返回 `io::Error`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// handle_stream(app, stream)?;
/// ```
fn handle_stream(app: &AppHandle, mut stream: TcpStream) -> io::Result<()> {
  stream.set_read_timeout(Some(Duration::from_secs(20)))?;
  let _ = stream.set_nodelay(true);
  let mut buf = [0u8; 8192];
  let mut filled = 0;
  while filled < buf.len() {
    let n = stream.read(&mut buf[filled..])?;
    if n == 0 {
      break;
    }
    filled += n;
    if buf[..filled].windows(4).any(|item| item == b"\r\n\r\n") {
      break;
    }
  }
  let text = String::from_utf8_lossy(&buf[..filled]);
  let Some((method, path, _)) = split_request(&text) else {
    return Ok(());
  };
  if method.eq_ignore_ascii_case("OPTIONS") {
    return write_options(&mut stream);
  }
  let media = app.state::<MediaState>();
  let token = media.endpoint().token;
  let Some(id) = file_id(path, &token) else {
    return write_body(&mut stream, &MediaBody::error(404, 0), false);
  };
  if !(method.eq_ignore_ascii_case("GET") || method.eq_ignore_ascii_case("HEAD")) {
    return write_body(&mut stream, &MediaBody::error(405, 0), false);
  }
  let db = app.state::<DbState>();
  let body = read_body(&db, &media, id, header_value(&text, "range").as_deref());
  write_body(&mut stream, &body, method.eq_ignore_ascii_case("HEAD"))
}

/// 用口令解开一个加密文件，并把密钥留在内存里
///
/// 未加密的文件直接成功。口令为空且这份文件已经解开过时也成功。密码不对时 ChaCha20 不会报错，这里看解开的开头像不像这种文件。
///
/// # 参数
/// - `db`: 数据库状态
/// - `media`: 媒体状态
/// - `id`: 文件 ID
/// - `password`: 文件密码。用完会清掉
///
/// # 返回值
/// 可以播放时返回空成功
///
/// # 错误
/// 记录不存在返回 `NotFound`。已封存返回 `OwnerSealed`。加密但没口令返回 `NeedPassword`。开头不像这种文件返回 `Password`。格式不对返回 `Format`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// open_media(db, media, id, password)?;
/// ```
pub(crate) fn open_media(db: &DbState, media: &MediaState, id: &str, mut password: String) -> Result<(), UploadError> {
  let result = open_with(db, media, id, &password);
  password.zeroize();
  result
}

/// 把文件的明文写到用户选的文件夹
///
/// 未加密时直接复制。加密文件沿用已经解开的密钥；这次带了密码就先解开再写，文件名去掉末尾的 `.env`。目标里已有同名文件时，在名字后加上序号。
///
/// # 参数
/// - `db`: 数据库状态
/// - `media`: 播放地址和文件密钥
/// - `id`: 文件标识
/// - `dir`: 用户选择的文件夹
/// - `password`: 文件密码。未加密、或这次会话已经解开过时可以是空字符串。用完会清掉
///
/// # 返回值
/// 写好的文件绝对路径
///
/// # 错误
/// 记录不存在返回 `NotFound`。已封存返回 `OwnerSealed`。加密但没口令返回 `NeedPassword`。口令不对返回 `Password`。文件夹无效或写盘失败返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let saved = export_file(db, media, id, dir, password)?;
/// ```
pub(crate) fn export_file(
  db: &DbState,
  media: &MediaState,
  id: &str,
  dir: &str,
  password: String,
) -> Result<String, UploadError> {
  open_media(db, media, id, password)?;
  let row = load_row(db, id)?;
  let source = stored_path(db, &row)?;
  let folder = Path::new(dir.trim());
  if !folder.is_dir() {
    return Err(UploadError::Internal);
  }
  let label = if row.encrypted { plain_name(&row.name) } else { row.name.clone() };
  let dest = vacant_path(folder, &export_name(&label, &row.id, &row.rel_path));
  if row.encrypted {
    write_plain(media, &row.id, &source, &dest)?;
  } else if std::fs::copy(&source, &dest).is_err() {
    let _ = std::fs::remove_file(&dest);
    return Err(UploadError::Internal);
  }
  Ok(dest.to_string_lossy().into_owned())
}

/// 用已经记下的密钥把密文解密写到新文件
///
/// # 参数
/// - `media`: 播放地址和文件密钥
/// - `id`: 文件标识
/// - `source`: 上传目录里的密文
/// - `dest`: 明文输出路径
///
/// # 返回值
/// 写完时返回空成功
///
/// # 错误
/// 还没有密钥返回 `NeedPassword`。打不开密文返回 `NotFound`。格式不对返回 `Format`。写入中断时删掉半成品并返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// write_plain(media, id, &source, &dest)?;
/// ```
fn write_plain(media: &MediaState, id: &str, source: &Path, dest: &Path) -> Result<(), UploadError> {
  let key = media.key(id).ok_or(UploadError::NeedPassword)?;
  let file = File::open(source).map_err(|_| UploadError::NotFound)?;
  let mut reader = CipherReader::with_key(file, key).map_err(map_file)?;
  let mut out = File::create(dest).map_err(|_| UploadError::Internal)?;
  if io::copy(&mut reader, &mut out).is_err() {
    drop(out);
    let _ = std::fs::remove_file(dest);
    return Err(UploadError::Internal);
  }
  Ok(())
}

/// 导出明文时去掉加密文件名末尾的 `.env`
///
/// 名字不长于 `.env`，或不是以它结尾时原样返回。
///
/// # 参数
/// - `name`: 记录里的文件名
///
/// # 返回值
/// 明文文件名
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(plain_name("a.jpg.env"), "a.jpg");
/// assert_eq!(plain_name("a.jpg"), "a.jpg");
/// ```
fn plain_name(name: &str) -> String {
  if name.len() > 4 && name.to_ascii_lowercase().ends_with(".env") {
    return name[..name.len() - 4].to_string();
  }
  name.to_string()
}

/// 从原始文件名取出可以落盘的名字
///
/// 丢掉路径和 Windows 不允许的字符。名字空掉时用文件标识，并尽量带上存储路径里的扩展名。
///
/// # 参数
/// - `name`: 原始文件名
/// - `id`: 文件标识
/// - `rel`: 存储相对路径
///
/// # 返回值
/// 单个路径分量
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let name = export_name("a/b.jpg", "id", "upload/image/id.jpg");
/// assert_eq!(name, "b.jpg");
/// ```
fn export_name(name: &str, id: &str, rel: &str) -> String {
  let file_name = Path::new(name.trim()).file_name().and_then(|item| item.to_str()).unwrap_or("");
  let cleaned: String = file_name
    .chars()
    .filter(|item| !item.is_control() && !matches!(item, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'))
    .collect();
  let cleaned = cleaned.trim().trim_matches('.').trim();
  if !cleaned.is_empty() {
    return cleaned.to_string();
  }
  let ext = Path::new(rel).extension().and_then(|item| item.to_str()).unwrap_or("");
  if ext.is_empty() { id.to_string() } else { format!("{id}.{ext}") }
}

/// 在文件夹里找一个还没被占用的文件名
///
/// # 参数
/// - `dir`: 目标文件夹
/// - `name`: 希望使用的文件名
///
/// # 返回值
/// 不存在的路径。同名文件已在时变成 `名字 (2).扩展名`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let dest = vacant_path(dir, "a.jpg");
/// ```
fn vacant_path(dir: &Path, name: &str) -> std::path::PathBuf {
  let first = dir.join(name);
  if !first.exists() {
    return first;
  }
  let path = Path::new(name);
  let stem = path.file_stem().and_then(|item| item.to_str()).unwrap_or(name);
  let ext = path.extension().and_then(|item| item.to_str());
  for index in 2..10_000 {
    let next = match ext {
      Some(ext) => format!("{stem} ({index}).{ext}"),
      None => format!("{stem} ({index})"),
    };
    let candidate = dir.join(next);
    if !candidate.exists() {
      return candidate;
    }
  }
  dir.join(format!("{stem}-{}", new_uuid_v4()))
}

/// 校验口令并记下密钥
///
/// # 参数
/// - `db`: 数据库状态
/// - `media`: 媒体状态
/// - `id`: 文件 ID
/// - `password`: 文件密码
///
/// # 返回值
/// 可以播放时返回空成功
///
/// # 错误
/// 同 [`open_media`]
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// open_with(db, media, id, password)?;
/// ```
fn open_with(db: &DbState, media: &MediaState, id: &str, password: &str) -> Result<(), UploadError> {
  let row = load_row(db, id)?;
  if !row.encrypted {
    return Ok(());
  }
  if password.is_empty() {
    return if media.has(&row.id) { Ok(()) } else { Err(UploadError::NeedPassword) };
  }
  let path = stored_path(db, &row)?;
  let mut file = File::open(&path).map_err(|_| UploadError::NotFound)?;
  let key = FileKey::read(&mut file, password).map_err(map_file)?;
  let mut reader = CipherReader::with_key(file, key.clone()).map_err(map_file)?;
  let mut head = [0u8; 512];
  let n = reader.read(&mut head).map_err(|_| UploadError::Internal)?;
  if !looks_plain(&row.kind, &row.name, &head[..n]) {
    return Err(UploadError::Password);
  }
  media.insert(row.id, key);
  Ok(())
}

/// 按区间读出一份文件的明文
///
/// # 参数
/// - `db`: 数据库状态
/// - `media`: 媒体状态
/// - `id`: 文件 ID
/// - `range`: `Range` 头的值，没有时为 `None`
///
/// # 返回值
/// 可直接写成 HTTP 响应的一段字节
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let body = read_body(db, media, id, Some("bytes=0-1023"));
/// ```
fn read_body(db: &DbState, media: &MediaState, id: &str, range: Option<&str>) -> MediaBody {
  let row = match load_row(db, id) {
    Ok(row) => row,
    Err(err) => return MediaBody::error(status_of(&err), 0),
  };
  let mime = content_type(&row.mime);
  if row.encrypted {
    let Some(key) = media.key(&row.id) else {
      return MediaBody::error(401, 0);
    };
    return read_encrypted(db, &row, &mime, key, range);
  }
  read_plain(db, &row, &mime, range)
}

/// 读出未加密文件的一段
///
/// # 参数
/// - `db`: 数据库状态
/// - `row`: 文件记录
/// - `mime`: 响应类型
/// - `range`: `Range` 头
///
/// # 返回值
/// 这一段明文
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let body = read_plain(db, &row, mime, range);
/// ```
fn read_plain(db: &DbState, row: &Upload, mime: &str, range: Option<&str>) -> MediaBody {
  let path = match stored_path(db, row) {
    Ok(path) => path,
    Err(err) => return MediaBody::error(status_of(&err), 0),
  };
  let total = match File::open(&path).and_then(|file| file.metadata()) {
    Ok(meta) => meta.len(),
    Err(_) => return MediaBody::error(404, 0),
  };
  let (status, start, end) = match byte_window(total, range, allow_full(&row.kind, total)) {
    Ok(window) => window,
    Err(()) => return MediaBody::error(416, total),
  };
  let body = match read_at(&path, start, end) {
    Ok(body) => body,
    Err(err) => return MediaBody::error(status_of(&err), total),
  };
  sliced(status, mime, total, start, body)
}

/// 用已记下的密钥读出密文里的一段明文
///
/// # 参数
/// - `db`: 数据库状态
/// - `row`: 文件记录
/// - `mime`: 响应类型
/// - `key`: 这个文件的密钥
/// - `range`: `Range` 头
///
/// # 返回值
/// 这一段明文
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let body = read_encrypted(db, &row, mime, key, range);
/// ```
fn read_encrypted(db: &DbState, row: &Upload, mime: &str, key: FileKey, range: Option<&str>) -> MediaBody {
  let path = match stored_path(db, row) {
    Ok(path) => path,
    Err(err) => return MediaBody::error(status_of(&err), 0),
  };
  let file = match File::open(&path) {
    Ok(file) => file,
    Err(_) => return MediaBody::error(404, 0),
  };
  let mut reader = match CipherReader::with_key(file, key) {
    Ok(reader) => reader,
    Err(_) => return MediaBody::error(500, 0),
  };
  let total = reader.len();
  let (status, start, end) = match byte_window(total, range, allow_full(&row.kind, total)) {
    Ok(window) => window,
    Err(()) => return MediaBody::error(416, total),
  };
  if reader.seek(SeekFrom::Start(start)).is_err() {
    return MediaBody::error(500, total);
  }
  let mut body = vec![0u8; (end - start) as usize];
  if read_exact_io(&mut reader, &mut body).is_err() {
    return MediaBody::error(500, total);
  }
  sliced(status, mime, total, start, body)
}

/// 装一段成功的响应
///
/// # 参数
/// - `status`: 200 或 206
/// - `mime`: 响应类型
/// - `total`: 明文总长度
/// - `start`: 这段起点
/// - `body`: 这段字节
///
/// # 返回值
/// `length` 等于正文长度的响应
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let body = sliced(206, "video/mp4", total, 0, bytes);
/// ```
fn sliced(status: u16, mime: &str, total: u64, start: u64, body: Vec<u8>) -> MediaBody {
  let length = body.len() as u64;
  MediaBody { status, mime: mime.to_string(), total, start, length, body }
}

/// 取出记录并拒绝已封存的所属
///
/// # 参数
/// - `db`: 数据库状态
/// - `id`: 文件 ID
///
/// # 返回值
/// 可以读取的文件记录
///
/// # 错误
/// 不存在、已封存或数据库未解锁时返回对应的上传错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = load_row(db, id)?;
/// ```
fn load_row(db: &DbState, id: &str) -> Result<Upload, UploadError> {
  let row = db.with_conn(|conn| super::repository::get(conn, id.trim()))?;
  let sealed =
    db.with_conn(|conn| crate::owner::sealed(conn, &row.owner, &row.owner_id).map_err(|_| UploadError::Internal))?;
  if sealed {
    return Err(UploadError::OwnerSealed);
  }
  Ok(row)
}

/// 把记录里的相对路径解析成上传目录中的文件
///
/// # 参数
/// - `db`: 数据库状态
/// - `row`: 文件记录
///
/// # 返回值
/// 文件绝对路径
///
/// # 错误
/// 路径不是 `upload/` 下的普通相对路径时返回 `NotFound`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let path = stored_path(db, &row)?;
/// ```
fn stored_path(db: &DbState, row: &Upload) -> Result<std::path::PathBuf, UploadError> {
  if !safe_rel(&row.rel_path) {
    return Err(UploadError::NotFound);
  }
  Ok(resolve(db, &row.rel_path))
}

/// 路径必须待在上传目录里
///
/// # 参数
/// - `rel`: 相对路径
///
/// # 返回值
/// 以 `upload/` 开头且没有 `.` 或 `..` 时为 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(safe_rel("upload/image/a.jpg"));
/// assert!(!safe_rel("upload/../db/a.db"));
/// ```
fn safe_rel(rel: &str) -> bool {
  let rel = rel.replace('\\', "/");
  rel.starts_with("upload/") && !rel.split('/').any(|part| part.is_empty() || part == "." || part == "..")
}

/// 从文件的绝对偏移读出 `[start, end)`
///
/// # 参数
/// - `path`: 文件路径
/// - `start`: 起点，含
/// - `end`: 终点，不含
///
/// # 返回值
/// 这一段字节
///
/// # 错误
/// 打不开返回 `NotFound`，读取不完整返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let bytes = read_at(path, 0, 4)?;
/// ```
fn read_at(path: &Path, start: u64, end: u64) -> Result<Vec<u8>, UploadError> {
  let mut file = File::open(path).map_err(|_| UploadError::NotFound)?;
  file.seek(SeekFrom::Start(start)).map_err(|_| UploadError::Internal)?;
  let mut body = vec![0u8; (end - start) as usize];
  read_exact_io(&mut file, &mut body)?;
  Ok(body)
}

/// 读满缓冲区
///
/// # 参数
/// - `reader`: 字节来源
/// - `buf`: 要填满的缓冲区
///
/// # 返回值
/// 读满时返回空成功
///
/// # 错误
/// 提前结束或读取失败时返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// read_exact_io(&mut file, &mut buf)?;
/// ```
fn read_exact_io(reader: &mut impl Read, buf: &mut [u8]) -> Result<(), UploadError> {
  let mut filled = 0;
  while filled < buf.len() {
    match reader.read(&mut buf[filled..]) {
      Ok(0) => return Err(UploadError::Internal),
      Ok(n) => filled += n,
      Err(err) if err.kind() == io::ErrorKind::Interrupted => {}
      Err(_) => return Err(UploadError::Internal),
    }
  }
  Ok(())
}

/// 视频以外、不超过 40 MiB 的文件可以一次给完
///
/// # 参数
/// - `kind`: 文件类型
/// - `total`: 明文长度
///
/// # 返回值
/// 允许整份返回时为 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(allow_full("image", 1024));
/// assert!(!allow_full("video", 1024));
/// ```
fn allow_full(kind: &str, total: u64) -> bool {
  kind != "video" && total <= FULL_MAX
}

/// 把 `Range` 收成一段不超过 [`CHUNK`] 的半开区间
///
/// 没有 `Range` 时，允许整份就返回 200，否则返回开头的一段 206。区间不合法时返回 `Err(())`，调用方回应 416。
///
/// # 参数
/// - `total`: 明文总长度
/// - `range`: `Range` 头，例如 `bytes=0-1023`
/// - `full`: 没有 `Range` 时是否可以返回整份
///
/// # 返回值
/// 状态码、起点和终点（不含）
///
/// # 错误
/// 区间单位不是 `bytes`，或起点落在文件外时返回 `Err(())`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let (status, start, end) = byte_window(10, Some("bytes=3-7"), false).unwrap();
/// assert_eq!((status, start, end), (206, 3, 8));
/// ```
fn byte_window(total: u64, range: Option<&str>, full: bool) -> Result<(u16, u64, u64), ()> {
  if total == 0 {
    return Ok((200, 0, 0));
  }
  let Some(range) = range.map(str::trim).filter(|value| !value.is_empty()) else {
    if full || total <= CHUNK {
      return Ok((200, 0, total));
    }
    return Ok((206, 0, CHUNK));
  };
  let Some(spec) = range.strip_prefix("bytes=") else {
    return Err(());
  };
  let spec = spec.split(',').next().unwrap_or(spec).trim();
  if let Some(suffix) = spec.strip_prefix('-') {
    let n = parse_u64(suffix).filter(|n| *n > 0).ok_or(())?;
    let len = n.min(total).min(CHUNK);
    return Ok((206, total - len, total));
  }
  let Some((start_text, end_text)) = spec.split_once('-') else {
    return Err(());
  };
  let start = parse_u64(start_text).ok_or(())?;
  if start >= total {
    return Err(());
  }
  let end = if end_text.is_empty() { total } else { parse_u64(end_text).ok_or(())?.saturating_add(1).min(total) };
  if end <= start {
    return Err(());
  }
  let end = if end - start > CHUNK { start + CHUNK } else { end };
  Ok((206, start, end))
}

/// 只接受十进制数字
///
/// # 参数
/// - `text`: 数字文本
///
/// # 返回值
/// 解析出的整数。空串或含其他字符时为 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(parse_u64("12"), Some(12));
/// assert_eq!(parse_u64("1a"), None);
/// ```
fn parse_u64(text: &str) -> Option<u64> {
  if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
    return None;
  }
  text.parse().ok()
}

/// 解开后的开头像不像这种文件
///
/// 密码错了不会报错，只会得到另一串字节。图片和视频对一下常见文件头。
///
/// # 参数
/// - `kind`: 文件类型
/// - `name`: 原始文件名，用来看扩展名
/// - `head`: 明文开头
///
/// # 返回值
/// 像这种文件时为 `true`。其他文件无法判断，一律通过
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(looks_plain("image", "a.jpg", b"\xFF\xD8\xFF"));
/// assert!(!looks_plain("image", "a.jpg", b"nope"));
/// ```
fn looks_plain(kind: &str, name: &str, head: &[u8]) -> bool {
  match kind {
    "image" => image_magic(head) || ftyp(head),
    "video" => video_magic(name, head) || ftyp(head),
    "book" => book_magic(name, head),
    _ => true,
  }
}

/// 常见图片文件头
///
/// # 参数
/// - `head`: 明文开头
///
/// # 返回值
/// 像 JPEG、PNG、GIF、WebP、BMP、TIFF 或 JPEG XL 时为 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(image_magic(b"\x89PNG\r\n\x1a\n"));
/// ```
fn image_magic(head: &[u8]) -> bool {
  head.starts_with(b"\xFF\xD8\xFF")
    || head.starts_with(b"\x89PNG")
    || head.starts_with(b"GIF87a")
    || head.starts_with(b"GIF89a")
    || head.starts_with(b"BM")
    || head.starts_with(b"II*\x00")
    || head.starts_with(b"MM\x00*")
    || head.starts_with(b"\xFF\x0A")
    || head.starts_with(b"\x00\x00\x00\x0CJXL ")
    || (head.len() >= 12 && &head[0..4] == b"RIFF" && &head[8..12] == b"WEBP")
}

/// ISO 容器，AVIF 和 MP4 都是这个头
///
/// # 参数
/// - `head`: 明文开头
///
/// # 返回值
/// 第 4 个字节起是 `ftyp` 时为 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(ftyp(b"\x00\x00\x00\x18ftyp"));
/// ```
fn ftyp(head: &[u8]) -> bool {
  head.len() >= 8 && &head[4..8] == b"ftyp"
}

/// 常见视频文件头
///
/// # 参数
/// - `name`: 原始文件名
/// - `head`: 明文开头
///
/// # 返回值
/// 像 WebM、MKV、AVI、MPEG 或 TS 时为 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(video_magic("a.webm", b"\x1A\x45\xDF\xA3"));
/// ```
fn video_magic(name: &str, head: &[u8]) -> bool {
  let ebml = head.starts_with(b"\x1A\x45\xDF\xA3");
  let avi = head.len() >= 12 && &head[0..4] == b"RIFF" && &head[8..12] == b"AVI ";
  let mpeg = head.starts_with(b"\x00\x00\x01\xBA") || head.starts_with(b"\x00\x00\x01\xB3");
  ebml || avi || mpeg || ts_magic(name, head)
}

/// MPEG-TS 在 0 和 188 字节处都是 `0x47`
///
/// 只有扩展名是 `ts` 或 `m2ts` 时才认，避免任意字节碰巧对上。
///
/// # 参数
/// - `name`: 原始文件名
/// - `head`: 明文开头
///
/// # 返回值
/// 扩展名符合且同步字节在时为 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let mut head = vec![0u8; 189];
/// head[0] = 0x47;
/// head[188] = 0x47;
/// assert!(ts_magic("a.ts", &head));
/// ```
fn ts_magic(name: &str, head: &[u8]) -> bool {
  let ext = ext_of_name(name);
  if ext != "ts" && ext != "m2ts" {
    return false;
  }
  head.first() == Some(&0x47) && (head.len() <= 188 || head.get(188) == Some(&0x47))
}

/// 常见书籍文件头
///
/// # 参数
/// - `name`: 原始文件名
/// - `head`: 明文开头
///
/// # 返回值
/// 像 PDF、EPUB、MOBI，或文本是合法 UTF-8 时为 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(book_magic("a.pdf", b"%PDF-1.7"));
/// ```
fn book_magic(name: &str, head: &[u8]) -> bool {
  match ext_of_name(name).as_str() {
    "pdf" => head.starts_with(b"%PDF"),
    "epub" => head.starts_with(b"PK\x03\x04"),
    "mobi" | "azw" | "azw3" => mobi_magic(head),
    "txt" => std::str::from_utf8(head).is_ok(),
    _ => head.starts_with(b"%PDF") || head.starts_with(b"PK\x03\x04") || mobi_magic(head),
  }
}

/// Palm 数据库头后面的 MOBI 标记
///
/// # 参数
/// - `head`: 明文开头
///
/// # 返回值
/// 第 60 字节起是 `BOOKMOBI` 或 `TEXtREAd` 时为 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let mut head = vec![0u8; 68];
/// head[60..68].copy_from_slice(b"BOOKMOBI");
/// assert!(mobi_magic(&head));
/// ```
fn mobi_magic(head: &[u8]) -> bool {
  head.len() >= 68 && (&head[60..68] == b"BOOKMOBI" || &head[60..68] == b"TEXtREAd")
}

/// 取文件名的小写扩展名
///
/// 末尾是 `.env` 且前面还有扩展名时，用前面那个。加密入库的 `a.epub.env` 仍按 epub 识别。
///
/// # 参数
/// - `name`: 文件名
///
/// # 返回值
/// 扩展名的小写文本。没有点时为空串
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(ext_of_name("A.JPG"), "jpg");
/// assert_eq!(ext_of_name("a.epub.env"), "epub");
/// assert_eq!(ext_of_name("a.env"), "env");
/// ```
fn ext_of_name(name: &str) -> String {
  let path = Path::new(name.trim());
  let ext = path.extension().and_then(|item| item.to_str()).unwrap_or("").to_ascii_lowercase();
  if ext == "env" {
    let stem = path.file_stem().and_then(|item| item.to_str()).unwrap_or("");
    if let Some(inner) = Path::new(stem).extension().and_then(|item| item.to_str()) {
      if !inner.is_empty() {
        return inner.to_ascii_lowercase();
      }
    }
  }
  ext
}

/// 去掉 MIME 里的控制字符
///
/// # 参数
/// - `mime`: 记录里的 MIME
///
/// # 返回值
/// 可用的 `Content-Type`。空的或含控制字符时用 `application/octet-stream`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(content_type("video/mp4"), "video/mp4");
/// assert_eq!(content_type(""), "application/octet-stream");
/// ```
fn content_type(mime: &str) -> String {
  let clean = mime.trim();
  if clean.is_empty() || clean.bytes().any(|byte| byte.is_ascii_control()) {
    return "application/octet-stream".to_string();
  }
  clean.to_string()
}

/// 把上传错误收成 HTTP 状态
///
/// # 参数
/// - `err`: 上传错误
///
/// # 返回值
/// 404、403、401 或 500
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(status_of(&UploadError::NotFound), 404);
/// ```
fn status_of(err: &UploadError) -> u16 {
  match err {
    UploadError::NotFound | UploadError::ReferencedMissing => 404,
    UploadError::OwnerSealed => 403,
    UploadError::NeedPassword | UploadError::Password | UploadError::Encrypted | UploadError::Locked => 401,
    _ => 500,
  }
}

/// 把文件加解密错误收成上传错误
///
/// # 参数
/// - `err`: 文件加解密错误
///
/// # 返回值
/// 口令为空、格式不对，或内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(matches!(map_file(FileError::PasswordEmpty), UploadError::NeedPassword));
/// ```
fn map_file(err: FileError) -> UploadError {
  match err {
    FileError::PasswordEmpty => UploadError::NeedPassword,
    FileError::BadMagic | FileError::BadVersion | FileError::Truncated => UploadError::Format,
    FileError::Canceled | FileError::SamePath | FileError::Internal => UploadError::Internal,
  }
}

/// 拆开请求行
///
/// # 参数
/// - `text`: 请求头文本
///
/// # 返回值
/// 方法、路径和协议。请求行不完整时为 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let (method, path, version) = split_request("GET /m/a/b HTTP/1.1\r\n\r\n").unwrap();
/// assert_eq!(method, "GET");
/// assert_eq!(path, "/m/a/b");
/// assert_eq!(version, "HTTP/1.1");
/// ```
fn split_request(text: &str) -> Option<(&str, &str, &str)> {
  let line = text.lines().next()?;
  let mut parts = line.split_whitespace();
  Some((parts.next()?, parts.next()?, parts.next()?))
}

/// 取出一个请求头
///
/// # 参数
/// - `text`: 请求头文本
/// - `name`: 头名称，不区分大小写
///
/// # 返回值
/// 去掉空白的值
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(header_value("GET / HTTP/1.1\r\nRange: bytes=0-1\r\n", "range").as_deref(), Some("bytes=0-1"));
/// ```
fn header_value(text: &str, name: &str) -> Option<String> {
  text.lines().find_map(|line| {
    let (key, value) = line.split_once(':')?;
    key.trim().eq_ignore_ascii_case(name).then(|| value.trim().to_string())
  })
}

/// 从 `/m/{token}/{id}` 取出文件 ID
///
/// # 参数
/// - `path`: 请求路径
/// - `token`: 期望的路径口令
///
/// # 返回值
/// 口令相符且 ID 是 UUID 时返回 ID
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(file_id("/m/nope/00000000-0000-4000-8000-000000000000", "token").is_none());
/// ```
fn file_id<'a>(path: &'a str, token: &str) -> Option<&'a str> {
  let path = path.split('?').next().unwrap_or(path);
  let mut parts = path.split('/').filter(|part| !part.is_empty());
  if parts.next() != Some("m") {
    return None;
  }
  let got = parts.next()?;
  if !token_eq(got, token) {
    return None;
  }
  let id = parts.next()?;
  if parts.next().is_some() || !uuid_ok(id) {
    return None;
  }
  Some(id)
}

/// 比较两段路径口令
///
/// # 参数
/// - `got`: 请求里的口令
/// - `expect`: 这次进程的口令
///
/// # 返回值
/// 长度和每个字节都相同为 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(token_eq("abc", "abc"));
/// assert!(!token_eq("abc", "abd"));
/// ```
fn token_eq(got: &str, expect: &str) -> bool {
  if got.len() != expect.len() {
    return false;
  }
  got.bytes().zip(expect.bytes()).fold(0u8, |acc, (left, right)| acc | (left ^ right)) == 0
}

/// ID 必须是带连字符的 36 位十六进制
///
/// # 参数
/// - `id`: 路径里的文件 ID
///
/// # 返回值
/// 形如 UUID 时为 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(uuid_ok("00000000-0000-4000-8000-000000000000"));
/// assert!(!uuid_ok("../secret"));
/// ```
fn uuid_ok(id: &str) -> bool {
  let bytes = id.as_bytes();
  if bytes.len() != 36 {
    return false;
  }
  bytes.iter().enumerate().all(|(index, byte)| match index {
    8 | 13 | 18 | 23 => *byte == b'-',
    _ => byte.is_ascii_hexdigit(),
  })
}

/// 写出跨源预检
///
/// # 参数
/// - `stream`: 这条连接
///
/// # 返回值
/// 写完时返回空成功
///
/// # 错误
/// 写入失败时返回 `io::Error`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// write_options(&mut stream)?;
/// ```
fn write_options(stream: &mut TcpStream) -> io::Result<()> {
  let text = "\
HTTP/1.1 204 No Content\r\n\
Access-Control-Allow-Origin: *\r\n\
Access-Control-Allow-Methods: GET, HEAD, OPTIONS\r\n\
Access-Control-Allow-Headers: Range\r\n\
Content-Length: 0\r\n\
Connection: close\r\n\
\r\n";
  stream.write_all(text.as_bytes())
}

/// 把一段明文写成 HTTP 响应
///
/// # 参数
/// - `stream`: 这条连接
/// - `body`: 要写的响应
/// - `head_only`: 为真时不写正文，`Content-Length` 仍是这段的长度
///
/// # 返回值
/// 写完时返回空成功
///
/// # 错误
/// 写入失败时返回 `io::Error`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// write_body(&mut stream, &body, false)?;
/// ```
fn write_body(stream: &mut TcpStream, body: &MediaBody, head_only: bool) -> io::Result<()> {
  let reason = reason(body.status);
  let mut text = format!(
    "HTTP/1.1 {status} {reason}\r\nContent-Type: {mime}\r\nAccept-Ranges: bytes\r\nContent-Length: {length}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Expose-Headers: Accept-Ranges, Content-Range, Content-Length\r\nConnection: close\r\n",
    status = body.status,
    mime = body.mime,
    length = body.length,
  );
  if body.status == 206 || body.status == 416 {
    let range = if body.status == 416 || body.length == 0 {
      format!("bytes */{}", body.total)
    } else {
      format!("bytes {}-{}/{}", body.start, body.start + body.length - 1, body.total)
    };
    text.push_str(&format!("Content-Range: {range}\r\n"));
  }
  text.push_str("\r\n");
  stream.write_all(text.as_bytes())?;
  if !head_only && !body.body.is_empty() {
    stream.write_all(&body.body)?;
  }
  Ok(())
}

/// HTTP 状态码的原因短语
///
/// # 参数
/// - `status`: 状态码
///
/// # 返回值
/// 对应的英文短语。不认识的码返回 `Error`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(reason(206), "Partial Content");
/// ```
fn reason(status: u16) -> &'static str {
  match status {
    200 => "OK",
    206 => "Partial Content",
    401 => "Unauthorized",
    403 => "Forbidden",
    404 => "Not Found",
    405 => "Method Not Allowed",
    416 => "Range Not Satisfiable",
    _ => "Error",
  }
}

#[cfg(test)]
mod tests {
  use std::fs;

  use super::*;
  use crate::chacha::chacha::encrypt_path;
  use crate::owner::Owner;
  use crate::upload::dto::UploadInput;
  use crate::upload::storage::{rel_path, resolve};

  fn setup() -> (tempfile::TempDir, DbState) {
    let root = tempfile::tempdir().expect("temp");
    let db = DbState::new(root.path().join("db")).with_migrators(vec![crate::upload::migrate]);
    db.create("test-password-123", "test-password-123").expect("create");
    (root, db)
  }

  fn store(db: &DbState, id: &str, name: &str, kind: &str, bytes: &[u8], password: &str) -> Upload {
    let rel = rel_path(kind, id, Some("jpg"));
    let dest = resolve(db, &rel);
    fs::create_dir_all(dest.parent().expect("parent")).expect("dir");
    if password.is_empty() {
      fs::write(&dest, bytes).expect("write");
    } else {
      let source = dest.with_extension("src");
      fs::write(&source, bytes).expect("source");
      encrypt_path(&source, &dest, password, &mut |_, _| Ok(())).expect("encrypt");
    }
    db.with_conn(|conn| {
      super::super::repository::insert(
        conn,
        id,
        Owner::Loose,
        &UploadInput {
          owner: Owner::Loose.as_str().into(),
          owner_id: "loose".into(),
          kind: kind.into(),
          rel_path: rel,
          mime: "image/jpeg".into(),
          name: name.into(),
          size: bytes.len() as i64,
          sort: 0,
          locked: false,
        },
        !password.is_empty(),
      )
    })
    .expect("insert")
  }

  #[test]
  fn plain_range_reads_the_middle() {
    let (_root, db) = setup();
    let id = new_uuid_v4();
    store(&db, &id, "a.jpg", "image", b"\xFF\xD8\xFFhello-range", "");
    let media = MediaState::offline();
    open_media(&db, &media, &id, String::new()).expect("open");
    let body = read_body(&db, &media, &id, Some("bytes=3-7"));
    assert_eq!(body.status, 206);
    assert_eq!(body.body, b"hello");
  }

  #[test]
  fn encrypted_seek_uses_plaintext_offset() {
    let (_root, db) = setup();
    let id = new_uuid_v4();
    let plain = b"\xFF\xD8\xFFhello-range-bytes";
    store(&db, &id, "a.jpg", "image", plain, "file-password");
    let media = MediaState::offline();
    assert!(matches!(
      open_media(&db, &media, &id, String::new()),
      Err(UploadError::NeedPassword)
    ));
    assert!(matches!(
      open_media(&db, &media, &id, "nope".into()),
      Err(UploadError::Password)
    ));
    open_media(&db, &media, &id, "file-password".into()).expect("open");
    let body = read_body(&db, &media, &id, Some("bytes=3-7"));
    assert_eq!(body.status, 206);
    assert_eq!(body.body, b"hello");
    assert_eq!(body.total, plain.len() as u64);
  }

  #[test]
  fn window_caps_open_ranges() {
    assert_eq!(byte_window(10, Some("bytes=3-7"), false).unwrap(), (206, 3, 8));
    assert!(byte_window(50, Some("bytes=100-"), false).is_err());
    let (_status, start, end) = byte_window(5_000_000, None, false).unwrap();
    assert_eq!((start, end), (0, CHUNK));
  }

  #[test]
  fn export_writes_the_chosen_folder() {
    let (root, db) = setup();
    let media = MediaState::offline();
    let dir = root.path().join("out");
    fs::create_dir(&dir).expect("out");
    let plain_id = new_uuid_v4();
    store(&db, &plain_id, "photo.jpg", "image", b"\xFF\xD8\xFFplain", "");
    let saved = export_file(&db, &media, &plain_id, dir.to_str().expect("dir"), String::new()).expect("export");
    assert!(saved.ends_with("photo.jpg"));
    assert_eq!(fs::read(&saved).expect("read"), b"\xFF\xD8\xFFplain");
    let again = export_file(&db, &media, &plain_id, dir.to_str().expect("dir"), String::new()).expect("again");
    assert!(again.ends_with("photo (2).jpg"));

    let enc_id = new_uuid_v4();
    let bytes = b"\xFF\xD8\xFFsecret";
    store(&db, &enc_id, "secret.jpg.env", "image", bytes, "file-password");
    assert!(matches!(
      export_file(&db, &media, &enc_id, dir.to_str().expect("dir"), String::new()),
      Err(UploadError::NeedPassword)
    ));
    let decrypted =
      export_file(&db, &media, &enc_id, dir.to_str().expect("dir"), "file-password".into()).expect("decrypt");
    assert!(decrypted.ends_with("secret.jpg"));
    assert_eq!(fs::read(decrypted).expect("read"), bytes);
  }

  #[test]
  fn clear_forgets_the_key() {
    let (_root, db) = setup();
    let id = new_uuid_v4();
    store(&db, &id, "a.jpg", "image", b"\xFF\xD8\xFFhi", "file-password");
    let media = MediaState::offline();
    open_media(&db, &media, &id, "file-password".into()).expect("open");
    media.clear();
    assert!(matches!(
      open_media(&db, &media, &id, String::new()),
      Err(UploadError::NeedPassword)
    ));
  }
}
