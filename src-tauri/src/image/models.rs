//! AVIF / JXL 编码参数。移动端也要能反序列化，执行只在桌面进行。
//!
//! 前端不传时，[`AvifEncodeParams::default`] 和 [`JxlEncodeParams::default`] 就是点默认就导出。
//! 位深、色度和范围是枚举。`codec`、`cicp`、`advanced`、`extra_hints` 仍可反序列化，界面不提供输入框。

use serde::{Deserialize, Serialize};

/// cjxl 的颜色编码
///
/// 组装参数时写成 `-x color_space=` 后面的名字。
///
/// # 变体
/// - `Srgb`: `sRGB`
/// - `LinearSrgb`: `RGB_D65_SRG_Lin_SRG`
/// - `SrgbLuma`: `Gray_D65_SRG_Per_SRG`
/// - `LinearSrgbLuma`: `Gray_D65_SRG_Lin_SRG`
///
/// # 示例
/// ```
/// assert_eq!(JxlColorEncoding::Srgb.hint(), "sRGB");
/// ```
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum JxlColorEncoding {
  /// `sRGB`
  Srgb,
  /// `RGB_D65_SRG_Lin_SRG`
  LinearSrgb,
  /// `Gray_D65_SRG_Per_SRG`
  SrgbLuma,
  /// `Gray_D65_SRG_Lin_SRG`
  LinearSrgbLuma,
}

impl JxlColorEncoding {
  /// 返回 cjxl `color_space` 能识别的名字
  ///
  /// # 返回值
  /// `sRGB`、`RGB_D65_SRG_Lin_SRG`、`Gray_D65_SRG_Per_SRG` 或 `Gray_D65_SRG_Lin_SRG`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert_eq!(JxlColorEncoding::LinearSrgb.hint(), "RGB_D65_SRG_Lin_SRG");
  /// ```
  pub fn hint(self) -> &'static str {
    match self {
      Self::Srgb => "sRGB",
      Self::LinearSrgb => "RGB_D65_SRG_Lin_SRG",
      Self::SrgbLuma => "Gray_D65_SRG_Per_SRG",
      Self::LinearSrgbLuma => "Gray_D65_SRG_Lin_SRG",
    }
  }
}

/// avifenc 位深
///
/// 写成 `-d` 后面的 `8`、`10` 或 `12`。
///
/// # 变体
/// - `Eight`: 8-bit，默认
/// - `Ten`: 10-bit
/// - `Twelve`: 12-bit
///
/// # 示例
/// ```
/// assert_eq!(AvifDepth::default().flag(), "8");
/// assert_eq!(AvifDepth::Twelve.flag(), "12");
/// ```
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AvifDepth {
  /// 8-bit。默认
  #[default]
  Eight,
  /// 10-bit
  Ten,
  /// 12-bit
  Twelve,
}

impl AvifDepth {
  /// 返回 avifenc `-d` 的参数
  ///
  /// # 返回值
  /// `"8"`、`"10"` 或 `"12"`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert_eq!(AvifDepth::Ten.flag(), "10");
  /// ```
  pub fn flag(self) -> &'static str {
    match self {
      Self::Eight => "8",
      Self::Ten => "10",
      Self::Twelve => "12",
    }
  }
}

/// avifenc 色度抽样
///
/// `Auto` 不传 `-y`。显式 `-y auto` 会报 `invalid format`。
///
/// # 变体
/// - `Auto`: 不传 `-y`。默认
/// - `Y444`: `-y 444`
/// - `Y422`: `-y 422`
/// - `Y420`: `-y 420`
/// - `Y400`: `-y 400`
///
/// # 示例
/// ```
/// assert_eq!(AvifYuv::Auto.flag(), None);
/// assert_eq!(AvifYuv::Y420.flag(), Some("420"));
/// ```
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AvifYuv {
  /// 不传 `-y`。默认
  #[default]
  Auto,
  /// `-y 444`
  #[serde(rename = "444")]
  Y444,
  /// `-y 422`
  #[serde(rename = "422")]
  Y422,
  /// `-y 420`
  #[serde(rename = "420")]
  Y420,
  /// `-y 400`
  #[serde(rename = "400")]
  Y400,
}

impl AvifYuv {
  /// 返回要写给 `-y` 的值
  ///
  /// `Auto` 不传这个参数。
  ///
  /// # 返回值
  /// `None` 表示不传。其余是 `"444"`、`"422"`、`"420"` 或 `"400"`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert_eq!(AvifYuv::Y444.flag(), Some("444"));
  /// ```
  pub fn flag(self) -> Option<&'static str> {
    match self {
      Self::Auto => None,
      Self::Y444 => Some("444"),
      Self::Y422 => Some("422"),
      Self::Y420 => Some("420"),
      Self::Y400 => Some("400"),
    }
  }
}

/// avifenc 色彩范围
///
/// `Auto` 不传 `-r`。
///
/// # 变体
/// - `Auto`: 不传 `-r`。默认
/// - `Full`: `-r full`
/// - `Limited`: `-r limited`
///
/// # 示例
/// ```
/// assert_eq!(AvifRange::Full.flag(), Some("full"));
/// ```
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AvifRange {
  /// 不传 `-r`。默认
  #[default]
  Auto,
  /// `-r full`
  Full,
  /// `-r limited`
  Limited,
}

impl AvifRange {
  /// 返回要写给 `-r` 的值
  ///
  /// `Auto` 不传这个参数。
  ///
  /// # 返回值
  /// `None` 表示不传。其余是 `"full"` 或 `"limited"`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert_eq!(AvifRange::Limited.flag(), Some("limited"));
  /// ```
  pub fn flag(self) -> Option<&'static str> {
    match self {
      Self::Auto => None,
      Self::Full => Some("full"),
      Self::Limited => Some("limited"),
    }
  }
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

/// avifenc 参数。缺省字段与默认导出一致：质量 60、速度 6、8-bit、自动分块。
///
/// 色彩和透明质量收到 0–100，速度收到 0–10。线程数至少为 1，分块 log2 最大 6。
/// 目标大小为 0 时当成没设置。旋转超出 0–3、镜像超出 0–1 时入队拒绝。
/// `Auto` 色度和范围不传对应参数。自动分块时不写行列 tile。
/// 要注入 EXIF 时不写 `--ignore-exif`。
///
/// # 字段
/// - `qcolor`: 质量，收到 0–100，默认 60。写成 `-q`
/// - `qalpha`: 透明质量，收到 0–100，默认 60。写成 `--qalpha`
/// - `speed`: 速度，收到 0–10，默认 6。写成 `-s`
/// - `lossless`: 为真时写 `-l`
/// - `jobs`: 有值时写 `-j`，至少为 1
/// - `depth`: 位深，见 [`AvifDepth`]。默认 8-bit。写成 `-d`
/// - `yuv`: 色度，见 [`AvifYuv`]。`Auto` 不传 `-y`
/// - `premultiply`: 为真时写 `-p`
/// - `sharpyuv`: 为真时写 `--sharpyuv`
/// - `ignore_exif`: 为真且没有注入 EXIF 时写 `--ignore-exif`
/// - `ignore_xmp`: 为真时写 `--ignore-xmp`
/// - `ignore_icc`: 为真时写 `--ignore-icc`
/// - `range`: 范围，见 [`AvifRange`]。`Auto` 不传 `-r`
/// - `cicp`: 非空时写 `--cicp`。界面不提供
/// - `autotiling`: 自动分块。默认开。为真时写 `--autotiling`，此时不写行列 tile
/// - `tilerowslog2`: 未自动分块且有值时写 `--tilerowslog2`，最大 6
/// - `tilecolslog2`: 未自动分块且有值时写 `--tilecolslog2`，最大 6
/// - `codec`: 非空时写 `-c`。界面不提供
/// - `target_size`: 大于 0 时写 `--target-size`。0 收成没有
/// - `progressive`: 为真时写 `--progressive`
/// - `pasp`: 非空时写 `--pasp`。界面不提供
/// - `crop`: 非空时写 `--crop`。界面不提供
/// - `irot`: 只接受 0 到 3，写成 `--irot`。界面不提供
/// - `imir`: 只接受 0 或 1，写成 `--imir`。界面不提供
/// - `clli`: 非空时写 `--clli`。界面不提供
/// - `advanced`: 非空项写成 `-a`。界面不提供
///
/// # 示例
/// ```
/// let params = AvifEncodeParams {
///   depth: AvifDepth::Ten,
///   yuv: AvifYuv::Y420,
///   ..AvifEncodeParams::default()
/// };
/// assert_eq!(params.depth, AvifDepth::Ten);
/// assert_eq!(params.qcolor, 60);
/// assert!(params.autotiling);
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AvifEncodeParams {
  /// 质量，收到 0–100，默认 60
  #[serde(default = "avif_qcolor")]
  pub qcolor: u8,
  /// 透明质量，收到 0–100，默认 60
  #[serde(default = "avif_qcolor")]
  pub qalpha: u8,
  /// 速度，收到 0–10，默认 6
  #[serde(default = "avif_speed")]
  pub speed: u8,
  /// 为真时写 `-l`
  pub lossless: bool,
  /// 有值时写 `-j`，至少为 1
  pub jobs: Option<u8>,
  /// 位深，见 [`AvifDepth`]。默认 8-bit
  #[serde(default)]
  pub depth: AvifDepth,
  /// 色度，见 [`AvifYuv`]。`Auto` 不传 `-y`
  #[serde(default)]
  pub yuv: AvifYuv,
  /// 为真时写 `-p`
  pub premultiply: bool,
  /// 为真时写 `--sharpyuv`
  pub sharpyuv: bool,
  /// 为真且没有注入 EXIF 时写 `--ignore-exif`
  pub ignore_exif: bool,
  /// 为真时写 `--ignore-xmp`
  pub ignore_xmp: bool,
  /// 为真时写 `--ignore-icc`
  pub ignore_icc: bool,
  /// 范围，见 [`AvifRange`]。`Auto` 不传 `-r`
  #[serde(default)]
  pub range: AvifRange,
  /// 非空时写 `--cicp`。界面不提供
  pub cicp: String,
  /// 自动分块。默认开。为真时不写行列 tile
  #[serde(default = "avif_autotiling")]
  pub autotiling: bool,
  /// 未自动分块且有值时写 `--tilerowslog2`，最大 6
  pub tilerowslog2: Option<u8>,
  /// 未自动分块且有值时写 `--tilecolslog2`，最大 6
  pub tilecolslog2: Option<u8>,
  /// 非空时写 `-c`。界面不提供
  pub codec: String,
  /// 大于 0 时写 `--target-size`。0 收成没有
  pub target_size: Option<u64>,
  /// 为真时写 `--progressive`
  pub progressive: bool,
  /// 非空时写 `--pasp`。界面不提供
  pub pasp: String,
  /// 非空时写 `--crop`。界面不提供
  pub crop: String,
  /// 只接受 0 到 3，写成 `--irot`。界面不提供
  pub irot: Option<u8>,
  /// 只接受 0 或 1，写成 `--imir`。界面不提供
  pub imir: Option<u8>,
  /// 非空时写 `--clli`。界面不提供
  pub clli: String,
  /// 非空项写成 `-a`。界面不提供
  pub advanced: Vec<String>,
}

impl Default for AvifEncodeParams {
  /// 返回默认的 AVIF 参数
  ///
  /// 色彩和透明质量都是 60，速度 6，8-bit，色度和范围自动，自动分块打开，其余开关关闭。
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
  /// assert_eq!(params.depth, AvifDepth::Eight);
  /// assert!(params.autotiling);
  /// ```
  fn default() -> Self {
    Self {
      qcolor: avif_qcolor(),
      qalpha: avif_qcolor(),
      speed: avif_speed(),
      lossless: false,
      jobs: None,
      depth: AvifDepth::Eight,
      yuv: AvifYuv::Auto,
      premultiply: false,
      sharpyuv: false,
      ignore_exif: false,
      ignore_xmp: false,
      ignore_icc: false,
      range: AvifRange::Auto,
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

/// 收紧 AVIF 参数，非法的旋转和镜像直接拒绝
///
/// 质量、透明质量和速度收到允许范围。线程数至少为 1。分块 log2 最大 6。目标大小为 0 时清空。
///
/// # 参数
/// - `params`: 入队前的 AVIF 参数，就地修改
///
/// # 返回值
/// 成功时无额外值
///
/// # 错误
/// 旋转不在 0 到 3，或镜像不是 0 或 1 时返回说明文字
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let mut params = AvifEncodeParams { speed: 99, ..AvifEncodeParams::default() };
/// normalize_avif(&mut params)?;
/// assert_eq!(params.speed, 10);
/// # Ok::<(), String>(())
/// ```
pub fn normalize_avif(params: &mut AvifEncodeParams) -> Result<(), String> {
  params.qcolor = params.qcolor.min(100);
  params.qalpha = params.qalpha.min(100);
  params.speed = params.speed.min(10);
  if let Some(jobs) = params.jobs {
    params.jobs = Some(jobs.max(1));
  }
  if let Some(rows) = params.tilerowslog2 {
    params.tilerowslog2 = Some(rows.min(6));
  }
  if let Some(cols) = params.tilecolslog2 {
    params.tilecolslog2 = Some(cols.min(6));
  }
  if params.target_size == Some(0) {
    params.target_size = None;
  }
  if params.irot.is_some_and(|angle| angle > 3) {
    return Err("旋转只接受 0 到 3".to_string());
  }
  if params.imir.is_some_and(|axis| axis > 1) {
    return Err("镜像只接受 0 或 1".to_string());
  }
  Ok(())
}

/// cjxl 参数。`quality` 是距离（0 无损，越大越糊），缺省 1.0、effort 7。
///
/// 无损时距离写 0，否则把质量收到 0–25。effort 收到 1–10，解码速度收到 0–4。
/// 不写 `--container=0`。`use_container` 为真或要注入 EXIF 时写 `--container=1`。
/// 可选数值超出允许集合时入队拒绝，不再悄悄丢掉。
///
/// # 字段
/// - `lossless`: 为真时距离写 0
/// - `quality`: 距离。0 无损，越大越糊。无损时写 0，否则收到 0–25，默认 1.0。写成 `-d`
/// - `effort`: 收到 1–10，默认 7。写成 `-e`
/// - `lossless_jpeg`: 写成 `--lossless_jpeg=0` 或 `1`
/// - `use_container`: 为真或要注入 EXIF 时写 `--container=1`。不写 `--container=0`
/// - `decoding_speed`: 解码速度，收到 0–4，默认 0。写成 `--faster_decoding`
/// - `color_encoding`: 颜色编码，见 [`JxlColorEncoding`]。有值时写 `-x color_space=`
/// - `target_intensity`: 有限且不小于 0 时写 `--intensity_target`。否则入队拒绝
/// - `alpha_distance`: 有限时收到 0–25，写 `--alpha_distance`。非有限数字入队拒绝
/// - `progressive`: 为真时写 `--progressive`
/// - `group_order`: 只接受 0 或 1，写成 `--group_order`。界面不提供
/// - `compress_boxes`: 只接受 0 或 1，写成 `--compress_boxes`。界面不提供
/// - `brotli_effort`: 最大 11，写成 `--brotli_effort`。界面不提供
/// - `modular`: 只接受 0 或 1，写成 `--modular`。空着表示编码器自己选
/// - `num_threads`: 至少为 1 时写 `--num_threads`。空着表示用机器默认
/// - `photon_noise_iso`: 有限且不小于 0 时写 `--photon_noise_iso`。空着表示不加颗粒
/// - `allow_jpeg_reconstruction`: 只接受 0 或 1。界面不提供
/// - `codestream_level`: 只接受 -1、5、10。界面不提供
/// - `buffering`: 只接受 -1 到 3。界面不提供
/// - `premultiply`: 只接受 -1 到 1。空着表示编码器自己选
/// - `keep_invisible`: 只接受 0 或 1。界面不提供
/// - `progressive_ac`: 为真时写 `--progressive_ac`。界面不提供
/// - `qprogressive_ac`: 为真时写 `--qprogressive_ac`。界面不提供
/// - `progressive_dc`: 只接受 -1 到 2。界面不提供
/// - `resampling`: 只接受 -1、1、2、4、8。空着表示只在很低画质时抽样
/// - `ec_resampling`: 只接受 -1、1、2、4、8。界面不提供
/// - `already_downsampled`: 为真时写 `--already_downsampled`。界面不提供
/// - `upsampling_mode`: 只接受 -1 到 1。界面不提供
/// - `epf`: 只接受 -1 到 3。界面不提供
/// - `gaborish`: 只接受 0 或 1。界面不提供
/// - `override_bitdepth`: 大于 0 时写 `--override_bitdepth`。空着表示沿用原图位深
/// - `noise`: 只接受 0 或 1。界面不提供
/// - `dots`: 只接受 0 或 1。界面不提供
/// - `patches`: 只接受 0 或 1。界面不提供
/// - `extra_hints`: 非空项写成 `-x`。界面不提供
///
/// # 示例
/// 见 [`JxlEncodeParams::default`]。
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct JxlEncodeParams {
  /// 为真时距离写 0
  pub lossless: bool,
  /// 距离。0 无损，越大越糊。默认 1.0
  #[serde(default = "jxl_quality")]
  pub quality: f32,
  /// 收到 1–10，默认 7
  #[serde(default = "jxl_effort")]
  pub effort: u8,
  /// 写成 `--lossless_jpeg=0` 或 `1`
  pub lossless_jpeg: bool,
  /// 为真或要注入 EXIF 时写 `--container=1`
  pub use_container: bool,
  /// 解码速度，收到 0–4，默认 0
  pub decoding_speed: i64,
  /// 颜色编码，见 [`JxlColorEncoding`]
  pub color_encoding: Option<JxlColorEncoding>,
  /// 有限且不小于 0 时写 `--intensity_target`
  pub target_intensity: Option<f32>,
  /// 有限时收到 0–25，写 `--alpha_distance`
  pub alpha_distance: Option<f32>,
  /// 为真时写 `--progressive`
  pub progressive: bool,
  /// 只接受 0 或 1。界面不提供
  pub group_order: Option<u8>,
  /// 只接受 0 或 1。界面不提供
  pub compress_boxes: Option<u8>,
  /// 最大 11。界面不提供
  pub brotli_effort: Option<u8>,
  /// 只接受 0 或 1。空着表示编码器自己选
  pub modular: Option<u8>,
  /// 至少为 1。空着表示用机器默认
  pub num_threads: Option<i32>,
  /// 有限且不小于 0。空着表示不加颗粒
  pub photon_noise_iso: Option<f32>,
  /// 只接受 0 或 1。界面不提供
  pub allow_jpeg_reconstruction: Option<u8>,
  /// 只接受 -1、5、10。界面不提供
  pub codestream_level: Option<i32>,
  /// 只接受 -1 到 3。界面不提供
  pub buffering: Option<i8>,
  /// 只接受 -1 到 1。空着表示编码器自己选
  pub premultiply: Option<i8>,
  /// 只接受 0 或 1。界面不提供
  pub keep_invisible: Option<u8>,
  /// 为真时写 `--progressive_ac`。界面不提供
  pub progressive_ac: bool,
  /// 为真时写 `--qprogressive_ac`。界面不提供
  pub qprogressive_ac: bool,
  /// 只接受 -1 到 2。界面不提供
  pub progressive_dc: Option<i8>,
  /// 只接受 -1、1、2、4、8。空着表示只在很低画质时抽样
  pub resampling: Option<i8>,
  /// 只接受 -1、1、2、4、8。界面不提供
  pub ec_resampling: Option<i8>,
  /// 为真时写 `--already_downsampled`。界面不提供
  pub already_downsampled: bool,
  /// 只接受 -1 到 1。界面不提供
  pub upsampling_mode: Option<i8>,
  /// 只接受 -1 到 3。界面不提供
  pub epf: Option<i8>,
  /// 只接受 0 或 1。界面不提供
  pub gaborish: Option<u8>,
  /// 大于 0 时写 `--override_bitdepth`。空着表示沿用原图位深
  pub override_bitdepth: Option<u32>,
  /// 只接受 0 或 1。界面不提供
  pub noise: Option<u8>,
  /// 只接受 0 或 1。界面不提供
  pub dots: Option<u8>,
  /// 只接受 0 或 1。界面不提供
  pub patches: Option<u8>,
  /// 非空项写成 `-x`。界面不提供
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

/// 收紧 JXL 参数，超出允许集合的可选数值直接拒绝
///
/// 距离收到 0–25，effort 收到 1–10，解码速度收到 0–4。透明距离在有限时收到 0–25。
///
/// # 参数
/// - `params`: 入队前的 JXL 参数，就地修改
///
/// # 返回值
/// 成功时无额外值
///
/// # 错误
/// 距离或可选数值不是允许的有限数字，或不在各自集合里时返回说明文字
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let mut params = JxlEncodeParams { effort: 0, ..JxlEncodeParams::default() };
/// normalize_jxl(&mut params)?;
/// assert_eq!(params.effort, 1);
/// # Ok::<(), String>(())
/// ```
pub fn normalize_jxl(params: &mut JxlEncodeParams) -> Result<(), String> {
  if !params.quality.is_finite() {
    return Err("距离必须是有限数字".to_string());
  }
  params.quality = params.quality.clamp(0.0, 25.0);
  params.effort = params.effort.clamp(1, 10);
  params.decoding_speed = params.decoding_speed.clamp(0, 4);

  if let Some(nits) = params.target_intensity {
    if !nits.is_finite() || nits < 0.0 {
      return Err("目标亮度必须是不小于 0 的有限数字".to_string());
    }
  }
  if let Some(distance) = params.alpha_distance.as_mut() {
    if !distance.is_finite() {
      return Err("透明距离必须是有限数字".to_string());
    }
    *distance = distance.clamp(0.0, 25.0);
  }

  reject_at_most(params.group_order, 1, "分组顺序只接受 0 或 1")?;
  reject_at_most(params.compress_boxes, 1, "压缩盒子只接受 0 或 1")?;
  reject_at_most(params.brotli_effort, 11, "Brotli effort 最大是 11")?;
  reject_at_most(params.modular, 1, "模块化只接受 0 或 1")?;
  if params.num_threads.is_some_and(|count| count < 1) {
    return Err("线程数至少为 1".to_string());
  }
  if let Some(iso) = params.photon_noise_iso {
    if !iso.is_finite() || iso < 0.0 {
      return Err("光子噪声必须是不小于 0 的有限数字".to_string());
    }
  }
  reject_at_most(params.allow_jpeg_reconstruction, 1, "JPEG 重建只接受 0 或 1")?;
  if params.codestream_level.is_some_and(|level| !matches!(level, -1 | 5 | 10)) {
    return Err("码流等级只接受 -1、5、10".to_string());
  }
  if params.buffering.is_some_and(|value| !(-1..=3).contains(&value)) {
    return Err("缓冲只接受 -1 到 3".to_string());
  }
  if params.premultiply.is_some_and(|value| !(-1..=1).contains(&value)) {
    return Err("预乘只接受 -1 到 1".to_string());
  }
  reject_at_most(params.keep_invisible, 1, "保留不可见像素只接受 0 或 1")?;
  if params.progressive_dc.is_some_and(|value| !(-1..=2).contains(&value)) {
    return Err("渐进 DC 只接受 -1 到 2".to_string());
  }
  if params.resampling.is_some_and(|value| !matches!(value, -1 | 1 | 2 | 4 | 8)) {
    return Err("重采样只接受 -1、1、2、4、8".to_string());
  }
  if params.ec_resampling.is_some_and(|value| !matches!(value, -1 | 1 | 2 | 4 | 8)) {
    return Err("额外通道重采样只接受 -1、1、2、4、8".to_string());
  }
  if params.upsampling_mode.is_some_and(|value| !(-1..=1).contains(&value)) {
    return Err("上采样模式只接受 -1 到 1".to_string());
  }
  if params.epf.is_some_and(|value| !(-1..=3).contains(&value)) {
    return Err("EPF 只接受 -1 到 3".to_string());
  }
  reject_at_most(params.gaborish, 1, "Gaborish 只接受 0 或 1")?;
  if params.override_bitdepth == Some(0) {
    return Err("覆盖位深必须大于 0".to_string());
  }
  reject_at_most(params.noise, 1, "噪声只接受 0 或 1")?;
  reject_at_most(params.dots, 1, "点只接受 0 或 1")?;
  reject_at_most(params.patches, 1, "补丁只接受 0 或 1")?;
  Ok(())
}

/// 可选的 `u8` 超过上限时返回说明
///
/// 没有值时通过。
///
/// # 参数
/// - `value`: 待检查的值
/// - `max`: 允许的最大值，含
/// - `message`: 超出时的说明
///
/// # 返回值
/// 在范围内时无额外值
///
/// # 错误
/// `value` 大于 `max` 时返回 `message`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(reject_at_most(Some(1), 1, "太大").is_ok());
/// assert!(reject_at_most(Some(2), 1, "太大").is_err());
/// ```
fn reject_at_most(value: Option<u8>, max: u8, message: &str) -> Result<(), String> {
  if value.is_some_and(|item| item > max) {
    return Err(message.to_string());
  }
  Ok(())
}
