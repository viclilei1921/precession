//! 农历常量数据

/// 农历年份数据 (1900-2100)
pub const LUNAR_INFO: [u32; 201] = [
  19416, 19168, 42352, 21717, 53856, 55632, 91476, 22176, 39632, 21970, 19168, 42422, 42192, 53840, 119381, 46400,
  54944, 44450, 38320, 84343, 18800, 42160, 46261, 27216, 27968, 109396, 11104, 38256, 21234, 18800, 25958, 54432,
  59984, 28309, 23248, 11104, 100067, 37600, 116951, 51536, 54432, 120998, 46416, 22176, 107956, 9680, 37584, 53938,
  43344, 46423, 27808, 46416, 86869, 19872, 42416, 83315, 21168, 43432, 59728, 27296, 44710, 43856, 19296, 43748,
  42352, 21088, 62051, 55632, 23383, 22176, 38608, 19925, 19152, 42192, 54484, 53840, 54616, 46400, 46752, 103846,
  38320, 18864, 43380, 42160, 45690, 27216, 27968, 44870, 43872, 38256, 19189, 18800, 25776, 29859, 59984, 27480,
  23232, 43872, 38613, 37600, 51552, 55636, 54432, 55888, 30034, 22176, 43959, 9680, 37584, 51893, 43344, 46240, 47780,
  44368, 21977, 19360, 42416, 86390, 21168, 43312, 31060, 27296, 44368, 23378, 19296, 42726, 42208, 53856, 60005,
  54576, 23200, 30371, 38608, 19195, 19152, 42192, 118966, 53840, 54560, 56645, 46496, 22224, 21938, 18864, 42359,
  42160, 43600, 111189, 27936, 44448, 84835, 37744, 18936, 18800, 25776, 92326, 59984, 27424, 108228, 43744, 41696,
  53987, 51552, 54615, 54432, 55888, 23893, 22176, 42704, 21972, 21200, 43448, 43344, 46240, 46758, 44368, 21920,
  43940, 42416, 21168, 45683, 26928, 29495, 27296, 44368, 84821, 19296, 42352, 21732, 53600, 59752, 54560, 55968,
  92838, 22224, 19168, 43476, 41680, 53584, 62034, 54560,
];

/// 公历每月天数 (平年)
pub const SOLAR_MONTH: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// 天干
pub const GAN: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];

/// 地支
pub const ZHI: [&str; 12] = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];

/// 生肖
pub const ANIMALS: [&str; 12] = ["鼠", "牛", "虎", "兔", "龙", "蛇", "马", "羊", "猴", "鸡", "狗", "猪"];

/// 二十四节气
pub const SOLAR_TERM: [&str; 24] = [
  "小寒", "大寒", "立春", "雨水", "惊蛰", "春分", "清明", "谷雨", "立夏", "小满", "芒种", "夏至", "小暑", "大暑",
  "立秋", "处暑", "白露", "秋分", "寒露", "霜降", "立冬", "小雪", "大雪", "冬至",
];

/// 数字转中文
pub const N_STR_1: [&str; 11] = ["日", "一", "二", "三", "四", "五", "六", "七", "八", "九", "十"];
pub const N_STR_2: [&str; 4] = ["初", "十", "廿", "卅"];
pub const N_STR_3: [&str; 12] = ["正", "二", "三", "四", "五", "六", "七", "八", "九", "十", "冬", "腊"];

/// 吉凶
pub const JX_NAMES: [&str; 2] = ["吉", "凶"];

/// 天干彭祖百忌
pub const M_PZ_STEM: [&str; 10] = [
  "甲不开仓财物耗散",
  "乙不栽植千株不长",
  "丙不修灶必见灾殃",
  "丁不剃头头必生疮",
  "戊不受田田主不祥",
  "己不破券二比并亡",
  "庚不经络织机虚张",
  "辛不合酱主人不尝",
  "壬不汲水更难提防",
  "癸不词讼理弱敌强",
];

/// 地支彭祖百忌
pub const M_PZ_BRANCH: [&str; 12] = [
  "子不问卜自惹祸殃",
  "丑不冠带主不还乡",
  "寅不祭祀神鬼不尝",
  "卯不穿井水泉不香",
  "辰不哭泣必主重丧",
  "巳不远行财物伏藏",
  "午不苫盖屋主更张",
  "未不服药毒气入肠",
  "申不安床鬼祟入房",
  "酉不宴客醉坐颠狂",
  "戌不吃犬作怪上床",
  "亥不嫁娶不利新郎",
];

/// 八方名称
pub const COMPASS_NAMES: [&str; 8] = ["正北", "东北", "正东", "东南", "正南", "西南", "正西", "西北"];

/// 值神
pub const ZHI_SHEN_NAMES: [&str; 12] =
  ["青龙", "明堂", "天刑", "朱雀", "金匮", "天德", "白虎", "玉堂", "天牢", "玄武", "司命", "勾陈"];

/// 建除十二神
pub const JIAN_CHU_NAMES: [&str; 12] =
  ["建日", "除日", "满日", "平日", "定日", "执日", "破日", "危日", "成日", "收日", "开日", "闭日"];

/// 当日宜忌数据
pub fn daily_suit_avoid_dict() -> &'static std::collections::HashMap<String, DailySuitAvoid> {
  use std::collections::HashMap;
  use std::sync::OnceLock;
  #[derive(serde::Deserialize)]
  struct Raw {
    j: String,
    y: String,
  }
  static CACHE: OnceLock<HashMap<String, DailySuitAvoid>> = OnceLock::new();
  CACHE.get_or_init(|| {
    let raw: HashMap<String, Raw> =
      serde_json::from_str(include_str!("./data/daily_suit_avoid.json")).unwrap_or_default();
    raw.into_iter().map(|(k, v)| (k, DailySuitAvoid { yi: v.y, ji: v.j })).collect()
  })
}

#[derive(Clone)]
pub struct DailySuitAvoid {
  pub yi: String,
  pub ji: String,
}

/// 六十甲子 × 十二时辰宜忌
pub fn almanac_core_dict() -> &'static [AlmanacCoreEntry] {
  use std::sync::OnceLock;
  static CACHE: OnceLock<Vec<AlmanacCoreEntry>> = OnceLock::new();
  CACHE.get_or_init(|| {
    serde_json::from_str::<Vec<AlmanacCoreEntry>>(include_str!("./data/almanac_core_dict.json")).unwrap_or_default()
  })
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AlmanacCoreEntry {
  pub yi0: String,
  pub ji0: String,
  pub yi1: String,
  pub ji1: String,
  pub yi2: String,
  pub ji2: String,
  pub yi3: String,
  pub ji3: String,
  pub yi4: String,
  pub ji4: String,
  pub yi5: String,
  pub ji5: String,
  pub yi6: String,
  pub ji6: String,
  pub yi7: String,
  pub ji7: String,
  pub yi8: String,
  pub ji8: String,
  pub yi9: String,
  pub ji9: String,
  pub yi10: String,
  pub ji10: String,
  pub yi11: String,
  pub ji11: String,
}

/// 吉神宜趋、凶神宜忌 (神煞) 字典，key 为 "月建码-日干支" 如 "1-甲子"
pub fn auspicious_gods_dict() -> &'static std::collections::HashMap<String, ShenShaEntry> {
  use std::collections::HashMap;
  use std::sync::OnceLock;
  static CACHE: OnceLock<HashMap<String, ShenShaEntry>> = OnceLock::new();
  CACHE.get_or_init(|| serde_json::from_str(include_str!("./data/auspicious_gods_dict.json")).unwrap_or_default())
}

#[derive(serde::Deserialize)]
pub struct ShenShaEntry {
  #[serde(rename = "JSYQ")]
  pub jsyq: String,
  #[serde(rename = "XSYJ")]
  pub xsyj: String,
}

/// 节气偏移数据 - 从 terms_offset.txt 解析
pub fn terms_offset() -> &'static [u16] {
  use std::sync::OnceLock;
  static CACHE: OnceLock<Vec<u16>> = OnceLock::new();
  CACHE
    .get_or_init(|| include_str!("./data/terms_offset.txt").split(',').filter_map(|s| s.trim().parse().ok()).collect())
}

/// 纳音五行 lookup
pub fn get_nayin(gz: &str) -> String {
  const MAP: [(&str, &str); 60] = [
    ("甲子", "海中金"),
    ("乙丑", "海中金"),
    ("丙寅", "炉中火"),
    ("丁卯", "炉中火"),
    ("戊辰", "大林木"),
    ("己巳", "大林木"),
    ("庚午", "路旁土"),
    ("辛未", "路旁土"),
    ("壬申", "剑锋金"),
    ("癸酉", "剑锋金"),
    ("甲戌", "山头火"),
    ("乙亥", "山头火"),
    ("丙子", "涧下水"),
    ("丁丑", "涧下水"),
    ("戊寅", "城头土"),
    ("己卯", "城头土"),
    ("庚辰", "白腊金"),
    ("辛巳", "白腊金"),
    ("壬午", "杨柳木"),
    ("癸未", "杨柳木"),
    ("甲申", "泉中水"),
    ("乙酉", "泉中水"),
    ("丙戌", "屋上土"),
    ("丁亥", "屋上土"),
    ("戊子", "霹雳火"),
    ("己丑", "霹雳火"),
    ("庚寅", "松柏木"),
    ("辛卯", "松柏木"),
    ("壬辰", "长流水"),
    ("癸巳", "长流水"),
    ("甲午", "沙中金"),
    ("乙未", "沙中金"),
    ("丙申", "山下火"),
    ("丁酉", "山下火"),
    ("戊戌", "平地木"),
    ("己亥", "平地木"),
    ("庚子", "壁上土"),
    ("辛丑", "壁上土"),
    ("壬寅", "金箔金"),
    ("癸卯", "金箔金"),
    ("甲辰", "覆灯火"),
    ("乙巳", "覆灯火"),
    ("丙午", "天河水"),
    ("丁未", "天河水"),
    ("戊申", "大驿土"),
    ("己酉", "大驿土"),
    ("庚戌", "钗钏金"),
    ("辛亥", "钗钏金"),
    ("壬子", "桑拓木"),
    ("癸丑", "桑拓木"),
    ("甲寅", "大溪水"),
    ("乙卯", "大溪水"),
    ("丙辰", "沙中土"),
    ("丁巳", "沙中土"),
    ("戊午", "天上火"),
    ("己未", "天上火"),
    ("庚申", "石榴木"),
    ("辛酉", "石榴木"),
    ("壬戌", "大海水"),
    ("癸亥", "大海水"),
  ];
  MAP.iter().find(|(k, _)| *k == gz).map(|(_, v)| (*v).to_string()).unwrap_or_default()
}

/// 天干 → 五行
pub fn gan_to_wuxing(gan: &str) -> String {
  const MAP: [(&str, &str); 10] = [
    ("甲", "木"),
    ("乙", "木"),
    ("丙", "火"),
    ("丁", "火"),
    ("戊", "土"),
    ("己", "土"),
    ("庚", "金"),
    ("辛", "金"),
    ("壬", "水"),
    ("癸", "水"),
  ];
  MAP.iter().find(|(k, _)| *k == gan).map(|(_, v)| (*v).to_string()).unwrap_or_default()
}

/// 地支 → 五行
pub fn zhi_to_wuxing(zhi: &str) -> String {
  const MAP: [(&str, &str); 12] = [
    ("子", "水"),
    ("丑", "土"),
    ("寅", "木"),
    ("卯", "木"),
    ("辰", "土"),
    ("巳", "火"),
    ("午", "火"),
    ("未", "土"),
    ("申", "金"),
    ("酉", "金"),
    ("戌", "土"),
    ("亥", "水"),
  ];
  MAP.iter().find(|(k, _)| *k == zhi).map(|(_, v)| (*v).to_string()).unwrap_or_default()
}

/// 胎神方位字典 (宜避)，key 为 "月建码-日干支" 如 "1-甲子"
pub fn inuspicious_gods_dict() -> &'static std::collections::HashMap<String, String> {
  use std::collections::HashMap;
  use std::sync::OnceLock;
  static CACHE: OnceLock<HashMap<String, String>> = OnceLock::new();
  CACHE.get_or_init(|| serde_json::from_str(include_str!("./data/inuspicious_gods_dict.json")).unwrap_or_default())
}

/// 月支到建除码
pub const MONTH_ZHI_TO_CODE: [(&str, u8); 12] = [
  ("子", 11),
  ("丑", 12),
  ("寅", 1),
  ("卯", 2),
  ("辰", 3),
  ("巳", 4),
  ("午", 5),
  ("未", 6),
  ("申", 7),
  ("酉", 8),
  ("戌", 9),
  ("亥", 10),
];
