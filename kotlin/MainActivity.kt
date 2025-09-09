package dev.dioxus.main;

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

import com.example.Trackfish.BuildConfig;
typealias BuildConfig = BuildConfig;

class MainActivity : WryActivity() 

