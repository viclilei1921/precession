use serde::{Deserialize, Serialize};

/// 裁剪片段
///
/// `start` / `duration` 为 `HH:MM:SS` 或秒数。
///
/// # 字段
/// - `start`: 起始时间，`HH:MM:SS` 或秒数
/// - `duration`: 时长，`HH:MM:SS` 或秒数
///
/// # 示例
/// ```
/// let seg = TimeSegment { start: "00:00:01".into(), duration: "2".into() };
/// assert_eq!(seg.start, "00:00:01");
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeSegment {
  /// 起始时间，`HH:MM:SS` 或秒数
  pub start: String,
  /// 时长，`HH:MM:SS` 或秒数
  pub duration: String,
}

/// 一条音轨
///
/// `index` 是 `Stream #0:N` 的 N，映射时写成 `-map 0:N?`。
/// `channel_layout` 如 stereo、mono、5.1。`language` 是流名字里的 eng、jpn，没有就是空。
/// `title` 是紧随其后的 Metadata title，没有就是空。
///
/// # 字段
/// - `index`: `Stream #0:N` 的 N，映射时写成 `-map 0:N?`
/// - `codec`: 音频编码名
/// - `sample_rate`: 采样率，单位 Hz
/// - `channels`: 声道数
/// - `channel_layout`: 声道布局，如 stereo、mono、5.1
/// - `bitrate_kbps`: 码率，单位 kbps
/// - `language`: 流名字里的 eng、jpn，没有就是空
/// - `title`: 紧随其后的 Metadata title，没有就是空
///
/// # 示例
/// ```
/// let track = AudioTrack {
///   index: 1,
///   codec: "aac".into(),
///   sample_rate: 48000,
///   channels: 2,
///   channel_layout: "stereo".into(),
///   bitrate_kbps: 128,
///   language: String::new(),
///   title: String::new(),
/// };
/// assert!(track.language.is_empty());
/// assert!(track.title.is_empty());
/// ```
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AudioTrack {
  /// `Stream #0:N` 的 N，映射时写成 `-map 0:N?`
  pub index: u32,
  /// 音频编码名
  pub codec: String,
  /// 采样率，单位 Hz
  pub sample_rate: u32,
  /// 声道数
  pub channels: u32,
  /// 声道布局，如 stereo、mono、5.1
  pub channel_layout: String,
  /// 码率，单位 kbps
  pub bitrate_kbps: u32,
  /// 流名字里的 eng、jpn，没有就是空
  pub language: String,
  /// 紧随其后的 Metadata title，没有就是空
  pub title: String,
}

/// ffmpeg `-i` 读到的视频信息
///
/// # 字段
/// - `path`: 视频路径
/// - `width`: 画面宽度
/// - `height`: 画面高度
/// - `fps`: 帧率
/// - `duration`: 时长，单位秒
/// - `video_codec`: 视频编码名
/// - `pix_fmt`: 像素格式
/// - `bitrate_kbps`: 容器码率，单位 kbps
/// - `audio_tracks`: 音轨
///
/// # 示例
/// ```
/// let info = VideoInfo { path: "a.mp4".into(), duration: 60.0, ..VideoInfo::default() };
/// assert_eq!(info.duration, 60.0);
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoInfo {
  /// 视频路径
  pub path: String,
  /// 画面宽度
  pub width: u32,
  /// 画面高度
  pub height: u32,
  /// 帧率
  pub fps: f64,
  /// 时长，单位秒
  pub duration: f64,
  /// 视频编码名
  pub video_codec: String,
  /// 像素格式
  pub pix_fmt: String,
  /// 容器码率，单位 kbps
  pub bitrate_kbps: u32,
  /// 音轨
  pub audio_tracks: Vec<AudioTrack>,
}

/// 写出容器
///
/// 侧车只打开了这三种封装。MOV 是 QuickTime 容器，和 MP4 同族，默认仍是 MP4。
/// WebM 能读不能写。VP9 只写入 MP4，不给 MPEG-TS。
///
/// # 变体
/// - `Mp4`: 封装名 `mp4`。默认容器
/// - `Mov`: QuickTime 容器，封装名 `mov`，和 MP4 同族
/// - `MpegTs`: 封装名 `mpegts`。VP9 不能写入
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum VideoContainer {
  /// 封装名 `mp4`。默认容器
  Mp4,
  /// QuickTime 容器，封装名 `mov`，和 MP4 同族
  Mov,
  /// 封装名 `mpegts`。VP9 不能写入
  MpegTs,
}

impl VideoContainer {
  /// 返回 ffmpeg 的封装名
  ///
  /// MP4 是 `mp4`，MOV 是 `mov`，MPEG-TS 是 `mpegts`。
  ///
  /// # 返回值
  /// 封装名
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert_eq!(VideoContainer::Mp4.muxer(), "mp4");
  /// ```
  pub fn muxer(self) -> &'static str {
    match self {
      Self::Mp4 => "mp4",
      Self::Mov => "mov",
      Self::MpegTs => "mpegts",
    }
  }
}

impl Default for VideoContainer {
  /// 返回默认容器 MP4
  ///
  /// # 返回值
  /// `VideoContainer::Mp4`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert_eq!(VideoContainer::default(), VideoContainer::Mp4);
  /// ```
  fn default() -> Self {
    Self::Mp4
  }
}

/// 像素位深
///
/// 默认 10。只有 AV1 和 HEVC 有这一项。
/// 8-bit 用 `yuv420p`，10-bit 用 `yuv420p10le`。HEVC 的 10-bit 同时用 `main10`。
/// 10-bit 渐变更干净，体积通常接近 8-bit；要兼容更多播放器再改回 8。
/// 这版 `libvpx-vp9` 没有 10-bit 像素格式，所以 VP9 没有这个字段。不是 8 或 10 的写法在反序列化时就失败。
///
/// # 变体
/// - `Eight`: 8-bit，像素格式 `yuv420p`
/// - `Ten`: 10-bit，像素格式 `yuv420p10le`。默认。HEVC 同时用 `main10`
///
/// # 示例
/// ```
/// assert_eq!(BitDepth::default(), BitDepth::Ten);
/// assert_eq!(BitDepth::Eight.pix_fmt(), "yuv420p");
/// assert_eq!(BitDepth::Ten.pix_fmt(), "yuv420p10le");
/// ```
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum BitDepth {
  /// 8-bit，像素格式 `yuv420p`
  Eight,
  /// 10-bit，像素格式 `yuv420p10le`。默认。HEVC 同时用 `main10`
  #[default]
  Ten,
}

impl BitDepth {
  /// 返回这一位深对应的像素格式
  ///
  /// 8-bit 是 `yuv420p`，10-bit 是 `yuv420p10le`。
  ///
  /// # 返回值
  /// 像素格式名
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert_eq!(BitDepth::Ten.pix_fmt(), "yuv420p10le");
  /// ```
  pub fn pix_fmt(self) -> &'static str {
    match self {
      Self::Eight => "yuv420p",
      Self::Ten => "yuv420p10le",
    }
  }
}

/// x265 的 preset 名字
///
/// 界面只给这五档，`ultrafast` 和 `placebo` 不进界面。
/// 很快 `veryfast` 文件更大；较快 `fast`；默认 `medium`；更小 `slow` 慢很多、体积只小一点；更慢是命令行的 `slower`。
///
/// # 变体
/// - `VeryFast`: `veryfast`。很快，文件更大
/// - `Fast`: `fast`。较快
/// - `Medium`: `medium`。默认
/// - `Slow`: `slow`。更小，慢很多，体积只小一点
/// - `Slower`: `slower`。更慢
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum HevcPreset {
  /// `veryfast`。很快，文件更大
  VeryFast,
  /// `fast`。较快
  Fast,
  /// `medium`。默认
  #[default]
  Medium,
  /// `slow`。更小，慢很多，体积只小一点
  Slow,
  /// `slower`。更慢
  Slower,
}

impl HevcPreset {
  /// 返回 x265 的 preset 名字
  ///
  /// # 返回值
  /// `veryfast`、`fast`、`medium`、`slow` 或 `slower`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert_eq!(HevcPreset::Medium.ffmpeg(), "medium");
  /// ```
  fn ffmpeg(self) -> &'static str {
    match self {
      Self::VeryFast => "veryfast",
      Self::Fast => "fast",
      Self::Medium => "medium",
      Self::Slow => "slow",
      Self::Slower => "slower",
    }
  }
}

/// x265 的 tune
///
/// 默认不传。`animation` 适合动画和线稿，边缘更干净，往往更小。`grain` 保留颗粒，文件变大。
/// 不提供 `psnr`、`ssim`、`zerolatency`、`fastdecode`：前两个是评测用的，后两个会让文件变大或只适合直播。
///
/// # 变体
/// - `None`: 不传。默认
/// - `Animation`: `animation`，适合动画和线稿，边缘更干净，往往更小
/// - `Grain`: `grain`，保留颗粒，文件变大
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum HevcTune {
  /// 不传。默认
  #[default]
  None,
  /// `animation`，适合动画和线稿，边缘更干净，往往更小
  Animation,
  /// `grain`，保留颗粒，文件变大
  Grain,
}

impl HevcTune {
  /// 返回要传给 x265 的 tune 名字
  ///
  /// `None` 不传。`Animation` 是 `animation`，`Grain` 是 `grain`。
  ///
  /// # 返回值
  /// tune 名字，不传时为 `None`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert_eq!(HevcTune::None.ffmpeg(), None);
  /// assert_eq!(HevcTune::Animation.ffmpeg(), Some("animation"));
  /// ```
  fn ffmpeg(self) -> Option<&'static str> {
    match self {
      Self::None => None,
      Self::Animation => Some("animation"),
      Self::Grain => Some("grain"),
    }
  }
}

/// 画面类型
///
/// 默认普通画面，命令行不传。`screen` 适合录屏，文字更清晰，往往更小。`film` 保留颗粒，文件变大。
///
/// # 变体
/// - `General`: 普通画面，命令行不传。默认
/// - `Screen`: `screen`，适合录屏，文字更清晰，往往更小
/// - `Film`: `film`，保留颗粒，文件变大
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Vp9Content {
  /// 普通画面，命令行不传。默认
  #[default]
  General,
  /// `screen`，适合录屏，文字更清晰，往往更小
  Screen,
  /// `film`，保留颗粒，文件变大
  Film,
}

impl Vp9Content {
  /// 返回要传给 VP9 的内容类型
  ///
  /// `General` 不传。`Screen` 是 `screen`，`Film` 是 `film`。
  ///
  /// # 返回值
  /// 内容类型，不传时为 `None`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert_eq!(Vp9Content::Screen.ffmpeg(), Some("screen"));
  /// ```
  fn ffmpeg(self) -> Option<&'static str> {
    match self {
      Self::General => None,
      Self::Screen => Some("screen"),
      Self::Film => Some("film"),
    }
  }
}

/// 返回 AV1 preset 的默认值
  ///
  /// # 返回值
  /// `6`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert_eq!(av1_preset(), 6);
  /// ```
fn av1_preset() -> u8 {
  6
}

/// 返回 AV1 CRF 的默认值
  ///
  /// # 返回值
  /// `35`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert_eq!(av1_crf(), 35);
  /// ```
fn av1_crf() -> u8 {
  35
}

/// 返回 HEVC CRF 的默认值
  ///
  /// # 返回值
  /// `26`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert_eq!(hevc_crf(), 26);
  /// ```
fn hevc_crf() -> u8 {
  26
}

/// 返回 VP9 `cpu-used` 的默认值
  ///
  /// # 返回值
  /// `2`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert_eq!(vp9_cpu_used(), 2);
  /// ```
fn vp9_cpu_used() -> u8 {
  2
}

/// 返回 VP9 CRF 的默认值
  ///
  /// # 返回值
  /// `33`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert_eq!(vp9_crf(), 33);
  /// ```
fn vp9_crf() -> u8 {
  33
}

/// 返回音频码率的默认值
  ///
  /// # 返回值
  /// `128`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert_eq!(audio_bitrate(), 128);
  /// ```
fn audio_bitrate() -> u32 {
  128
}

/// 返回 serde 用的布尔默认值
  ///
  /// # 返回值
  /// `true`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert!(default_true());
  /// ```
fn default_true() -> bool {
  true
}

/// 当前这次用哪种编码
///
/// 字段跟 `codec` 写在一起，缺的用该编码的默认值。
/// 不放共用的「快 / 均衡 / 慢」：三种编码器的刻度不能互换，任务里记下真正写给 FFmpeg 的值。
/// 界面可以给推荐档，选中后写入该编码器自己的字段。
///
/// 速度不改变质量档的含义。同一个 CRF 下，更慢压得更紧，文件更小，也更干净，等待更久。
/// CRF 决定观感和体积，数值越小越清晰、文件越大。想再压体积，优先把 CRF 调大一档，不要只把速度调快。
/// CRF 的数字不能跨编码器比较。大致相同的观感：更清晰是 AV1 30、HEVC 22、VP9 30；
/// 默认、适合保存是 AV1 35、HEVC 26、VP9 33；更小、运动和暗部会糊是 AV1 40、HEVC 30、VP9 38。
/// CRF 每加大约 2，体积大约少 15–25%，只供估算。
///
/// 保存、希望文件不要太大：AV1，preset 6，CRF 35，这就是默认。
/// 要更快点、播放器也更熟：HEVC，medium，CRF 26。VP9 只在明确需要这种编码时用。
/// `{ "codec": "hevc" }` 等于 medium、CRF 26、不调优、10-bit。`{ "codec": "av1", "bitDepth": "eight" }` 才写回 8-bit AV1。
///
/// # 变体
/// - `Av1`: `preset` 范围 0–13，越小越慢，默认 6。不用库默认的 10，同样 CRF 下 10 的文件会大一截。更小用 4，较快用 8，很快用 10，10 只适合赶时间。`crf` 范围 0–63，默认 35，是这个编码器常用的质量点。`bit_depth` 见 [`BitDepth`]。屏幕内容检测固定关闭
/// - `Hevc`: `preset` 见 [`HevcPreset`]。`crf` 范围 0–51，默认 26。比编码器自带的 28 干净一些，又比 22–24 小一截。`tune` 见 [`HevcTune`]。`bit_depth` 见 [`BitDepth`]，10-bit 同时用 `main10`
/// - `Vp9`: `cpu_used` 是 `deadline=good` 下的速度，只有 0–5，越小越慢，默认 2。更小用 1，较快用 4，很快用 5。`crf` 范围 0–63。压缩不如 AV1，默认停在 33：再往上提才能接近 AV1 的体积，观感会先掉下去。文件通常仍比 AV1 大，不作为默认编码。没有位深。`content` 见 [`Vp9Content`]。`deadline`、多线程、前瞻在 [`encode_args`] 里写死。只写入 MP4
///
/// # 示例
/// ```
/// # fn main() -> Result<(), serde_json::Error> {
/// let hevc: VideoParams = serde_json::from_str(r#"{"codec":"hevc"}"#)?;
/// assert!(matches!(
///   hevc,
///   VideoParams::Hevc {
///     preset: HevcPreset::Medium,
///     crf: 26,
///     tune: HevcTune::None,
///     bit_depth: BitDepth::Ten,
///   }
/// ));
/// let av1: VideoParams = serde_json::from_str(r#"{"codec":"av1","bitDepth":"eight"}"#)?;
/// assert!(matches!(av1, VideoParams::Av1 { bit_depth: BitDepth::Eight, .. }));
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "codec", rename_all = "camelCase")]
pub enum VideoParams {
  /// SVT-AV1。`preset` 0–13，默认 6；`crf` 0–63，默认 35；屏幕内容检测固定关闭
  Av1 {
    /// 速度，0–13，越小越慢，默认 6
    #[serde(default = "av1_preset")]
    preset: u8,
    /// 质量，0–63，越小越清晰，默认 35
    #[serde(default = "av1_crf")]
    crf: u8,
    /// 位深，见 [`BitDepth`]。默认 10
    #[serde(rename = "bitDepth", default)]
    bit_depth: BitDepth,
  },
  /// x265。CRF 默认 26，10-bit 同时用 `main10`
  Hevc {
    /// preset，见 [`HevcPreset`]。默认 `medium`
    #[serde(default)]
    preset: HevcPreset,
    /// 质量，0–51，默认 26
    #[serde(default = "hevc_crf")]
    crf: u8,
    /// tune，见 [`HevcTune`]。默认不传
    #[serde(default)]
    tune: HevcTune,
    /// 位深，见 [`BitDepth`]。默认 10，同时用 `main10`
    #[serde(rename = "bitDepth", default)]
    bit_depth: BitDepth,
  },
  /// libvpx-vp9。没有位深，只写入 MP4
  Vp9 {
    /// `deadline=good` 下的速度，0–5，越小越慢，默认 2
    #[serde(rename = "cpuUsed", default = "vp9_cpu_used")]
    cpu_used: u8,
    /// 质量，0–63，默认 33
    #[serde(default = "vp9_crf")]
    crf: u8,
    /// 画面类型，见 [`Vp9Content`]。默认不传
    #[serde(default)]
    content: Vp9Content,
  },
}

impl Default for VideoParams {
  /// 返回默认的 10-bit AV1 参数
  ///
  /// preset 为 6，CRF 为 35。
  ///
  /// # 返回值
  /// `VideoParams::Av1`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let video = VideoParams::default();
  /// assert!(matches!(video, VideoParams::Av1 { preset: 6, crf: 35, .. }));
  /// ```
  fn default() -> Self {
    Self::Av1 { preset: av1_preset(), crf: av1_crf(), bit_depth: BitDepth::Ten }
  }
}

impl VideoParams {
  /// 返回这一编码要写的像素格式
  ///
  /// AV1 和 HEVC 跟位深。VP9 固定 `yuv420p`。
  ///
  /// # 返回值
  /// 像素格式名
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert_eq!(VideoParams::default().pix_fmt(), "yuv420p10le");
  /// ```
  pub fn pix_fmt(&self) -> &'static str {
    match self {
      Self::Av1 { bit_depth, .. } | Self::Hevc { bit_depth, .. } => bit_depth.pix_fmt(),
      Self::Vp9 { .. } => "yuv420p",
    }
  }
}

/// 选哪条音轨
///
/// 默认第一条。
/// `Index` 在合并、追加里对每个输入用同一个序号；该文件没有这条轨道时退回第一条，调用方记一条警告。
/// 多条音轨全部保留先不做，拼接时音轨条数对不齐。
/// `First`、`Index` 映射成 `-map 0:{index}?`，`index` 是绝对流序号。`Drop` 写成 `-an`。
/// 反序列化用 `kind` 区分。
///
/// # 变体
/// - `First`: 第一条。默认。映射成 `-map 0:{index}?`
/// - `Index`: 指定绝对流序号 `index`。合并、追加里每个输入用同一个序号；没有这条轨道时退回第一条
/// - `Drop`: 不要音频，写成 `-an`
///
/// # 示例
/// ```
/// # fn main() -> Result<(), serde_json::Error> {
/// let mode: AudioMode = serde_json::from_str(r#"{"kind":"drop"}"#)?;
/// assert_eq!(mode, AudioMode::Drop);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AudioMode {
  /// 第一条。默认。映射成 `-map 0:{index}?`
  First,
  /// 指定绝对流序号。没有这条轨道时退回第一条
  Index {
    /// 绝对流序号
    index: u32,
  },
  /// 不要音频，写成 `-an`
  Drop,
}

impl Default for AudioMode {
  /// 返回默认音轨选择：第一条
  ///
  /// # 返回值
  /// `AudioMode::First`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert_eq!(AudioMode::default(), AudioMode::First);
  /// ```
  fn default() -> Self {
    Self::First
  }
}

/// 探测先把轨道列出来，选择留到这里
///
/// 编码格式固定 AAC，不暴露 `aac_coder`。
/// `channels` 空则自动：单声道保持单声道，两声道以上混成立体声。指定时只允许 1 或 2。
/// `sample_rate` 空则按操作决定，见 [`RatePolicy`]。指定时只允许 44100 或 48000。
/// `bitrate_kbps` 默认 128。界面给 96、128、160、192。声道用 `-ac`，采样率需要改时加 `-ar`，码率是 `-b:a`。
///
/// # 字段
/// - `mode`: 选哪条音轨，见 [`AudioMode`]。默认第一条
/// - `channels`: 声道数。空则自动：单声道保持单声道，两声道以上混成立体声。指定时只允许 1 或 2。写成 `-ac`
/// - `sample_rate`: 采样率。空则按 [`RatePolicy`] 决定。指定时只允许 44100 或 48000。需要改时加 `-ar`
/// - `bitrate_kbps`: 码率，单位 kbps，默认 128。界面给 96、128、160、192。写成 `-b:a`
///
/// # 示例
/// ```
/// let audio = AudioSelection {
///   channels: Some(1),
///   sample_rate: Some(44100),
///   ..AudioSelection::default()
/// };
/// assert_eq!(audio.bitrate_kbps, 128);
/// assert_eq!(audio.sample_rate, Some(44100));
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct AudioSelection {
  /// 选哪条音轨，见 [`AudioMode`]。默认第一条
  pub mode: AudioMode,
  /// 声道数。空则自动：单声道保持单声道，两声道以上混成立体声。指定时只允许 1 或 2
  pub channels: Option<u8>,
  /// 采样率。空则按 [`RatePolicy`] 决定。指定时只允许 44100 或 48000
  pub sample_rate: Option<u32>,
  /// 码率，单位 kbps，默认 128
  #[serde(default = "audio_bitrate")]
  pub bitrate_kbps: u32,
}

impl Default for AudioSelection {
  /// 返回默认音轨选择
  ///
  /// 模式是第一条，声道和采样率留空，码率 128。
  ///
  /// # 返回值
  /// 默认的 `AudioSelection`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let audio = AudioSelection::default();
  /// assert_eq!(audio.bitrate_kbps, 128);
  /// ```
  fn default() -> Self {
    Self { mode: AudioMode::First, channels: None, sample_rate: None, bitrate_kbps: audio_bitrate() }
  }
}

/// 画面怎么放进目标矩形
///
/// 默认 `Contain`：不足处补黑边。`Fill` 拉伸到目标矩形。
/// `Cover` 是铺满后裁掉多余画面，侧车没有 `crop`，入队直接拒绝。
///
/// # 变体
/// - `Contain`: 不足处补黑边。默认
/// - `Fill`: 拉伸到目标矩形
/// - `Cover`: 铺满后裁掉多余画面。侧车没有 `crop`，入队直接拒绝
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Fit {
  /// 不足处补黑边。默认
  #[default]
  Contain,
  /// 拉伸到目标矩形
  Fill,
  /// 铺满后裁掉多余画面。侧车没有 `crop`，入队直接拒绝
  Cover,
}

/// 目标画面
///
/// 空则转码、裁剪跟源，合并取各段最大宽高并补边，追加跟基准。
/// 宽或高为 0 视为没设置。奇数在滤镜里收成偶数。
///
/// # 字段
/// - `width`: 目标宽度。0 视为没设置。奇数在滤镜里收成偶数
/// - `height`: 目标高度。0 视为没设置。奇数在滤镜里收成偶数
/// - `fit`: 放进目标矩形的方式，见 [`Fit`]。默认 [`Fit::Contain`]
///
/// # 示例
/// ```
/// let size = VideoSize { width: 1919, height: 1080, fit: Fit::Contain };
/// assert_eq!(even_dim(size.width), 1918);
/// ```
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VideoSize {
  /// 目标宽度。0 视为没设置。奇数在滤镜里收成偶数
  pub width: u32,
  /// 目标高度。0 视为没设置。奇数在滤镜里收成偶数
  pub height: u32,
  /// 放进目标矩形的方式，见 [`Fit`]。默认 [`Fit::Contain`]
  #[serde(default)]
  pub fit: Fit,
}

/// 标题文字从哪来
///
/// `Text` 时文字不能为空。`Filename` 在转码、裁剪里用源文件名，在合并、追加里用每一段自己的文件名。
///
/// # 变体
/// - `Text`: 自定义文字，不能为空。默认
/// - `Filename`: 转码、裁剪用源文件名，合并、追加用每一段自己的文件名
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TitleSource {
  /// 自定义文字，不能为空。默认
  #[default]
  Text,
  /// 转码、裁剪用源文件名，合并、追加用每一段自己的文件名
  Filename,
}

/// 标题锚点
///
/// 默认左上。水平靠左用边距，靠中用 `(w-text_w)/2`，靠右用 `w-text_w-{margin}`。垂直同样换成高度。换算见 [`anchor_xy`]。
///
/// # 变体
/// - `TopStart`: 左上。默认
/// - `TopCenter`: 上中
/// - `TopEnd`: 右上
/// - `CenterStart`: 左中
/// - `Center`: 正中
/// - `CenterEnd`: 右中
/// - `BottomStart`: 左下
/// - `BottomCenter`: 下中
/// - `BottomEnd`: 右下
///
/// # 示例
/// 见 [`anchor_xy`]。
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TitleAnchor {
  /// 左上。默认
  #[default]
  TopStart,
  /// 上中
  TopCenter,
  /// 右上
  TopEnd,
  /// 左中
  CenterStart,
  /// 正中
  Center,
  /// 右中
  CenterEnd,
  /// 左下
  BottomStart,
  /// 下中
  BottomCenter,
  /// 右下
  BottomEnd,
}

/// 标题叠加
///
/// 默认不画。打开之后只改内容、位置和时间窗。颜色、底衬和字体缩放在滤镜里写死。
/// `font_size` 为 0 时按 24。这是 1920 宽时的像素，再按输出宽度缩放，最小 16。
/// `font_file` 空则用 `get_default_font_path()`。没有 fontconfig，标题走 `fontfile=`。
/// `start` / `end` 是成片时间轴上的秒。都空则整段都画。编码时换算到当前片段的局部时间，见 [`title_span`]。
/// 片段和窗口不相交就不画。合并、追加按各段在成片上的起止来换算。裁剪先按片段顺序拼成成片，再做同样的换算。
///
/// # 字段
/// - `source`: 文字从哪来，见 [`TitleSource`]。默认自定义文字
/// - `text`: 自定义文字。来源是 [`TitleSource::Text`] 时不能为空
/// - `anchor`: 位置，见 [`TitleAnchor`]。默认左上
/// - `font_size`: 1920 宽时的像素。0 时按 24，再按输出宽度缩放，最小 16
/// - `font_file`: 字体文件。空则用 `get_default_font_path()`。没有 fontconfig，标题走 `fontfile=`
/// - `start`: 成片时间轴上的开始秒。和 `end` 都空则整段都画
/// - `end`: 成片时间轴上的结束秒。和 `start` 都空则整段都画
///
/// # 示例
/// ```
/// let title = TitleOverlay { start: Some(8.0), end: Some(12.0), ..TitleOverlay::default() };
/// assert_eq!(title_span(&title, 10.0, 5.0), TitleSpan::Range(0.0, 2.0));
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct TitleOverlay {
  /// 文字从哪来，见 [`TitleSource`]。默认自定义文字
  pub source: TitleSource,
  /// 自定义文字。来源是 [`TitleSource::Text`] 时不能为空
  pub text: String,
  /// 位置，见 [`TitleAnchor`]。默认左上
  pub anchor: TitleAnchor,
  /// 1920 宽时的像素。0 时按 24，再按输出宽度缩放，最小 16
  pub font_size: u32,
  /// 字体文件。空则用 `get_default_font_path()`
  pub font_file: Option<String>,
  /// 成片时间轴上的开始秒。和 `end` 都空则整段都画
  pub start: Option<f64>,
  /// 成片时间轴上的结束秒。和 `start` 都空则整段都画
  pub end: Option<f64>,
}

impl Default for TitleOverlay {
  /// 返回一份空的标题配置
  ///
  /// 来源是自定义文字，文字为空，锚点左上，字号 0，没有字体和时间窗。
  ///
  /// # 返回值
  /// 默认的 `TitleOverlay`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let title = TitleOverlay::default();
  /// assert!(title.text.is_empty());
  /// ```
  fn default() -> Self {
    Self {
      source: TitleSource::Text,
      text: String::new(),
      anchor: TitleAnchor::TopStart,
      font_size: 0,
      font_file: None,
      start: None,
      end: None,
    }
  }
}

/// 字幕从哪来
///
/// # 变体
/// - `ExternalFile`: 外挂文件。默认。原样写进 `subtitles=`
/// - `EmbeddedTrack`: 内嵌轨，用该段自己的源文件和 `si={track_index}`
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SubtitleSource {
  /// 外挂文件。默认。原样写进 `subtitles=`
  #[default]
  ExternalFile,
  /// 内嵌轨，用该段自己的源文件和 `si={track_index}`
  EmbeddedTrack,
}

/// 字幕烧录
///
/// 默认不烧。烧进去之后输出不再带字幕轨。只支持文本字幕。
/// 外挂 `.srt`、`.ass`、`.webvtt` 加在滤镜末尾：`subtitles='{file}':fontsdir='{dir}'`。
/// 内嵌轨用该段自己的源文件和 `si={track_index}`。外挂文件原样写进 `subtitles=`。
/// `fonts_dir` 空则用默认字体所在目录。没有 fontconfig，字幕走 `fontsdir=`。
/// `force_style` 只对没有自带样式的 SRT、WebVTT 生效。字幕没有单独的时间窗，跟所在片段一起烧。
///
/// # 字段
/// - `source`: 字幕从哪来，见 [`SubtitleSource`]。默认外挂文件
/// - `file`: 外挂字幕路径。来源是外挂文件时不能为空
/// - `track_index`: 内嵌字幕序号。来源是内嵌轨时必须有
/// - `fonts_dir`: 字体目录。空则用默认字体所在目录。没有 fontconfig，字幕走 `fontsdir=`
/// - `force_style`: 强制样式。只对没有自带样式的 SRT、WebVTT 生效
///
/// # 示例
/// ```
/// let subtitle = SubtitleOverlay {
///   source: SubtitleSource::EmbeddedTrack,
///   track_index: Some(0),
///   ..SubtitleOverlay::default()
/// };
/// assert_eq!(subtitle.track_index, Some(0));
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct SubtitleOverlay {
  /// 字幕从哪来，见 [`SubtitleSource`]。默认外挂文件
  pub source: SubtitleSource,
  /// 外挂字幕路径。来源是外挂文件时不能为空
  pub file: Option<String>,
  /// 内嵌字幕序号。来源是内嵌轨时必须有
  pub track_index: Option<u32>,
  /// 字体目录。空则用默认字体所在目录
  pub fonts_dir: Option<String>,
  /// 强制样式。只对没有自带样式的 SRT、WebVTT 生效
  pub force_style: Option<String>,
}

impl Default for SubtitleOverlay {
  /// 返回一份空的外挂字幕配置
  ///
  /// # 返回值
  /// 来源是外挂文件，路径、序号、字体目录和样式都空着
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let subtitle = SubtitleOverlay::default();
  /// assert!(subtitle.file.is_none());
  /// ```
  fn default() -> Self {
    Self { source: SubtitleSource::ExternalFile, file: None, track_index: None, fonts_dir: None, force_style: None }
  }
}

/// 转码、裁剪、合并、追加共用的编码配置
///
/// 省略的字段由 serde 默认值补上。
/// 当前编码用不到的字段不出现，避免 AV1 任务里夹着 HEVC 的 preset。
///
/// 默认是 MP4 + AV1（preset 6，CRF 35，10-bit）+ 第一条音轨 AAC 128 kbps，faststart 开。
/// 分辨率和帧率跟源。标题和字幕不烧。MPEG-TS 忽略 faststart。
/// 普通 1080p30 实拍，视频大约 1.5–3.5 Mbps，10 分钟大约 110–260 MB，音频另加约 10 MB。
/// 动作多、噪点多会更大。4K 大约是 1080p 的 3–4 倍。这是量级，不是承诺码率。
///
/// `fps` 只对转码和裁剪生效。合并取各段帧率的上中位数，追加跟基准，都忽略传入值。
///
/// # 字段
/// - `container`: 写出容器，见 [`VideoContainer`]。默认 MP4。MPEG-TS 忽略 faststart
/// - `video`: 视频编码，见 [`VideoParams`]。默认 10-bit AV1，preset 6，CRF 35
/// - `fps`: 帧率。只对转码和裁剪生效，空则跟源。合并取各段帧率的上中位数，追加跟基准，都忽略传入值
/// - `size`: 目标画面，见 [`VideoSize`]。空则转码、裁剪跟源，合并取各段最大宽高并补边，追加跟基准
/// - `audio`: 音轨选择，见 [`AudioSelection`]。默认第一条，AAC 128 kbps
/// - `faststart`: 默认开。MPEG-TS 忽略。MP4 和 MOV 写成 `+faststart`
/// - `title`: 标题，见 [`TitleOverlay`]。空则不烧
/// - `subtitle`: 字幕，见 [`SubtitleOverlay`]。空则不烧
///
/// # 示例
/// ```
/// let encode = VideoEncode::default();
/// assert_eq!(encode.container, VideoContainer::Mp4);
/// assert!(matches!(
///   encode.video,
///   VideoParams::Av1 { preset: 6, crf: 35, bit_depth: BitDepth::Ten }
/// ));
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct VideoEncode {
  /// 写出容器，见 [`VideoContainer`]。默认 MP4
  pub container: VideoContainer,
  /// 视频编码，见 [`VideoParams`]。默认 10-bit AV1，preset 6，CRF 35
  #[serde(default)]
  pub video: VideoParams,
  /// 帧率。只对转码和裁剪生效，空则跟源。合并和追加忽略传入值
  pub fps: Option<u32>,
  /// 目标画面，见 [`VideoSize`]。空则转码、裁剪跟源
  pub size: Option<VideoSize>,
  /// 音轨选择，见 [`AudioSelection`]。默认第一条，AAC 128 kbps
  pub audio: AudioSelection,
  /// 默认开。MPEG-TS 忽略。MP4 和 MOV 写成 `+faststart`
  #[serde(default = "default_true")]
  pub faststart: bool,
  /// 标题，见 [`TitleOverlay`]。空则不烧
  pub title: Option<TitleOverlay>,
  /// 字幕，见 [`SubtitleOverlay`]。空则不烧
  pub subtitle: Option<SubtitleOverlay>,
}

impl Default for VideoEncode {
  /// 返回默认编码配置
  ///
  /// 容器 MP4，视频为默认 AV1，帧率和尺寸留空，音轨为默认选择，faststart 打开，不烧标题和字幕。
  ///
  /// # 返回值
  /// 默认的 `VideoEncode`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let encode = VideoEncode::default();
  /// assert!(encode.faststart);
  /// assert!(encode.title.is_none());
  /// ```
  fn default() -> Self {
    Self {
      container: VideoContainer::Mp4,
      video: VideoParams::default(),
      fps: None,
      size: None,
      audio: AudioSelection::default(),
      faststart: true,
      title: None,
      subtitle: None,
    }
  }
}

/// 把越界的速度、质量和帧率收进可用范围，拒绝不合法组合
///
/// AV1 preset 超过 13 收成 13，VP9 `cpu-used` 超过 5 收成 5。CRF 超过上限时收成上限：AV1 和 VP9 是 63，HEVC 是 51。帧率不在 1 到 120 时改成 `None`。宽或高为 0 时清掉尺寸。字体文件或字体目录不存在时清掉并记警告。容器是 MPEG-TS 时关掉 faststart。
///
/// # 参数
/// - `encode`: 待收紧的编码配置
///
/// # 返回值
/// 成功时无额外值
///
/// # 错误
/// 声道不是 1 或 2、采样率不是 44100 或 48000、码率不在 64 到 320、适配方式是 `Cover`、自定义标题为空、标题结束不晚于开始、外挂字幕路径为空或文件不存在、内嵌字幕没有序号、VP9 写入 MPEG-TS 时，返回对应的中文 `String`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// normalize_encode(&mut encode)?;
/// ```
pub fn normalize_encode(encode: &mut VideoEncode) -> Result<(), String> {
  match &mut encode.video {
    VideoParams::Av1 { preset, crf, .. } => {
      *preset = (*preset).min(13);
      *crf = (*crf).min(63);
    }
    VideoParams::Hevc { crf, .. } => {
      *crf = (*crf).min(51);
    }
    VideoParams::Vp9 { cpu_used, crf, .. } => {
      *cpu_used = (*cpu_used).min(5);
      *crf = (*crf).min(63);
    }
  }

  if encode.fps.is_some_and(|fps| fps < 1 || fps > 120) {
    encode.fps = None;
  }

  if encode.audio.channels.is_some_and(|channels| channels != 1 && channels != 2) {
    return Err("声道只支持 1 或 2".to_string());
  }
  if encode.audio.sample_rate.is_some_and(|rate| rate != 44100 && rate != 48000) {
    return Err("采样率只支持 44100 或 48000".to_string());
  }
  if !(64..=320).contains(&encode.audio.bitrate_kbps) {
    return Err("音频码率要在 64 到 320 之间".to_string());
  }

  if let Some(size) = encode.size {
    if size.width == 0 || size.height == 0 {
      encode.size = None;
    } else if size.fit == Fit::Cover {
      return Err("当前版本不支持裁剪铺满".to_string());
    }
  }

  if let Some(title) = &encode.title {
    if title.source == TitleSource::Text && title.text.trim().is_empty() {
      return Err("标题文字不能为空".to_string());
    }
    if let (Some(start), Some(end)) = (title.start, title.end) {
      if end <= start {
        return Err("标题结束时间要晚于开始".to_string());
      }
    }
  }

  if let Some(subtitle) = &encode.subtitle {
    match subtitle.source {
      SubtitleSource::ExternalFile => {
        let file = subtitle.file.as_deref().unwrap_or("").trim();
        if file.is_empty() {
          return Err("字幕文件不能为空".to_string());
        }
        if !std::path::Path::new(file).is_file() {
          return Err("字幕文件不存在".to_string());
        }
      }
      SubtitleSource::EmbeddedTrack => {
        if subtitle.track_index.is_none() {
          return Err("未指定字幕轨".to_string());
        }
      }
    }
  }

  if let Some(title) = &mut encode.title {
    if title.font_file.as_deref().is_some_and(|path| !std::path::Path::new(path).is_file()) {
      tauri_plugin_log::log::warn!("字体不存在，改用系统字体");
      title.font_file = None;
    }
  }
  if let Some(subtitle) = &mut encode.subtitle {
    if subtitle.fonts_dir.as_deref().is_some_and(|path| !std::path::Path::new(path).is_dir()) {
      tauri_plugin_log::log::warn!("字体目录不存在，改用系统字体目录");
      subtitle.fonts_dir = None;
    }
  }

  if matches!(encode.video, VideoParams::Vp9 { .. }) && encode.container == VideoContainer::MpegTs {
    return Err("VP9 不能写入 MPEG-TS".to_string());
  }
  if encode.container == VideoContainer::MpegTs {
    encode.faststart = false;
  }
  Ok(())
}

/// 把尺寸收成不小于 2 的偶数
///
/// 小于 2 时变成 2。奇数减 1。
///
/// # 参数
/// - `value`: 原始尺寸
///
/// # 返回值
/// 偶数尺寸
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(even_dim(0), 2);
/// assert_eq!(even_dim(5), 4);
/// ```
pub fn even_dim(value: u32) -> u32 {
  let value = value.max(2);
  if value % 2 == 0 { value } else { value - 1 }
}

/// 采样率怎么定
///
/// 合并固定 48000。追加在基准被复制时跟基准，基准被重编码时用 48000。
/// 转码和裁剪在没指定采样率时，44100 和 48000 保持原样，其他值改成 48000。
///
/// # 变体
/// - `Source`: 没指定采样率时保留 44100 和 48000，其他改成 48000
/// - `Force`: 使用给定采样率。载荷是采样率
#[derive(Clone, Copy, Debug)]
pub enum RatePolicy {
  /// 没指定采样率时保留 44100 和 48000，其他改成 48000
  Source,
  /// 使用给定采样率
  Force(u32),
}

/// 一条音轨计划
///
/// 丢掉音频或音轨列表为空时不选流。指定序号不存在时退回第一条，并把 `fallback` 设为真。
///
/// # 字段
/// - `stream_index`: 绝对流序号。空表示不要音频
/// - `channels`: 声道数。未指定时，源不超过 1 声道就用 1，否则用 2
/// - `sample_rate`: 采样率。不要音频时记 0
/// - `bitrate_kbps`: 码率，单位 kbps
/// - `fallback`: 指定序号不存在、退回第一条时为真
///
/// # 示例
/// 见 [`plan_audio`]。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioPlan {
  /// 绝对流序号。空表示不要音频。
  pub stream_index: Option<u32>,
  /// 声道数。未指定时，源不超过 1 声道就用 1，否则用 2
  pub channels: u8,
  /// 采样率。不要音频时记 0
  pub sample_rate: u32,
  /// 码率，单位 kbps
  pub bitrate_kbps: u32,
  /// 指定序号不存在、退回第一条时为真
  pub fallback: bool,
}

/// 按选择和采样率策略挑出一条音轨计划
///
/// 丢掉音频或音轨列表为空时不选流，采样率记 0，声道用指定值或 2。指定序号不存在时退回第一条，并把 `fallback` 设为真。声道未指定时，源不超过 1 声道就用 1，否则用 2。`Force` 使用给定采样率。`Source` 在未指定时保留 44100 和 48000，其他改成 48000。
///
/// # 参数
/// - `selection`: 音轨选择
/// - `tracks`: 探测到的音轨
/// - `rate`: 采样率策略
///
/// # 返回值
/// 音轨计划
///
/// # Panics
/// `AudioMode::Drop` 若落到后面的 `unreachable!()` 会 panic。`tracks` 为空时 `tracks.first().expect` 会 panic。这两种情况都先在函数开头返回
///
/// # 示例
/// ```
/// let plan = plan_audio(&AudioSelection::default(), &[], RatePolicy::Source);
/// assert!(plan.stream_index.is_none());
/// ```
pub fn plan_audio(selection: &AudioSelection, tracks: &[AudioTrack], rate: RatePolicy) -> AudioPlan {
  if matches!(selection.mode, AudioMode::Drop) || tracks.is_empty() {
    return AudioPlan {
      stream_index: None,
      channels: selection.channels.unwrap_or(2),
      sample_rate: 0,
      bitrate_kbps: selection.bitrate_kbps,
      fallback: false,
    };
  }

  let (track, fallback) = match selection.mode {
    AudioMode::Drop => unreachable!(),
    AudioMode::First => (tracks.first().expect("tracks 非空"), false),
    AudioMode::Index { index } => match tracks.iter().find(|track| track.index == index) {
      Some(track) => (track, false),
      None => (tracks.first().expect("tracks 非空"), true),
    },
  };

  let channels = selection.channels.unwrap_or(if track.channels <= 1 { 1 } else { 2 });
  let sample_rate = match rate {
    RatePolicy::Force(rate) => rate,
    RatePolicy::Source => selection
      .sample_rate
      .unwrap_or(if track.sample_rate == 44100 || track.sample_rate == 48000 { track.sample_rate } else { 48000 }),
  };

  AudioPlan { stream_index: Some(track.index), channels, sample_rate, bitrate_kbps: selection.bitrate_kbps, fallback }
}

/// 在要保留音频却没有可选音轨时返回错误
///
/// # 参数
/// - `mode`: 音轨选择模式
/// - `plan`: 音轨计划
/// - `path`: 视频路径，写进错误文本
///
/// # 返回值
/// 可以继续时无额外值
///
/// # 错误
/// 模式不是丢掉音频，且 `stream_index` 为空时，返回 `没有音轨: {path}`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let plan = AudioPlan {
///   stream_index: None,
///   channels: 2,
///   sample_rate: 0,
///   bitrate_kbps: 128,
///   fallback: false,
/// };
/// let err = ensure_audio(AudioMode::First, &plan, "a.mp4").unwrap_err();
/// assert!(err.contains("没有音轨"));
/// ```
pub fn ensure_audio(mode: AudioMode, plan: &AudioPlan, path: &str) -> Result<(), String> {
  if !matches!(mode, AudioMode::Drop) && plan.stream_index.is_none() {
    Err(format!("没有音轨: {path}"))
  } else {
    Ok(())
  }
}

/// 把参数名和值依次推进列表
///
/// # 参数
/// - `args`: 参数列表
/// - `key`: 参数名
/// - `value`: 参数值
///
/// # 返回值
/// 无返回值
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let mut args = Vec::new();
/// push_pair(&mut args, "-c:v", "libx265");
/// assert_eq!(args, ["-c:v", "libx265"]);
/// ```
fn push_pair(args: &mut Vec<String>, key: &str, value: impl Into<String>) {
  args.push(key.to_string());
  args.push(value.into());
}

/// 按宽度选取 VP9 的 `tile-columns`
///
/// 大于 0 且小于 1280 用 1，大于 1920 用 3，其余（含 0 和 1280 到 1920）用 2。
///
/// # 参数
/// - `width`: 输出宽度
///
/// # 返回值
/// `tile-columns` 数值
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(tile_columns(1280), 2);
/// assert_eq!(tile_columns(1921), 3);
/// ```
fn tile_columns(width: u32) -> u8 {
  if width > 0 && width < 1280 {
    1
  } else if width > 1920 {
    3
  } else {
    2
  }
}

/// 写出视频和音频的编码参数
///
/// 不写流选择。AV1 写 `libsvtav1`、CRF 和 preset。HEVC 写 `libx265`、CRF、preset，有 tune 时写上；10-bit 再加 `main10`。VP9 写 `libvpx-vp9`、CRF、`-b:v 0`、`deadline good`、`cpu-used`、`row-mt`、`auto-alt-ref`、`lag-in-frames 25`、`frame-parallel 0`，以及按宽度取的 `tile-columns`。有音轨序号时写 AAC、声道、码率和大于 0 的采样率，否则写 `-an`。HEVC 且容器是 MP4 或 MOV 时加 `hvc1`。faststart 且容器是 MP4 或 MOV 时加 `+faststart`。
///
/// # 参数
/// - `encode`: 编码配置
/// - `audio`: 音轨计划
/// - `width`: 用来计算 VP9 `tile-columns` 的宽度
///
/// # 返回值
/// ffmpeg 参数
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let plan = AudioPlan {
///   stream_index: Some(0),
///   channels: 2,
///   sample_rate: 48000,
///   bitrate_kbps: 128,
///   fallback: false,
/// };
/// let args = encode_args(&VideoEncode::default(), &plan, 1920);
/// assert!(args.join(" ").contains("-c:v libsvtav1"));
/// ```
pub fn encode_args(encode: &VideoEncode, audio: &AudioPlan, width: u32) -> Vec<String> {
  let mut args = Vec::new();

  match &encode.video {
    VideoParams::Av1 { preset, crf, .. } => {
      push_pair(&mut args, "-c:v", "libsvtav1");
      push_pair(&mut args, "-crf", crf.to_string());
      push_pair(&mut args, "-preset", preset.to_string());
    }
    VideoParams::Hevc { preset, crf, tune, bit_depth } => {
      push_pair(&mut args, "-c:v", "libx265");
      push_pair(&mut args, "-crf", crf.to_string());
      push_pair(&mut args, "-preset", preset.ffmpeg());
      if let Some(tune) = tune.ffmpeg() {
        push_pair(&mut args, "-tune", tune);
      }
      if *bit_depth == BitDepth::Ten {
        push_pair(&mut args, "-profile:v", "main10");
      }
    }
    VideoParams::Vp9 { cpu_used, crf, content } => {
      push_pair(&mut args, "-c:v", "libvpx-vp9");
      push_pair(&mut args, "-crf", crf.to_string());
      push_pair(&mut args, "-b:v", "0");
      push_pair(&mut args, "-deadline", "good");
      push_pair(&mut args, "-cpu-used", cpu_used.to_string());
      push_pair(&mut args, "-row-mt", "1");
      push_pair(&mut args, "-auto-alt-ref", "1");
      push_pair(&mut args, "-lag-in-frames", "25");
      push_pair(&mut args, "-frame-parallel", "0");
      push_pair(&mut args, "-tile-columns", tile_columns(width).to_string());
      if let Some(content) = content.ffmpeg() {
        push_pair(&mut args, "-tune-content", content);
      }
    }
  }

  push_pair(&mut args, "-pix_fmt", encode.video.pix_fmt());

  if audio.stream_index.is_some() {
    push_pair(&mut args, "-c:a", "aac");
    push_pair(&mut args, "-ac", audio.channels.to_string());
    push_pair(&mut args, "-b:a", format!("{}k", audio.bitrate_kbps));
    if audio.sample_rate > 0 {
      push_pair(&mut args, "-ar", audio.sample_rate.to_string());
    }
  } else {
    args.push("-an".to_string());
  }

  if matches!(encode.video, VideoParams::Hevc { .. })
    && matches!(encode.container, VideoContainer::Mp4 | VideoContainer::Mov)
  {
    push_pair(&mut args, "-tag:v", "hvc1");
  }
  if encode.faststart && matches!(encode.container, VideoContainer::Mp4 | VideoContainer::Mov) {
    push_pair(&mut args, "-movflags", "+faststart");
  }

  args
}

/// 在参数末尾写上封装、覆盖、输出路径和 `-hide_banner`
///
/// # 参数
/// - `args`: 参数列表
/// - `encode`: 编码配置，用来取封装名
/// - `output`: 输出路径
///
/// # 返回值
/// 无返回值
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let mut args = Vec::new();
/// push_output(&mut args, &VideoEncode::default(), "out.mp4");
/// assert!(args.join(" ").contains("-f mp4"));
/// ```
pub fn push_output(args: &mut Vec<String>, encode: &VideoEncode, output: &str) {
  push_pair(args, "-f", encode.container.muxer());
  args.push("-y".to_string());
  args.push(output.to_string());
  args.push("-hide_banner".to_string());
}

/// 把成片时间窗换算到当前片段的局部时间
///
/// 都空则整段都画。片段和窗口不相交就跳过。换算后起点加 0.001 仍不早于终点时也跳过。
///
/// # 变体
/// - `All`: 开始和结束都空，整段都画
/// - `Range`: 局部时间窗。`0` 是起点秒，`1` 是终点秒
/// - `Skip`: 片段和窗口不相交，或换算后起点加 0.001 仍不早于终点
///
/// # 示例
/// 见 [`title_span`]。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TitleSpan {
  /// 开始和结束都空，整段都画
  All,
  /// 局部时间窗。`0` 是起点秒，`1` 是终点秒
  Range(f64, f64),
  /// 片段和窗口不相交，或换算后起点加 0.001 仍不早于终点
  Skip,
}

/// 把成片时间窗换算到当前片段的局部时间
///
/// 开始和结束都空时返回 `All`。开始空按 0，结束空按无穷大。换算后起点加 0.001 仍不早于终点时返回 `Skip`。
///
/// # 参数
/// - `title`: 标题配置
/// - `offset`: 这一段在成片上的起点
/// - `duration`: 这一段的时长
///
/// # 返回值
/// `All`、`Range` 或 `Skip`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(title_span(&TitleOverlay::default(), 3.0, 4.0), TitleSpan::All);
/// ```
pub fn title_span(title: &TitleOverlay, offset: f64, duration: f64) -> TitleSpan {
  if title.start.is_none() && title.end.is_none() {
    return TitleSpan::All;
  }
  let start = title.start.unwrap_or(0.0);
  let end = title.end.unwrap_or(f64::MAX);
  let local_start = (start - offset).max(0.0);
  let local_end = if end.is_finite() { (end - offset).min(duration) } else { duration };
  if local_start + 0.001 >= local_end { TitleSpan::Skip } else { TitleSpan::Range(local_start, local_end) }
}

/// 判断源编码名是否属于目标编码
///
/// 比较前把源编码转成小写。AV1 看是否包含 `av1`。HEVC 看 `hevc` 或 `h265`。VP9 看 `vp9`。
///
/// # 参数
/// - `codec`: 源编码名
/// - `video`: 目标视频参数
///
/// # 返回值
/// 属于该编码时为 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(source_codec_matches("HEVC", &VideoParams::Hevc {
///   preset: HevcPreset::Medium,
///   crf: 26,
///   tune: HevcTune::None,
///   bit_depth: BitDepth::Ten,
/// }));
/// ```
pub fn source_codec_matches(codec: &str, video: &VideoParams) -> bool {
  let codec = codec.to_ascii_lowercase();
  match video {
    VideoParams::Av1 { .. } => codec.contains("av1"),
    VideoParams::Hevc { .. } => codec.contains("hevc") || codec.contains("h265"),
    VideoParams::Vp9 { .. } => codec.contains("vp9"),
  }
}

/// 给滤镜参数加上单引号，并转义反斜杠和单引号
///
/// 反斜杠写成两个反斜杠。单引号写成 `'\''`。
///
/// # 参数
/// - `value`: 原始文本
///
/// # 返回值
/// 加引号后的文本
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(quote_filter("a'b"), "'a'\\''b'");
/// ```
fn quote_filter(value: &str) -> String {
  let mut out = String::from("'");
  for ch in value.chars() {
    if ch == '\\' {
      out.push_str("\\\\");
    } else if ch == '\'' {
      out.push_str("'\\''");
    } else {
      out.push(ch);
    }
  }
  out.push('\'');
  out
}

/// 按输出宽度相对 1920 缩放一个像素值
///
/// 宽度为 0 时按 1920。结果四舍五入，并且不小于 `min`。
///
/// # 参数
/// - `base`: 1920 宽时的像素值
/// - `width`: 输出宽度
/// - `min`: 最小值
///
/// # 返回值
/// 缩放后的像素值
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(scaled(24.0, 960, 16), 16);
/// ```
fn scaled(base: f64, width: u32, min: u32) -> u32 {
  let width = if width == 0 { 1920 } else { width };
  ((base * width as f64 / 1920.0).round() as u32).max(min)
}

/// 把标题锚点换成 drawtext 的 x、y 表达式
///
/// 水平靠中间用 `(w-text_w)/2`，靠右用 `w-text_w-{margin}`。垂直靠中间用 `(h-text_h)/2`，靠下用 `h-text_h-{margin}`。靠左或靠上直接用边距。
///
/// # 参数
/// - `anchor`: 锚点
/// - `margin`: 边距像素
///
/// # 返回值
/// `(x 表达式, y 表达式)`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let (x, y) = anchor_xy(TitleAnchor::TopStart, 8);
/// assert_eq!((x.as_str(), y.as_str()), ("8", "8"));
/// ```
fn anchor_xy(anchor: TitleAnchor, margin: u32) -> (String, String) {
  let margin = margin.to_string();
  let x = match anchor {
    TitleAnchor::TopStart | TitleAnchor::CenterStart | TitleAnchor::BottomStart => margin.clone(),
    TitleAnchor::TopCenter | TitleAnchor::Center | TitleAnchor::BottomCenter => "(w-text_w)/2".to_string(),
    TitleAnchor::TopEnd | TitleAnchor::CenterEnd | TitleAnchor::BottomEnd => format!("w-text_w-{margin}"),
  };
  let y = match anchor {
    TitleAnchor::TopStart | TitleAnchor::TopCenter | TitleAnchor::TopEnd => margin,
    TitleAnchor::CenterStart | TitleAnchor::Center | TitleAnchor::CenterEnd => "(h-text_h)/2".to_string(),
    TitleAnchor::BottomStart | TitleAnchor::BottomCenter | TitleAnchor::BottomEnd => format!("h-text_h-{margin}"),
  };
  (x, y)
}

/// 画到滤镜里的标题
///
/// 字号为 0 时按 24。字号和边距按 `width` 相对 1920 缩放，最小分别是 16 和 8。
///
/// # 字段
/// - `text`: 标题文字
/// - `anchor`: 锚点，见 [`TitleAnchor`]
/// - `font_file`: 字体文件，写成 `fontfile=`
/// - `font_size`: 字号。0 时按 24，再按 `width` 相对 1920 缩放，最小 16
/// - `width`: 输出宽度，用来缩放字号和边距。0 时按 1920
/// - `window`: 局部时间窗。有值时写成 `enable='between(t,{start:.3},{end:.3})'`。空则不限时间
///
/// # 示例
/// ```
/// let filter = video_filter(&VideoFilter {
///   fps: None,
///   size: None,
///   title: Some(TitleDraw {
///     text: "片名",
///     anchor: TitleAnchor::TopStart,
///     font_file: "font.ttf",
///     font_size: 0,
///     width: 1920,
///     window: None,
///   }),
///   subtitle: None,
///   pix_fmt: "yuv420p",
/// });
/// assert!(filter.as_deref().is_some_and(|text| text.contains("fontsize=24")));
/// ```
pub struct TitleDraw<'a> {
  /// 标题文字
  pub text: &'a str,
  /// 锚点，见 [`TitleAnchor`]
  pub anchor: TitleAnchor,
  /// 字体文件，写成 `fontfile=`
  pub font_file: &'a str,
  /// 字号。0 时按 24，再按宽度相对 1920 缩放，最小 16
  pub font_size: u32,
  /// 输出宽度，用来缩放字号和边距。0 时按 1920
  pub width: u32,
  /// 局部时间窗。空则不限时间
  pub window: Option<(f64, f64)>,
}

/// 画到滤镜里的字幕
///
/// `source_start` 大于 0 时，前后各加一次 `setpts`。
///
/// # 字段
/// - `file`: 字幕文件，写成 `subtitles=`
/// - `fonts_dir`: 字体目录。非空时写成 `fontsdir=`
/// - `track_index`: 内嵌字幕序号。有值时写成 `si=`
/// - `force_style`: 强制样式。非空时写成 `force_style=`
/// - `source_start`: 裁剪时把时间轴拨回源视频，字幕时间才能对上。大于 0 时前后各加一次 `setpts`
///
/// # 示例
/// ```
/// let filter = video_filter(&VideoFilter {
///   fps: None,
///   size: None,
///   title: None,
///   subtitle: Some(SubtitleDraw {
///     file: "a.srt",
///     fonts_dir: None,
///     track_index: None,
///     force_style: None,
///     source_start: Some(5.0),
///   }),
///   pix_fmt: "yuv420p",
/// });
/// assert!(filter.as_deref().is_some_and(|text| text.contains("setpts=PTS+5")));
/// ```
pub struct SubtitleDraw<'a> {
  /// 字幕文件，写成 `subtitles=`
  pub file: &'a str,
  /// 字体目录。非空时写成 `fontsdir=`
  pub fonts_dir: Option<&'a str>,
  /// 内嵌字幕序号。有值时写成 `si=`
  pub track_index: Option<u32>,
  /// 强制样式。非空时写成 `force_style=`
  pub force_style: Option<&'a str>,
  /// 裁剪时把时间轴拨回源视频，字幕时间才能对上。
  pub source_start: Option<f64>,
}

/// 一条视频滤镜的内容
///
/// 顺序是 `fps`、尺寸、`drawtext`、`subtitles`，最后 `format`。前面没有任何滤镜时不单独写 `format`。
///
/// # 字段
/// - `fps`: 帧率。大于 0 时写成 `fps=`
/// - `size`: 目标尺寸，见 [`VideoSize`]
/// - `title`: 标题，见 [`TitleDraw`]
/// - `subtitle`: 字幕，见 [`SubtitleDraw`]
/// - `pix_fmt`: 像素格式，最后写成 `format=`。前面没有其他滤镜时整段省略
///
/// # 示例
/// 见 [`video_filter`]。
pub struct VideoFilter<'a> {
  /// 帧率。大于 0 时写成 `fps=`
  pub fps: Option<f64>,
  /// 目标尺寸，见 [`VideoSize`]
  pub size: Option<VideoSize>,
  /// 标题，见 [`TitleDraw`]
  pub title: Option<TitleDraw<'a>>,
  /// 字幕，见 [`SubtitleDraw`]
  pub subtitle: Option<SubtitleDraw<'a>>,
  /// 像素格式，最后写成 `format=`
  pub pix_fmt: &'a str,
}

/// 按固定顺序拼出视频滤镜
///
/// 顺序是 `fps`、尺寸、`drawtext`、`subtitles`，最后 `format`。尺寸先把源宽高取偶并 `setsar=1`，`Fill` 直接拉伸，`Contain` 和 `Cover` 都是缩小后补边，然后再 `setsar=1`。字号为 0 时按 24。字号和边距按宽度相对 1920 缩放，最小分别是 16 和 8。字幕的 `source_start` 大于 0 时，前后各加一次 `setpts`。前面没有任何滤镜时返回 `None`，不单独写 `format`。
///
/// # 参数
/// - `spec`: 滤镜内容
///
/// # 返回值
/// 用逗号接起来的滤镜。没有帧率、尺寸、标题和字幕时返回 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let filter = video_filter(&VideoFilter {
///   fps: Some(30.0),
///   size: None,
///   title: None,
///   subtitle: None,
///   pix_fmt: "yuv420p",
/// });
/// assert_eq!(filter.as_deref(), Some("fps=30,format=yuv420p"));
/// ```
pub fn video_filter(spec: &VideoFilter<'_>) -> Option<String> {
  let mut filters = Vec::new();
  if let Some(fps) = spec.fps.filter(|fps| *fps > 0.0) {
    filters.push(format!("fps={fps}"));
  }
  if let Some(size) = spec.size {
    let width = even_dim(size.width);
    let height = even_dim(size.height);
    filters.push("scale='trunc(iw*sar/2)*2':'trunc(ih/2)*2'".to_string());
    filters.push("setsar=1".to_string());
    match size.fit {
      Fit::Fill => filters.push(format!("scale={width}:{height}")),
      Fit::Contain | Fit::Cover => {
        filters.push(format!("scale={width}:{height}:force_original_aspect_ratio=decrease"));
        filters.push(format!("pad={width}:{height}:(ow-iw)/2:(oh-ih)/2"));
      }
    }
    filters.push("setsar=1".to_string());
  }
  if let Some(title) = &spec.title {
    let font_size = if title.font_size == 0 { 24 } else { title.font_size };
    let size = scaled(font_size as f64, title.width, 16);
    let margin = scaled(10.0, title.width, 8);
    let (x, y) = anchor_xy(title.anchor, margin);
    let mut draw = format!(
      "drawtext=fontfile={}:text={}:expansion=none:fontcolor=white:fontsize={size}:x={}:y={}",
      quote_filter(title.font_file),
      quote_filter(title.text),
      quote_filter(&x),
      quote_filter(&y),
    );
    if let Some((start, end)) = title.window {
      draw.push_str(&format!(":enable='between(t,{start:.3},{end:.3})'"));
    }
    filters.push(draw);
  }
  if let Some(subtitle) = &spec.subtitle {
    if subtitle.source_start.is_some_and(|start| start > 0.0) {
      filters.push(format!("setpts=PTS+{}/TB", subtitle.source_start.unwrap_or(0.0)));
    }
    let mut sub = format!("subtitles={}", quote_filter(subtitle.file));
    if let Some(index) = subtitle.track_index {
      sub.push_str(&format!(":si={index}"));
    }
    if let Some(dir) = subtitle.fonts_dir.filter(|dir| !dir.is_empty()) {
      sub.push_str(&format!(":fontsdir={}", quote_filter(dir)));
    }
    if let Some(style) = subtitle.force_style.filter(|style| !style.is_empty()) {
      sub.push_str(&format!(":force_style={}", quote_filter(style)));
    }
    filters.push(sub);
    if subtitle.source_start.is_some_and(|start| start > 0.0) {
      filters.push("setpts=PTS-STARTPTS".to_string());
    }
  }
  if filters.is_empty() {
    return None;
  }
  filters.push(format!("format={}", spec.pix_fmt));
  Some(filters.join(","))
}

#[cfg(test)]
mod tests {
  use super::*;

  fn stereo() -> AudioPlan {
    AudioPlan { stream_index: Some(1), channels: 2, sample_rate: 48000, bitrate_kbps: 128, fallback: false }
  }

  #[test]
  fn default_json_is_av1_mp4() {
    let encode: VideoEncode = serde_json::from_str("{}").expect("默认对象");
    let joined = encode_args(&encode, &stereo(), 1920).join(" ");
    assert!(joined.contains("libsvtav1"), "{joined}");
    assert!(joined.contains("-preset 6"), "{joined}");
    assert!(joined.contains("-crf 35"), "{joined}");
    assert!(joined.contains("-pix_fmt yuv420p10le"), "{joined}");
    assert!(joined.contains("-c:a aac"), "{joined}");
    assert!(joined.contains("-b:a 128k"), "{joined}");
    assert!(joined.contains("-movflags +faststart"), "{joined}");
    assert!(!joined.contains("hvc1"), "{joined}");
  }

  #[test]
  fn hevc_shortcut_uses_its_own_defaults() {
    let encode: VideoEncode = serde_json::from_str(r#"{"video":{"codec":"hevc"}}"#).expect("hevc");
    match encode.video {
      VideoParams::Hevc { preset, crf, tune, bit_depth } => {
        assert_eq!(preset, HevcPreset::Medium);
        assert_eq!(crf, 26);
        assert_eq!(tune, HevcTune::None);
        assert_eq!(bit_depth, BitDepth::Ten);
      }
      other => panic!("应是 hevc: {other:?}"),
    }
  }

  #[test]
  fn av1_ten_bit_json_reaches_yuv420p10le() {
    let raw = r#"{
      "container": "mp4",
      "video": { "codec": "av1", "preset": 6, "crf": 35, "bitDepth": "ten" },
      "audio": { "mode": { "kind": "first" }, "channels": null, "sampleRate": null, "bitrateKbps": 128 },
      "faststart": true
    }"#;
    let encode: VideoEncode = serde_json::from_str(raw).expect("界面提交的 10-bit");
    let joined = encode_args(&encode, &stereo(), 1920).join(" ");
    assert!(joined.contains("-pix_fmt yuv420p10le"), "{joined}");
    assert!(!joined.contains("yuv420p "), "{joined}");
  }

  #[test]
  fn hevc_ten_bit_mp4_tags_hvc1_and_mpegts_does_not() {
    let mut encode = VideoEncode {
      video: VideoParams::Hevc {
        preset: HevcPreset::Fast,
        crf: 22,
        tune: HevcTune::Animation,
        bit_depth: BitDepth::Ten,
      },
      ..VideoEncode::default()
    };
    let joined = encode_args(&encode, &AudioPlan { stream_index: None, ..stereo() }, 1280).join(" ");
    assert!(joined.contains("libx265"), "{joined}");
    assert!(joined.contains("-preset fast"), "{joined}");
    assert!(joined.contains("-tune animation"), "{joined}");
    assert!(joined.contains("-profile:v main10"), "{joined}");
    assert!(joined.contains("-pix_fmt yuv420p10le"), "{joined}");
    assert!(joined.contains("-tag:v hvc1"), "{joined}");
    assert!(joined.contains("-an"), "{joined}");

    encode.container = VideoContainer::MpegTs;
    normalize_encode(&mut encode).expect("mpegts");
    let joined = encode_args(&encode, &stereo(), 1280).join(" ");
    assert!(!joined.contains("hvc1"), "{joined}");
    assert!(!joined.contains("faststart"), "{joined}");
  }

  #[test]
  fn vp9_locks_internal_speed_flags() {
    let encode = VideoEncode {
      container: VideoContainer::Mp4,
      video: VideoParams::Vp9 { cpu_used: 2, crf: 33, content: Vp9Content::Screen },
      faststart: true,
      ..VideoEncode::default()
    };
    let joined = encode_args(&encode, &stereo(), 3840).join(" ");
    assert!(joined.contains("libvpx-vp9"), "{joined}");
    assert!(joined.contains("-b:v 0"), "{joined}");
    assert!(joined.contains("-deadline good"), "{joined}");
    assert!(joined.contains("-cpu-used 2"), "{joined}");
    assert!(joined.contains("-row-mt 1"), "{joined}");
    assert!(joined.contains("-tile-columns 3"), "{joined}");
    assert!(joined.contains("-tune-content screen"), "{joined}");
  }

  #[test]
  fn normalize_clamps_and_rejects() {
    let mut encode = VideoEncode {
      video: VideoParams::Av1 { preset: 99, crf: 80, bit_depth: BitDepth::Eight },
      fps: Some(400),
      ..VideoEncode::default()
    };
    normalize_encode(&mut encode).expect("夹取");
    match encode.video {
      VideoParams::Av1 { preset, crf, .. } => {
        assert_eq!(preset, 13);
        assert_eq!(crf, 63);
      }
      other => panic!("{other:?}"),
    }
    assert_eq!(encode.fps, None);

    encode.video = VideoParams::Vp9 { cpu_used: 2, crf: 33, content: Vp9Content::General };
    encode.container = VideoContainer::MpegTs;
    assert!(normalize_encode(&mut encode).unwrap_err().contains("VP9"));

    encode.container = VideoContainer::Mp4;
    encode.size = Some(VideoSize { width: 100, height: 100, fit: Fit::Cover });
    assert!(normalize_encode(&mut encode).unwrap_err().contains("铺满"));
  }

  #[test]
  fn title_window_maps_onto_a_segment() {
    let title = TitleOverlay { start: Some(8.0), end: Some(12.0), ..TitleOverlay::default() };
    assert_eq!(title_span(&title, 0.0, 5.0), TitleSpan::Skip);
    assert_eq!(title_span(&title, 10.0, 5.0), TitleSpan::Range(0.0, 2.0));
    assert_eq!(title_span(&TitleOverlay::default(), 3.0, 4.0), TitleSpan::All);
  }

  #[test]
  fn filter_puts_drawtext_before_subtitles() {
    let filter = video_filter(&VideoFilter {
      fps: Some(30.0),
      size: Some(VideoSize { width: 1919, height: 1080, fit: Fit::Contain }),
      title: Some(TitleDraw {
        text: "片名",
        anchor: TitleAnchor::TopStart,
        font_file: "/System/Library/Fonts/PingFang.ttc",
        font_size: 0,
        width: 1920,
        window: None,
      }),
      subtitle: Some(SubtitleDraw {
        file: "/tmp/a.srt",
        fonts_dir: Some("/Library/Fonts"),
        track_index: None,
        force_style: None,
        source_start: Some(5.0),
      }),
      pix_fmt: "yuv420p",
    })
    .expect("有滤镜");
    let title_at = filter.find("drawtext").expect("标题");
    let subtitle_at = filter.find("subtitles").expect("字幕");
    assert!(title_at < subtitle_at, "{filter}");
    assert!(filter.contains("scale=1918:1080"), "{filter}");
    assert!(filter.contains("setpts=PTS+5"), "{filter}");
    assert!(filter.ends_with("format=yuv420p"), "{filter}");
  }

  #[test]
  fn missing_audio_track_falls_back_to_the_first() {
    let tracks = vec![AudioTrack { index: 1, channels: 1, sample_rate: 44100, ..AudioTrack::default() }];
    let selection = AudioSelection { mode: AudioMode::Index { index: 9 }, ..AudioSelection::default() };
    let plan = plan_audio(&selection, &tracks, RatePolicy::Source);
    assert_eq!(plan.stream_index, Some(1));
    assert!(plan.fallback);
    assert_eq!(plan.channels, 1);
    assert_eq!(plan.sample_rate, 44100);
  }
}
