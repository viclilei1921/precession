/// macOS 平台的启动挂钩
///
/// 编译到 macOS 时被选为 `Platform` 的实现，并挂上 shell 和对话框插件。
///
/// # 示例
/// 见 [`Platform`](super::Platform)
pub(crate) struct Macos;

#[cfg(target_os = "macos")]
mod imp {
  use tauri::{App, Manager, Webview, Window};

  impl super::super::Platform for super::Macos {
    /// 挂上 shell 和对话框插件
    ///
    /// # 参数
    /// - `builder`: 待挂插件的 Tauri 构建器
    ///
    /// # 返回值
    /// 挂上两个插件后的构建器
    ///
    /// # Panics
    /// 不会主动 panic
    ///
    /// # 示例
    /// ```ignore
    /// let builder = Macos::plugins(builder);
    /// ```
    fn plugins(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
      // 初始化 shell 插件(为了侧车功能)。
      builder.plugin(tauri_plugin_shell::init()).plugin(tauri_plugin_dialog::init())
    }

    /// 登记侧车进程槽和托盘，并调整主窗口
    ///
    /// 托盘注册失败时只记错误日志。
    ///
    /// # 参数
    /// - `app`: 正在启动的应用
    ///
    /// # 返回值
    /// 不返回数据
    ///
    /// # Panics
    /// 不会主动 panic
    ///
    /// # 示例
    /// ```ignore
    /// Macos::setup(app);
    /// ```
    fn setup(app: &mut App) {
      // 管理侧车进程，并注册托盘菜单。
      app.manage(crate::sidecar::ProcessSlot::default());
      if let Err(e) = crate::tray::register_tray_menu(app) {
        tauri_plugin_log::log::error!("tray menu registration failed: {e}");
      }
      tune_main_window(app);
    }

    /// 页面加载完成后再次关闭弹性回弹
    ///
    /// 只处理 `Finished`。关闭失败时记错误日志。
    ///
    /// # 参数
    /// - `webview`: 发生加载的 WebView
    /// - `payload`: 加载事件内容
    ///
    /// # 返回值
    /// 不返回数据
    ///
    /// # Panics
    /// 不会主动 panic
    ///
    /// # 示例
    /// ```ignore
    /// Macos::on_page_load(webview, payload);
    /// ```
    fn on_page_load(webview: &Webview, payload: &tauri::webview::PageLoadPayload<'_>) {
      // 页面加载完再关一次。启动时滚动视图可能还没建好，只关那一次盖不住回弹。
      if payload.event() != tauri::webview::PageLoadEvent::Finished {
        return;
      }
      if let Err(error) = webview.with_webview(|inner| unsafe {
        let view: &objc2_web_kit::WKWebView = &*inner.inner().cast();
        disable_elastic_overscroll(view);
      }) {
        tauri_plugin_log::log::error!("disable webview elastic overscroll failed: {error}");
      }
    }

    /// 拦截关闭，并在缩放后再次关闭回弹
    ///
    /// 关闭请求改为隐藏窗口。`Resized` 时再关一次主 WebView 的回弹。
    ///
    /// # 参数
    /// - `window`: 发出事件的窗口
    /// - `event`: 窗口事件
    ///
    /// # 返回值
    /// 不返回数据
    ///
    /// # Panics
    /// 不会主动 panic
    ///
    /// # 示例
    /// ```ignore
    /// Macos::on_window_event(window, event);
    /// ```
    fn on_window_event(window: &Window, event: &tauri::WindowEvent) {
      // 关闭窗口时隐藏到托盘。
      if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        let _ = window.hide();
      }
      // 缩放时系统会把回弹开关复位，再关掉。红绿灯在标题栏 layout 末尾对齐，这里再改会和系统位置来回抢。
      if let tauri::WindowEvent::Resized(_) = event {
        disable_main_elastic_overscroll(window);
      }
    }
  }

  /// 调整主窗口的回弹和红绿灯位置
  ///
  /// 找不到名为 `main` 的窗口时只记错误日志。
  ///
  /// # 参数
  /// - `app`: 正在启动的应用
  ///
  /// # 返回值
  /// 不返回数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// tune_main_window(app);
  /// ```
  fn tune_main_window(app: &App) {
    match app.get_webview_window("main") {
      Some(window) => {
        if let Err(error) = window.with_webview(|webview| unsafe {
          let view: &objc2_web_kit::WKWebView = &*webview.inner().cast();
          disable_elastic_overscroll(view);
          let ns_window: &objc2_app_kit::NSWindow = &*webview.ns_window().cast();
          align_traffic_lights(ns_window);
        }) {
          tauri_plugin_log::log::error!("disable webview elastic overscroll failed: {error}");
        }
      }
      None => {
        tauri_plugin_log::log::error!("main window missing, elastic overscroll left enabled");
      }
    }
  }

  /// 标题栏高度，和前端 `.title-bar` 的 `2.25rem` 一致（根字号 16px 时为 36px）。红绿灯中线和标题中线对齐。
  const TITLEBAR_HEIGHT: f64 = 36.0;

  static TRAFFIC_MAIN_WINDOW: std::sync::atomic::AtomicPtr<std::ffi::c_void> =
    std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());

  static TRAFFIC_LAYOUT_IMP: std::sync::atomic::AtomicPtr<std::ffi::c_void> =
    std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());

  /// 安装红绿灯布局挂钩并立刻摆放一次
  ///
  /// # 参数
  /// - `window`: 主窗口的 `NSWindow`
  ///
  /// # 返回值
  /// 不返回数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// align_traffic_lights(window);
  /// ```
  fn align_traffic_lights(window: &objc2_app_kit::NSWindow) {
    install_traffic_light_hook(window);
    position_traffic_lights(window);
  }

  /// 把红绿灯修正挂到主题框的 `layout` 末尾
  ///
  /// 只执行一次。找不到主题框、类不能挂钩或替换失败时记警告并返回。
  ///
  /// # 参数
  /// - `window`: 主窗口的 `NSWindow`
  ///
  /// # 返回值
  /// 不返回数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// install_traffic_light_hook(window);
  /// ```
  fn install_traffic_light_hook(window: &objc2_app_kit::NSWindow) {
    use std::sync::Once;

    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
      let Some(host) = theme_frame_host(window) else {
        tauri_plugin_log::log::warn!("traffic light hook skipped, theme frame missing");
        return;
      };
      let Some(host_window) = host.window() else {
        return;
      };
      let ptr = objc2::rc::Retained::as_ptr(&host_window).cast::<std::ffi::c_void>().cast_mut();
      TRAFFIC_MAIN_WINDOW.store(ptr, std::sync::atomic::Ordering::Release);

      let class = host.class();
      if !hook_able_theme_class(class) {
        tauri_plugin_log::log::warn!("traffic light hook skipped, class {}", class.name().to_string_lossy());
        return;
      }
      let imp = unsafe {
        std::mem::transmute::<
          unsafe extern "C-unwind" fn(*mut objc2::runtime::AnyObject, objc2::runtime::Sel),
          objc2::runtime::Imp,
        >(traffic_light_layout)
      };
      if !hook_theme_method(class, objc2::sel!(layout), imp, &TRAFFIC_LAYOUT_IMP) {
        tauri_plugin_log::log::warn!("traffic light hook failed, class {}", class.name().to_string_lossy());
      }
    });
  }

  /// 沿父视图向上查找可挂钩的主题框
  ///
  /// 没有内容视图，或最终宿主类不能挂钩时返回 `None`。
  ///
  /// # 参数
  /// - `window`: 主窗口的 `NSWindow`
  ///
  /// # 返回值
  /// 主题框宿主视图；找不到时为 `None`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let host = theme_frame_host(window);
  /// ```
  fn theme_frame_host(window: &objc2_app_kit::NSWindow) -> Option<objc2::rc::Retained<objc2_app_kit::NSView>> {
    let content = window.contentView()?;
    let mut host = unsafe { content.superview() }?;
    while let Some(parent) = unsafe { host.superview() } {
      if !hook_able_theme_class(parent.class()) {
        break;
      }
      host = parent;
    }
    if !hook_able_theme_class(host.class()) {
      return None;
    }
    Some(host)
  }

  /// 判断类名是否可以作为主题框挂钩目标
  ///
  /// `NSView`、`NSResponder`、`NSObject` 返回 false。
  ///
  /// # 参数
  /// - `class`: Objective-C 类
  ///
  /// # 返回值
  /// 类名不属于这三个基类时为 true
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let ok = hook_able_theme_class(class);
  /// ```
  fn hook_able_theme_class(class: &objc2::runtime::AnyClass) -> bool {
    let name = class.name();
    name != c"NSView" && name != c"NSResponder" && name != c"NSObject"
  }

  /// 替换主题框实例方法，并保存原来的实现
  ///
  /// 方法不存在、类型编码为空，或继承方法添加失败时返回 false，并在添加失败时清掉已保存的原实现。
  ///
  /// # 参数
  /// - `class`: 要修改的类
  /// - `sel`: 被替换的选择子
  /// - `new_imp`: 新的方法实现
  /// - `slot`: 存放原实现指针的位置
  ///
  /// # 返回值
  /// 替换成功时为 true
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let ok = hook_theme_method(class, sel, new_imp, slot);
  /// ```
  fn hook_theme_method(
    class: &objc2::runtime::AnyClass,
    sel: objc2::runtime::Sel,
    new_imp: objc2::runtime::Imp,
    slot: &std::sync::atomic::AtomicPtr<std::ffi::c_void>,
  ) -> bool {
    use std::sync::atomic::Ordering;

    use objc2::ffi::{
      class_addMethod, class_getInstanceMethod, method_getImplementation, method_getTypeEncoding,
      method_setImplementation,
    };

    let class_ptr = std::ptr::from_ref(class);
    let method = unsafe { class_getInstanceMethod(class_ptr, sel) };
    if method.is_null() {
      return false;
    }

    let inherited = class.superclass().is_some_and(|superclass| {
      let super_method = unsafe { class_getInstanceMethod(std::ptr::from_ref(superclass), sel) };
      super_method == method
    });

    if inherited {
      let Some(original) = (unsafe { method_getImplementation(method) }) else {
        return false;
      };
      let encoding = unsafe { method_getTypeEncoding(method) };
      if encoding.is_null() {
        return false;
      }
      slot.store(unsafe { std::mem::transmute(original) }, Ordering::Release);
      let added = unsafe { class_addMethod(class_ptr.cast_mut(), sel, new_imp, encoding) };
      if !added.as_bool() {
        slot.store(std::ptr::null_mut(), Ordering::Release);
        return false;
      }
      return true;
    }

    let Some(original) = (unsafe { method_setImplementation(method, new_imp) }) else {
      return false;
    };
    slot.store(unsafe { std::mem::transmute(original) }, Ordering::Release);
    true
  }

  /// 先调用原来的 `layout`，再把主窗口红绿灯摆回中线
  ///
  /// 正在对齐、指针为空，或窗口不是记下的主窗口时直接返回。
  ///
  /// # 参数
  /// - `this`: 主题框对象
  /// - `cmd`: 本次调用的选择子
  ///
  /// # 返回值
  /// 不返回数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// traffic_light_layout(this, cmd);
  /// ```
  unsafe extern "C-unwind" fn traffic_light_layout(this: *mut objc2::runtime::AnyObject, cmd: objc2::runtime::Sel) {
    if TRAFFIC_ALIGNING.with(|flag| flag.get()) {
      return;
    }
    let original = TRAFFIC_LAYOUT_IMP.load(std::sync::atomic::Ordering::Acquire);
    if !original.is_null() {
      let original: unsafe extern "C-unwind" fn(*mut objc2::runtime::AnyObject, objc2::runtime::Sel) =
        unsafe { std::mem::transmute(original) };
      unsafe { original(this, cmd) };
    }
    if this.is_null() || TRAFFIC_ALIGNING.with(|flag| flag.replace(true)) {
      return;
    }
    let _guard = ClearTrafficAligning;
    let view: &objc2_app_kit::NSView = unsafe { &*this.cast() };
    let Some(window) = view.window() else {
      return;
    };
    let current = objc2::rc::Retained::as_ptr(&window).cast::<std::ffi::c_void>();
    if current == TRAFFIC_MAIN_WINDOW.load(std::sync::atomic::Ordering::Acquire).cast_const() {
      position_traffic_lights(&window);
    }
  }

  thread_local! {
    static TRAFFIC_ALIGNING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
  }

  /// 红绿灯对齐的重入标记
  ///
  /// 离开作用域时把 `TRAFFIC_ALIGNING` 清回 false。
  ///
  /// # 示例
  /// ```ignore
  /// let _guard = ClearTrafficAligning;
  /// ```
  struct ClearTrafficAligning;

  impl Drop for ClearTrafficAligning {
    /// 把红绿灯对齐标记清回 false
    ///
    /// # 返回值
    /// 不返回数据
    ///
    /// # Panics
    /// 不会主动 panic
    ///
    /// # 示例
    /// ```ignore
    /// drop(guard);
    /// ```
    fn drop(&mut self) {
      TRAFFIC_ALIGNING.with(|flag| flag.set(false));
    }
  }

  /// 计算标题栏中线并摆放红绿灯
  ///
  /// 全屏、缺少内容视图或按钮时直接返回。能找到 `CATransaction` 时先关掉隐式动画再改位置。
  ///
  /// # 参数
  /// - `window`: 主窗口的 `NSWindow`
  ///
  /// # 返回值
  /// 不返回数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// position_traffic_lights(window);
  /// ```
  fn position_traffic_lights(window: &objc2_app_kit::NSWindow) {
    use objc2::runtime::{AnyClass, Bool};
    use objc2_app_kit::{NSWindowButton, NSWindowStyleMask};
    use objc2_foundation::NSPoint;

    if window.styleMask().contains(NSWindowStyleMask::FullScreen) {
      return;
    }
    let Some(content) = window.contentView() else {
      return;
    };
    let Some(close) = window.standardWindowButton(NSWindowButton::CloseButton) else {
      return;
    };
    let Some(button_super) = (unsafe { close.superview() }) else {
      return;
    };

    let button_height = close.frame().size.height;
    let center_y = content.bounds().size.height - TITLEBAR_HEIGHT / 2.0;
    let center = content.convertPoint_toView(NSPoint::new(0.0, center_y), Some(&*button_super));
    let origin_y = center.y - button_height / 2.0;

    let Some(transaction) = AnyClass::get(c"CATransaction") else {
      place_traffic_lights(window, origin_y);
      return;
    };
    unsafe {
      let _: () = objc2::msg_send![transaction, begin];
      let _: () = objc2::msg_send![transaction, setDisableActions: Bool::YES];
    }
    place_traffic_lights(window, origin_y);
    unsafe {
      let _: () = objc2::msg_send![transaction, commit];
    }
  }

  /// 把关闭、最小化、全屏按钮的纵坐标设为给定值
  ///
  /// 按钮不存在，或当前纵坐标与目标相差小于 0.5 时跳过。
  ///
  /// # 参数
  /// - `window`: 主窗口的 `NSWindow`
  /// - `origin_y`: 按钮原点的目标纵坐标
  ///
  /// # 返回值
  /// 不返回数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// place_traffic_lights(window, origin_y);
  /// ```
  fn place_traffic_lights(window: &objc2_app_kit::NSWindow, origin_y: f64) {
    use objc2_app_kit::NSWindowButton;

    for kind in [NSWindowButton::CloseButton, NSWindowButton::MiniaturizeButton, NSWindowButton::ZoomButton] {
      let Some(button) = window.standardWindowButton(kind) else {
        continue;
      };
      let mut origin = button.frame().origin;
      if (origin.y - origin_y).abs() < 0.5 {
        continue;
      }
      origin.y = origin_y;
      button.setFrameOrigin(origin);
    }
  }

  /// 关闭该窗口 WebView 的弹性回弹
  ///
  /// 按窗口标签取 WebView；取不到时直接返回。
  ///
  /// # 参数
  /// - `window`: 发出缩放事件的窗口
  ///
  /// # 返回值
  /// 不返回数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// disable_main_elastic_overscroll(window);
  /// ```
  fn disable_main_elastic_overscroll(window: &Window) {
    let Some(webview) = window.get_webview_window(window.label()) else {
      return;
    };
    let _ = webview.with_webview(|webview| unsafe {
      let view: &objc2_web_kit::WKWebView = &*webview.inner().cast();
      disable_elastic_overscroll(view);
    });
  }

  /// 关闭 WKWebView 的橡皮筋回弹和始终回弹
  ///
  /// 选择子不存在时跳过；`_setRubberBandingEnabled:` 缺失时只警告一次。最后再关视图树上的滚动弹性。
  ///
  /// # 参数
  /// - `view`: 页面所在的 `WKWebView`
  ///
  /// # 返回值
  /// 不返回数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// disable_elastic_overscroll(view);
  /// ```
  fn disable_elastic_overscroll(view: &objc2_web_kit::WKWebView) {
    use std::sync::Once;

    use objc2::msg_send;
    use objc2::runtime::{Bool, NSObjectProtocol};
    use objc2::sel;

    let rubber_banding = sel!(_setRubberBandingEnabled:);
    if view.respondsToSelector(rubber_banding) {
      // `_WKRectEdge` 的 0 表示四边都不回弹。
      let _: () = unsafe { msg_send![view, _setRubberBandingEnabled: 0usize] };
    } else {
      static WARNED: Once = Once::new();
      WARNED.call_once(|| {
        tauri_plugin_log::log::warn!("WKWebView _setRubberBandingEnabled: unavailable");
      });
    }

    let bounce_y = sel!(_setAlwaysBounceVertical:);
    if view.respondsToSelector(bounce_y) {
      let _: () = unsafe { msg_send![view, _setAlwaysBounceVertical: Bool::NO] };
    }
    let bounce_x = sel!(_setAlwaysBounceHorizontal:);
    if view.respondsToSelector(bounce_x) {
      let _: () = unsafe { msg_send![view, _setAlwaysBounceHorizontal: Bool::NO] };
    }

    disable_scroll_view_elasticity(view);
  }

  /// 递归关闭视图树上 `NSScrollView` 的弹性
  ///
  /// 当前视图是滚动视图时把横向和纵向弹性都设为 `None`，再对每个子视图做同样的事。
  ///
  /// # 参数
  /// - `view`: 起始 `NSView`
  ///
  /// # 返回值
  /// 不返回数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// disable_scroll_view_elasticity(view);
  /// ```
  fn disable_scroll_view_elasticity(view: &objc2_app_kit::NSView) {
    use objc2_app_kit::{NSScrollElasticity, NSScrollView};

    if let Some(scroll) = view.downcast_ref::<NSScrollView>() {
      scroll.setHorizontalScrollElasticity(NSScrollElasticity::None);
      scroll.setVerticalScrollElasticity(NSScrollElasticity::None);
    }

    let subviews = view.subviews();
    for index in 0..subviews.count() {
      let child = subviews.objectAtIndex(index);
      disable_scroll_view_elasticity(&child);
    }
  }
}

#[cfg(not(target_os = "macos"))]
impl super::Platform for Macos {}
