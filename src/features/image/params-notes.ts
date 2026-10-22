/** 说明来自 cjxl 手册：距离 1 为视觉无损，建议 0.5–3；质量 90 对应距离 1。 */
export const jxlDistance =
  '这不是 0–100 的画质百分比，而是和原图的视觉差距，数字越小越接近原图。0 是像素无损。1 是默认，cjxl 称为看不出差别，大约相当于 JPEG 质量 90。2 到 3 文件更小，平滑区域可能看出一点损失。官方建议日常只用 0.5 到 3，允许写到 25，但再大就会明显发糊。要像素一致就填 0，或勾选无损。';

export const jxlEffort =
  '不改变你设的距离，只决定编码器肯花多少时间去贴近它。7 是默认。调到 9 更慢：有损时通常更准、文件更小，无损时也往往更小。1 到 4 只适合先快速出一张看看。';

export const jxlDecoding =
  '0 是默认，文件更小，打开稍慢。调到 2 或 4，打开更快，文件会变大，画质也可能略差。只存档、偶尔打开就留 0。';

export const jxlAlphaDistance =
  '只作用于透明通道，刻度和距离一样，越小边缘越精确。留空就跟色彩距离走。抠图和标志要干净边缘就填 0。柔和的半透明可以比色彩距离再大一点，仍建议落在 0.5 到 3。';

export const jxlLossless = '勾上后按距离 0 编码，像素和原图一致，文件明显更大。已经打算用距离压缩就不要勾。';

export const jxlLosslessJpeg =
  '只对 JPEG 原图有意义：按原来的 JPEG 码流保存，以后还能还原那一份 JPEG，而不只是看起来一样。PNG 勾了没有这个效果。勾上之后，距离不再决定这张 JPEG 的画面。';

export const avifLossless =
  '勾上会要求像素无损。色彩质量和透明质量这时应是 100，色度用 4:4:4，范围用全幅。否则编码器会警告，并退回有损。';

export const avifQuality =
  '数字越大越接近原图，100 是无损，0 最差。60 适合日常照片，80 以上更不容易看出差别，40 以下方块和涂抹会明显。它不管编码快慢。';

export const avifAlpha =
  '同一套 0–100，只作用于透明通道。没有透明的图可以忽略。有柔边就设得和色彩质量接近；简单剪影可以再低一些。';

export const avifSpeed =
  '0 最慢，10 最快，默认 6。同一个质量下，更慢通常文件更小，画面目标不变。想要更清楚请加质量，不要把速度调低当成加清晰。';

export const avifTargetSize =
  '单位是字节，例如 200000 大约是 200 KB。编码器会来回试质量去靠近这个大小，最多慢到原来的 7 倍。留空就完全按上面的质量来。';

export const avifDepth =
  '普通照片和截图用 8-bit。10-bit 或 12-bit 让天空、暗部的渐变更顺。原图本身只有 8-bit 时不会多出细节，文件更大，部分看图软件也只按 8-bit 显示。';

export const avifYuv =
  '自动时，JPEG 尽量沿用它自己的抽样，灰度图用 4:0:0，其余用 4:4:4。4:4:4 保留全部色彩，适合文字和截图。4:2:0 是常见照片的做法，更小，彩色文字边缘可能发糊。4:0:0 会丢掉颜色，只剩灰度。';

export const avifRange =
  '静图用全幅（0–255）。有限是视频的 16–235，普通图片选它会发灰。不确定就留自动，照片和 PNG 会按全幅。';

export const avifSharpYuv =
  '只在色度是 4:2:0 时生效，让彩色边缘少渗开。文字和图标要抽样成 4:2:0 时可以勾上。自动或 4:4:4 时这项会被忽略。';

export const avifPremultiply =
  '只有原图的颜色已经乘过透明度时才勾，例如某些渲染导出。普通 PNG 勾上后，透明边缘会发暗。';

export const avifExif = '拍摄信息和方向标记。忽略后不写进新文件。方向如果只记在 EXIF 里，有的图会横过来。';

export const avifXmp = '附加说明。忽略后不写进新文件，画面不变。';

export const avifIcc = '色彩配置。忽略后如果原图不是 sRGB，颜色可能偏移。';

export const avifTiles = '大图拆开解码，默认开着即可。关掉后，下面的分块行和分块列才生效。';

export const avifTileCount = '切成 2 的多少次方份：0 是 1 份，1 是 2 份，6 是 64 份。份数越多，单块越小。';

export const avifProgressive = '可以先显示一个粗画面，再补全。适合从网络打开。只在本地存档可以不勾。';

export const avifJobs = '编码用几条线程。留空就尽量占满核心。机器还要做别的事时再填小一点。';

export const jxlColor =
  '保持原样就对了。sRGB 是普通屏幕。线性 sRGB 只给已经是线性光的数据，普通照片标成线性会看起来不对。灰度会丢掉颜色。';

export const jxlIntensity =
  'HDR 画面的峰值亮度，单位尼特。普通照片留空，编码器按色彩空间自己选。只有 HDR 母版才需要填，例如 1000 或 4000。填错会改变高光怎么被压缩。';

export const jxlModular =
  '自动时，照片多走 VarDCT，插画、截图和无损多走模块化。VarDCT 适合照片。模块化适合边缘清楚的图形。拿不准就留自动。';

export const jxlResampling =
  '编码前把色彩缩小，打开时再放大。自动只在画质很低时才缩小。1 是不缩小，适合文字和截图。2、4、8 文件更小，彩色边缘更糊。';

export const jxlBitDepth =
  '跟随原图就对了。这是写进文件的位深，不会把 8-bit 照片变成更细的渐变。原图只有 8-bit 时，选 10、12 或 16 不会多出细节，文件可能更大。只有原图本身有这么多级，例如 16-bit PNG，才值得改。';

export const jxlPremultiply =
  '只有原图的颜色已经乘过透明度时才选预乘，例如某些渲染导出。普通 PNG 选预乘后，透明边缘会发暗。拿不准留自动。';

export const jxlPhoton =
  '按相机 ISO 往画面里加颗粒，用来抵消大距离时平滑区域发塑料的感觉。100 很少，3200 很多。留空或 0 不加。这会改变像素。';

export const jxlThreads = '编码用几条线程。留空就按这台机器的默认，通常尽量占满核心。填 1 就是单线程。';

export const jxlContainer =
  '勾上后一定使用 JPEG XL 容器，用来装拍摄信息和 JPEG 还原数据。不勾时，只有要写入 EXIF 才会自动打开。这不是画质开关。';

export const jxlProgressive = '由粗到细解码，网络上可以更早看到大概。压缩效率会差一点。存档不必勾。';
