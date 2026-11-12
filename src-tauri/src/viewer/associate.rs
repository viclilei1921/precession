use super::error::ViewerError;
use super::folder::EXTENSIONS;

/// 把当前程序登记成这些图片的打开方式，并打开系统的关联对话框
///
/// Windows 上先写入当前用户的程序关联，再请系统让用户勾选要交给本程序的类型。对话框打不开时改为打开默认应用设置。其他系统返回 `ViewerError::WindowsOnly`。
///
/// # 返回值
/// 登记完成，或已经打开系统设置
///
/// # 错误
/// 写注册表失败返回 `ViewerError::Unreadable`。系统设置也打不开时返回 `ViewerError::Settings`
///
/// # Panics
/// 不会主动 panic
pub fn register_default() -> Result<(), ViewerError> {
  #[cfg(windows)]
  {
    windows::register()
  }
  #[cfg(not(windows))]
  {
    Err(ViewerError::WindowsOnly)
  }
}

#[cfg(windows)]
mod windows {
  use std::path::PathBuf;

  use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx};
  use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_WRITE, REG_OPTION_NON_VOLATILE, REG_SZ, RegCloseKey, RegCreateKeyExW, RegSetValueExW,
  };
  use windows::Win32::UI::Shell::{
    ApplicationAssociationRegistrationUI, IApplicationAssociationRegistrationUI, SHCNE_ASSOCCHANGED, SHCNF_IDLIST,
    SHChangeNotify,
  };
  use windows::core::PCWSTR;

  use super::EXTENSIONS;
  use super::ViewerError;

  const APP: &str = "Precession";
  const PROG_ID: &str = "Precession.Image";

  /// 写入当前用户的关联并打开系统对话框
  pub fn register() -> Result<(), ViewerError> {
    let exe = std::env::current_exe().map_err(|_| ViewerError::Unreadable)?;
    write_keys(&exe)?;
    unsafe {
      SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None);
    }
    if launch_dialog().is_err() {
      open_settings()?;
    }
    Ok(())
  }

  /// 打开 Windows 的默认应用设置
  fn open_settings() -> Result<(), ViewerError> {
    std::process::Command::new("cmd")
      .args(["/C", "start", "", "ms-settings:defaultapps"])
      .spawn()
      .map_err(|_| ViewerError::Settings)?;
    Ok(())
  }

  /// 在单独的线程上打开「设置程序关联」
  fn launch_dialog() -> Result<(), ViewerError> {
    std::thread::spawn(|| unsafe {
      let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
      let ui: IApplicationAssociationRegistrationUI =
        CoCreateInstance(&ApplicationAssociationRegistrationUI, None, CLSCTX_INPROC_SERVER)
          .map_err(|_| ViewerError::Settings)?;
      let name = wide(APP);
      ui.LaunchAdvancedAssociationUI(PCWSTR(name.as_ptr())).map_err(|_| ViewerError::Settings)
    })
    .join()
    .map_err(|_| ViewerError::Settings)?
  }

  /// 写入 ProgID、能力声明和「打开方式」
  fn write_keys(exe: &PathBuf) -> Result<(), ViewerError> {
    let command = format!("\"{}\" \"%1\"", exe.display());
    let icon = format!("\"{}\",0", exe.display());
    set_default(&format!("Software\\Classes\\{PROG_ID}"), "Precession Image")?;
    set_default(&format!("Software\\Classes\\{PROG_ID}\\DefaultIcon"), &icon)?;
    set_default(&format!("Software\\Classes\\{PROG_ID}\\shell\\open\\command"), &command)?;
    let capabilities = format!("Software\\{APP}\\Capabilities");
    set_named(&capabilities, "ApplicationName", APP)?;
    set_named(&capabilities, "ApplicationDescription", APP)?;
    set_named(&capabilities, "ApplicationIcon", &icon)?;
    for ext in EXTENSIONS {
      set_named(&format!("{capabilities}\\FileAssociations"), &format!(".{ext}"), PROG_ID)?;
      set_named(&format!("Software\\Classes\\.{ext}\\OpenWithProgids"), PROG_ID, "")?;
    }
    set_named("Software\\RegisteredApplications", APP, &capabilities)?;
    Ok(())
  }

  /// 写入一个键的默认值
  fn set_default(subkey: &str, value: &str) -> Result<(), ViewerError> {
    let key = create_key(subkey)?;
    let result = set_sz(key, None, value);
    close(key);
    result
  }

  /// 写入一个具名值
  fn set_named(subkey: &str, name: &str, value: &str) -> Result<(), ViewerError> {
    let key = create_key(subkey)?;
    let result = set_sz(key, Some(name), value);
    close(key);
    result
  }

  /// 打开或建出键
  fn create_key(subkey: &str) -> Result<HKEY, ViewerError> {
    let name = wide(subkey);
    let mut key = HKEY::default();
    unsafe {
      RegCreateKeyExW(
        HKEY_CURRENT_USER,
        PCWSTR(name.as_ptr()),
        Some(0),
        PCWSTR::null(),
        REG_OPTION_NON_VOLATILE,
        KEY_WRITE,
        None,
        &mut key,
        None,
      )
      .ok()
      .map_err(|_| ViewerError::Unreadable)?;
    }
    Ok(key)
  }

  /// 写入 REG_SZ。`name` 为空时写默认值
  fn set_sz(key: HKEY, name: Option<&str>, value: &str) -> Result<(), ViewerError> {
    let stored;
    let value_name = match name {
      Some(name) => {
        stored = wide(name);
        PCWSTR(stored.as_ptr())
      }
      None => PCWSTR::null(),
    };
    let data = wide(value);
    let bytes = unsafe { std::slice::from_raw_parts(data.as_ptr().cast::<u8>(), data.len() * 2) };
    unsafe {
      RegSetValueExW(key, value_name, Some(0), REG_SZ, Some(bytes)).ok().map_err(|_| ViewerError::Unreadable)?;
    }
    Ok(())
  }

  /// 关掉注册表键
  fn close(key: HKEY) {
    unsafe {
      let _ = RegCloseKey(key);
    }
  }

  /// UTF-16，带结尾的 0
  fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
  }
}
