package com.kkrainbow.easytier

import android.content.Intent
import android.os.Build
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.webkit.JavascriptInterface
import android.webkit.WebView

class MainActivity : TauriActivity() {
    private val mainHandler = Handler(Looper.getMainLooper())

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        initService()
    }

    override fun onWebViewCreate(webView: WebView) {
        super.onWebViewCreate(webView)
        webView.isLongClickable = true
        webView.addJavascriptInterface(ThemeBridge(), "_easytier_theme")
        // Disable pinch-to-zoom (two-finger gesture) on Android WebView
        webView.settings.setSupportZoom(false)
        webView.settings.builtInZoomControls = false
    }

    private inner class ThemeBridge {
        @JavascriptInterface
        @Suppress("DEPRECATION")
        fun setStatusBarStyle(dark: Boolean) {
            mainHandler.post {
                val window = window ?: return@post
                if (dark) {
                    window.statusBarColor = 0xFF121212.toInt()
                    window.navigationBarColor = 0xFF121212.toInt()
                    window.decorView.systemUiVisibility = 0
                } else {
                    window.statusBarColor = 0xFFF5F5F5.toInt()
                    window.navigationBarColor = 0xFFF5F5F5.toInt()
                    window.decorView.systemUiVisibility = android.view.View.SYSTEM_UI_FLAG_LIGHT_STATUS_BAR
                }
            }
        }
    }

    private fun initService() {
        val serviceIntent = Intent(this, MainForegroundService::class.java)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            startForegroundService(serviceIntent)
        } else {
            startService(serviceIntent)
        }
    }
}
