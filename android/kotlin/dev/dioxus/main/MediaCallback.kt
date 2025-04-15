package dev.dioxus.main

import android.media.session.MediaSession
import android.media.session.PlaybackState
import android.util.Log
import android.os.Handler
import android.os.Looper

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

external fun nativeOnPlay()
external fun nativeOnPause()
external fun nativeOnNext()
external fun nativeOnPrevious()
external fun nativeOnSeekTo(pos: Long)