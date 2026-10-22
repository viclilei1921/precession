//! 图片转码两侧共用的扩展名和侧车启动。

use std::path::Path;
use std::time::{Duration, Instant};

use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

use crate::sidecar::{SidecarError, SidecarResult, clear_current_child, secure_spawn_step};
use crate::task::JobCtx;

/// 枚举与转码共用的宽扩展名（非侧车原生格式经 PNG 桥接）
pub const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "bmp", "gif", "tif", "tiff"];

/// 创建输出路径的父目录
///
/// 没有父目录，或父目录是空路径时什么都不做。
///
/// # 参数
/// - `output_path`: 输出路径
///
/// # 返回值
/// 成功时无额外值
///
/// # 错误
/// 创建目录失败时返回 `SidecarError::Io`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// ensure_output_parent(output_path).await?;
/// ```
pub async fn ensure_output_parent(output_path: &str) -> SidecarResult<()> {
  let out = Path::new(output_path);
  if let Some(parent) = out.parent() {
    if !parent.as_os_str().is_empty() {
      std::fs::create_dir_all(parent).map_err(|e| SidecarError::io(format!("创建输出目录失败: {e}")))?;
    }
  }
  Ok(())
}

/// 启动侧车编码并上报进度
///
/// 开始时进度记 10。输出到达且距上次上报至少 500 毫秒时，进度记 40 并带上已用秒数。标准错误尾部最多留 4000 字节。退出码 0 时进度记 100。
///
/// # 参数
/// - `app`: Tauri 应用句柄
/// - `job`: 任务执行期句柄
/// - `sidecar_name`: 侧车名
/// - `args`: 参数
/// - `progress_label`: 进度说明
///
/// # 返回值
/// 成功时无额外值
///
/// # 错误
/// 取消返回 `SidecarError::Canceled`。启动失败返回 `Spawn`。非 0 退出返回 `Exit`，并附上错误尾部最后最多 800 字节。没有退出码返回 `Exit`，消息是「异常退出（无退出码）」。循环结束仍未退出返回 `Exit`，消息是「进程意外结束」
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// run_sidecar_encode(app, job, "avifenc", &args, label).await?;
/// ```
pub async fn run_sidecar_encode(
  app: &AppHandle,
  job: &JobCtx,
  sidecar_name: &str,
  args: &[String],
  progress_label: &str,
) -> SidecarResult<()> {
  let encode_started = Instant::now();
  job.report(10.0, progress_label).await;

  let shell = app.shell();
  tauri_plugin_log::log::info!("{sidecar_name} {}", args.join(" "));
  let (mut rx, child) = shell
    .sidecar(sidecar_name)
    .map_err(|e| SidecarError::sidecar_spawn(sidecar_name, e))?
    .args(args)
    .spawn()
    .map_err(|e| SidecarError::sidecar_spawn(sidecar_name, e))?;

  secure_spawn_step(app, job, child).await?;

  let mut stderr_tail = String::new();
  let mut last_tick = Instant::now();

  while let Some(event) = rx.recv().await {
    match event {
      CommandEvent::Stderr(line) | CommandEvent::Stdout(line) => {
        if job.is_canceled().await {
          break;
        }
        let text = String::from_utf8_lossy(&line);
        let trimmed = text.trim();
        if !trimmed.is_empty() && stderr_tail.len() < 4000 {
          if !stderr_tail.is_empty() {
            stderr_tail.push('\n');
          }
          stderr_tail.push_str(trimmed);
          if stderr_tail.len() > 4000 {
            stderr_tail.truncate(4000);
          }
        }
        if last_tick.elapsed() >= Duration::from_millis(500) {
          last_tick = Instant::now();
          let secs = encode_started.elapsed().as_secs();
          job.report(40.0, &format!("{progress_label} · 已用 {secs}s")).await;
        }
      }
      CommandEvent::Terminated(status) => {
        clear_current_child(app).await;
        if job.is_canceled().await {
          return Err(SidecarError::canceled());
        }
        if let Some(code) = status.code {
          if code == 0 {
            job.report(100.0, "完成").await;
            tauri_plugin_log::log::info!("{sidecar_name} encode done: elapsed={:?}", encode_started.elapsed());
            return Ok(());
          }
          let detail = if stderr_tail.is_empty() {
            String::new()
          } else {
            let snippet =
              if stderr_tail.len() > 800 { &stderr_tail[stderr_tail.len() - 800..] } else { stderr_tail.as_str() };
            format!("：{snippet}")
          };
          return Err(SidecarError::sidecar_exit(sidecar_name, format!("退出代码 {code}{detail}")));
        }
        return Err(SidecarError::sidecar_exit(sidecar_name, "异常退出（无退出码）"));
      }
      _ => {}
    }
  }

  clear_current_child(app).await;

  if job.is_canceled().await {
    return Err(SidecarError::canceled());
  }

  Err(SidecarError::sidecar_exit(sidecar_name, "进程意外结束"))
}
