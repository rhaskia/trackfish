// Modified Tauri Kotlin Code

// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package dev.dioxus.main

import dev.dioxus.main.RustWebView
import dev.dioxus.main.Logger
import android.annotation.SuppressLint
import android.os.Build
import android.os.Bundle
import android.webkit.WebView
import android.view.KeyEvent
import androidx.appcompat.app.AppCompatActivity
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Intent
import android.os.IBinder
import android.util.Log
import android.widget.Button
import android.Manifest
import android.content.pm.PackageManager
import android.widget.Toast
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import android.os.PowerManager
import android.view.WindowManager
import android.view.View
import android.graphics.Color

abstract class WryActivity : AppCompatActivity() {
    private lateinit var mWebView: RustWebView

    open fun onWebViewCreate(webView: WebView) { }

    fun setWebView(webView: RustWebView) {
        mWebView = webView
        onWebViewCreate(webView)
    }

    val version: String
        @SuppressLint("WebViewApiAvailability", "ObsoleteSdkInt")
        get() {
            // Check getCurrentWebViewPackage() directly if above Android 8
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                return WebView.getCurrentWebViewPackage()?.versionName ?: ""
            }

            // Otherwise manually check WebView versions
            var webViewPackage = "com.google.android.webview"
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
              webViewPackage = "com.android.chrome"
            }
            try {
                @Suppress("DEPRECATION")
                val info = packageManager.getPackageInfo(webViewPackage, 0)
                return info.versionName.toString()
            } catch (ex: Exception) {
                Logger.warn("Unable to get package info for '$webViewPackage'$ex")
            }

            try {
                @Suppress("DEPRECATION")
                val info = packageManager.getPackageInfo("com.android.webview", 0)
                return info.versionName.toString()
            } catch (ex: Exception) {
                Logger.warn("Unable to get package info for 'com.android.webview'$ex")
            }

            // Could not detect any webview, return empty string
            return ""
        }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        requestPermissions()

        if (Build.VERSION.SDK_INT >= 19 && Build.VERSION.SDK_INT < 21) {
            setWindowFlag(WindowManager.LayoutParams.FLAG_TRANSLUCENT_STATUS, true)
        }
        if (Build.VERSION.SDK_INT >= 19) {
            window.decorView.systemUiVisibility = View.SYSTEM_UI_FLAG_LAYOUT_STABLE or View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
        }
        if (Build.VERSION.SDK_INT >= 21) {
            setWindowFlag(WindowManager.LayoutParams.FLAG_TRANSLUCENT_STATUS, false)
            window.statusBarColor = Color.TRANSPARENT
        }

        if (KeepAliveService.serviceInstance == null) {
            Intent().setClassName("com.example.Trackfish", "dev.dioxus.main.KeepAliveService")
            val intent = Intent(this, KeepAliveService::class.java)

            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                startForegroundService(intent)
            } else {
                startService(intent)
            }
            Log.i("com.example.Music", "started KeepAliveService")

            create(this)
        } else {
            Log.i("com.example.Trackfish", "Keep Alive Service already exists")
        }
    }

    private fun setWindowFlag(bits: Int, on: Boolean) {
        val win = window
        val winParams = win.attributes
        if (on) {
            winParams.flags = winParams.flags or bits
        } else {
            winParams.flags = winParams.flags and bits.inv()
        }
        win.attributes = winParams
    }

    override fun onStart() {
        super.onStart()
        start()
    }

    override fun onResume() {
        super.onResume()
        resume()
    }

    override fun onPause() {
        super.onPause()
        pause()
    }

    override fun onStop() {
        super.onStop()
        stop()
    }

    override fun onWindowFocusChanged(hasFocus: Boolean) {
        super.onWindowFocusChanged(hasFocus)
        focus(hasFocus)
    }

    override fun onSaveInstanceState(outState: Bundle) {
        super.onSaveInstanceState(outState)
        save()
    }

    override fun onDestroy() {
        KeepAliveService.serviceInstance?.stopService()
        super.onDestroy()
        destroy()
        onActivityDestroy()
    }

    override fun onLowMemory() {
        super.onLowMemory()
        memory()
    }

    override fun onKeyDown(keyCode: Int, event: KeyEvent?): Boolean {
        if (keyCode == KeyEvent.KEYCODE_BACK && mWebView.canGoBack()) {
            mWebView.goBack()
            return true
        }
        return super.onKeyDown(keyCode, event)
    }

    private fun requestPermissions() {
        val permissions = mutableListOf(
            Manifest.permission.POST_NOTIFICATIONS
        )

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            permissions.add(
                Manifest.permission.READ_MEDIA_AUDIO
            )
        } else {
            permissions.add(
                Manifest.permission.READ_EXTERNAL_STORAGE
            )
        }

        val permissionsToRequest = permissions.filter { permission ->
            ContextCompat.checkSelfPermission(this, permission) != PackageManager.PERMISSION_GRANTED
        }

        if (permissionsToRequest.isNotEmpty()) {
            ActivityCompat.requestPermissions(
                this,
                permissionsToRequest.toTypedArray(), // Convert list to array
                1001 // Pass the request code
            )
        }
    }

    fun getAppClass(name: String): Class<*> {
        return Class.forName(name)
    }
    private external fun create(activity: WryActivity)
    private external fun start()
    private external fun resume()
    private external fun pause()
    private external fun stop()
    private external fun save()
    private external fun destroy()
    private external fun onActivityDestroy()
    private external fun memory()
    private external fun focus(focus: Boolean)

    companion object {
        init {
            NativeLoader.ensureLoaded()
        }
    }
}

