import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');

const usage = `用法: pnpm version:set <版本>

  pnpm version:set 0.1.1      应用版本 0.1.1，商店清单 0.1.1.0
  pnpm version:set 0.1.1.2    应用版本仍是 0.1.1，商店清单第四位是 2

三段版本写入 package.json、tauri.conf.json、Cargo.toml 和 Cargo.lock。
四段版本写入 Package.appxmanifest。省略第四位时按 0。
同一应用版本重新提交商店时，第四位必须高于上次。`;

function fail(message) {
  console.error(message);
  process.exit(1);
}

function parseVersion(input) {
  const raw = input.startsWith('v') ? input.slice(1) : input;
  const parts = raw.split('.');
  if (parts.length !== 3 && parts.length !== 4) {
    fail('版本写成 0.1.1 或 0.1.1.2');
  }
  for (const part of parts) {
    if (!/^(0|[1-9]\d*)$/.test(part)) {
      fail('每一段必须是不含前导零的非负整数');
    }
    if (Number(part) > 65535) {
      fail('每一段不能超过 65535，MSIX 不接受');
    }
  }
  const semver = `${parts[0]}.${parts[1]}.${parts[2]}`;
  const msix = `${semver}.${parts[3] ?? '0'}`;
  return { msix, semver };
}

function replaceOnce(text, pattern, replacement, label) {
  const flags = pattern.flags.includes('g') ? pattern.flags : `${pattern.flags}g`;
  const found = text.match(new RegExp(pattern.source, flags));
  if (found?.length !== 1) {
    fail(`${label} 里应恰好有一处版本，实际是 ${found?.length ?? 0} 处`);
  }
  return text.replace(pattern, replacement);
}

function replaceCargoPackageVersion(text, version) {
  const start = text.search(/^\[package\]/m);
  if (start < 0) {
    fail('src-tauri/Cargo.toml 缺少 [package]');
  }
  const after = text.slice(start + 1).search(/^\[/m);
  const end = after < 0 ? text.length : start + 1 + after;
  const section = text.slice(start, end);
  const next = replaceOnce(
    section,
    /^version = "[^"]+"/m,
    `version = "${version}"`,
    'src-tauri/Cargo.toml 的 [package]'
  );
  return text.slice(0, start) + next + text.slice(end);
}

function read(relativePath) {
  return readFileSync(join(root, relativePath), 'utf8');
}

function writeIfChanged(relativePath, next) {
  const path = join(root, relativePath);
  const current = readFileSync(path, 'utf8');
  if (current === next) {
    return false;
  }
  writeFileSync(path, next);
  return true;
}

const arg = process.argv[2];
if (!arg || process.argv.length > 3) {
  fail(usage);
}

const { msix, semver } = parseVersion(arg);

const targets = [
  {
    path: 'package.json',
    next: replaceOnce(read('package.json'), /^(\s*"version": ")[^"]+(")/m, `$1${semver}$2`, 'package.json')
  },
  {
    path: 'src-tauri/tauri.conf.json',
    next: replaceOnce(
      read('src-tauri/tauri.conf.json'),
      /^(\s*"version": ")[^"]+(")/m,
      `$1${semver}$2`,
      'src-tauri/tauri.conf.json'
    )
  },
  {
    path: 'src-tauri/Cargo.toml',
    next: replaceCargoPackageVersion(read('src-tauri/Cargo.toml'), semver)
  },
  {
    path: 'src-tauri/Cargo.lock',
    next: replaceOnce(
      read('src-tauri/Cargo.lock'),
      /(name = "precession"\r?\nversion = ")[^"]+(")/,
      `$1${semver}$2`,
      'src-tauri/Cargo.lock'
    )
  },
  {
    path: 'Package.appxmanifest',
    next: replaceOnce(
      read('Package.appxmanifest'),
      /(<Identity\b[^>]*\bVersion=")[^"]+(")/,
      `$1${msix}$2`,
      'Package.appxmanifest'
    )
  }
];

const changed = [];
for (const target of targets) {
  if (writeIfChanged(target.path, target.next)) {
    changed.push(target.path);
  }
}

if (changed.length === 0) {
  console.log(`已经是 ${semver}，商店清单 ${msix}`);
} else {
  console.log(`应用版本 ${semver}`);
  console.log(`商店清单 ${msix}`);
  for (const path of changed) {
    console.log(path);
  }
}
