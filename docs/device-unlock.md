# 设备槽解锁

档案密码仍然是换机和找回用的主密钥。启用设备槽之后，这台电脑已经登录、并且没有处于锁定时，打开应用会直接进入，不再输入档案密码，也不弹出系统验证。

点「锁定」会清掉内存里的 DEK，并在 `db/user.lock` 留下标记。这个标记在重启后仍然存在，所以再次打开会停在解锁页。点「解锁」后才弹出系统验证：Windows Hello、macOS Touch ID、Android 生物识别。验证成功才打开档案。取消或失败时可以改输入档案密码。

拷走应用数据目录里的文件无法解开数据库。解包用的设备密钥留在系统密钥库里，不可导出，并且绑定这一台机器、这一个系统用户。已经登录的同一系统用户下的其他程序，可以像本应用一样静默使用这把密钥。这是不弹窗的边界。

## 威胁模型

打得开数据库的只有两样东西：

- 档案密码（密码槽）
- 本机已登录用户能使用的设备密钥（设备槽，不额外弹窗）

下面这些材料单独或一起被拷走，都解不开 `data.sqlite`：

- `data.sqlite`
- `key.header.json`
- `device.wrap`

`device.wrap` 只是密文。没有系统密钥库里的那把设备密钥，密文没有用。

残留风险：已经登录同一系统用户、并且能调用同一密钥库的本机程序，可以不经过 Hello、Touch ID 或生物识别就使用设备密钥。`user.lock` 只让本应用在锁定后要求系统验证，删掉这个文件会回到静默打开。拷走文件仍然解不开。不用未公开的 Passport `NgcCacheType`。

锁屏之后、进程退出之后，内存里的 DEK 会清掉。没有 `user.lock` 时，下次打开用设备槽静默进入。有 `user.lock` 时保持锁定。设备槽还在，不需要重新启用。

## 钥匙

数据库页密钥仍是建库时生成的随机 DEK（32 字节）。DEK 明文只出现在解锁后的进程内存里。磁盘上有两个槽：

| 槽 | 用什么包 DEK | 放在哪 | 何时使用 |
| --- | --- | --- | --- |
| 密码槽 | Argon2id 派生的 KEK，算法 XChaCha20-Poly1305 | `key.header.json` | 建库、换机、设备槽失效、用户选择输入密码 |
| 设备槽 | 系统密钥库中的设备密钥 | 密文在 `device.wrap`，密钥在系统里 | 用户在这台设备上启用之后的日常打开 |

改档案密码只重包密码槽，DEK 和 `device.wrap` 都不变。关闭设备槽只删除 `device.wrap` 和系统里的设备密钥，不重加密数据库。

启用设备槽时，库必须已经用密码解开，并且要再输入一次档案密码。用这次密码解开头文件，确认和内存里的 DEK 一致，再用这把 DEK 去登记设备槽。未解锁的会话不能打开免密。

锁定丢掉内存里的会话，并写入 `user.lock`。`device.wrap` 保留。下次启动时只要这个标记还在，就停在解锁页，不会静默进入。解锁成功后删除标记。

## 文件

`device.wrap` 与 `data.sqlite`、`key.header.json` 放在同一目录（应用本地数据目录下的 `db/`）。写入方式与密钥头相同：先写 `device.wrap.tmp`，再改名。

```json
{
  "v": 1,
  "kind": "windows-cng",
  "ct": "<base64>"
}
```

`kind` 取值：

- `windows-cng`：`ct` 是 TPM 上 RSA-2048 用 OAEP-SHA256 加密 DEK 的密文
- `macos-keychain`：`ct` 是 24 字节 nonce，后面接着 XChaCha20-Poly1305 密文。包装密钥是钥匙串里的 32 字节随机 KEK
- `android-keystore`：`ct` 由 Android 侧解释为大端 4 字节 IV 长度、IV、AES-GCM 密文（含 tag）
- `memory`：只用于测试的假槽，格式与 macOS 的 nonce 拼接相同

文件里没有密码，也没有 DEK 明文。`kind` 与当前平台不一致时，设备解锁失败，密码槽仍然可用。

## 平台

接口在 `src-tauri/src/crypto/device/`：`enroll` 返回要写入 `device.wrap` 的字节，`open` 读回 DEK，`forget` 删除系统里的设备密钥。

### Windows

- 密钥提供程序：`Microsoft Platform Crypto Provider`
- 密钥名：`precession.device.v1`
- 算法：不可导出的 RSA-2048，只允许解密，导出策略为 0
- 没有 TPM 时，不能启用设备槽
- 没有 `user.lock` 时直接解密，不弹出 Windows Hello
- 有 `user.lock` 时，解密前调用 `IUserConsentVerifierInterop` 弹出 Windows Hello。用户取消则不打开数据库，标记保留

### macOS

- 钥匙串通用密码，服务名 `precession`，账户 `device-kek-v2`
- 保护级别 `kSecAttrAccessibleWhenUnlockedThisDeviceOnly`。钥匙串读取本身不要求 Touch ID
- 有 `user.lock` 时，读取前用 LocalAuthentication 要求 Touch ID。没有 Touch ID 时这次解锁失败，可以改输入档案密码
- 写入数据保护钥匙串，并标记为不同步 iCloud
- 未签名的开发包可能被系统拒绝创建这条钥匙串项。这时设备槽不可用，密码解锁不受影响
- 旧的 `device-kek-v1`（每次都要用户在场）不再读取。需要重新启用一次设备槽

### Android

实现放在应用内插件 `src-tauri/device-slot`，不改 `src-tauri/gen/android`（那份工程会被 `tauri android init` 覆盖）。Kotlin 使用 Android Keystore。DEK 只在应用进程里从 Rust 传到 Kotlin，不进入 WebView，能力文件里也不开放这三条插件命令。

- API 28 以下不能启用设备槽
- 密钥别名 `precession.device.v2`，AES-256-GCM
- `setUnlockedDeviceRequired(true)`。密钥本身不要求每次生物识别
- 有 `user.lock` 时，解开前弹出生物识别。验证在回调里结束，避免堵住界面线程
- 设备锁屏后密钥不可用；手机已解锁且没有 `user.lock` 时，应用可以静默打开
- 旧别名 `precession.device.v1` 不再使用。需要重新启用一次设备槽

### Linux

没有设备槽。只有档案密码。

## 界面与命令

`db_status` 增加 `deviceUnlock` 和 `userLocked`。前者只表示 `device.wrap` 是否存在，后者表示 `user.lock` 是否存在。查询状态时不弹出系统验证。

| 命令 | 作用 |
| --- | --- |
| `db_enable_device_unlock` | 已解锁，并且密码正确。登记设备槽并写入 `device.wrap` |
| `db_unlock_device` | 用设备槽解开 DEK。有 `user.lock` 时先做系统验证 |
| `db_disable_device_unlock` | 删除设备密钥和 `device.wrap`。锁定或未锁定都可以调用 |
| `db_lock` | 用户点锁定：清内存会话，并写入 `user.lock`。退出应用只清内存，不写这个标记 |

启动时如果 `deviceUnlock` 为真且 `userLocked` 为假，自动调用 `db_unlock_device`，成功则进入已解锁页面，不弹系统验证。失败则留下档案密码表单。

`userLocked` 为真时不自动解锁。点「解锁」后弹出 Windows Hello、Touch ID 或生物识别。取消或失败时可以改输入档案密码，也可以再试一次系统验证。设备槽失效时可以移除它。

已解锁页面用档案密码确认来启用设备槽，也可以关闭它。

面向界面的错误：

- 这台设备不支持设备解锁
- 已取消设备验证
- 设备解锁已失效

其他失败仍显示「操作失败」，细节只写日志。

## 测试

自动化测试使用内存假槽，不碰 TPM、钥匙串或 Android Keystore：

- enroll / open 往返
- 再次 enroll 后，旧密文打不开
- 另一只假槽打不开这份密文
- forget 之后 open 失败
- 损坏的 `device.wrap` 失败
- 密码错误不能启用；未解锁不能启用；启用、锁定、设备解锁、关闭的完整路径

手动确认：

1. Windows：有 TPM 的机器上启用设备槽。退出应用再打开，应直接进入，不弹 Hello。点锁定后退出再打开，应停在解锁页；点解锁后弹出 Hello，通过后进入。取消 Hello 后可以输入档案密码。把 `db/` 目录拷到另一台电脑，不能打开数据库。
2. macOS：启用后退出再打开，不出现 Touch ID。点锁定后退出再打开，点解锁应出现 Touch ID。钥匙串项不应出现在 iCloud 钥匙串同步里。账户名是 `device-kek-v2`。
3. Android（API 28+）：手机已解锁时，启用后再次打开应直接进入。点锁定后再次打开应停在解锁页，点解锁弹出生物识别。API 27 及以下只能输入密码。
