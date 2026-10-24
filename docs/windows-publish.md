# Windows 商店发布

用 Microsoft Store 的 MSIX 安装，避免「Windows 已保护你的电脑」。个人开发者账号不会给 NSIS 安装包签名。`pnpm build:win` 打出的安装包仍然会报警。

商店身份写在仓库根目录的 [Package.appxmanifest](../Package.appxmanifest)。这份清单可以提交。`devcert.pfx` 和 `precession.msix` 不会提交。

## 1. 合作伙伴中心身份

在 [合作伙伴中心](https://partner.microsoft.com/dashboard/apps/overview) 用个人开发者账号登录，新产品选 **MSIX 或 PWA 应用**，名称填 `precession`。保留成功后，打开 **产品管理 → 产品标识**，把下面三项原样抄进清单。`CN=` 要保留。

| 页面标签                                | 清单字段               | 当前值                                    |
| --------------------------------------- | ---------------------- | ----------------------------------------- |
| Package/Identity/Name                   | `Identity Name`        | `viclilei.precession`                     |
| Package/Identity/Publisher              | `Identity Publisher`   | `CN=A342B656-55E4-4555-9649-7D1585494CF3` |
| Package/Properties/PublisherDisplayName | `PublisherDisplayName` | `viclilei`                                |

`com.viclilei.precession` 只是 Tauri 内部标识，商店不认。

清单里其余固定值：

| 字段        | 值                                                               |
| ----------- | ---------------------------------------------------------------- |
| DisplayName | `precession`                                                     |
| Version     | 四段式。当前是 `0.1.0.1`。每次覆盖安装都必须更高，例如 `0.1.0.2` |
| Executable  | `precession.exe`                                                 |
| EntryPoint  | `Windows.FullTrustApplication`                                   |
| Language    | `zh-CN`                                                          |
| 能力        | `runFullTrust`                                                   |
| 架构        | `x64`                                                            |

图标用 `src-tauri/icons/` 里现成的 `StoreLogo.png`、`Square44x44Logo.png`、`Square150x150Logo.png`。不要单独写 `Square310x310Logo`：指定它时还必须有一张 310×150 的宽磁贴，项目里没有这张图。

## 2. 编译

```powershell
pnpm tauri build --no-bundle
```

这个版本的 Tauri 不接受 `--bundles none`。`--bundles` 只有 `msi` 和 `nsis`。只要可执行文件时用 `--no-bundle`。

编译完成后，程序旁边的旁路文件名已经去掉架构后缀。打包必须用这一组，不要用 `src-tauri/binaries/` 里带 `-x86_64-pc-windows-msvc` 的源文件。运行时在 `precession.exe` 同目录查找短文件名，找不到就报「系统找不到指定的文件」。

| 源文件                               | 打进 MSIX 的文件                       |
| ------------------------------------ | -------------------------------------- |
| `ffmpeg-x86_64-pc-windows-msvc.exe`  | `src-tauri/target/release/ffmpeg.exe`  |
| `cjxl-x86_64-pc-windows-msvc.exe`    | `src-tauri/target/release/cjxl.exe`    |
| `avifenc-x86_64-pc-windows-msvc.exe` | `src-tauri/target/release/avifenc.exe` |

## 3. 组装并签名

本机已有 Windows SDK 10.0.26100，直接用其中的 `makeappx` 和 `signtool`。不要用 `winapp pack`：它会再从 NuGet 下载一套 `Microsoft.Windows.SDK.BuildTools`，体积大约几百 MB。

先把清单版本改高，再在项目根目录执行：

```powershell
$layout = "src-tauri\target\msix-layout"
New-Item -ItemType Directory -Force "$layout\src-tauri\icons" | Out-Null
Copy-Item src-tauri\target\release\precession.exe,src-tauri\target\release\ffmpeg.exe,src-tauri\target\release\cjxl.exe,src-tauri\target\release\avifenc.exe $layout -Force
Copy-Item src-tauri\icons\StoreLogo.png,src-tauri\icons\Square44x44Logo.png,src-tauri\icons\Square150x150Logo.png "$layout\src-tauri\icons\" -Force
Copy-Item Package.appxmanifest "$layout\AppxManifest.xml" -Force
& "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\makeappx.exe" pack /d $layout /p precession.msix /o
& "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\signtool.exe" sign /fd SHA256 /f devcert.pfx /p password precession.msix
```

包内的清单必须叫 `AppxManifest.xml`。放成 `Package.appxmanifest` 时，MakeAppx 会报缺少清单。

还没有 `devcert.pfx` 时，先装 [winapp CLI](https://learn.microsoft.com/windows/apps/dev-tools/winapp-cli/)，再生成证书。证书发布者必须和清单里的 `Publisher` 一致，所以要在改完清单之后生成：

```powershell
winget install Microsoft.winappCli
winapp cert generate --password "你的密码" --if-exists overwrite
```

`--password` 不写时，密码是 `password`。已经生成过 `devcert.pfx` 时，`--if-exists skip` 会留下旧文件和旧密码，要改密码必须加 `--if-exists overwrite`。改完后，上面的 `signtool` 把 `/p password` 换成同一个密码，安装证书时也要带上：

```powershell
winapp cert install .\devcert.pfx --password "你的密码"
```

## 4. 本机安装

证书要装进本机受信任存储，必须用管理员 PowerShell。

```powershell
cd D:\precession
winapp cert install .\devcert.pfx --password "你的密码"
```

用户账户控制选「是」。证书只需装一次。然后双击项目根目录的 `precession.msix`。

卸掉这张信任证书时，用管理员 PowerShell。这只取消「用这张证书签名的包可以安装」，不会卸掉已经装上的 precession。

```powershell
Get-ChildItem Cert:\LocalMachine\TrustedPeople, Cert:\CurrentUser\TrustedPeople, Cert:\CurrentUser\My |
  Where-Object { $_.Subject -eq 'CN=A342B656-55E4-4555-9649-7D1585494CF3' } |
  Remove-Item
```

应用本身在「设置 → 应用 → 已安装的应用」里卸载 precession。

推送 `v*` tag 或手动运行 `.github/workflows/build.yml` 时，Windows 任务在 NSIS 之后用同一次编译结果打 `precession.msix`，并放进 Artifact。tag 构建还会把它附到 Release 草稿。版本沿用 `Package.appxmanifest`，覆盖安装前先把清单版本改高。

稳定签名配置两个仓库 Secrets，不要把 pfx 提交进仓库：

| Secret                         | 内容                    |
| ------------------------------ | ----------------------- |
| `WINDOWS_CERTIFICATE_BASE64`   | `devcert.pfx` 的 Base64 |
| `WINDOWS_CERTIFICATE_PASSWORD` | 该 pfx 的密码           |

本机生成 Base64：

```powershell
[Convert]::ToBase64String([IO.File]::ReadAllBytes("devcert.pfx")) | Set-Clipboard
```

两个 Secret 缺一不可。未配置时 Windows 任务在打 MSIX 这一步失败，不会改用临时证书。

## 5. 提交商店

回到合作伙伴中心该产品，点「开始提交」。只面向 Windows 电脑，不要勾选 Xbox。

| 页           | 填写                                                                                                         |
| ------------ | ------------------------------------------------------------------------------------------------------------ |
| 定价和可用性 | 免费。市场先选中国。只想自己安装时，可见性选专用受众，填入这个微软账号的邮箱                                 |
| 属性         | 类别选生产力。子类别、网站、支持联系信息留空                                                                 |
| 年龄分级     | 全部选「无」：无暴力、无性内容、无粗话、无毒品、无赌博、无应用内购买、无用户之间交流、不把个人信息发给第三方 |
| 包           | 上传 `precession.msix`。设备系列只留桌面。架构 x64。最低系统 Windows 10                                      |
| 商店列表     | 语言选简体中文。说明和功能见下文。截图至少 1 张 PNG，宽高不小于 1366×768。`StoreLogo.png` 不能当作截图       |

隐私政策这一项要填一个公网能打开的 https 地址，不能把正文直接贴进网址栏。桌面应用即使数据只留在本机，这一项也必填。页面正文可以用：

> precession 是一款在你的电脑上运行的本地应用。计划、手记、书库、成长记录和加密档案都保存在本机，应用不会把这些内容上传到开发者的服务器，也没有账号系统。应用不收集广告标识，不向第三方出售或共享你的记录。视频和图片处理调用随应用附带的本地程序，文件不会离开这台电脑。

商店列表说明：

> precession 把一天里的安排、手记、阅读和回顾放在同一处。计划按年、月、周查看；手记、成长和书库记在本机；图片转码、视频处理和文件加密也在应用内完成。数据留在你的电脑上。

产品功能每行一条，不要自己加项目符号：

- 按年、月、周查看计划
- 手记、成长与阅读记录留在本机
- 图片转码与视频处理
- 文件加密

第一版的「此版本的新增内容」留空。搜索关键字填 `计划,手记,日记,书库,日历`。版权信息填 `© 2026 viclilei`。开发者与产品标识页的 PublisherDisplayName 相同，当前是 `viclilei`。

名称保留大约 3 个月，过期未提交就会被释放。认证通常要几个工作日。通过后用同一个微软账号在微软商店安装，安装时不会再出现 SmartScreen 拦截。
