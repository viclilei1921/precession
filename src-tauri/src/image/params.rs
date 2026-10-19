//! AVIF / JXL 编码参数。移动端也要能反序列化，执行只在桌面进行。

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum JxlColorEncoding {
  Srgb,
  LinearSrgb,
  SrgbLuma,
  LinearSrgbLuma,
}

/// 返回 AVIF 质量的默认值
///
/// # 返回值
/// `60`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(avif_qcolor(), 60);
/// ```
fn avif_qcolor() -> u8 {
  60
}

/// 返回 AVIF 速度的默认值
///
/// # 返回值
/// `6`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(avif_speed(), 6);
/// ```
fn avif_speed() -> u8 {
  6
}

/// 返回 AVIF 位深的默认值
///
/// # 返回值
/// `"8"`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(avif_depth(), "8");
/// ```
fn avif_depth() -> String {
  "8".to_string()
}

/// 返回 AVIF 自动分块的默认值
///
/// # 返回值
/// `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(avif_autotiling());
/// ```
fn avif_autotiling() -> bool {
  true
}

/// 返回 JXL 距离的默认值
///
/// # 返回值
/// `1.0`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(jxl_quality(), 1.0);
/// ```
fn jxl_quality() -> f32 {
  1.0
}

/// 返回 JXL effort 的默认值
///
/// # 返回值
/// `7`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(jxl_effort(), 7);
/// ```
fn jxl_effort() -> u8 {
  7
}

/// avifenc 参数。缺省字段与原先界面一致：质量 60、速度 6、自动分块。
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AvifEncodeParams {
  #[serde(default = "avif_qcolor")]
  pub qcolor: u8,
  #[serde(default = "avif_qcolor")]
  pub qalpha: u8,
  #[serde(default = "avif_speed")]
  pub speed: u8,
  pub lossless: bool,
  pub jobs: Option<u8>,
  #[serde(default = "avif_depth")]
  pub depth: String,
  pub yuv: String,
  pub premultiply: bool,
  pub sharpyuv: bool,
  pub ignore_exif: bool,
  pub ignore_xmp: bool,
  pub ignore_icc: bool,
  pub range: String,
  pub cicp: String,
  #[serde(default = "avif_autotiling")]
  pub autotiling: bool,
  pub tilerowslog2: Option<u8>,
  pub tilecolslog2: Option<u8>,
  pub codec: String,
  pub target_size: Option<u64>,
  pub progressive: bool,
  pub pasp: String,
  pub crop: String,
  pub irot: Option<u8>,
  pub imir: Option<u8>,
  pub clli: String,
  pub advanced: Vec<String>,
}

impl Default for AvifEncodeParams {
  /// 返回默认的 AVIF 参数
  ///
  /// 色彩和透明质量都是 60，速度 6，位深 `"8"`，自动分块打开，其余开关关闭、文本为空。
  ///
  /// # 返回值
  /// 默认的 `AvifEncodeParams`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let params = AvifEncodeParams::default();
  /// assert_eq!(params.qcolor, 60);
  /// assert!(params.autotiling);
  /// ```
  fn default() -> Self {
    Self {
      qcolor: avif_qcolor(),
      qalpha: avif_qcolor(),
      speed: avif_speed(),
      lossless: false,
      jobs: None,
      depth: avif_depth(),
      yuv: String::new(),
      premultiply: false,
      sharpyuv: false,
      ignore_exif: false,
      ignore_xmp: false,
      ignore_icc: false,
      range: String::new(),
      cicp: String::new(),
      autotiling: avif_autotiling(),
      tilerowslog2: None,
      tilecolslog2: None,
      codec: String::new(),
      target_size: None,
      progressive: false,
      pasp: String::new(),
      crop: String::new(),
      irot: None,
      imir: None,
      clli: String::new(),
      advanced: Vec::new(),
    }
  }
}

/// cjxl 参数。`quality` 是距离（0 无损，越大越糊），缺省 1.0、effort 7。
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct JxlEncodeParams {
  pub lossless: bool,
  #[serde(default = "jxl_quality")]
  pub quality: f32,
  #[serde(default = "jxl_effort")]
  pub effort: u8,
  pub lossless_jpeg: bool,
  pub use_container: bool,
  /// 保留以兼容任务契约；cjxl 无同名 flag，不映射到 CLI
  pub uses_original_profile: bool,
  pub decoding_speed: i64,
  pub color_encoding: Option<JxlColorEncoding>,
  pub target_intensity: Option<f32>,
  pub alpha_distance: Option<f32>,
  pub progressive: bool,
  pub group_order: Option<u8>,
  pub compress_boxes: Option<u8>,
  pub brotli_effort: Option<u8>,
  pub modular: Option<u8>,
  pub num_threads: Option<i32>,
  pub photon_noise_iso: Option<f32>,
  pub allow_jpeg_reconstruction: Option<u8>,
  pub codestream_level: Option<i32>,
  pub buffering: Option<i8>,
  pub premultiply: Option<i8>,
  pub keep_invisible: Option<u8>,
  pub progressive_ac: bool,
  pub qprogressive_ac: bool,
  pub progressive_dc: Option<i8>,
  pub resampling: Option<i8>,
  pub ec_resampling: Option<i8>,
  pub already_downsampled: bool,
  pub upsampling_mode: Option<i8>,
  pub epf: Option<i8>,
  pub gaborish: Option<u8>,
  pub override_bitdepth: Option<u32>,
  pub noise: Option<u8>,
  pub dots: Option<u8>,
  pub patches: Option<u8>,
  pub extra_hints: Vec<String>,
}

impl Default for JxlEncodeParams {
  /// 返回默认的 JXL 参数
  ///
  /// 距离 1.0，effort 7，解码速度 0，有损，不用容器。
  ///
  /// # 返回值
  /// 默认的 `JxlEncodeParams`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let params = JxlEncodeParams::default();
  /// assert_eq!(params.quality, 1.0);
  /// assert_eq!(params.effort, 7);
  /// ```
  fn default() -> Self {
    Self {
      lossless: false,
      quality: jxl_quality(),
      effort: jxl_effort(),
      lossless_jpeg: false,
      use_container: false,
      uses_original_profile: false,
      decoding_speed: 0,
      color_encoding: None,
      target_intensity: None,
      alpha_distance: None,
      progressive: false,
      group_order: None,
      compress_boxes: None,
      brotli_effort: None,
      modular: None,
      num_threads: None,
      photon_noise_iso: None,
      allow_jpeg_reconstruction: None,
      codestream_level: None,
      buffering: None,
      premultiply: None,
      keep_invisible: None,
      progressive_ac: false,
      qprogressive_ac: false,
      progressive_dc: None,
      resampling: None,
      ec_resampling: None,
      already_downsampled: false,
      upsampling_mode: None,
      epf: None,
      gaborish: None,
      override_bitdepth: None,
      noise: None,
      dots: None,
      patches: None,
      extra_hints: Vec::new(),
    }
  }
}
