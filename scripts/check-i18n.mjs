#!/usr/bin/env node
/**
 * 多语言自检：
 * 1. zh-CN / en-US 语言包文件与 key 结构必须一致；
 * 2. 代码里静态写出的 t('key') 必须能在对应命名空间找到；
 * 3. 列出仍然含中文的源码行（注释除外）。
 *
 * 用法：node scripts/check-i18n.mjs [--strict]
 * --strict 时，遗留中文也会让命令失败。
 */
import { readdirSync, readFileSync } from 'node:fs';
import { join, relative, sep } from 'node:path';

const ROOT = process.cwd();
const LOCALES = join(ROOT, 'src', 'i18n', 'locales');
const strict = process.argv.includes('--strict');

function walk(dir, filter, out = []) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) {
      walk(path, filter, out);
    } else if (filter(path)) {
      out.push(path);
    }
  }
  return out;
}

function flatten(value, prefix = '', out = []) {
  for (const [key, child] of Object.entries(value)) {
    const path = prefix ? `${prefix}.${key}` : key;
    if (child && typeof child === 'object' && !Array.isArray(child)) {
      flatten(child, path, out);
    } else {
      out.push(path);
    }
  }
  return out;
}

const errors = [];
const warnings = [];

// 1. 语言包结构
const files = readdirSync(join(LOCALES, 'zh-CN')).filter((name) => name.endsWith('.json'));
const enFiles = readdirSync(join(LOCALES, 'en-US')).filter((name) => name.endsWith('.json'));
for (const file of files) {
  if (!enFiles.includes(file)) {
    errors.push(`en-US 缺少语言包：${file}`);
  }
}
for (const file of enFiles) {
  if (!files.includes(file)) {
    errors.push(`zh-CN 缺少语言包：${file}`);
  }
}

const namespaces = new Map();
for (const file of files.filter((name) => enFiles.includes(name))) {
  const ns = file.replace(/\.json$/, '');
  const zh = JSON.parse(readFileSync(join(LOCALES, 'zh-CN', file), 'utf8'));
  const en = JSON.parse(readFileSync(join(LOCALES, 'en-US', file), 'utf8'));
  const zhKeys = new Set(flatten(zh));
  const enKeys = new Set(flatten(en));
  for (const key of zhKeys) {
    if (!enKeys.has(key)) {
      errors.push(`en-US/${file} 缺少 key：${key}`);
    }
  }
  for (const key of enKeys) {
    if (!zhKeys.has(key)) {
      errors.push(`zh-CN/${file} 缺少 key：${key}`);
    }
  }
  namespaces.set(ns, zhKeys);
}

// 2. 静态 t() key
const sourceFiles = walk(join(ROOT, 'src'), (path) => /\.(ts|tsx)$/.test(path)).filter(
  (path) => !path.includes(`${sep}i18n${sep}`)
);

for (const file of sourceFiles) {
  const source = readFileSync(file, 'utf8');
  const rel = relative(ROOT, file);
  const hook = source.match(/useTranslation\(\s*\[?\s*['"]([\w-]+)['"]/);
  const nsHint = source.match(/i18n-namespace:\s*([\w-]+)/);
  const defaultNs = nsHint?.[1] ?? hook?.[1] ?? 'common';
  const pattern = /\bt\(\s*['"]([\w.:-]+)['"]/g;
  for (const match of source.matchAll(pattern)) {
    const raw = match[1];
    if (!raw || raw.includes('${')) {
      continue;
    }
    const separator = raw.indexOf(':');
    const ns = separator >= 0 ? raw.slice(0, separator) : defaultNs;
    const key = separator >= 0 ? raw.slice(separator + 1) : raw;
    const keys = namespaces.get(ns);
    if (!keys) {
      errors.push(`${rel}: 引用了不存在的命名空间 "${ns}"（${raw}）`);
      continue;
    }
    if (!keys.has(key)) {
      // i18next 的复数 key 以 _one / _other / _zero / _two / _few / _many 结尾，
      // 代码里引用的是不带后缀的基础 key。
      const plural = ['_zero', '_one', '_two', '_few', '_many', '_other'].some((suffix) => keys.has(`${key}${suffix}`));
      if (!plural) {
        errors.push(`${rel}: ${ns} 缺少 key "${key}"`);
      }
    }
  }
}

// 3. 遗留中文
const chinese = /[\u4e00-\u9fff]/;
const hardcoded = [];
for (const file of sourceFiles) {
  const lines = readFileSync(file, 'utf8').split(/\r?\n/);
  let blockComment = false;
  lines.forEach((line, index) => {
    const trimmed = line.trim();
    if (blockComment) {
      if (trimmed.includes('*/')) {
        blockComment = false;
      }
      return;
    }
    if (trimmed.startsWith('/*')) {
      blockComment = !trimmed.includes('*/');
      return;
    }
    if (trimmed.startsWith('//') || trimmed.startsWith('*')) {
      return;
    }
    if (!chinese.test(line)) {
      return;
    }
    // 去掉行尾注释后再判断
    const code = line.replace(/\/\/.*$/, '');
    if (chinese.test(code)) {
      hardcoded.push(`${relative(ROOT, file)}:${index + 1}: ${trimmed}`);
    }
  });
}

if (hardcoded.length > 0) {
  warnings.push(`仍有 ${hardcoded.length} 行源码含中文（可能是注释、稳定错误码或未迁移文案）：`);
  warnings.push(...hardcoded.slice(0, 200));
  if (hardcoded.length > 200) {
    warnings.push(`……其余 ${hardcoded.length - 200} 行省略`);
  }
}

for (const warning of warnings) {
  console.warn(`[i18n] ${warning}`);
}
for (const error of errors) {
  console.error(`[i18n] ${error}`);
}

if (errors.length > 0 || (strict && hardcoded.length > 0)) {
  console.error(`\n[i18n] 检查失败：${errors.length} 个错误，${hardcoded.length} 行遗留中文。`);
  process.exit(1);
}

console.log(`[i18n] 语言包结构一致，静态 key 检查通过（${namespaces.size} 个命名空间）。`);
