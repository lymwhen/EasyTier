package com.kkrainbow.easytier

import android.app.DownloadManager
import android.content.ClipData
import android.content.ContentValues
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.Environment
import android.os.Handler
import android.os.Looper
import android.provider.MediaStore
import android.webkit.GeolocationPermissions
import android.webkit.JavascriptInterface
import android.webkit.JsResult
import android.webkit.CookieManager
import android.webkit.MimeTypeMap
import android.webkit.PermissionRequest
import android.webkit.ValueCallback
import android.webkit.WebChromeClient
import android.webkit.WebView
import android.webkit.WebSettings
import androidx.activity.OnBackPressedCallback
import androidx.activity.result.ActivityResult
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.contract.ActivityResultContracts
import java.io.File
import java.io.FileOutputStream
import java.io.IOException
import java.io.OutputStream
import java.net.HttpURLConnection
import java.net.URL
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors
import org.json.JSONObject

class MainActivity : TauriActivity() {
    private val mainHandler = Handler(Looper.getMainLooper())
    private lateinit var fileChooserLauncher: ActivityResultLauncher<Intent>
    private var pendingFileChooser: ValueCallback<Array<Uri?>?>? = null
    private val downloadExecutor: ExecutorService = Executors.newCachedThreadPool()
    private var mainWebView: WebView? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        fileChooserLauncher = registerForActivityResult(
            ActivityResultContracts.StartActivityForResult(),
            ::handleFileChooserResult
        )
        onBackPressedDispatcher.addCallback(this, object : OnBackPressedCallback(true) {
            override fun handleOnBackPressed() {
                moveTaskToBack(true)
            }
        })
        initService()
    }

    override fun onWebViewCreate(webView: WebView) {
        super.onWebViewCreate(webView)
        mainWebView = webView
        webView.isLongClickable = true
        webView.addJavascriptInterface(ThemeBridge(), "_easytier_theme")
        // Disable pinch-to-zoom (two-finger gesture) on Android WebView
        webView.settings.cacheMode = android.webkit.WebSettings.LOAD_NO_CACHE
        webView.settings.setSupportZoom(false)
        webView.settings.builtInZoomControls = false

        webView.setDownloadListener { url, userAgent, contentDisposition, mimeType, _ ->
            startRouterDownload(url, userAgent, contentDisposition, mimeType)
        }

        // Wry installs its WebChromeClient after onWebViewCreate returns.
        // Replace it on the next UI turn with a wrapper that hardens file selection
        // while delegating JavaScript dialogs and permissions back to Wry.
        webView.post {
            val wryClient = webView.webChromeClient ?: return@post
            webView.webChromeClient = RouterWebChromeClient(wryClient)
        }
    }

    private fun handleFileChooserResult(result: ActivityResult) {
        val callback = pendingFileChooser
        pendingFileChooser = null
        val selected = mutableListOf<Uri>()
        val data = result.data
        if (result.resultCode == RESULT_OK && data != null) {
            val clipData: ClipData? = data.clipData
            if (clipData != null) {
                for (index in 0 until clipData.itemCount) {
                    clipData.getItemAt(index)?.uri?.let(selected::add)
                }
            }
            data.data?.let(selected::add)
        }

        if (callback == null) return
        if (selected.isEmpty()) {
            callback.onReceiveValue(null)
            return
        }
        val uris = arrayOfNulls<Uri>(selected.size)
        selected.forEachIndexed { index, uri -> uris[index] = uri }
        callback.onReceiveValue(uris)
    }

    private fun startRouterDownload(
        url: String?,
        userAgent: String?,
        contentDisposition: String?,
        mimeType: String?
    ) {
        if (url.isNullOrBlank() || (!url.startsWith("http://") && !url.startsWith("https://"))) {
            emitNativeTransfer("failed", url ?: "file", "Unsupported download URL")
            return
        }

        var fileName = url.substringAfterLast('/').ifBlank { "download" }
        try {
            fileName = android.webkit.URLUtil.guessFileName(
                url,
                contentDisposition,
                mimeType?.takeIf { it.isNotBlank() } ?: "application/octet-stream"
            )
            if (isLocalProxyUrl(url)) {
                startLocalDownload(url, fileName, mimeType, userAgent)
                return
            }

            val request = DownloadManager.Request(Uri.parse(url))
                .setTitle(fileName)
                .setDescription(fileName)
                .setNotificationVisibility(DownloadManager.Request.VISIBILITY_VISIBLE_NOTIFY_COMPLETED)
                .setAllowedOverMetered(true)
                .setAllowedOverRoaming(true)
            if (!mimeType.isNullOrBlank()) request.setMimeType(mimeType)
            if (!userAgent.isNullOrBlank()) request.addRequestHeader("User-Agent", userAgent)

            try {
                request.setDestinationInExternalPublicDir(Environment.DIRECTORY_DOWNLOADS, fileName)
            } catch (_: Exception) {
                request.setDestinationInExternalFilesDir(this, Environment.DIRECTORY_DOWNLOADS, fileName)
            }

            val manager = getSystemService(Context.DOWNLOAD_SERVICE) as DownloadManager
            manager.enqueue(request)
            emitNativeTransfer("external-started", fileName)
        } catch (error: Exception) {
            emitNativeTransfer("failed", fileName, error.message)
        }
    }

    private fun isLocalProxyUrl(url: String): Boolean {
        val host = Uri.parse(url).host?.lowercase()
        return host == "127.0.0.1" || host == "localhost" || host == "::1"
    }

    private data class LocalDownloadTarget(val uri: Uri? = null, val file: File? = null)

    private fun createLocalDownloadTarget(fileName: String, mimeType: String?): LocalDownloadTarget {
        val resolvedType = mimeType?.takeIf { it.isNotBlank() } ?: "application/octet-stream"
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            val values = ContentValues().apply {
                put(MediaStore.MediaColumns.DISPLAY_NAME, fileName)
                put(MediaStore.MediaColumns.MIME_TYPE, resolvedType)
                put(MediaStore.MediaColumns.RELATIVE_PATH, Environment.DIRECTORY_DOWNLOADS)
                put(MediaStore.MediaColumns.IS_PENDING, 1)
            }
            val uri = contentResolver.insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, values)
                ?: throw IOException("Unable to create download entry")
            return LocalDownloadTarget(uri = uri)
        }

        val publicDir = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DOWNLOADS)
        val publicFile = File(publicDir, fileName)
        return try {
            publicDir.mkdirs()
            FileOutputStream(publicFile).use { }
            LocalDownloadTarget(file = publicFile)
        } catch (_: Exception) {
            val fallbackDir = getExternalFilesDir(Environment.DIRECTORY_DOWNLOADS) ?: filesDir
            fallbackDir.mkdirs()
            LocalDownloadTarget(file = File(fallbackDir, fileName))
        }
    }

    private fun openLocalDownloadOutput(target: LocalDownloadTarget): OutputStream {
        target.uri?.let { uri ->
            return contentResolver.openOutputStream(uri, "w")
                ?: throw IOException("Unable to open download output")
        }
        val file = target.file ?: throw IOException("Missing download target")
        return FileOutputStream(file)
    }

    private fun finishLocalDownload(target: LocalDownloadTarget) {
        target.uri?.let { uri ->
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                val values = ContentValues().apply { put(MediaStore.MediaColumns.IS_PENDING, 0) }
                contentResolver.update(uri, values, null, null)
            }
        }
    }

    private fun cleanupLocalDownload(target: LocalDownloadTarget) {
        target.uri?.let { uri ->
            runCatching { contentResolver.delete(uri, null, null) }
        }
        target.file?.let { file ->
            runCatching { if (file.exists()) file.delete() }
        }
    }

    private fun emitNativeTransfer(status: String, fileName: String, error: String? = null) {
        val detail = JSONObject().apply {
            put("status", status)
            put("filename", fileName)
            put("error", error ?: JSONObject.NULL)
        }
        mainHandler.post {
            mainWebView?.evaluateJavascript(
                "window.dispatchEvent(new CustomEvent('easytier-native-transfer', { detail: $detail }))",
                null
            )
        }
    }

    private fun startLocalDownload(url: String, fileName: String, mimeType: String?, userAgent: String?) {
        downloadExecutor.execute {
            var target: LocalDownloadTarget? = null
            try {
                val replayUrl = Uri.parse(url)
                    .buildUpon()
                    .path("/__easytier_download__")
                    .clearQuery()
                    .appendQueryParameter("url", url)
                    .build()
                    .toString()
                val connection = URL(replayUrl).openConnection() as HttpURLConnection
                connection.connectTimeout = 15_000
                connection.readTimeout = 60_000
                connection.instanceFollowRedirects = true
                connection.setRequestProperty("Accept", "*/*")
                connection.setRequestProperty(
                    "User-Agent",
                    userAgent?.takeIf { it.isNotBlank() } ?: WebSettings.getDefaultUserAgent(this)
                )
                CookieManager.getInstance().getCookie(replayUrl)
                    ?.takeIf { it.isNotBlank() }
                    ?.let { connection.setRequestProperty("Cookie", it) }
                URL(replayUrl).let { parsed ->
                    connection.setRequestProperty(
                        "Referer",
                        "${parsed.protocol}://${parsed.authority}/cgi-bin/luci/admin/"
                    )
                }
                connection.connect()
                if (connection.responseCode !in 200..299) {
                    val detail = connection.errorStream
                        ?.bufferedReader()
                        ?.use { it.readText() }
                        ?.replace('\n', ' ')
                        ?.take(300)
                    val suffix = if (detail.isNullOrBlank()) "" else ": $detail"
                    throw IOException("Download failed with HTTP ${connection.responseCode}$suffix")
                }

                target = createLocalDownloadTarget(fileName, mimeType)
                connection.inputStream.use { input ->
                    openLocalDownloadOutput(target).use { output ->
                        input.copyTo(output, 64 * 1024)
                    }
                }
                finishLocalDownload(target)
                emitNativeTransfer("saved", fileName)
            } catch (error: Exception) {
                target?.let(::cleanupLocalDownload)
                emitNativeTransfer("failed", fileName, error.message)
            }
        }
    }

    private fun acceptedMimeTypes(params: WebChromeClient.FileChooserParams): Array<String> {
        val types = params.acceptTypes
            .map { it.trim() }
            .filter { it.isNotEmpty() && it != "*" && it != "*/*" }
            .map { accept ->
                if (accept.startsWith(".")) {
                    val extension = accept.removePrefix(".").substringAfterLast('.').lowercase()
                    MimeTypeMap.getSingleton().getMimeTypeFromExtension(extension) ?: "application/octet-stream"
                } else {
                    accept
                }
            }
            .distinct()
        return types.toTypedArray()
    }

    private fun handleFileChooser(
        filePathCallback: ValueCallback<Array<Uri?>?>,
        params: WebChromeClient.FileChooserParams
    ): Boolean {
        pendingFileChooser?.onReceiveValue(null)
        pendingFileChooser = filePathCallback

        val mimeTypes = acceptedMimeTypes(params)
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = mimeTypes.singleOrNull()?.takeUnless { it.endsWith("/*") } ?: "*/*"
            putExtra(Intent.EXTRA_ALLOW_MULTIPLE, params.mode == WebChromeClient.FileChooserParams.MODE_OPEN_MULTIPLE)
            addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            if (mimeTypes.isNotEmpty()) putExtra(Intent.EXTRA_MIME_TYPES, mimeTypes)
        }

        return try {
            fileChooserLauncher.launch(intent)
            true
        } catch (_: Exception) {
            pendingFileChooser = null
            filePathCallback.onReceiveValue(null)
            false
        }
    }

    private inner class RouterWebChromeClient(
        private val delegate: WebChromeClient
    ) : WebChromeClient() {
        override fun onShowFileChooser(
            webView: WebView,
            filePathCallback: ValueCallback<Array<Uri?>?>,
            fileChooserParams: FileChooserParams
        ): Boolean {
            if (fileChooserParams.isCaptureEnabled) {
                return delegate.onShowFileChooser(webView, filePathCallback, fileChooserParams)
            }
            return handleFileChooser(filePathCallback, fileChooserParams)
        }

        override fun onShowCustomView(view: android.view.View?, callback: CustomViewCallback?) {
            if (view != null && callback != null) delegate.onShowCustomView(view, callback)
            else super.onShowCustomView(view, callback)
        }

        override fun onHideCustomView() {
            delegate.onHideCustomView()
        }

        override fun onPermissionRequest(request: PermissionRequest) {
            delegate.onPermissionRequest(request)
        }

        override fun onJsAlert(view: WebView, url: String, message: String, result: JsResult): Boolean {
            return delegate.onJsAlert(view, url, message, result)
        }

        override fun onJsConfirm(view: WebView, url: String, message: String, result: JsResult): Boolean {
            return delegate.onJsConfirm(view, url, message, result)
        }

        override fun onJsPrompt(
            view: WebView,
            url: String,
            message: String,
            defaultValue: String?,
            result: android.webkit.JsPromptResult
        ): Boolean {
            return delegate.onJsPrompt(view, url, message, defaultValue, result)
        }

        override fun onGeolocationPermissionsShowPrompt(
            origin: String,
            callback: GeolocationPermissions.Callback
        ) {
            delegate.onGeolocationPermissionsShowPrompt(origin, callback)
        }
    }

    private inner class ThemeBridge {
        @JavascriptInterface
        @Suppress("DEPRECATION")
        fun setStatusBarStyle(dark: Boolean) {
            mainHandler.post {
                val window = window ?: return@post
                if (dark) {
                    window.statusBarColor = 0xFF000000.toInt()
                    window.navigationBarColor = 0xFF000000.toInt()
                    window.decorView.systemUiVisibility = 0
                } else {
                    window.statusBarColor = 0xFFFCFCFC.toInt()
                    window.navigationBarColor = 0xFFFCFCFC.toInt()
                    window.decorView.systemUiVisibility = android.view.View.SYSTEM_UI_FLAG_LIGHT_STATUS_BAR
                }
            }
        }
    }

    override fun onDestroy() {
        mainWebView = null
        downloadExecutor.shutdownNow()
        super.onDestroy()
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
