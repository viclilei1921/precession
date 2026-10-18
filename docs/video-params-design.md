# 视频处理参数

面向 Tauri 2 桌面端。侧车能力以 [mac-ffmpeg.md](./mac-ffmpeg.md)、[window-ffmpeg.md](./window-ffmpeg.md) 为准。转码、裁剪、合并、追加共用一套 `VideoEncode`。前端不传时，后端 `Default` 就是「点默认就导出」。

## 1. 侧车边界

| 能力 | 现状 | 对参数的影响 |
| --- | --- | --- |
| 视频编码器 | `libsvtav1`、`libx265`、`libvpx-vp9`，纯软件 | 三选一。没有 GPU，也不能编 H.264 |
| 音频编码器 | 内置 `aac` | 只能选轨道、声道、采样率、码率。编码格式固定 AAC |
| 可写封装 | `mp4`、`mov`、`mpegts` | 只能写出这三种。WebM 只有 demuxer，能读不能写 |
| 视频滤镜 | `fps`、`scale`、`setsar`、`pad`、`drawtext`、`format`、`concat`、`subtitles`、`ass` | 画面适配只有「缩放加黑边」和「拉伸」。没有 `crop`，铺满裁切做不到 |
| 音频滤镜 | `aresample`、`aformat` | 合并、追加靠它们把各段音频对齐 |
| 字体 | 没有 fontconfig | 标题用 `fontfile=`，字幕用 `fontsdir=` |
| 其他 | 无网络、无图片、无位图字幕 | 标题和字幕只烧文字。PGS、DVD 字幕不可用 |

MOV 是 QuickTime 容器，和 MP4 同族。侧车已经能写。默认导出仍是 MP4。

## 2. 默认导出

默认是 **MP4 + AV1 + AAC**。

| 项 | 默认 |
| --- | --- |
| 封装 | MP4 |
| 视频 | AV1，`preset 6`，CRF 35，10-bit。AV1 和 HEVC 可改回 8-bit |
| 分辨率 / 帧率 | 跟源。合并、追加另有对齐规则，见第 6 节 |
| 音频 | 第一条音轨。单声道保持单声道，两声道以上混成立体声。AAC 128 kbps |
| faststart | 开。MPEG-TS 忽略 |
| 标题 / 字幕 | 不烧 |

普通 1080p30 实拍，视频大约 1.5–3.5 Mbps，10 分钟大约 110–260 MB，音频另加约 10 MB。动作多、噪点多会更大。4K 大约是 1080p 的 3–4 倍。这是量级，不是承诺码率。

## 3. 速度和质量

数据模型里不放共用的「快 / 均衡 / 慢」。三种编码器的刻度不能互换，任务里要记下真正写给 FFmpeg 的值。界面可以给推荐档，选中后写入该编码器自己的字段。

两个旋钮分开：

- **速度**不改变质量档的含义。同一个 CRF 下，更慢的速度压得更紧，文件更小，也更干净，等待更久。
- **质量（CRF）**决定观感和体积。数值越小越清晰、文件越大。想再压体积，优先把 CRF 调大一档，不要只把速度调快。

CRF 的数字不能跨编码器比较。同一行才是大致相同的观感。

| 观感 | AV1 `libsvtav1` | HEVC `libx265` | VP9 `libvpx-vp9` |
| --- | --- | --- | --- |
| 更清晰，文件偏大 | CRF 30 | CRF 22 | CRF 30 |
| **默认，适合保存** | **CRF 35** | **CRF 26** | **CRF 33** |
| 更小，运动和暗部会糊 | CRF 40 | CRF 30 | CRF 38 |

CRF 每加大约 2，体积大约少 15–25%，只供估算。

### 3.1 AV1

`preset` 范围 0–13，越小越慢。CRF 范围 0–63。

| 档位 | preset | 说明 |
| --- | --- | --- |
| 更小 | 4 | 同样 CRF 下更小，明显更慢 |
| **默认** | **6** | 落盘保存的平衡点。不用库默认的 10，同样 CRF 下 10 的文件会大一截 |
| 较快 | 8 | 文件大约多一成，耗时下降 |
| 很快 | 10 | 文件更大，只适合赶时间 |

默认 CRF 35，是这个编码器常用的质量点。

`bit_depth` 默认 10，像素格式用 `yuv420p10le`，渐变更干净，体积通常接近 8-bit。要兼容更多播放器时再改回 8。屏幕内容检测继续固定关闭。

### 3.2 HEVC

`preset` 用 x265 的名字。CRF 范围 0–51。

| 档位 | preset | 说明 |
| --- | --- | --- |
| 很快 | veryfast | 文件更大 |
| 较快 | fast | |
| **默认** | **medium** | |
| 更小 | slow | 慢很多，体积只小一点 |
| 更慢 | slower | 命令行名字是 `slower` |

默认 CRF 26。比编码器自带的 28 干净一些，又比 22–24 小一截。

另有两项：

- `tune`：默认不传。`animation` 适合动画和线稿，边缘更干净，往往更小。`grain` 保留颗粒，文件变大。不提供 `psnr`、`ssim`、`zerolatency`、`fastdecode`，前两个是评测用的，后两个会让文件变大或只适合直播。
- `bit_depth`：和 AV1 同一套，默认 10，可改回 8。10-bit 用 `yuv420p10le` 和 `main10`。MP4、MOV 自动加 `-tag:v hvc1`。

界面上的 preset 只给上表五档。`ultrafast`、`placebo` 不进界面。

### 3.3 VP9

速度是 `cpu-used`，在 `deadline=good` 下只有 0–5，越小越慢。CRF 范围 0–63。VP9 压缩不如 AV1，默认停在 CRF 33：再往上提才能接近 AV1 的体积，观感会先掉下去。文件通常仍比 AV1 大。不作为默认编码。

| 档位 | cpu-used |
| --- | --- |
| 更小 | 1 |
| **默认** | **2** |
| 较快 | 4 |
| 很快 | 5 |

`content` 默认普通画面。`screen` 适合录屏，文字更清晰，往往更小。`film` 保留颗粒，文件变大。

这版侧车的 `libvpx-vp9` 没有 10-bit 像素格式，所以 VP9 没有 `bit_depth`。`deadline`、多线程、前瞻固定写死，见第 5 节。VP9 只写入 MP4，不给 MPEG-TS。

### 3.4 怎么选

- 保存、希望文件不要太大：AV1，preset 6，CRF 35。这就是默认。
- 要更快点、播放器也更熟：HEVC，medium，CRF 26。
- VP9 只在明确需要这种编码时用。

## 4. 参数模型

省略的字段由 `#[serde(default)]` 补成默认值。当前编码用不到的字段不出现，避免 AV1 任务里夹着 HEVC 的 preset。

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "codec", rename_all = "camelCase")]
pub enum VideoParams {
  Av1(Av1Params),
  Hevc(HevcParams),
  Vp9(Vp9Params),
}

pub enum BitDepth { Eight, Ten } // 默认 Ten。只有 AV1 和 HEVC 有这个字段

pub struct Av1Params {
  pub preset: u8,         // 默认 6，范围 0–13
  pub crf: u8,            // 默认 35，范围 0–63
  pub bit_depth: BitDepth, // 默认 Ten
}

pub struct HevcParams {
  pub preset: HevcPreset, // 默认 Medium
  pub crf: u8,            // 默认 26，范围 0–51
  pub tune: HevcTune,     // 默认 None
  pub bit_depth: BitDepth, // 默认 Ten
}

pub enum HevcPreset { VeryFast, Fast, Medium, Slow, Slower }
pub enum HevcTune { None, Animation, Grain }

pub struct Vp9Params {
  pub cpu_used: u8,          // 默认 2，范围 0–5
  pub crf: u8,               // 默认 33，范围 0–63
  pub content: Vp9Content,   // 默认 General
}

pub enum Vp9Content { General, Screen, Film } // 命令行：default / screen / film

pub struct VideoEncode {
  pub container: VideoContainer,     // 默认 Mp4
  pub video: VideoParams,            // 默认 Av1
  pub fps: Option<u32>,              // 空=自动，只对转码和裁剪生效
  pub size: Option<VideoSize>,       // 空=自动
  pub audio: AudioSelection,
  pub faststart: bool,               // 默认 true
  pub title: Option<TitleOverlay>,   // 默认不画
  pub subtitle: Option<SubtitleOverlay>, // 默认不烧
}
```

`VideoParams` 的默认变体是 `Av1`。某个变体内部缺字段时，用该结构体的默认值。例如 `{ "codec": "hevc" }` 等于 medium、CRF 26、不调优、10-bit。`{ "codec": "av1", "bitDepth": "eight" }` 才写回 8-bit AV1。

### 4.1 音频

探测先把轨道列出来，选择留到编码参数里。

```rust
pub struct AudioTrack {
  pub index: u32,             // Stream #0:N 的 N，-map 0:N 用它
  pub codec: String,
  pub sample_rate: u32,
  pub channels: u32,
  pub channel_layout: String, // stereo、mono、5.1
  pub bitrate_kbps: u32,
  pub language: String,       // 流名字里的 eng、jpn，没有就是空
  pub title: String,          // 紧随其后的 Metadata title，没有就是空
}

pub enum AudioMode {
  First,          // 默认
  Index { index: u32 },
  Drop,
}

pub struct AudioSelection {
  pub mode: AudioMode,
  pub channels: Option<u8>,     // 空=自动：1 保持 1，大于 1 混成 2。指定时只允许 1 或 2
  pub sample_rate: Option<u32>, // 空=按操作决定。指定时只允许 44100 或 48000
  pub bitrate_kbps: u32,        // 默认 128。界面给 96 / 128 / 160 / 192
}
```

`VideoInfo` 保留现在的路径、宽高、帧率、时长、视频编码、总码率，把单个 `audioCodec` 换成 `audio_tracks`。默认仍用第一条。合并、追加里，`Index` 对每个输入用同一个序号；该文件没有这条轨道时退回第一条并记一条警告。多条音轨全部保留先不做，拼接时音轨条数对不齐。

映射：

- `First`、`Index`：`-map 0:a:{n}?`，再加上 `-c:a aac -b:a {bitrate}k`。声道用 `-ac`，采样率需要改时加 `-ar`。
- `Drop`：`-an`。

### 4.2 画面尺寸

```rust
pub enum Fit { Contain, Fill } // 默认 Contain

pub struct VideoSize {
  pub width: u32,
  pub height: u32,
  pub fit: Fit,
}
```

- 空：转码、裁剪跟源。合并取各段最大宽高。追加跟基准。
- `contain`：放进目标矩形，不足处补黑边。
- `fill`：拉伸到目标矩形。
- 铺满后裁掉多余画面需要 `crop`，侧车没有这个滤镜，入队直接拒绝。

宽或高为 0 视为没设置。奇数在滤镜里收成偶数。

### 4.3 标题

默认不画。打开之后只改内容、位置和时间窗。颜色、底衬、字体缩放固定，见第 5 节。

```rust
pub enum TitleSource { Text, Filename }

pub enum TitleAnchor {
  TopStart, TopCenter, TopEnd,
  CenterStart, Center, CenterEnd,
  BottomStart, BottomCenter, BottomEnd,
}

pub struct TitleOverlay {
  pub source: TitleSource,     // Text 时 text 不能为空
  pub text: String,
  pub anchor: TitleAnchor,     // 默认 TopStart
  pub font_size: u32,          // 0=按 24。这是 1920 宽时的像素，再按输出宽度缩放，最小 16
  pub font_file: Option<String>, // 空=get_default_font_path()
  pub start: Option<f64>,      // 成片时间轴上的秒。都空=整段都画
  pub end: Option<f64>,
}
```

`Filename` 在转码、裁剪里用源文件名；在合并、追加里用每一段自己的文件名。

时间窗按成片时间填写，编码时换算到当前片段的局部时间。片段和窗口不相交就不画。合并、追加按各段在成片上的起止来换算。裁剪先按片段顺序拼成成片，再做同样的换算。

### 4.4 字幕

默认不烧。烧进去之后输出不再带字幕轨。只支持文本字幕。

```rust
pub enum SubtitleSource { ExternalFile, EmbeddedTrack }

pub struct SubtitleOverlay {
  pub source: SubtitleSource,      // 默认 ExternalFile
  pub file: Option<String>,        // 外挂 .srt / .ass / .webvtt
  pub track_index: Option<u32>,    // 内嵌文本轨的流序号
  pub fonts_dir: Option<String>,   // 空=默认字体所在目录
  pub force_style: Option<String>, // 只对没有自带样式的 SRT、WebVTT 生效
}
```

外挂字幕加在滤镜末尾：`subtitles='{file}':fontsdir='{dir}'`。内嵌轨用 `subtitles=源文件:si={track_index}:fontsdir=...`。合并、追加有多个输入时，滤镜里的输入序号按实际命令换算。时间窗规则和标题相同。

## 5. 写死、不开放的参数

这些影响体积、速度或能不能播放，但不交给用户改。

| 编码 | 固定项 |
| --- | --- |
| AV1 | 8-bit 用 `yuv420p`，10-bit 用 `yuv420p10le`。不传 film-grain，不加 `svtav1-params` |
| HEVC | 8-bit 用 `yuv420p`，10-bit 用 `yuv420p10le` 和 `main10`。MP4、MOV 加 `-tag:v hvc1` |
| VP9 | 只有 8-bit `yuv420p`。另加 `-b:v 0`、`-deadline good`、`-row-mt 1`、`-auto-alt-ref 1`、`-lag-in-frames 25`、`-frame-parallel 0`。`tile-columns` 按宽度取：小于 1280 用 1，不超过 1920 用 2，更宽用 3 |
| 音频 | 编码器只有 AAC。不暴露 `aac_coder` |
| 标题 | 白字，没有底衬。1920 宽时字号 24、边距 10，按宽度缩放，字号最小 16、边距最小 8。`expansion=none`，转义 `\`、`:`、`'`。字体默认走 `get_default_font_path()` |
| 封装 | MP4、MOV 且 `faststart` 为真时加 `-movflags +faststart` |

不提供 `svtav1-params`、`x265-params` 这类自由字符串，避免和上面的具名字段互相覆盖。

关键帧间隔用编码器自己的默认（大约数秒），方便拖动进度，也不为了再小一点把间隔拉得很长。

## 6. 四种操作

| 参数 | 转码 | 裁剪 | 合并 | 追加 |
| --- | --- | --- | --- | --- |
| 容器、编码、音频码率、faststart | 可改 | 可改 | 可改 | 可改 |
| 帧率 | 可改，空则跟源 | 同左 | 取各段帧率的上中位数，忽略传入值 | 跟基准，忽略传入值 |
| 尺寸 | 可改，空则跟源 | 同左 | 空则取最大宽高并补边 | 空则跟基准 |
| 音轨 | 按 `AudioSelection` | 同左 | 每段各选一条，再统一采样率和声道 | 同合并。基准若直接复制，音频也复制 |
| 标题、字幕 | 整片 | 按成片时间换算到各片段 | 每段各烧 | 新片段各烧。基准要烧就必须重编码 |

裁剪仍是逐段编码，再无损拼起来。编码参数只作用在前面的编码上。

追加先把各段收成同一种编码、同尺寸、同帧率、同像素格式，再用 MPEG-TS 拼好，最后复制进目标容器。基准已经和这个目标一致，而且不用在基准上烧标题或字幕时，基准直接复制。否则基准也走同一套参数重编码。默认 AV1 时，常见的 H.264 基准会重编码。这一步不单独做开关。

有滤镜时顺序固定：

`fps → 宽高取偶 → scale（contain 或 fill）→ pad → setsar=1 → drawtext → subtitles → format`

`format` 跟位深走：8-bit 是 `yuv420p`，AV1 或 HEVC 的 10-bit 是 `yuv420p10le`。转码和裁剪如果没有改帧率、没有改尺寸、没有标题、没有字幕，就不加滤镜，像素格式只靠 `-pix_fmt`。

合并、追加的音频滤镜把采样率对齐到本节规则：合并固定 48000。追加在基准被复制时跟基准，基准被重编码时用 48000。转码和裁剪在没指定采样率时，44100 和 48000 保持原样，其他值改成 48000。

## 7. 入队校验

| 情况 | 处理 |
| --- | --- |
| AV1 preset 超出 0–13，VP9 `cpu-used` 超出 0–5 | 夹到范围内 |
| CRF 超出该编码器上限 | 夹到范围内。AV1、VP9 上限 63，HEVC 上限 51 |
| AV1 或 HEVC 的 `bit_depth` 不是 8 或 10 | 入队失败 |
| 帧率不在 1–120 | 忽略，按自动处理 |
| `channels` 有值但不是 1 或 2 | 入队失败 |
| `sample_rate` 有值但不是 44100 或 48000 | 入队失败 |
| `bitrate_kbps` 不在 64–320 | 入队失败 |
| 尺寸宽或高为 0 | 视为没设置 |
| `fit` 要裁切铺满 | 入队失败，侧车没有 `crop` |
| 音轨序号超出该文件的音轨数 | 退回第一条并警告 |
| 标题是自定义文字但文本为空，或结束时间不晚于开始 | 入队失败 |
| 外挂字幕没填路径、文件不存在，或内嵌字幕没填序号 | 入队失败 |
| 字体文件或 `fonts_dir` 找不到 | 回退系统字体并警告 |
| VP9 且容器是 MPEG-TS | 入队失败 |
| `faststart` 且容器是 MPEG-TS | 忽略 faststart |

## 8. 这次不做

- 铺满裁切：侧车要加上 `crop`。
- 写出 WebM：侧车要加上 webm muxer。
- 保留全部音轨，或每段各自选不同序号。
- 固定码率、两遍编码。
- 编码器原始参数字符串。VP9 10-bit 也不做：这版 `libvpx-vp9` 没有 10-bit 像素格式。
- 位图字幕、图片水印。水印还要 `overlay` 滤镜。
