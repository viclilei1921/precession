//! Windows 精准触摸板（PTP）的 Raw Input 接触监听。
//!
//! WebView 的滚轮事件流里没有「手指是否还在板上」这个信号，翻页结算只能靠
//! 静默计时猜。这里用 Win32 Raw Input 订阅触摸板 HID（Usage Page 0x0D /
//! Usage 0x05），读报告里的 Contact Count（Usage 0x54），手指数变化时 emit
//! `ptp:fingers` 事件给前端。前端据此把「手指停在板上」和「轻抬无惯性」
//! 区分开：前者永不结算，后者 0ms 确定性结算。
//!
//! 设计约束：
//! - 观察者模式：注册 RIDEV_INPUTSINK 只观察不消费，原生滚轮/捏合事件流
//!   原样走 WebView2，互不影响。
//! - 读 Contact Count 一个值（0x54）就得到手指数，不逐点解析 Tip Switch，
//!   不碰 X/Y 坐标，把 HID 解析复杂度降到最低。
//! - 事件只在手指数变化时发（平均 < 10Hz），不冲爆 IPC。
//! - 老触摸板（PS/2 兼容模式）收不到 HID raw input：注册成功不等于有事件，
//!   前端「从未收到过 ptp:fingers」就整体回退静默结算，零感知。
//! - wndproc 用 SetWindowSubclass 子类化，处理后透传给原窗口过程，
//!   不动窗口消息链。

use std::sync::OnceLock;
use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};

use tauri::{Emitter, Manager};
use windows::Win32::Devices::HumanInterfaceDevice::{
  HIDP_CAPS, HidP_GetCaps, HidP_GetUsageValue, HidP_Input, PHIDP_PREPARSED_DATA,
};
use windows::Win32::Foundation::{HANDLE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::{
  GetRawInputData, GetRawInputDeviceInfoW, GetRawInputDeviceList, HRAWINPUT, RAWINPUTDEVICE, RAWINPUTDEVICELIST,
  RAWINPUTHEADER, RID_DEVICE_INFO, RID_INPUT, RIDEV_INPUTSINK, RIDI_DEVICEINFO, RIDI_PREPARSEDDATA,
  RegisterRawInputDevices,
};
use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::WM_INPUT;

/// 子类过程 ID（同一窗口上区分不同子类的标记）
const SUBCLASS_ID: usize = 0x5054_5050;
/// HID Usage Page: Digitizers（0x0D）
const DIGITIZER_PAGE: u16 = 0x0D;
/// HID Usage: Touch Pad（0x05）
const TOUCH_PAD_USAGE: u16 = 0x05;
/// HID Usage: Contact Count（0x54），报告里有效接触点的数量
const CONTACT_COUNT_USAGE: u16 = 0x54;
/// RIM_TYPEHID：HID 设备的 raw input 类型（crate 里是 RID_DEVICE_INFO_TYPE 包装）
const RIM_TYPEHID_RAW: u32 = 2;

/// 上次上报的手指数。-1 表示还没有过任何接触（此时静默降级由前端兜底）。
static LAST_FINGERS: AtomicI32 = AtomicI32::new(-1);
/// 已发出的诊断日志条数（WM_INPUT 到达 / 解析失败 各自限 3 条，防止刷爆日志）
static SEEN_INPUT: AtomicU32 = AtomicU32::new(0);
static SEEN_PARSE_FAIL: AtomicU32 = AtomicU32::new(0);
/// 子类化过的窗口，退出时移除。
static SUBCLASSED: OnceLock<isize> = OnceLock::new();
/// 应用句柄，用于 emit 事件
static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();

/// 在主窗口上挂 Raw Input 触摸板监听。
///
/// 失败只记日志不 panic：监听是增强，不是必需，前端会走静默结算兜底。
pub(crate) fn attach(app: &tauri::AppHandle) {
  let Some(window) = app.get_webview_window("main") else {
    tauri_plugin_log::log::warn!("ptp: main window not found, finger signal disabled");
    return;
  };
  let Ok(hwnd) = window.hwnd() else {
    tauri_plugin_log::log::warn!("ptp: main window has no hwnd, finger signal disabled");
    return;
  };

  let _ = APP_HANDLE.set(app.clone());
  log_touchpad_collection();

  // 只观察不消费（INPUTSINK），原生输入事件照常分发
  let devices = [RAWINPUTDEVICE {
    usUsagePage: DIGITIZER_PAGE,
    usUsage: TOUCH_PAD_USAGE,
    dwFlags: RIDEV_INPUTSINK,
    hwndTarget: hwnd,
  }];
  let registered = unsafe { RegisterRawInputDevices(&devices, std::mem::size_of::<RAWINPUTDEVICE>() as u32) };
  if registered.is_err() {
    tauri_plugin_log::log::warn!("ptp: RegisterRawInputDevices failed, finger signal disabled");
    return;
  }

  let subclassed = unsafe { SetWindowSubclass(hwnd, Some(subclass_proc), SUBCLASS_ID, 0usize) };
  if subclassed.as_bool() {
    let _ = SUBCLASSED.set(hwnd.0 as isize);
    tauri_plugin_log::log::info!("ptp: raw input finger listener attached");
  } else {
    tauri_plugin_log::log::warn!("ptp: SetWindowSubclass failed, finger signal disabled");
  }
}

/// 枚举 Raw Input HID 设备，报告哪些带 Digitizer/Touch Pad 采集集合。
///
/// 用来回答「注册成功但收不到事件」到底是设备形态问题（PS/2 兼容触摸板没有
/// HID 采集集合）还是链路问题。诊断信息，失败不阻塞 attach。
fn log_touchpad_collection() {
  let mut count: u32 = 0;
  if unsafe { GetRawInputDeviceList(None, &mut count, std::mem::size_of::<RAWINPUTDEVICELIST>() as u32) } != 0 {
    tauri_plugin_log::log::warn!("ptp: GetRawInputDeviceList count query failed");
    return;
  }
  if count == 0 {
    tauri_plugin_log::log::warn!("ptp: no raw input devices at all");
    return;
  }
  let mut list = vec![RAWINPUTDEVICELIST::default(); count as usize];
  let got = unsafe {
    GetRawInputDeviceList(
      Some(list.as_mut_ptr()),
      &mut count,
      std::mem::size_of::<RAWINPUTDEVICELIST>() as u32,
    )
  };
  if got == u32::MAX {
    tauri_plugin_log::log::warn!("ptp: GetRawInputDeviceList failed");
    return;
  }
  let mut digitizer_count = 0;
  for device in list.iter().take(got as usize) {
    if device.dwType.0 != RIM_TYPEHID_RAW {
      continue;
    }
    // HID 硬件形态（厂商/产品 ID）：诊断哪个设备是触摸板
    let mut info = RID_DEVICE_INFO::default();
    let mut size = std::mem::size_of::<RID_DEVICE_INFO>() as u32;
    let ok = unsafe {
      GetRawInputDeviceInfoW(
        Some(device.hDevice),
        RIDI_DEVICEINFO,
        Some(&mut info as *mut _ as *mut core::ffi::c_void),
        &mut size,
      )
    };
    if ok == 0 || ok == u32::MAX || info.dwType.0 != RIM_TYPEHID_RAW {
      continue;
    }
    let hid = unsafe { info.Anonymous.hid };
    if hid.usUsagePage != DIGITIZER_PAGE || hid.usUsage != TOUCH_PAD_USAGE {
      continue;
    }
    digitizer_count += 1;
    tauri_plugin_log::log::info!(
      "ptp: touchpad hid collection found: vid={:04x} pid={:04x} usage_page={:02x} usage={:02x}",
      hid.dwVendorId,
      hid.dwProductId,
      hid.usUsagePage,
      hid.usUsage
    );
  }
  if digitizer_count == 0 {
    tauri_plugin_log::log::warn!(
      "ptp: no Touch Pad hid collection — touchpad likely in PS/2 compatibility mode, finger signal will stay silent"
    );
  }
}

/// 退出时移除子类，避免窗口销毁后子类过程被调用。
pub(crate) fn detach() {
  if let Some(handle) = SUBCLASSED.get() {
    let hwnd = HWND(*handle as *mut core::ffi::c_void);
    unsafe {
      let _ = RemoveWindowSubclass(hwnd, Some(subclass_proc), SUBCLASS_ID);
    }
  }
}

/// 子类窗口过程：处理 WM_INPUT，其余透传给原窗口过程。
unsafe extern "system" fn subclass_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
  _subclass_id: usize,
  _ref: usize,
) -> LRESULT {
  if msg == WM_INPUT {
    unsafe { on_raw_input(lparam) };
  }
  unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

/// 处理一条 WM_INPUT：解析 HID 报告里的 Contact Count 并上报。
unsafe fn on_raw_input(lparam: LPARAM) {
  let handle = HRAWINPUT(lparam.0 as *mut core::ffi::c_void);

  // 先问缓冲区要多大
  let mut size: u32 = 0;
  let header_size = std::mem::size_of::<RAWINPUTHEADER>() as u32;
  if unsafe { GetRawInputData(handle, RID_INPUT, None, &mut size, header_size) } == u32::MAX {
    return;
  }
  if size == 0 || size > 4096 {
    return;
  }

  let mut buffer = vec![0u8; size as usize];
  let copied = unsafe {
    GetRawInputData(
      handle,
      RID_INPUT,
      Some(buffer.as_mut_ptr() as *mut core::ffi::c_void),
      &mut size,
      header_size,
    )
  };
  if copied == u32::MAX || copied == 0 {
    return;
  }
  let copied = copied as usize;
  if copied < std::mem::size_of::<RAWINPUTHEADER>() + 8 {
    return;
  }

  // RAWINPUT 布局：header + 按设备类型变化的 body。HID 设备的 body 是
  // RAWHID { dwSizeHid: u32, dwCount: u32, bRawData[...] }。不引入完整
  // RAWINPUT 包装类型，手工解构，减少一层 unsafe 转换。
  let raw = &buffer[..copied];
  let header = unsafe { &*(raw.as_ptr() as *const RAWINPUTHEADER) };
  // 只关心自己注册的触摸板设备（hDevice 传给 HID 解析要用）
  if header.dwType != RIM_TYPEHID_RAW {
    return;
  }
  let device = header.hDevice;
  let body_offset = std::mem::size_of::<RAWINPUTHEADER>();
  let dw_size_hid =
    u32::from_le_bytes([raw[body_offset], raw[body_offset + 1], raw[body_offset + 2], raw[body_offset + 3]]) as usize;
  if dw_size_hid == 0 || dw_size_hid > 256 {
    return;
  }
  let data_offset = body_offset + 8;
  if raw.len() < data_offset + dw_size_hid {
    return;
  }
  let report = &raw[data_offset..data_offset + dw_size_hid];

  // 诊断探针：WM_INPUT 链路到达的前 3 条留痕（证明信号进来了）
  if SEEN_INPUT.fetch_add(1, Ordering::Relaxed) < 3 {
    tauri_plugin_log::log::info!("ptp: wm_input arrived, hid report {} bytes", dw_size_hid);
  }

  let fingers = unsafe { read_contact_count(device, report) };
  match fingers {
    Ok(count) => {
      let previous = LAST_FINGERS.swap(count, Ordering::Relaxed);
      if previous != count {
        tauri_plugin_log::log::info!("ptp: fingers {} -> {}", previous, count);
        if let Some(app) = APP_HANDLE.get() {
          let _ = app.emit("ptp:fingers", count);
        }
      }
    }
    Err(reason) => {
      // 解析失败的前 3 条留痕（含原因），之后静默忽略
      if SEEN_PARSE_FAIL.fetch_add(1, Ordering::Relaxed) < 3 {
        tauri_plugin_log::log::warn!("ptp: contact count parse failed: {}", reason);
      }
    }
  }
}

/// 读 HID 输入报告里的 Contact Count（Usage 0x54）。
///
/// PTP 规范规定报告顶层就是 Contact Count 值，一个 HidP_GetUsageValue 直接
/// 取到当前有效接触点数，不需要逐点解析 Tip Switch 位。返回 Err 带
/// 可读原因，供诊断日志用。
unsafe fn read_contact_count(device: HANDLE, report: &[u8]) -> Result<i32, String> {
  // 拿设备 pre-parsed data：先问大小再取内容
  let mut preparsed_size: u32 = 0;
  let queried = unsafe { GetRawInputDeviceInfoW(Some(device), RIDI_PREPARSEDDATA, None, &mut preparsed_size) };
  if queried == 0 || queried == u32::MAX || preparsed_size == 0 || preparsed_size > 8192 {
    return Err(format!("preparsed size query returned {queried} / {preparsed_size}"));
  }
  let mut preparsed = vec![0u8; preparsed_size as usize];
  let filled = unsafe {
    GetRawInputDeviceInfoW(
      Some(device),
      RIDI_PREPARSEDDATA,
      Some(preparsed.as_mut_ptr() as *mut core::ffi::c_void),
      &mut preparsed_size,
    )
  };
  if filled == u32::MAX || filled == 0 {
    return Err(format!("preparsed fetch returned {filled}"));
  }

  let data = PHIDP_PREPARSED_DATA(preparsed.as_ptr() as isize);
  let mut caps = HIDP_CAPS::default();
  let caps_status = unsafe { HidP_GetCaps(data, &mut caps) };
  if caps_status.is_err() {
    return Err(format!("HidP_GetCaps failed: {}", caps_status.0));
  }

  // Contact Count 是顶层值能力：LinkCollection 传 None（0 = 根集合）
  let mut value: u32 = 0;
  let status =
    unsafe { HidP_GetUsageValue(HidP_Input, DIGITIZER_PAGE, None, CONTACT_COUNT_USAGE, &mut value, data, report) };
  if status.is_err() {
    return Err(format!(
      "HidP_GetUsageValue(ContactCount) failed: {} (input value caps = {})",
      status.0, caps.NumberInputValueCaps
    ));
  }
  Ok(value as i32)
}
