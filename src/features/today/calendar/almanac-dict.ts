import { getCurrentLanguage } from '@/i18n';

/** 宜忌词中英映射。英文黄历惯用译法，按词表全量覆盖。 */
const ALMANAC_WORDS_EN: Record<string, string> = {
  安床: 'Set the bed',
  安机械: 'Install machinery',
  安门: 'Install doors',
  安香: 'Place incense',
  安葬: 'Burial',
  安装器具: 'Install appliances',
  搬家: 'Move house',
  搬新房: 'Move into a new home',
  补垣: 'Repair walls',
  拆卸: 'Dismantling',
  成服: 'Wear mourning garments',
  成人礼: 'Coming-of-age ceremony',
  乘船: 'Board a ship',
  酬神: 'Thank the gods',
  出火: 'Move the stove',
  出货财: 'Dispatch goods',
  出行: 'Travel',
  除虫: 'Pest control',
  除服: 'Remove mourning garments',
  打猎: 'Hunting',
  打扫: 'Cleaning',
  打鱼: 'Fishing',
  雕刻: 'Carving',
  订盟: 'Betrothal',
  定磉: 'Lay the foundation stone',
  动土: 'Groundbreaking',
  断蚁: 'Termite control',
  伐木: 'Felling trees',
  放水: 'Drain water',
  分居: 'Separate households',
  赴任: 'Take office',
  盖屋: 'Build a house',
  割蜜: 'Harvest honey',
  雇庸: 'Hire laborers',
  挂匾: 'Hang a plaque',
  归宁: 'Visit the parental home',
  归岫: 'Return home',
  合婚订婚: 'Betrothal',
  合嵴: 'Install the roof ridge',
  合寿木: 'Prepare a coffin',
  合帐: 'Hang curtains',
  坏垣: 'Demolish walls',
  会亲友: 'Meet friends and relatives',
  祭祀: 'Worship ancestors',
  架马: 'Set up scaffolding',
  见贵: 'Meet dignitaries',
  交易: 'Transactions',
  教牛马: 'Train cattle and horses',
  结婚: 'Wedding',
  结网: 'Set nets',
  掘井: 'Dig a well',
  开仓: 'Open the granary',
  开厕: 'Build a latrine',
  开池: 'Dig a pond',
  开光: 'Consecration',
  开渠: 'Dig canals',
  开生坟: 'Prepare a tomb',
  开业: 'Business opening',
  开柱眼: 'Cut column mortises',
  理发: 'Haircut',
  立碑: 'Erect a monument',
  买车: 'Buy a vehicle',
  买房: 'Buy a house',
  买衣服: 'Buy clothes',
  美甲浴足: 'Manicure and foot bath',
  沐浴: 'Bathing',
  牧养: 'Grazing',
  纳财: 'Receive wealth',
  纳畜: 'Acquire livestock',
  纳婿: 'Welcome a son-in-law',
  破土: 'Break ground',
  破屋: 'Demolish a house',
  普渡: 'Universal salvation',
  铺路: 'Pave roads',
  祈福: 'Pray for blessings',
  起基: 'Lay the foundation',
  迁坟: 'Relocate a grave',
  签订合同: 'Sign a contract',
  求财: 'Seek wealth',
  求医: 'Seek medical treatment',
  求职入学: 'Job hunting and enrollment',
  求子: 'Pray for children',
  入殓: 'Encoffin the deceased',
  塞穴: 'Plug openings',
  上梁: 'Install the roof beam',
  收养子女: 'Adopt children',
  竖柱: 'Erect pillars',
  诉讼: 'Litigation',
  塑绘: 'Sculpting and painting',
  探病: 'Visit the sick',
  问名: 'Betrothal inquiry',
  习艺: 'Learn a craft',
  谢土: 'Give thanks to the Earth God',
  行丧: 'Funeral procession',
  修坟: 'Repair a grave',
  修造: 'Repairs and construction',
  移柩: 'Move the coffin',
  馀事勿取: 'Refrain from other matters',
  栽种: 'Planting',
  造仓: 'Build a granary',
  造车器: 'Build vehicles',
  造畜稠: 'Build a livestock pen',
  造船: 'Shipbuilding',
  造庙: 'Build a temple',
  造桥: 'Build a bridge',
  斋醮: 'Taoist rites',
  针灸: 'Acupuncture',
  治病: 'Treat illness',
  诸事不宜: 'Nothing auspicious',
  筑堤: 'Build an embankment',
  装修: 'Renovation',
  作梁: 'Make beams',
  作灶: 'Build a stove'
};

/** 十二地支中英映射，使用通用罗马拼音 */
const ZHI_EN: Record<string, string> = {
  子: 'Zi',
  丑: 'Chou',
  寅: 'Yin',
  卯: 'Mao',
  辰: 'Chen',
  巳: 'Si',
  午: 'Wu',
  未: 'Wei',
  申: 'Shen',
  酉: 'You',
  戌: 'Xu',
  亥: 'Hai'
};

/** 生肖中英映射 */
const ANIMALS_EN: Record<string, string> = {
  鼠: 'Rat',
  牛: 'Ox',
  虎: 'Tiger',
  兔: 'Rabbit',
  龙: 'Dragon',
  蛇: 'Snake',
  马: 'Horse',
  羊: 'Goat',
  猴: 'Monkey',
  鸡: 'Rooster',
  狗: 'Dog',
  猪: 'Pig'
};

/** 天干中英映射 */
const GAN_EN: Record<string, string> = {
  甲: 'Jia',
  乙: 'Yi',
  丙: 'Bing',
  丁: 'Ding',
  戊: 'Wu',
  己: 'Ji',
  庚: 'Geng',
  辛: 'Xin',
  壬: 'Ren',
  癸: 'Gui'
};

/** 八方方位中英映射（不含「正」前缀） */
const DIRECTIONS_EN: Record<string, string> = {
  北: 'North',
  东北: 'Northeast',
  东: 'East',
  东南: 'Southeast',
  南: 'South',
  西南: 'Southwest',
  西: 'West',
  西北: 'Northwest'
};

/** 建除十二神中英映射 */
const JIAN_CHU_EN: Record<string, string> = {
  建日: 'Build day',
  除日: 'Remove day',
  满日: 'Full day',
  平日: 'Level day',
  定日: 'Settle day',
  执日: 'Hold day',
  破日: 'Break day',
  危日: 'Danger day',
  成日: 'Success day',
  收日: 'Receive day',
  开日: 'Open day',
  闭日: 'Close day'
};

/** 彭祖百忌中英映射，覆盖十天干和十二地支共 22 句 */
const PENG_ZU_EN: Record<string, string> = {
  甲不开仓财物耗散: 'Jia: no granary opening, wealth scatters',
  乙不栽植千株不长: 'Yi: no planting, nothing will grow',
  丙不修灶必见灾殃: 'Bing: no stove repair, calamity follows',
  丁不剃头头必生疮: 'Ding: no shaving, sores will appear',
  戊不受田田主不祥: 'Wu: no land deals, misfortune for the owner',
  己不破券二比并亡: 'Ji: no breaking pacts, both sides perish',
  庚不经络织机虚张: 'Geng: no weaving, the loom fails',
  辛不合酱主人不尝: 'Xin: no sauce-making, the host will not taste it',
  壬不汲水更难提防: 'Ren: no drawing water, danger is hard to guard against',
  癸不词讼理弱敌强: 'Gui: no litigation, your case is weak',
  子不问卜自惹祸殃: 'Zi: no divination, trouble will follow',
  丑不冠带主不还乡: 'Chou: no formal dress, the master will not return home',
  寅不祭祀神鬼不尝: 'Yin: no worship, spirits will not accept',
  卯不穿井水泉不香: 'Mao: no well-digging, the spring turns foul',
  辰不哭泣必主重丧: 'Chen: no weeping, mourning doubles',
  巳不远行财物伏藏: 'Si: no long travel, wealth stays hidden',
  午不苫盖屋主更张: 'Wu: no roof thatching, the house changes',
  未不服药毒气入肠: 'Wei: no medicine, poison enters the gut',
  申不安床鬼祟入房: 'Shen: no bed-setting, spirits enter the room',
  酉不宴客醉坐颠狂: 'You: no banquets, guests drink wildly',
  戌不吃犬作怪上床: 'Xu: no dog meat, mischief comes to bed',
  亥不嫁娶不利新郎: 'Hai: no weddings, unlucky for the groom'
};

/** 二十四节气中英映射，使用国际通用译名 */
const SOLAR_TERMS_EN: Record<string, string> = {
  立春: 'Start of Spring',
  雨水: 'Rain Water',
  惊蛰: 'Awakening of Insects',
  春分: 'Spring Equinox',
  清明: 'Pure Brightness',
  谷雨: 'Grain Rain',
  立夏: 'Start of Summer',
  小满: 'Grain Full',
  芒种: 'Grain in Ear',
  夏至: 'Summer Solstice',
  小暑: 'Minor Heat',
  大暑: 'Major Heat',
  立秋: 'Start of Autumn',
  处暑: 'End of Heat',
  白露: 'White Dew',
  秋分: 'Autumn Equinox',
  寒露: 'Cold Dew',
  霜降: "Frost's Descent",
  立冬: 'Start of Winter',
  小雪: 'Minor Snow',
  大雪: 'Major Snow',
  冬至: 'Winter Solstice',
  小寒: 'Minor Cold',
  大寒: 'Major Cold'
};

/** 当前是否为英文界面 */
function isEnglish(): boolean {
  return getCurrentLanguage() === 'en-US';
}

/** 单个宜忌词。中文界面或词典外词原样返回。 */
export function translateAlmanacWord(word: string): string {
  if (!isEnglish()) {
    return word;
  }
  return ALMANAC_WORDS_EN[word] ?? word;
}

/** 地支。中文界面或词典外原样返回。 */
export function translateZhi(zhi: string): string {
  if (!isEnglish()) {
    return zhi;
  }
  return ZHI_EN[zhi] ?? zhi;
}

/**
 * 空格分隔的宜忌词串。空串、「暂无」、「-」返回空串（由调用方换占位），
 * 其余逐个翻译后拼回。英文下用逗号分隔，方便区分短语边界。
 */
export function translateYiJiText(text: string): string {
  const trimmed = text.trim();
  if (!trimmed || trimmed === '-' || trimmed === '暂无') {
    return '';
  }
  if (!isEnglish()) {
    return trimmed;
  }
  return trimmed
    .split(/\s+/)
    .map((word) => ALMANAC_WORDS_EN[word] ?? word)
    .join(', ');
}

/** 生肖。中文界面或词典外原样返回。 */
export function translateAnimal(animal: string): string {
  if (!isEnglish()) {
    return animal;
  }
  return ANIMALS_EN[animal] ?? animal;
}

/** 干支（如「辛巳」）。逐字查天干地支后拼成拼音。 */
export function translateGanZhi(ganZhi: string): string {
  if (!isEnglish()) {
    return ganZhi;
  }
  const chars = [...ganZhi.trim()];
  if (chars.length !== 2) {
    return ganZhi;
  }
  const gan = GAN_EN[chars[0]];
  const zhi = ZHI_EN[chars[1]];
  if (!gan || !zhi) {
    return ganZhi;
  }
  return `${gan}${zhi}`;
}

/** 建除日（如「危日」）。中文界面或词典外原样返回。 */
export function translateJianChu(jianChu: string): string {
  if (!isEnglish()) {
    return jianChu;
  }
  return JIAN_CHU_EN[jianChu] ?? jianChu;
}

/** 方位（如「正东」「西南」）。自动去掉「正」前缀后查表。 */
export function translateDirection(direction: string): string {
  if (!isEnglish()) {
    return direction;
  }
  return DIRECTIONS_EN[direction.replace(/^正/, '')] ?? direction;
}

/** 冲煞（如「冲猪煞东」）。拆出生肖和方位分别翻译。 */
export function translateChongSha(chongSha: string): string {
  if (!isEnglish()) {
    return chongSha;
  }
  const match = chongSha.match(/^冲(.+?)煞(.+)$/);
  if (!match) {
    return chongSha;
  }
  const animal = ANIMALS_EN[match[1]] ?? match[1];
  const direction = DIRECTIONS_EN[match[2]] ?? match[2];
  return `Clash ${animal} · Sha ${direction}`;
}

/** 彭祖百忌（如「辛不合酱主人不尝 已不远行财物伏藏」）。逐句翻译后逗号连接。 */
export function translatePengZu(pengZu: string): string {
  const trimmed = pengZu.trim();
  if (!trimmed) {
    return trimmed;
  }
  if (!isEnglish()) {
    return trimmed;
  }
  return trimmed
    .split(/\s+/)
    .map((sentence) => PENG_ZU_EN[sentence] ?? sentence)
    .join(', ');
}

/** 节气（如「立冬」）。中文界面或词典外原样返回。 */
export function translateSolarTerm(solarTerm: string): string {
  if (!isEnglish()) {
    return solarTerm;
  }
  return SOLAR_TERMS_EN[solarTerm] ?? solarTerm;
}
