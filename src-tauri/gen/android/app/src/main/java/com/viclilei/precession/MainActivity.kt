package com.viclilei.precession

import android.annotation.SuppressLint
import android.os.Build
import android.os.Bundle
import android.webkit.WebView
import android.window.OnBackInvokedCallback
import android.window.OnBackInvokedDispatcher
import androidx.activity.OnBackPressedCallback
import androidx.activity.enableEdgeToEdge

class MainActivity : TauriActivity() {
  private var backCallback: OnBackPressedCallback? = null
  private var backToken: Any? = null
  private var pending = false
  private var backGeneration = 0

  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
  }

  override fun onWebViewCreate(webView: WebView) {
    super.onWebViewCreate(webView)
    backCallback?.remove()
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
      PredictiveBack.unregister(this, backToken)
    }

    val callback = object : OnBackPressedCallback(true) {
      override fun handleOnBackPressed() {
        dispatchBack(webView)
      }
    }
    backCallback = callback
    onBackPressedDispatcher.addCallback(this, callback)

    // WebView 自己也注册了返回回调，优先级相同，后注册的会吃掉侧滑，hash 路由又不会动。
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
      backToken = PredictiveBack.register(this) { dispatchBack(webView) }
    }
  }

  override fun onDestroy() {
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
      PredictiveBack.unregister(this, backToken)
      backToken = null
    }
    backCallback?.remove()
    backCallback = null
    super.onDestroy()
  }

  private fun dispatchBack(webView: WebView) {
    if (pending || isFinishing) {
      return
    }
    pending = true
    val generation = ++backGeneration
    // 等这次系统返回结束再跑脚本，避免 history.back 被当成手势内部调用而丢掉。
    webView.post {
      if (isDestroyed || generation != backGeneration) {
        if (generation == backGeneration) {
          pending = false
        }
        return@post
      }
      webView.evaluateJavascript(BACK_SCRIPT) { raw ->
        if (generation != backGeneration) {
          return@evaluateJavascript
        }
        pending = false
        val action = raw?.trim()?.removeSurrounding("\"") ?: "exit"
        if (action == "exit") {
          runOnUiThread {
            if (!isFinishing && !moveTaskToBack(true)) {
              finish()
            }
          }
        }
      }
      webView.postDelayed({
        if (generation == backGeneration) {
          pending = false
        }
      }, 600)
    }
  }

  private companion object {
    const val BACK_SCRIPT =
      "(function(){try{return window.__precessionBack?window.__precessionBack():'exit';}catch(e){return 'exit';}})()"
  }
}

@SuppressLint("NewApi")
private object PredictiveBack {
  fun register(activity: MainActivity, onBack: () -> Unit): OnBackInvokedCallback {
    val callback = OnBackInvokedCallback { onBack() }
    activity.onBackInvokedDispatcher.registerOnBackInvokedCallback(
      OnBackInvokedDispatcher.PRIORITY_OVERLAY,
      callback
    )
    return callback
  }

  fun unregister(activity: MainActivity, token: Any?) {
    if (token is OnBackInvokedCallback) {
      activity.onBackInvokedDispatcher.unregisterOnBackInvokedCallback(token)
    }
  }
}
