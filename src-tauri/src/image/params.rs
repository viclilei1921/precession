//! AVIF / JXL 编码参数。移动端也要能反序列化，执行只在桌面进行。

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
///
/// 色彩和透明质量夹到 0–100，速度夹到 0–10。位深只接受 8、10、12。
/// `yuv` 为 `auto` 或其他未列出的值时不传 `-y`。自动分块时不写行列 tile。
/// 要注入 EXIF 时不写 `--ignore-exif`。`irot` 只接受 0 到 3，`imir` 只接受 0 或 1。
///
/// # 字段
/// - `qcolor`: 质量，夹到 0–100，默认 60。写成 `-q`
/// - `qalpha`: 透明质量，夹到 0–100，默认 60。写成 `--qalpha`
/// - `speed`: 速度，夹到 0–10，默认 6。写成 `-s`
/// - `lossless`: 为真时写 `-l`
/// - `jobs`: 有值时写 `-j`，至少为 1
/// - `depth`: 位深，只接受 `"8"`、`"10"`、`"12"`，默认 `"8"`。其他值不传。写成 `-d`
/// - `yuv`: `444`、`422`、`420`、`400` 时写 `-y`。`auto` 或其他值不传
/// - `premultiply`: 为真时写 `-p`
/// - `sharpyuv`: 为真时写 `--sharpyuv`
/// - `ignore_exif`: 为真且没有注入 EXIF 时写 `--ignore-exif`
/// - `ignore_xmp`: 为真时写 `--ignore-xmp`
/// - `ignore_icc`: 为真时写 `--ignore-icc`
/// - `range`: `full`、`f`、`limited`、`l` 时写 `-r`
/// - `cicp`: 非空时写 `--cicp`
/// - `autotiling`: 自动分块。默认开。为真时写 `--autotiling`，此时不写行列 tile
/// - `tilerowslog2`: 未自动分块且有值时写 `--tilerowslog2`，最大 6
/// - `tilecolslog2`: 未自动分块且有值时写 `--tilecolslog2`，最大 6
/// - `codec`: 非空时写 `-c`
/// - `target_size`: 大于 0 时写 `--target-size`
/// - `progressive`: 为真时写 `--progressive`
/// - `pasp`: 非空时写 `--pasp`
/// - `crop`: 非空时写 `--crop`
/// - `irot`: 只接受 0 到 3，写成 `--irot`
/// - `imir`: 只接受 0 或 1，写成 `--imir`
/// - `clli`: 非空时写 `--clli`
/// - `advanced`: 非空项写成 `-a`
///
/// # 示例
/// ```
/// let params = AvifEncodeParams { depth: "10".into(), yuv: "420".into(), ..AvifEncodeParams::default() };
/// assert_eq!(params.depth, "10");
/// assert_eq!(params.qcolor, 60);
/// assert!(params.autotiling);
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AvifEncodeParams {
  /// 质量，夹到 0–100，默认 60
  #[serde(default = "avif_qcolor")]
  pub qcolor: u8,
  /// 透明质量，夹到 0–100，默认 60
  #[serde(default = "avif_qcolor")]
  pub qalpha: u8,
  /// 速度，夹到 0–10，默认 6
  #[serde(default = "avif_speed")]
  pub speed: u8,
  /// 为真时写 `-l`
  pub lossless: bool,
  /// 有值时写 `-j`，至少为 1
  pub jobs: Option<u8>,
  /// 位深，只接受 `"8"`、`"10"`、`"12"`，默认 `"8"`
  #[serde(default = "avif_depth")]
  pub depth: String,
  /// `444`、`422`、`420`、`400` 时写 `-y`。`auto` 或其他值不传
  pub yuv: String,
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
  /// `full`、`f`、`limited`、`l` 时写 `-r`
  pub range: String,
  /// 非空时写 `--cicp`
  pub cicp: String,
  /// 自动分块。默认开。为真时不写行列 tile
  #[serde(default = "avif_autotiling")]
  pub autotiling: bool,
  /// 未自动分块且有值时写 `--tilerowslog2`，最大 6
  pub tilerowslog2: Option<u8>,
  /// 未自动分块且有值时写 `--tilecolslog2`，最大 6
  pub tilecolslog2: Option<u8>,
  /// 非空时写 `-c`
  pub codec: String,
  /// 大于 0 时写 `--target-size`
  pub target_size: Option<u64>,
  /// 为真时写 `--progressive`
  pub progressive: bool,
  /// 非空时写 `--pasp`
  pub pasp: String,
  /// 非空时写 `--crop`
  pub crop: String,
  /// 只接受 0 到 3，写成 `--irot`
  pub irot: Option<u8>,
  /// 只接受 0 或 1，写成 `--imir`
  pub imir: Option<u8>,
  /// 非空时写 `--clli`
  pub clli: String,
  /// 非空项写成 `-a`
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
///
/// 无损时距离写 0，否则把质量夹到 0–25。effort 夹到 1–10，解码速度夹到 0–4。
/// 不写 `--container=0`。`use_container` 为真或要注入 EXIF 时写 `--container=1`。
/// 超出各自允许范围的可选数值不写入。
///
/// # 字段
/// - `lossless`: 为真时距离写 0
/// - `quality`: 距离。0 无损，越大越糊。无损时写 0，否则夹到 0–25，默认 1.0。写成 `-d`
/// - `effort`: 夹到 1–10，默认 7。写成 `-e`
/// - `lossless_jpeg`: 写成 `--lossless_jpeg=0` 或 `1`
/// - `use_container`: 为真或要注入 EXIF 时写 `--container=1`。不写 `--container=0`
/// - `uses_original_profile`: 保留以兼容任务契约；cjxl 无同名 flag，不映射到 CLI
/// - `decoding_speed`: 解码速度，夹到 0–4，默认 0。写成 `--faster_decoding`
/// - `color_encoding`: 颜色编码，见 [`JxlColorEncoding`]。有值时写 `-x color_space=`
/// - `target_intensity`: 有限且不小于 0 时写 `--intensity_target`
/// - `alpha_distance`: 有限时夹到 0–25，写 `--alpha_distance`
/// - `progressive`: 为真时写 `--progressive`
/// - `group_order`: 不大于 1 时写 `--group_order`
/// - `compress_boxes`: 不大于 1 时写 `--compress_boxes`
/// - `brotli_effort`: 不大于 11 时写 `--brotli_effort`
/// - `modular`: 不大于 1 时写 `--modular`
/// - `num_threads`: 有值时写 `--num_threads`
/// - `photon_noise_iso`: 有限且不小于 0 时写 `--photon_noise_iso`
/// - `allow_jpeg_reconstruction`: 不大于 1 时写 `--allow_jpeg_reconstruction`
/// - `codestream_level`: 只接受 -1、5、10，写成 `--codestream_level`
/// - `buffering`: 只接受 -1 到 3，写成 `--buffering`
/// - `premultiply`: 只接受 -1 到 1，写成 `--premultiply`
/// - `keep_invisible`: 不大于 1 时写 `--keep_invisible`
/// - `progressive_ac`: 为真时写 `--progressive_ac`
/// - `qprogressive_ac`: 为真时写 `--qprogressive_ac`
/// - `progressive_dc`: 只接受 -1 到 2，写成 `--progressive_dc`
/// - `resampling`: 只接受 -1、1、2、4、8，写成 `--resampling`
/// - `ec_resampling`: 只接受 -1、1、2、4、8，写成 `--ec_resampling`
/// - `already_downsampled`: 为真时写 `--already_downsampled`
/// - `upsampling_mode`: 只接受 -1 到 1，写成 `--upsampling_mode`
/// - `epf`: 只接受 -1 到 3，写成 `--epf`
/// - `gaborish`: 不大于 1 时写 `--gaborish`
/// - `override_bitdepth`: 大于 0 时写 `--override_bitdepth`
/// - `noise`: 不大于 1 时写 `--noise`
/// - `dots`: 不大于 1 时写 `--dots`
/// - `patches`: 不大于 1 时写 `--patches`
/// - `extra_hints`: 非空项写成 `-x`
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
  /// 夹到 1–10，默认 7
  #[serde(default = "jxl_effort")]
  pub effort: u8,
  /// 写成 `--lossless_jpeg=0` 或 `1`
  pub lossless_jpeg: bool,
  /// 为真或要注入 EXIF 时写 `--container=1`
  pub use_container: bool,
  /// 保留以兼容任务契约；cjxl 无同名 flag，不映射到 CLI
  pub uses_original_profile: bool,
  /// 解码速度，夹到 0–4，默认 0
  pub decoding_speed: i64,
  /// 颜色编码，见 [`JxlColorEncoding`]
  pub color_encoding: Option<JxlColorEncoding>,
  /// 有限且不小于 0 时写 `--intensity_target`
  pub target_intensity: Option<f32>,
  /// 有限时夹到 0–25，写 `--alpha_distance`
  pub alpha_distance: Option<f32>,
  /// 为真时写 `--progressive`
  pub progressive: bool,
  /// 不大于 1 时写 `--group_order`
  pub group_order: Option<u8>,
  /// 不大于 1 时写 `--compress_boxes`
  pub compress_boxes: Option<u8>,
  /// 不大于 11 时写 `--brotli_effort`
  pub brotli_effort: Option<u8>,
  /// 不大于 1 时写 `--modular`
  pub modular: Option<u8>,
  /// 有值时写 `--num_threads`
  pub num_threads: Option<i32>,
  /// 有限且不小于 0 时写 `--photon_noise_iso`
  pub photon_noise_iso: Option<f32>,
  /// 不大于 1 时写 `--allow_jpeg_reconstruction`
  pub allow_jpeg_reconstruction: Option<u8>,
  /// 只接受 -1、5、10
  pub codestream_level: Option<i32>,
  /// 只接受 -1 到 3
  pub buffering: Option<i8>,
  /// 只接受 -1 到 1
  pub premultiply: Option<i8>,
  /// 不大于 1 时写 `--keep_invisible`
  pub keep_invisible: Option<u8>,
  /// 为真时写 `--progressive_ac`
  pub progressive_ac: bool,
  /// 为真时写 `--qprogressive_ac`
  pub qprogressive_ac: bool,
  /// 只接受 -1 到 2
  pub progressive_dc: Option<i8>,
  /// 只接受 -1、1、2、4、8
  pub resampling: Option<i8>,
  /// 只接受 -1、1、2、4、8
  pub ec_resampling: Option<i8>,
  /// 为真时写 `--already_downsampled`
  pub already_downsampled: bool,
  /// 只接受 -1 到 1
  pub upsampling_mode: Option<i8>,
  /// 只接受 -1 到 3
  pub epf: Option<i8>,
  /// 不大于 1 时写 `--gaborish`
  pub gaborish: Option<u8>,
  /// 大于 0 时写 `--override_bitdepth`
  pub override_bitdepth: Option<u32>,
  /// 不大于 1 时写 `--noise`
  pub noise: Option<u8>,
  /// 不大于 1 时写 `--dots`
  pub dots: Option<u8>,
  /// 不大于 1 时写 `--patches`
  pub patches: Option<u8>,
  /// 非空项写成 `-x`
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
