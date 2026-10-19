use std::path::Path;

/// 返回第一个存在的系统字体路径
///
/// Windows、macOS 和其他系统各查一组候选。命中后把反斜杠换成 `/`。都不存在时记警告并返回 `arial.ttf`。
///
/// # 返回值
/// 字体文件路径
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let path = get_default_font_path();
/// ```
pub fn get_default_font_path() -> String {
  let candidates = if cfg!(target_os = "windows") {
    vec![
      r"C:\Windows\Fonts\msyh.ttc",
      r"C:\Windows\Fonts\msyh.ttf",
      r"C:\Windows\Fonts\simhei.ttf",
      r"C:\Windows\Fonts\segoeui.ttf",
      r"C:\Windows\Fonts\arial.ttf",
    ]
  } else if cfg!(target_os = "macos") {
    vec![
      "/System/Library/Fonts/PingFang.ttc",
      "/Library/Fonts/Arial Unicode.ttf",
      "/System/Library/Fonts/STHeiti Light.ttc",
      "/System/Library/Fonts/Helvetica.ttc",
    ]
  } else {
    vec![
      "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
      "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
      "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
      "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    ]
  };

  for path in candidates {
    if Path::new(path).exists() {
      return path.replace('\\', "/");
    }
  }

  tauri_plugin_log::log::warn!("warn: no default font found, ffmpeg may fail.");
  "arial.ttf".to_string()
}

/// 返回默认字体所在目录
///
/// 父目录缺失或为空时，返回字体路径本身。
///
/// # 返回值
/// 目录路径；没有可用父目录时为字体路径
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let dir = default_fonts_dir();
/// ```
pub fn default_fonts_dir() -> String {
  let font = get_default_font_path();
  Path::new(&font)
    .parent()
    .filter(|path| !path.as_os_str().is_empty())
    .map(|path| path.to_string_lossy().replace('\\', "/"))
    .unwrap_or(font)
}
