//! 侧车版本在各平台上怎么探测。手机端不启动进程。

/// 桌面：并行启动三个侧车，每个最多等 5 秒。
#[cfg(desktop)]
mod imp {
  use std::time::Duration;

  use tauri::AppHandle;
  use tauri_plugin_shell::ShellExt;

  use super::super::constants::{SIDECAR_AVIFENC, SIDECAR_FFMPEG, UNAVAILABLE};
  use super::super::service::{SidecarVersions, version_from_output};

  /// 侧车在时限内没有退出
  const TIMED_OUT: &str = "超时";

  /// 单个侧车最多等待的时间
  const PROBE_TIMEOUT: Duration = Duration::from_secs(5);

  /// cjxl 侧车名
  const SIDECAR_CJXL: &str = "cjxl";

  /// FFmpeg 的版本参数
  const ARG_FFMPEG_VERSION: &str = "-version";

  /// cjxl 和 avifenc 的版本参数
  const ARG_VERSION: &str = "--version";

  /// 并行读取三个侧车的版本
  ///
  /// 不写入转码用的进程槽。
  ///
  /// # 参数
  /// - `app`: Tauri 应用句柄
  ///
  /// # 返回值
  /// 三个侧车的版本文本
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let versions = probe_sidecars(app).await;
  /// ```
  pub async fn probe_sidecars(app: AppHandle) -> SidecarVersions {
    let (ffmpeg, cjxl, avifenc) = tokio::join!(
      probe_one(&app, SIDECAR_FFMPEG, &[ARG_FFMPEG_VERSION]),
      probe_one(&app, SIDECAR_CJXL, &[ARG_VERSION]),
      probe_one(&app, SIDECAR_AVIFENC, &[ARG_VERSION]),
    );
    tauri_plugin_log::log::info!("sidecar versions ffmpeg={ffmpeg} cjxl={cjxl} avifenc={avifenc}");
    SidecarVersions { ffmpeg, cjxl, avifenc }
  }

  /// 启动一个侧车并截取版本
  ///
  /// 不放入转码进程槽。启动失败返回「不可用」，超时返回「超时」。
  ///
  /// # 参数
  /// - `app`: Tauri 应用句柄
  /// - `name`: 侧车名，`ffmpeg`、`cjxl` 或 `avifenc`
  /// - `args`: 版本参数
  ///
  /// # 返回值
  /// 短版本字符串，或「不可用」「超时」
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let version = probe_one(app, "ffmpeg", &["-version"]).await;
  /// ```
  async fn probe_one(app: &AppHandle, name: &str, args: &[&str]) -> String {
    let command = match app.shell().sidecar(name) {
      Ok(command) => command.args(args),
      Err(err) => {
        tauri_plugin_log::log::warn!("sidecar {name} version: {err}");
        return UNAVAILABLE.to_string();
      }
    };
    match tokio::time::timeout(PROBE_TIMEOUT, command.output()).await {
      Ok(Ok(output)) => version_from_output(name, &output.stdout, &output.stderr),
      Ok(Err(err)) => {
        tauri_plugin_log::log::warn!("sidecar {name} version: {err}");
        UNAVAILABLE.to_string()
      }
      Err(_elapsed) => TIMED_OUT.to_string(),
    }
  }
}

/// 手机端没有打包侧车，三项都返回「未包含」。
#[cfg(not(desktop))]
mod imp {
  use tauri::AppHandle;

  use super::super::service::SidecarVersions;

  /// 当前平台没有打包侧车
  const NOT_BUNDLED: &str = "未包含";

  /// 手机端不启动侧车
  ///
  /// # 参数
  /// - `_app`: Tauri 应用句柄，手机端不使用
  ///
  /// # 返回值
  /// 三项都是「未包含」
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let versions = probe_sidecars(app).await;
  /// ```
  pub async fn probe_sidecars(_app: AppHandle) -> SidecarVersions {
    SidecarVersions { ffmpeg: NOT_BUNDLED.to_string(), cjxl: NOT_BUNDLED.to_string(), avifenc: NOT_BUNDLED.to_string() }
  }
}

pub(super) use imp::probe_sidecars;
