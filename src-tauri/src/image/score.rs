//! 转码成功后用 SSIMULACRA2 比对原图。
//!
//! 打分失败不改转码结果。只有取消会中断任务。AVIF 先用 avifdec 解成临时 PNG：8-bit 保持 8-bit，10-bit 和 12-bit 写成 16-bit，避免收成 8 位。JPEG XL 直接交给 ssimulacra2。

use std::path::{Path, PathBuf};
use std::time::Instant;

use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

use crate::image::models::{AvifDepth, ImageBand, ImageQuality, ImageScoreProblem};
use crate::sidecar::cache::get_cache_temp_dir;
use crate::sidecar::{SidecarError, SidecarResult, clear_current_child, secure_spawn_step};
use crate::task::JobCtx;

/// 按 SSIMULACRA2 的刻度分档
///
/// 90、70、50、30 是这个指标自己的切分。
///
/// # 参数
/// - `score`: SSIMULACRA2 分数。可以是负数
///
/// # 返回值
/// 分数对应的档
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(quality_band(90.0), ImageBand::Indistinguishable);
/// assert_eq!(quality_band(29.9), ImageBand::Degraded);
/// ```
pub fn quality_band(score: f64) -> ImageBand {
  if score >= 90.0 {
    ImageBand::Indistinguishable
  } else if score >= 70.0 {
    ImageBand::High
  } else if score >= 50.0 {
    ImageBand::Medium
  } else if score >= 30.0 {
    ImageBand::Visible
  } else {
    ImageBand::Degraded
  }
}

/// 从侧车输出里取出第一个有限浮点数
///
/// # 参数
/// - `text`: 标准输出或标准错误
///
/// # 返回值
/// 找到有限数字时返回它，否则 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(parse_score("72.413940\n"), Some(72.413940));
/// assert_eq!(parse_score("没有分数"), None);
/// ```
pub fn parse_score(text: &str) -> Option<f64> {
  for token in text.split_whitespace() {
    let token = token.trim_matches(|ch: char| matches!(ch, ',' | ':' | ';'));
    if let Ok(value) = token.parse::<f64>() {
      if value.is_finite() {
        return Some(value);
      }
    }
  }
  None
}

/// 把打分失败的说明收成问题码
///
/// 宽高不一致单独成一类。其余都算比对没有完成。
///
/// # 参数
/// - `text`: 侧车的标准错误和标准输出
///
/// # 返回值
/// 宽高不一致或其它失败
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(
///   classify_failure("Image size mismatch: 8x8 vs 4x4"),
///   ImageScoreProblem::SizeMismatch
/// );
/// ```
pub fn classify_failure(text: &str) -> ImageScoreProblem {
  let lower = text.to_lowercase();
  if lower.contains("size mismatch")
    || lower.contains("same size")
    || lower.contains("different size")
    || lower.contains("dimensions differ")
    || lower.contains("dimension mismatch")
  {
    return ImageScoreProblem::SizeMismatch;
  }
  ImageScoreProblem::Failed
}

/// 启动失败记成缺少侧车。取消继续向外传。其它错误按说明分类
///
/// # 参数
/// - `err`: 侧车错误
///
/// # 返回值
/// 取消时返回 `Canceled`。启动失败和其它错误都变成比对结果，不让转码失败
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let quality = interpret_error(&SidecarError::sidecar_spawn("ssimulacra2", "missing")).unwrap();
/// assert_eq!(quality.problem, Some(ImageScoreProblem::MissingTool));
/// ```
pub fn interpret_error(err: &SidecarError) -> Result<ImageQuality, SidecarError> {
  match err {
    SidecarError::Canceled => Err(SidecarError::canceled()),
    SidecarError::Spawn { .. } => Ok(ImageQuality::problem(ImageScoreProblem::MissingTool)),
    other => Ok(ImageQuality::problem(classify_failure(&other.to_string()))),
  }
}

/// 按退出码和输出决定分数或问题码
///
/// 退出码 0 时从输出里取分数。没有分数，或退出码不是 0，按说明分类。
///
/// # 参数
/// - `code`: 进程退出码
/// - `stdout`: 标准输出
/// - `stderr`: 标准错误
///
/// # 返回值
/// 有分数时带分档。否则是问题码
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let quality = interpret_score(Some(0), "91.2\n", "");
/// assert_eq!(quality.band, Some(ImageBand::Indistinguishable));
/// ```
pub fn interpret_score(code: Option<i32>, stdout: &str, stderr: &str) -> ImageQuality {
  if code == Some(0) {
    if let Some(score) = parse_score(stdout).or_else(|| parse_score(stderr)) {
      return ImageQuality::scored(score, quality_band(score));
    }
    return ImageQuality::problem(ImageScoreProblem::Failed);
  }
  ImageQuality::problem(classify_failure(&format!("{stderr}\n{stdout}")))
}

/// 编码成功后，把 AVIF 和送进编码器的原图比对
///
/// 先按位深把 AVIF 解成临时 PNG，打完分就删掉。8-bit 用 8-bit PNG，10-bit 和 12-bit 用 16-bit PNG。
///
/// # 参数
/// - `app`: Tauri 应用句柄
/// - `job`: 任务执行期句柄
/// - `reference`: 送进 avifenc 的文件
/// - `encoded`: 写出的 AVIF
/// - `depth`: 编码位深
///
/// # 返回值
/// 成功时返回分数或问题码
///
/// # 错误
/// 取消返回 `SidecarError::Canceled`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let quality = score_avif(app, job, reference, encoded, AvifDepth::Eight).await?;
/// ```
pub async fn score_avif(
  app: &AppHandle,
  job: &JobCtx,
  reference: &str,
  encoded: &str,
  depth: AvifDepth,
) -> SidecarResult<ImageQuality> {
  job.report(95.0, "正在比对画质…").await;
  if job.is_canceled().await {
    return Err(SidecarError::canceled());
  }

  let temp = match decoded_png(app) {
    Ok(path) => path,
    Err(err) => {
      tauri_plugin_log::log::warn!("avif score temp: {err}");
      return Ok(ImageQuality::problem(ImageScoreProblem::Failed));
    }
  };
  let _cleanup = TempFile(temp.clone());
  let depth_flag = match depth {
    AvifDepth::Eight => "8",
    AvifDepth::Ten | AvifDepth::Twelve => "16",
  };
  let decoded = path_text(&temp);
  let output = match run_tool(
    app,
    job,
    "avifdec",
    &["-d".to_string(), depth_flag.to_string(), encoded.to_string(), decoded.clone()],
  )
  .await
  {
    Ok(output) => output,
    Err(err) => return interpret_error(&err),
  };
  if output.code != Some(0) || !temp.is_file() {
    return Ok(interpret_score(output.code, &output.stdout, &output.stderr));
  }
  if let Some(problem) = size_problem(Path::new(reference), &temp) {
    return Ok(ImageQuality::problem(problem));
  }
  score_pair(app, job, reference, &decoded).await
}

/// 编码成功后，把 JPEG XL 和送进编码器的原图比对
///
/// ssimulacra2 直接读 `.jxl`，不再解成 PNG。
///
/// # 参数
/// - `app`: Tauri 应用句柄
/// - `job`: 任务执行期句柄
/// - `reference`: 送进 cjxl 的文件
/// - `encoded`: 写出的 JPEG XL
///
/// # 返回值
/// 成功时返回分数或问题码
///
/// # 错误
/// 取消返回 `SidecarError::Canceled`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let quality = score_jxl(app, job, reference, encoded).await?;
/// ```
pub async fn score_jxl(app: &AppHandle, job: &JobCtx, reference: &str, encoded: &str) -> SidecarResult<ImageQuality> {
  job.report(95.0, "正在比对画质…").await;
  if job.is_canceled().await {
    return Err(SidecarError::canceled());
  }
  score_pair(app, job, reference, encoded).await
}

/// 跑 ssimulacra2。第一个参数是原图，第二个是转码结果
async fn score_pair(app: &AppHandle, job: &JobCtx, reference: &str, distorted: &str) -> SidecarResult<ImageQuality> {
  match run_tool(app, job, "ssimulacra2", &[reference.to_string(), distorted.to_string()]).await {
    Ok(output) => Ok(interpret_score(output.code, &output.stdout, &output.stderr)),
    Err(err) => interpret_error(&err),
  }
}

/// 两边都能读出宽高、并且不一致时返回问题码
fn size_problem(reference: &Path, decoded: &Path) -> Option<ImageScoreProblem> {
  let left = raster_size(reference)?;
  let right = raster_size(decoded)?;
  if left == right { None } else { Some(ImageScoreProblem::SizeMismatch) }
}

/// 读图片头里的宽高。读不到时返回空，交给后面的打分程序判断
fn raster_size(path: &Path) -> Option<(u32, u32)> {
  image::ImageReader::open(path).ok()?.into_dimensions().ok()
}

/// 在缓存临时目录里留一个 PNG 路径。文件要等 avifdec 去创建
fn decoded_png(app: &AppHandle) -> SidecarResult<PathBuf> {
  let dir = get_cache_temp_dir(app)?;
  Ok(dir.join(format!("score-{}.png", crate::utils::id::new_uuid_v4())))
}

fn path_text(path: &Path) -> String {
  path.to_string_lossy().into_owned()
}

struct ToolOutput {
  code: Option<i32>,
  stdout: String,
  stderr: String,
}

/// 函数结束时删掉 avifdec 写出的临时 PNG
struct TempFile(PathBuf);

impl Drop for TempFile {
  fn drop(&mut self) {
    let _ = std::fs::remove_file(&self.0);
  }
}

/// 启动侧车并收齐输出。取消向外传，启动失败也向外传，非 0 退出仍把输出交回去
async fn run_tool(app: &AppHandle, job: &JobCtx, name: &str, args: &[String]) -> Result<ToolOutput, SidecarError> {
  let started = Instant::now();
  tauri_plugin_log::log::info!("{name} {}", args.join(" "));
  let (mut rx, child) = app
    .shell()
    .sidecar(name)
    .map_err(|err| SidecarError::sidecar_spawn(name, err))?
    .args(args)
    .spawn()
    .map_err(|err| SidecarError::sidecar_spawn(name, err))?;
  secure_spawn_step(app, job, child).await?;

  let mut stdout = String::new();
  let mut stderr = String::new();
  while let Some(event) = rx.recv().await {
    match event {
      CommandEvent::Stdout(line) => {
        if job.is_canceled().await {
          break;
        }
        push_tail(&mut stdout, &line);
      }
      CommandEvent::Stderr(line) => {
        if job.is_canceled().await {
          break;
        }
        push_tail(&mut stderr, &line);
      }
      CommandEvent::Terminated(status) => {
        clear_current_child(app).await;
        if job.is_canceled().await {
          return Err(SidecarError::canceled());
        }
        tauri_plugin_log::log::info!("{name} done: code={:?} elapsed={:?}", status.code, started.elapsed());
        return Ok(ToolOutput { code: status.code, stdout, stderr });
      }
      _ => {}
    }
  }

  clear_current_child(app).await;
  if job.is_canceled().await {
    return Err(SidecarError::canceled());
  }
  Err(SidecarError::sidecar_exit(name, "进程意外结束"))
}

/// 输出尾部最多留 4000 字节，避免大图日志把内存撑满
fn push_tail(buf: &mut String, line: &[u8]) {
  let text = String::from_utf8_lossy(line);
  let trimmed = text.trim();
  if trimmed.is_empty() || buf.len() >= 4000 {
    return;
  }
  if !buf.is_empty() {
    buf.push('\n');
  }
  buf.push_str(trimmed);
  if buf.len() > 4000 {
    buf.truncate(4000);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn bands_follow_the_ssimulacra2_scale() {
    assert_eq!(quality_band(100.0), ImageBand::Indistinguishable);
    assert_eq!(quality_band(90.0), ImageBand::Indistinguishable);
    assert_eq!(quality_band(89.9), ImageBand::High);
    assert_eq!(quality_band(70.0), ImageBand::High);
    assert_eq!(quality_band(69.9), ImageBand::Medium);
    assert_eq!(quality_band(50.0), ImageBand::Medium);
    assert_eq!(quality_band(49.9), ImageBand::Visible);
    assert_eq!(quality_band(30.0), ImageBand::Visible);
    assert_eq!(quality_band(29.9), ImageBand::Degraded);
    assert_eq!(quality_band(-4.0), ImageBand::Degraded);
  }

  #[test]
  fn parses_the_score_line() {
    assert_eq!(parse_score("72.413940\n"), Some(72.413940));
    assert_eq!(parse_score("  -3.5\n"), Some(-3.5));
    assert_eq!(parse_score("没有分数"), None);
    assert_eq!(parse_score("nan"), None);
  }

  #[test]
  fn size_mismatch_and_missing_tool_stay_successful() {
    let missing =
      interpret_error(&SidecarError::sidecar_spawn("ssimulacra2", "not found")).expect("缺少侧车仍算比对结果");
    assert_eq!(missing.problem, Some(ImageScoreProblem::MissingTool));
    assert!(missing.score.is_none());
    assert!(missing.band.is_none());

    let mismatch = interpret_score(Some(1), "", "Image size mismatch: 8x8 vs 4x4");
    assert_eq!(mismatch.problem, Some(ImageScoreProblem::SizeMismatch));
    assert!(mismatch.score.is_none());

    let scored = interpret_score(Some(0), "91.2\n", "");
    assert_eq!(scored.score, Some(91.2));
    assert_eq!(scored.band, Some(ImageBand::Indistinguishable));
    assert!(scored.problem.is_none());

    let canceled = interpret_error(&SidecarError::canceled());
    assert!(matches!(canceled, Err(SidecarError::Canceled)));
  }
}
