use std::path::Path;

use tauri::{AppHandle, Emitter, Manager};

use super::folder::is_image;

/// 系统打开图片时暂存的路径。前端取走后清空。
pub struct PendingOpen(std::sync::Mutex<Option<String>>);

impl Default for PendingOpen {
  /// 启动时还没有待打开的图片
  fn default() -> Self {
    Self(std::sync::Mutex::new(None))
  }
}

/// 从进程参数里找出第一张图片，记下来并通知前端
///
/// 参数里没有图片时什么都不做。开发模式带上的地址和开关会被跳过。
///
/// # 参数
/// - `app`: 应用句柄
/// - `args`: 进程参数，第一项通常是程序自己
///
/// # Panics
/// 不会主动 panic
pub fn note_open(app: &AppHandle, args: &[String]) {
  let Some(path) = image_from_args(args) else {
    return;
  };
  if let Some(state) = app.try_state::<PendingOpen>() {
    if let Ok(mut slot) = state.0.lock() {
      *slot = Some(path.clone());
    }
  }
  let _ = app.emit("viewer-open", path);
}

/// 取走暂存的图片路径
///
/// 没有暂存时返回空。取走后再次调用也是空，直到下一次 [`note_open`]。
///
/// # 参数
/// - `app`: 应用句柄
///
/// # 返回值
/// 暂存的路径
///
/// # Panics
/// 不会主动 panic
pub fn take(app: &AppHandle) -> Option<String> {
  let state = app.try_state::<PendingOpen>()?;
  state.0.lock().ok()?.take()
}

/// 参数里第一张存在的图片
///
/// 跳过程序名和以 `-` 开头的开关。`file://` 会先解开。
///
/// # 参数
/// - `args`: 进程参数
///
/// # 返回值
/// 找到的图片路径
///
/// # Panics
/// 不会主动 panic
pub fn image_from_args(args: &[String]) -> Option<String> {
  args.iter().skip(1).find_map(|arg| {
    let path = normalize_arg(arg);
    if path.is_empty() || path.starts_with('-') {
      return None;
    }
    let file = Path::new(&path);
    if file.is_file() && is_image(file) { Some(path) } else { None }
  })
}

/// 去掉引号，并解开 `file://`
fn normalize_arg(arg: &str) -> String {
  let decoded = percent_decode(arg.trim().trim_matches('"'));
  from_file_url(&decoded).unwrap_or(decoded)
}

/// 把 `file://` 收成本地路径。不是这种地址时返回空
fn from_file_url(value: &str) -> Option<String> {
  let rest = value.strip_prefix("file://")?;
  let rest = rest.strip_prefix("localhost").unwrap_or(rest);
  #[cfg(windows)]
  {
    let rest = rest.trim_start_matches('/');
    Some(rest.replace('/', "\\"))
  }
  #[cfg(not(windows))]
  {
    if rest.starts_with('/') { Some(rest.to_string()) } else { Some(format!("/{rest}")) }
  }
}

/// 解开 `%HH`。不合法的百分号原样留下
fn percent_decode(input: &str) -> String {
  let bytes = input.as_bytes();
  let mut out = Vec::with_capacity(bytes.len());
  let mut index = 0;
  while index < bytes.len() {
    if bytes[index] == b'%' && index + 2 < bytes.len() {
      if let Ok(byte) = u8::from_str_radix(&input[index + 1..index + 3], 16) {
        out.push(byte);
        index += 3;
        continue;
      }
    }
    out.push(bytes[index]);
    index += 1;
  }
  String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn picks_an_image_argument() {
    let dir = tempfile::tempdir().expect("tempdir");
    let image = dir.path().join("a.png");
    std::fs::write(&image, b"p").expect("write");
    let args = vec!["precession.exe".to_string(), image.to_string_lossy().into_owned()];
    assert_eq!(image_from_args(&args).as_deref(), Some(image.to_string_lossy().as_ref()));
  }

  #[test]
  fn skips_flags_and_other_files() {
    let dir = tempfile::tempdir().expect("tempdir");
    let note = dir.path().join("a.txt");
    std::fs::write(&note, b"t").expect("write");
    let args = vec!["precession.exe".to_string(), "--dev".to_string(), note.to_string_lossy().into_owned()];
    assert!(image_from_args(&args).is_none());
  }
}
