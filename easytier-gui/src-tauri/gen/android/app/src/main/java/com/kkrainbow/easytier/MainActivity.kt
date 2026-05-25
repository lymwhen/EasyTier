package com.kkrainbow.easytier

import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.webkit.JavascriptInterface
import android.webkit.WebView
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

class MainActivity : TauriActivity() {
    private val mainHandler = Handler(Looper.getMainLooper())

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        initService()
    }

    override fun onWebViewCreate(webView: WebView) {
        super.onWebViewCreate(webView)
        webView.isLongClickable = true
        webView.addJavascriptInterface(PasteBridge(webView), "_easytier_paste")
    }

    private inner class PasteBridge(private val webView: WebView) {
        @JavascriptInterface
        fun readClipboard(): String {
            val latch = CountDownLatch(1)
            var text = ""
            mainHandler.post {
                val cm = webView.context.getSystemService(Context.CLIPBOARD_SERVICE) as? ClipboardManager
                text = cm?.primaryClip?.getItemAt(0)?.text?.toString() ?: ""
                latch.countDown()
            }
            latch.await(500, TimeUnit.MILLISECONDS)
            return text
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

