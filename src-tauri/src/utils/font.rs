use std::path::Path;

/// 合并、追加视频时，给 drawtext 用的系统默认字体。
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
      if cfg!(target_os = "windows") {
        let clean_font_path = path.replace("\\", "/");
        return clean_font_path.replace(":", "\\\\:");
      }
      return path.to_string();
    }
  }

  tauri_plugin_log::log::warn!("warn: no default font found, ffmpeg may fail.");
  "arial.ttf".to_string()
}
