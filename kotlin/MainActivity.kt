package dev.dioxus.main

// need to re-export buildconfig down from the parent
import com.example.Trackfish.BuildConfig
typealias BuildConfig = BuildConfig

import android.Manifest
import android.annotation.SuppressLint
import android.app.*
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import android.os.Bundle
import android.util.Log
import android.view.KeyEvent
import android.webkit.WebView
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import androidx.core.content.ContextCompat

abstract class BaseWryActivity : AppCompatActivity() {
    private lateinit var mWebView: RustWebView

    open fun onWebViewCreate(webView: WebView) { }

    fun setWebView(webView: RustWebView) {
        mWebView = webView
        onWebViewCreate(webView)
    }

    val version: String
        @SuppressLint("WebViewApiAvailability", "ObsoleteSdkInt")
        get() {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                return WebView.getCurrentWebViewPackage()?.versionName ?: ""
            }

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

            return ""
        }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Start a foreground service to keep this process alive
        val intent = Intent(this, KeepAliveService::class.java)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            startForegroundService(intent)
        } else {
            startService(intent)
        }

        create(this)
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

    fun getAppClass(name: String): Class<*> {
        return Class.forName(name)
    }

    companion object {
        init {
            System.loadLibrary("dioxusmain")
        }
    }

    private external fun create(activity: BaseWryActivity)
    private external fun start()
    private external fun resume()
    private external fun pause()
    private external fun stop()
    private external fun save()
    private external fun destroy()
    private external fun onActivityDestroy()
    private external fun memory()
    private external fun focus(focus: Boolean)

    class KeepAliveService : Service() {

        companion object {
            const val NOTIF_ID = 1
            const val CHANNEL_ID = "media_channel"
        }

        override fun onCreate() {
            super.onCreate()
            createNotificationChannel()

            mediaSession = MediaSessionCompat(this, "MusicService")

            val baseNotification = NotificationCompat.Builder(this, CHANNEL_ID)
                .setContentTitle("Trackfish")
                .setContentText("App is running")
                .setSmallIcon(android.R.drawable.ic_media_play)
                .setOngoing(true)
                .setStyle(
                    MediaStyle()
                        .setMediaSession(mediaSession.sessionToken)
                        .setShowActionsInCompactView() // No actions yet, but layout is set
                )
                .build()

            startForeground(NOTIF_ID, baseNotification)
        }

        override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
            // Service logic here if any

            return START_STICKY // keep service running
        }

        override fun onBind(intent: Intent?): IBinder? = null

        private fun createNotificationChannel() {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                val channel = NotificationChannel(
                    CHANNEL_ID,
                    "Media Playback",
                    NotificationManager.IMPORTANCE_LOW
                )
                val manager = getSystemService(NotificationManager::class.java)
                manager.createNotificationChannel(channel)
            }
        }
    }

}

class MainActivity : BaseWryActivity() {

    private val permissionsToRequest = arrayOf(
        Manifest.permission.READ_MEDIA_AUDIO,
        Manifest.permission.WAKE_LOCK,
        Manifest.permission.FOREGROUND_SERVICE,
        Manifest.permission.POST_NOTIFICATIONS
    )

    private val requestPermissionsLauncher = registerForActivityResult(
        ActivityResultContracts.RequestMultiplePermissions()
    ) { permissions ->
        val allGranted = permissions.entries.all { it.value }
        if (allGranted) {
            onPermissionsGranted()
        } else {
            onPermissionsDenied()
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        Log.i("com.example.Music", "Info message")

        if (!hasAllPermissions()) {
            requestPermissionsLauncher.launch(permissionsToRequest)
        } else {
            onPermissionsGranted()
        }
    }

    private fun hasAllPermissions(): Boolean {
        return permissionsToRequest.all { perm ->
            ContextCompat.checkSelfPermission(this, perm) == PackageManager.PERMISSION_GRANTED
        }
    }

    private fun onPermissionsGranted() {
        // Permissions granted — safe to proceed with audio access, notifications, etc.
    }

    private fun onPermissionsDenied() {
        // Permissions denied — show UI or disable related features gracefully
    }
}

fun createBaseNotification(context: Context) {
    val channelId = "media_channel"
    val channelName = "Media Controls"

    val manager = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager

    // Create the channel if not exists
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
        val channel = NotificationChannel(channelId, channelName, NotificationManager.IMPORTANCE_LOW)
        manager.createNotificationChannel(channel)
    }

    val notif = NotificationCompat.Builder(context, channelId)
        .setContentTitle("Loading...")
        .setContentText("Preparing media controls")
        .setSmallIcon(android.R.drawable.ic_media_play)
        .setOngoing(true)
        .build()

    manager.notify(1, notif) // Known ID = 1
}
