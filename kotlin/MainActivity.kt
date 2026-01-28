package dev.dioxus.main

import android.app.NativeActivity
import android.os.Bundle
import android.view.View
import android.view.ViewGroup
import androidx.core.view.ViewCompat
import androidx.core.view.OnApplyWindowInsetsListener
import androidx.core.view.WindowInsetsCompat
import dev.dioxus.main.Logger
import android.annotation.SuppressLint
import android.os.Build
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
import android.graphics.Color

// Makes basic text input work with NativeActivity
// Copied from https://github.com/rust-mobile/android-activity/pull/178#issuecomment-2572817579
class MainActivity : NativeActivity() { // ,OnApplyWindowInsetsListener {
    private fun getNativeActivityView(): View {
        // This is hacky as hell, but NativeActivity does not give any proper way of accessing it.
        var parent = window.decorView as ViewGroup
        parent = parent.getChildAt(0) as ViewGroup
        parent = parent.getChildAt(1) as ViewGroup
        return parent.getChildAt(0)
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val nativeActivityView = getNativeActivityView()
        nativeActivityView.isFocusable = true
        nativeActivityView.isFocusableInTouchMode = true
        nativeActivityView.requestFocus()

        // ViewCompat.setOnApplyWindowInsetsListener(nativeActivityView, this)
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

    override fun onDestroy() {
        KeepAliveService.serviceInstance?.stopService()
        super.onDestroy()
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

    companion object {
        init {
            NativeLoader.ensureLoaded()
        }
    }
}