//! 使用平台原生 API 获取 GPU 信息，替代 wgpu 以减小依赖体积

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GpuInfo {
  pub name: String,
  pub backend: String,     // 如 Vulkan, Metal, DirectX 12
  pub device_type: String, // 如 "DiscreteGpu" 或 "IntegratedGpu" 等
}

static GPU_INFO: OnceLock<Vec<GpuInfo>> = OnceLock::new();

fn init_gpu_info() -> Vec<GpuInfo> {
  let gpu_list = init_gpu_info_impl();

  if gpu_list.is_empty() {
    tauri_plugin_log::log::error!("未检测到可用的图形适配器");
  }

  gpu_list
}

#[cfg(windows)]
fn init_gpu_info_impl() -> Vec<GpuInfo> {
  crate::utils::gpu::windows::enumerate_gpus()
}

#[cfg(target_os = "macos")]
fn init_gpu_info_impl() -> Vec<GpuInfo> {
  crate::utils::gpu::macos::enumerate_gpus()
}

#[cfg(target_os = "linux")]
fn init_gpu_info_impl() -> Vec<GpuInfo> {
  crate::utils::gpu::linux::enumerate_gpus()
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
fn init_gpu_info_impl() -> Vec<GpuInfo> {
  Vec::new()
}

pub async fn get_gpu_info() -> Vec<GpuInfo> {
  GPU_INFO.get_or_init(init_gpu_info).clone()
}

// ---------------------------------------------------------------------------
// Windows: DXGI API
// ---------------------------------------------------------------------------
#[cfg(windows)]
pub(crate) mod windows {
  use super::GpuInfo;
  use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, DXGI_ADAPTER_DESC1, IDXGIFactory1};

  const DXGI_ADAPTER_FLAG_SOFTWARE: u32 = 1;

  fn decode_utf16(utf16: &[u16]) -> String {
    String::from_utf16_lossy(utf16).trim_matches('\0').trim().to_string()
  }

  fn infer_device_type(desc: &DXGI_ADAPTER_DESC1, name: &str) -> &'static str {
    // 软件适配器（如 WARP、远程桌面）
    if (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE) != 0 {
      return "Cpu";
    }
    let name_lower = name.to_lowercase();
    // Microsoft 基本显示 / 远程
    if name_lower.contains("microsoft") || name_lower.contains("basic display") {
      return "VirtualGpu";
    }
    // 独显通常有较大专用显存，集显通常较小
    // 64MB 作为粗略阈值：集显多为 0~128MB 共享，独显一般 >= 256MB
    const DISCRETE_THRESHOLD: usize = 64 * 1024 * 1024; // 64MB
    if desc.DedicatedVideoMemory >= DISCRETE_THRESHOLD { "DiscreteGpu" } else { "IntegratedGpu" }
  }

  pub fn enumerate_gpus() -> Vec<GpuInfo> {
    let mut list = Vec::new();

    unsafe {
      let factory: IDXGIFactory1 = match CreateDXGIFactory1() {
        Ok(f) => f,
        Err(e) => {
          tauri_plugin_log::log::error!("CreateDXGIFactory1 失败: {}", e);
          return list;
        }
      };

      let mut i = 0u32;
      loop {
        let adapter = match factory.EnumAdapters1(i) {
          Ok(a) => a,
          Err(_) => break, // DXGI_ERROR_NOT_FOUND 或其它错误时结束枚举
        };
        match adapter.GetDesc1() {
          Ok(desc) => {
            let name = decode_utf16(&desc.Description);
            if !name.is_empty() {
              let device_type = infer_device_type(&desc, &name).to_string();
              list.push(GpuInfo { name, backend: "DirectX 12".to_string(), device_type });
            }
          }
          Err(e) => tauri_plugin_log::log::warn!("GetDesc1 失败 (adapter {}): {}", i, e),
        }
        i += 1;
      }
    }

    list
  }
}

// ---------------------------------------------------------------------------
// macOS: system_profiler（无额外依赖）
// ---------------------------------------------------------------------------
#[cfg(target_os = "macos")]
pub(crate) mod macos {
  use super::GpuInfo;
  use std::process::Command;

  pub fn enumerate_gpus() -> Vec<GpuInfo> {
    let output = match Command::new("system_profiler").args(["SPDisplaysDataType", "-json"]).output() {
      Ok(o) if o.status.success() => o,
      Ok(o) => {
        tauri_plugin_log::log::warn!("system_profiler 失败: {:?}", o.stderr);
        return Vec::new();
      }
      Err(e) => {
        tauri_plugin_log::log::error!("执行 system_profiler 失败: {}", e);
        return Vec::new();
      }
    };

    let json: serde_json::Value = match serde_json::from_slice(&output.stdout) {
      Ok(v) => v,
      Err(e) => {
        tauri_plugin_log::log::error!("解析 system_profiler 输出失败: {}", e);
        return Vec::new();
      }
    };

    let mut list = Vec::new();
    let empty = Vec::new();
    let displays = json.get("SPDisplaysDataType").and_then(|d| d.as_array()).unwrap_or(&empty);

    for disp in displays {
      let chip = disp
        .get("sppci_chipset_model")
        .or_else(|| disp.get("sppci_model"))
        .or_else(|| disp.get("_name"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown GPU")
        .to_string();

      // Apple Silicon 统一显存，视为 IntegratedGpu；外接 eGPU 少见
      let device_type = if chip.to_lowercase().contains("apple") {
        "IntegratedGpu"
      } else if chip.to_lowercase().contains("intel") {
        "IntegratedGpu"
      } else {
        "DiscreteGpu"
      };

      list.push(GpuInfo { name: chip, backend: "Metal".to_string(), device_type: device_type.to_string() });
    }

    list
  }
}

// ---------------------------------------------------------------------------
// Linux: lspci（无额外依赖）
// ---------------------------------------------------------------------------
#[cfg(target_os = "linux")]
pub(crate) mod linux {
  use super::GpuInfo;
  use std::process::Command;

  fn infer_device_type(vendor: &str) -> &'static str {
    let v = vendor.to_lowercase();
    if v.contains("nvidia") || v.contains("amd") || v.contains("ati") {
      "DiscreteGpu"
    } else if v.contains("intel") {
      if v.contains("arc") || v.contains("a-series") { "DiscreteGpu" } else { "IntegratedGpu" }
    } else {
      "Other"
    }
  }

  pub fn enumerate_gpus() -> Vec<GpuInfo> {
    let output = match Command::new("lspci").args(["-vmm", "-D"]).output() {
      Ok(o) if o.status.success() => o,
      Ok(_) => return Vec::new(),
      Err(e) => {
        tauri_plugin_log::log::error!("执行 lspci 失败: {}", e);
        return Vec::new();
      }
    };

    let text = match String::from_utf8(output.stdout) {
      Ok(t) => t,
      Err(_) => return Vec::new(),
    };

    let mut list = Vec::new();
    let mut class = String::new();
    let mut vendor = String::new();
    let mut device = String::new();

    for line in text.lines() {
      if line.is_empty() {
        if (class.contains("VGA") || class.contains("3D")) && !vendor.is_empty() && !device.is_empty() {
          let name = format!("{} {}", vendor, device).trim().to_string();
          let device_type = infer_device_type(&vendor).to_string();
          list.push(GpuInfo { name, backend: "Vulkan".to_string(), device_type });
        }
        class.clear();
        vendor.clear();
        device.clear();
        continue;
      }
      if let Some((k, v)) = line.split_once(':') {
        let k = k.trim();
        let v = v.trim();
        match k {
          "Class" => class = v.to_string(),
          "Vendor" => vendor = v.to_string(),
          "Device" => device = v.to_string(),
          _ => {}
        }
      }
    }

    if (class.contains("VGA") || class.contains("3D")) && !vendor.is_empty() && !device.is_empty() {
      let name = format!("{} {}", vendor, device).trim().to_string();
      let device_type = infer_device_type(&vendor).to_string();
      list.push(GpuInfo { name, backend: "Vulkan".to_string(), device_type });
    }

    list
  }
}
