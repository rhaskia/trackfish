package dev.dioxus.main

import android.media.session.MediaSession
import android.media.session.PlaybackState
import android.util.Log
import android.os.Handler
import android.os.Looper
import android.app.*
import android.content.Intent
import android.os.IBinder
import androidx.core.app.NotificationCompat
import android.app.Notification
import android.content.Context

class MediaCallback : MediaSession.Callback() {
    override fun onPlay() {
        Log.d("MediaCallback", "onPlay called from JNI!")
        nativeOnPlay();
    }

    override fun onPause() {
        Log.d("MediaCallback", "onPause called from JNI!")
        nativeOnPause();
    }

    override fun onSkipToNext() {
        Log.d("MediaCallback", "Skipped to next track")
        nativeOnNext();
    }

    override fun onSkipToPrevious() {
        Log.d("MediaCallback", "Skipped to previous track")
	nativeOnPrevious();
    }

    override fun onSeekTo(pos: Long) {
        Log.d("MediaCallback", "Seek to a position")
	nativeOnSeekTo(pos);
    }
}

object MediaHelper {
	@JvmStatic
	fun setMediaCallback(mediaSession: MediaSession, callback: MediaSession.Callback) {
 	   mediaSession.setCallback(callback, Handler(Looper.getMainLooper()))
	}
}

class RustAudioService : Service() {
    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val notification = getNotification(this)
	
        startForeground(1, notification)

	Log.d("AudioService", "Service started and notification displayed.")
        return START_STICKY
    }

    override fun onBind(intent: Intent?): IBinder? = null
}

external fun getNotification(context: Context): Notification

external fun nativeOnPlay()
external fun nativeOnPause()
external fun nativeOnNext()
external fun nativeOnPrevious()
external fun nativeOnSeekTo(pos: Long)

