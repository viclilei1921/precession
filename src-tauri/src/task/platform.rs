//! 任务在各平台上怎么跑侧车、怎么在文件管理器里打开路径。手机端这些入口直接拒绝。

/// 桌面：跑侧车，并用系统文件管理器打开路径。
#[cfg(desktop)]
mod imp {
  use std::sync::Arc;

  use tauri::AppHandle;

  use super::super::error::TaskError;
  use super::super::job::JobCtx;
  use super::super::queue::Ticket;
  use super::super::service::Spec;

  /// 允许继续准备桌面任务
  ///
  /// # 返回值
  /// 成功时无额外值
  ///
  /// # 错误
  /// 不返回错误
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// ensure_desktop()?;
  /// ```
  pub fn ensure_desktop() -> Result<(), TaskError> {
    Ok(())
  }

  /// 在当前线程上阻塞跑完侧车任务
  ///
  /// # 参数
  /// - `app`: Tauri 应用句柄
  /// - `ticket`: 任务票
  ///
  /// # 返回值
  /// 成功时返回 `None`
  ///
  /// # 错误
  /// 侧车取消变成 `TaskError::Canceled`，其他侧车错误变成 `TaskError::Failed`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let upload_id = run_sidecar(app, ticket)?;
  /// ```
  pub fn run_sidecar(app: &AppHandle, ticket: &Ticket) -> Result<Option<String>, TaskError> {
    let job = JobCtx::new(app.clone(), ticket.id.clone(), Arc::clone(&ticket.cancel));
    tauri::async_runtime::block_on(dispatch_sidecar(app, &job, &ticket.spec)).map(|()| None).map_err(TaskError::from)
  }

  /// 按规格调用对应的视频或图片侧车
  ///
  /// # 参数
  /// - `app`: Tauri 应用句柄
  /// - `job`: 任务执行期句柄
  /// - `spec`: 任务规格
  ///
  /// # 返回值
  /// 成功时无额外值
  ///
  /// # 错误
  /// 文件、导入、上传文件规格返回 `SidecarError::Invalid`，消息是「不是转码任务」。其余失败返回侧车错误
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// dispatch_sidecar(app, job, spec).await?;
  /// ```
  async fn dispatch_sidecar(app: &AppHandle, job: &JobCtx, spec: &Spec) -> Result<(), crate::sidecar::SidecarError> {
    match spec {
      Spec::ConvertVideo { input, output, encode } => crate::ffmpeg::run_convert(app, job, input, output, encode).await,
      Spec::CutVideo { input, output, segments, encode } => {
        crate::ffmpeg::run_cut(app, job, input, output, segments.clone(), encode).await
      }
      Spec::MergeVideo { inputs, output, encode } => {
        crate::ffmpeg::run_merge_smart(app, job, inputs.clone(), output, encode).await
      }
      Spec::AppendVideo { base, inputs, output, encode } => {
        crate::ffmpeg::run_append_smart(app, job, base, inputs.clone(), output, encode).await
      }
      Spec::ConvertAvif { input, output, params } => {
        crate::image::run_convert_avif(app, job, input, output, params.clone()).await
      }
      Spec::ConvertJxl { input, output, params } => {
        crate::image::run_convert_jxl(app, job, input, output, params.clone()).await
      }
      Spec::File { .. } | Spec::Import { .. } | Spec::Upload { .. } => {
        Err(crate::sidecar::SidecarError::invalid("不是转码任务"))
      }
    }
  }

  /// 在系统文件管理器中打开路径
  ///
  /// macOS 用 `open`，Windows 用资源管理器，其他桌面用 `xdg-open`。
  ///
  /// # 参数
  /// - `path`: 要打开的文件或目录
  ///
  /// # 返回值
  /// 成功时无额外值
  ///
  /// # 错误
  /// 路径不存在或启动系统程序失败时返回 `TaskError`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// reveal(path)?;
  /// ```
  pub fn reveal(path: &str) -> Result<(), TaskError> {
    reveal_in_folder(path)
  }

  /// 用 macOS 的 `open` 打开文件或其所在目录
  ///
  /// 路径是文件时加 `-R` 选中它。否则父目录存在就打开父目录。
  ///
  /// # 参数
  /// - `path`: 要打开的路径
  ///
  /// # 返回值
  /// 成功时无额外值
  ///
  /// # 错误
  /// 既不是文件、父目录也不存在时返回 `TaskError::Missing`。`open` 启动失败时返回 `TaskError::Failed`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// reveal_in_folder(path)?;
  /// ```
  #[cfg(target_os = "macos")]
  fn reveal_in_folder(path: &str) -> Result<(), TaskError> {
    let target = std::path::Path::new(path);
    let mut command = std::process::Command::new("open");
    if target.is_file() {
      command.args(["-R", path]);
    } else if let Some(parent) = target.parent().filter(|parent| parent.is_dir()) {
      command.arg(parent);
    } else {
      return Err(TaskError::Missing);
    }
    command.spawn().map_err(|err| TaskError::Failed(err.to_string()))?;
    Ok(())
  }

  /// 用资源管理器打开文件或其所在目录
  ///
  /// 路径是文件时带 `/select,` 选中它。否则父目录存在就打开父目录。
  ///
  /// # 参数
  /// - `path`: 要打开的路径
  ///
  /// # 返回值
  /// 成功时无额外值
  ///
  /// # 错误
  /// 既不是文件、父目录也不存在时返回 `TaskError::Missing`。`explorer` 启动失败时返回 `TaskError::Failed`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// reveal_in_folder(path)?;
  /// ```
  #[cfg(target_os = "windows")]
  fn reveal_in_folder(path: &str) -> Result<(), TaskError> {
    use std::os::windows::process::CommandExt;

    let target = std::path::Path::new(path);
    let mut command = std::process::Command::new("explorer");
    if target.is_file() {
      command.raw_arg(format!("/select,{path}"));
    } else if let Some(parent) = target.parent().filter(|parent| parent.is_dir()) {
      command.arg(parent);
    } else {
      return Err(TaskError::Missing);
    }
    command.spawn().map_err(|err| TaskError::Failed(err.to_string()))?;
    Ok(())
  }

  /// 用 `xdg-open` 打开目录
  ///
  /// 路径本身是目录就打开它，否则打开父目录；没有父目录时仍把原路径交给 `xdg-open`。
  ///
  /// # 参数
  /// - `path`: 要打开的路径
  ///
  /// # 返回值
  /// 成功时无额外值
  ///
  /// # 错误
  /// `xdg-open` 启动失败时返回 `TaskError::Failed`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// reveal_in_folder(path)?;
  /// ```
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  fn reveal_in_folder(path: &str) -> Result<(), TaskError> {
    let target = std::path::Path::new(path);
    let folder = if target.is_dir() { target } else { target.parent().unwrap_or(target) };
    std::process::Command::new("xdg-open").arg(folder).spawn().map_err(|err| TaskError::Failed(err.to_string()))?;
    Ok(())
  }
}

/// 手机端不能跑侧车，也不能打开系统文件管理器。
#[cfg(not(desktop))]
mod imp {
  use tauri::AppHandle;

  use super::super::error::TaskError;
  use super::super::queue::Ticket;

  /// 拒绝只在桌面存在的准备步骤
  ///
  /// # 返回值
  /// 不返回成功
  ///
  /// # 错误
  /// 始终返回 `TaskError::DesktopOnly`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// ensure_desktop()?;
  /// ```
  pub fn ensure_desktop() -> Result<(), TaskError> {
    Err(TaskError::DesktopOnly)
  }

  /// 手机端不跑侧车
  ///
  /// # 参数
  /// - `app`: Tauri 应用句柄，手机端不使用
  /// - `ticket`: 任务票，手机端不使用
  ///
  /// # 返回值
  /// 不返回成功
  ///
  /// # 错误
  /// 始终返回 `TaskError::DesktopOnly`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let upload_id = run_sidecar(app, ticket)?;
  /// ```
  pub fn run_sidecar(app: &AppHandle, ticket: &Ticket) -> Result<Option<String>, TaskError> {
    let _ = (app, ticket);
    Err(TaskError::DesktopOnly)
  }

  /// 手机端没有系统文件管理器入口
  ///
  /// # 参数
  /// - `path`: 要打开的路径，手机端不使用
  ///
  /// # 返回值
  /// 不返回成功
  ///
  /// # 错误
  /// 始终返回 `TaskError::DesktopOnly`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// reveal(path)?;
  /// ```
  pub fn reveal(path: &str) -> Result<(), TaskError> {
    let _ = path;
    Err(TaskError::DesktopOnly)
  }
}

pub(super) use imp::{ensure_desktop, reveal, run_sidecar};
