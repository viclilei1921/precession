use std::path::Path;

/// 标题和字幕用的系统字体。返回普通路径，滤镜里再转义。
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

/// 默认字体所在目录，给字幕的 `fontsdir` 用。
pub fn default_fonts_dir() -> String {
  let font = get_default_font_path();
  Path::new(&font)
    .parent()
    .filter(|path| !path.as_os_str().is_empty())
    .map(|path| path.to_string_lossy().replace('\\', "/"))
    .unwrap_or(font)
}
