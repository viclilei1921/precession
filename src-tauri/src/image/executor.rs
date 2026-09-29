use std::path::Path;
use std::time::{Duration, Instant};

use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

use crate::image::bridge::{
  BridgeInput, EncodeTarget, is_supported_image_ext, needs_bridge, path_ext_lower, prepare_bridge_input,
};
use crate::image::params::{AvifEncodeParams, JxlColorEncoding, JxlEncodeParams};
use crate::sidecar::{SidecarError, SidecarResult, clear_current_child, secure_spawn_step};
use crate::task::JobCtx;

/// cjxl `-x color_space=` 可识别值
fn jxl_color_space_hint(value: JxlColorEncoding) -> &'static str {
  match value {
    JxlColorEncoding::Srgb => "sRGB",
    JxlColorEncoding::LinearSrgb => "RGB_D65_SRG_Lin_SRG",
    JxlColorEncoding::SrgbLuma => "Gray_D65_SRG_Per_SRG",
    JxlColorEncoding::LinearSrgbLuma => "Gray_D65_SRG_Lin_SRG",
  }
}

fn build_avifenc_args(
  input_path: &str,
  output_path: &str,
  params: &AvifEncodeParams,
  exif_inject: Option<&str>,
) -> Vec<String> {
  let mut args = Vec::with_capacity(36);

  if params.lossless {
    args.push("-l".to_string());
  }

  args.push("-q".to_string());
  args.push(params.qcolor.clamp(0, 100).to_string());
  args.push("--qalpha".to_string());
  args.push(params.qalpha.clamp(0, 100).to_string());
  args.push("-s".to_string());
  args.push(params.speed.clamp(0, 10).to_string());

  if let Some(jobs) = params.jobs {
    args.push("-j".to_string());
    args.push(jobs.max(1).to_string());
  }

  let depth = params.depth.trim();
  if matches!(depth, "8" | "10" | "12") {
    args.push("-d".to_string());
    args.push(depth.to_string());
  }

  let yuv = params.yuv.trim().to_ascii_lowercase();
  // `auto` 是工具默认值；显式传 `-y auto` 会报 ERROR: invalid format: auto
  if matches!(yuv.as_str(), "444" | "422" | "420" | "400") {
    args.push("-y".to_string());
    args.push(yuv);
  }

  if params.premultiply {
    args.push("-p".to_string());
  }
  if params.sharpyuv {
    args.push("--sharpyuv".to_string());
  }
  if params.ignore_exif && exif_inject.is_none() {
    args.push("--ignore-exif".to_string());
  }
  if params.ignore_xmp {
    args.push("--ignore-xmp".to_string());
  }
  if params.ignore_icc {
    args.push("--ignore-icc".to_string());
  }

  let range = params.range.trim().to_ascii_lowercase();
  if matches!(range.as_str(), "full" | "f" | "limited" | "l") {
    args.push("-r".to_string());
    args.push(range);
  }

  let cicp = params.cicp.trim();
  if !cicp.is_empty() {
    args.push("--cicp".to_string());
    args.push(cicp.to_string());
  }

  if params.autotiling {
    args.push("--autotiling".to_string());
  } else {
    if let Some(r) = params.tilerowslog2 {
      args.push("--tilerowslog2".to_string());
      args.push(r.min(6).to_string());
    }
    if let Some(c) = params.tilecolslog2 {
      args.push("--tilecolslog2".to_string());
      args.push(c.min(6).to_string());
    }
  }

  let codec = params.codec.trim();
  if !codec.is_empty() {
    args.push("-c".to_string());
    args.push(codec.to_string());
  }

  if let Some(size) = params.target_size {
    if size > 0 {
      args.push("--target-size".to_string());
      args.push(size.to_string());
    }
  }

  if params.progressive {
    args.push("--progressive".to_string());
  }

  let pasp = params.pasp.trim();
  if !pasp.is_empty() {
    args.push("--pasp".to_string());
    args.push(pasp.to_string());
  }

  let crop = params.crop.trim();
  if !crop.is_empty() {
    args.push("--crop".to_string());
    args.push(crop.to_string());
  }

  if let Some(angle) = params.irot {
    if angle <= 3 {
      args.push("--irot".to_string());
      args.push(angle.to_string());
    }
  }

  if let Some(axis) = params.imir {
    if axis <= 1 {
      args.push("--imir".to_string());
      args.push(axis.to_string());
    }
  }

  let clli = params.clli.trim();
  if !clli.is_empty() {
    args.push("--clli".to_string());
    args.push(clli.to_string());
  }

  for adv in &params.advanced {
    let kv = adv.trim();
    if kv.is_empty() {
      continue;
    }
    args.push("-a".to_string());
    args.push(kv.to_string());
  }

  if let Some(exif) = exif_inject {
    args.push("--exif".to_string());
    args.push(exif.to_string());
  }

  args.push(input_path.to_string());
  args.push(output_path.to_string());
  args
}

fn build_cjxl_args(
  input_path: &str,
  output_path: &str,
  params: &JxlEncodeParams,
  exif_inject: Option<&str>,
) -> Vec<String> {
  let effort = params.effort.clamp(1, 10);
  let decoding_speed = params.decoding_speed.clamp(0, 4);
  let distance = if params.lossless { 0.0 } else { params.quality.clamp(0.0, 25.0) };

  let mut args = Vec::with_capacity(48);
  args.push("-d".to_string());
  args.push(format!("{distance}"));
  args.push("-e".to_string());
  args.push(effort.to_string());
  args.push(format!("--lossless_jpeg={}", if params.lossless_jpeg { 1 } else { 0 }));
  // 勿默认传 --container=0：cjxl 会据此清空 EXIF/XMP。
  // 注入 EXIF 时必须使用容器。
  if params.use_container || exif_inject.is_some() {
    args.push("--container=1".to_string());
  }
  args.push(format!("--faster_decoding={decoding_speed}"));

  if let Some(nits) = params.target_intensity {
    if nits.is_finite() && nits >= 0.0 {
      args.push(format!("--intensity_target={nits}"));
    }
  }

  if let Some(ad) = params.alpha_distance {
    if ad.is_finite() {
      args.push(format!("--alpha_distance={}", ad.clamp(0.0, 25.0)));
    }
  }

  if params.progressive {
    args.push("--progressive".to_string());
  }

  if let Some(g) = params.group_order {
    if g <= 1 {
      args.push(format!("--group_order={g}"));
    }
  }

  if let Some(c) = params.compress_boxes {
    if c <= 1 {
      args.push(format!("--compress_boxes={c}"));
    }
  }

  if let Some(b) = params.brotli_effort {
    if b <= 11 {
      args.push(format!("--brotli_effort={b}"));
    }
  }

  if let Some(m) = params.modular {
    if m <= 1 {
      args.push(format!("--modular={m}"));
    }
  }

  if let Some(n) = params.num_threads {
    args.push(format!("--num_threads={n}"));
  }

  if let Some(iso) = params.photon_noise_iso {
    if iso.is_finite() && iso >= 0.0 {
      args.push(format!("--photon_noise_iso={iso}"));
    }
  }

  if let Some(a) = params.allow_jpeg_reconstruction {
    if a <= 1 {
      args.push(format!("--allow_jpeg_reconstruction={a}"));
    }
  }

  if let Some(level) = params.codestream_level {
    if matches!(level, -1 | 5 | 10) {
      args.push(format!("--codestream_level={level}"));
    }
  }

  if let Some(buf) = params.buffering {
    if (-1..=3).contains(&buf) {
      args.push(format!("--buffering={buf}"));
    }
  }

  if let Some(p) = params.premultiply {
    if (-1..=1).contains(&p) {
      args.push(format!("--premultiply={p}"));
    }
  }

  if let Some(k) = params.keep_invisible {
    if k <= 1 {
      args.push(format!("--keep_invisible={k}"));
    }
  }

  if params.progressive_ac {
    args.push("--progressive_ac".to_string());
  }
  if params.qprogressive_ac {
    args.push("--qprogressive_ac".to_string());
  }

  if let Some(dc) = params.progressive_dc {
    if (-1..=2).contains(&dc) {
      args.push(format!("--progressive_dc={dc}"));
    }
  }

  if let Some(r) = params.resampling {
    if matches!(r, -1 | 1 | 2 | 4 | 8) {
      args.push(format!("--resampling={r}"));
    }
  }

  if let Some(r) = params.ec_resampling {
    if matches!(r, -1 | 1 | 2 | 4 | 8) {
      args.push(format!("--ec_resampling={r}"));
    }
  }

  if params.already_downsampled {
    args.push("--already_downsampled".to_string());
  }

  if let Some(m) = params.upsampling_mode {
    if (-1..=1).contains(&m) {
      args.push(format!("--upsampling_mode={m}"));
    }
  }

  if let Some(e) = params.epf {
    if (-1..=3).contains(&e) {
      args.push(format!("--epf={e}"));
    }
  }

  if let Some(g) = params.gaborish {
    if g <= 1 {
      args.push(format!("--gaborish={g}"));
    }
  }

  if let Some(bits) = params.override_bitdepth {
    if bits > 0 {
      args.push(format!("--override_bitdepth={bits}"));
    }
  }

  if let Some(n) = params.noise {
    if n <= 1 {
      args.push(format!("--noise={n}"));
    }
  }
  if let Some(d) = params.dots {
    if d <= 1 {
      args.push(format!("--dots={d}"));
    }
  }
  if let Some(p) = params.patches {
    if p <= 1 {
      args.push(format!("--patches={p}"));
    }
  }

  if let Some(enc) = params.color_encoding {
    args.push("-x".to_string());
    args.push(format!("color_space={}", jxl_color_space_hint(enc)));
  }

  for hint in &params.extra_hints {
    let kv = hint.trim();
    if kv.is_empty() {
      continue;
    }
    args.push("-x".to_string());
    args.push(kv.to_string());
  }

  if let Some(exif) = exif_inject {
    args.push("-x".to_string());
    args.push(format!("exif={exif}"));
  }

  args.push(input_path.to_string());
  args.push(output_path.to_string());
  args
}

async fn ensure_output_parent(output_path: &str) -> SidecarResult<()> {
  let out = Path::new(output_path);
  if let Some(parent) = out.parent() {
    if !parent.as_os_str().is_empty() {
      std::fs::create_dir_all(parent).map_err(|e| SidecarError::io(format!("创建输出目录失败: {e}")))?;
    }
  }
  Ok(())
}

async fn run_sidecar_encode(
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

fn resolve_encode_input(
  app: &AppHandle,
  input_path: &str,
  target: EncodeTarget,
  inject_exif: bool,
) -> SidecarResult<(String, Option<BridgeInput>, Option<String>)> {
  let ext = path_ext_lower(input_path).ok_or_else(|| SidecarError::invalid("无法识别输入文件扩展名"))?;
  if !is_supported_image_ext(&ext) {
    return Err(SidecarError::invalid("仅支持 jpg/jpeg/png/webp/bmp/gif/tif/tiff 输入"));
  }

  if !needs_bridge(&ext, target) {
    return Ok((input_path.to_string(), None, None));
  }

  let bridge = prepare_bridge_input(app, input_path, inject_exif)?;
  let encode_path = bridge.png_path.to_str().ok_or_else(|| SidecarError::invalid("临时 PNG 路径无效"))?.to_string();
  let exif = bridge.exif_path.as_ref().and_then(|p| p.to_str().map(|s| s.to_string()));
  Ok((encode_path, Some(bridge), exif))
}

pub async fn run_convert_avif(
  app: &AppHandle,
  job: &JobCtx,
  input_path: &str,
  output_path: &str,
  params: AvifEncodeParams,
) -> SidecarResult<()> {
  if job.is_canceled().await {
    return Err(SidecarError::canceled());
  }

  job.report(5.0, "准备转码...").await;
  if job.is_canceled().await {
    return Err(SidecarError::canceled());
  }

  ensure_output_parent(output_path).await?;

  let ext = path_ext_lower(input_path).unwrap_or_default();
  let bridged = needs_bridge(&ext, EncodeTarget::Avif);
  if bridged {
    job.report(8.0, "桥接转 PNG…").await;
  }

  let inject_exif = !params.ignore_exif;
  let (encode_input, bridge, exif_path) = resolve_encode_input(app, input_path, EncodeTarget::Avif, inject_exif)?;
  // 持有 bridge 至函数结束，Drop 时清理临时文件
  let _bridge = bridge;

  if job.is_canceled().await {
    return Err(SidecarError::canceled());
  }

  let params = AvifEncodeParams {
    qcolor: params.qcolor.clamp(0, 100),
    qalpha: params.qalpha.clamp(0, 100),
    speed: params.speed.clamp(0, 10),
    ..params
  };

  let exif_inject = exif_path.as_deref();
  let args = build_avifenc_args(&encode_input, output_path, &params, exif_inject);
  let label = format!(
    "正在 AVIF 编码（-q {}, --qalpha {}, -s {}, lossless={}）...",
    params.qcolor, params.qalpha, params.speed, params.lossless
  );
  run_sidecar_encode(app, job, "avifenc", &args, &label).await
}

pub async fn run_convert_jxl(
  app: &AppHandle,
  job: &JobCtx,
  input_path: &str,
  output_path: &str,
  params: JxlEncodeParams,
) -> SidecarResult<()> {
  if job.is_canceled().await {
    return Err(SidecarError::canceled());
  }

  job.report(5.0, "准备转码...").await;
  if job.is_canceled().await {
    return Err(SidecarError::canceled());
  }

  ensure_output_parent(output_path).await?;

  let ext = path_ext_lower(input_path).unwrap_or_default();
  let bridged = needs_bridge(&ext, EncodeTarget::Jxl);
  if bridged {
    job.report(8.0, "桥接转 PNG…").await;
  }

  let (encode_input, bridge, exif_path) = resolve_encode_input(app, input_path, EncodeTarget::Jxl, true)?;
  let _bridge = bridge;

  if job.is_canceled().await {
    return Err(SidecarError::canceled());
  }

  let params = JxlEncodeParams {
    quality: params.quality.clamp(0.0, 25.0),
    effort: params.effort.clamp(1, 10),
    decoding_speed: params.decoding_speed.clamp(0, 4),
    ..params
  };

  let exif_inject = exif_path.as_deref();
  let args = build_cjxl_args(&encode_input, output_path, &params, exif_inject);
  let distance = if params.lossless { 0.0 } else { params.quality };
  let label = format!(
    "正在 JXL 编码（lossless={}, distance={distance:.2}, effort={}, lossless_jpeg={}）...",
    params.lossless, params.effort, params.lossless_jpeg
  );
  run_sidecar_encode(app, job, "cjxl", &args, &label).await
}
