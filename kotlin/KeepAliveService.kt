package dev.dioxus.main

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Intent
import android.graphics.Bitmap
import android.media.MediaMetadata
import android.media.session.MediaController
import android.media.session.MediaSession
import android.media.session.PlaybackState
import android.os.Build
import android.os.IBinder
import android.app.Notification
import android.app.Notification.MediaStyle
import android.util.Log
import android.media.AudioManager
import android.content.Context
import android.media.AudioFocusRequest
import android.media.AudioAttributes

class KeepAliveService : Service() {
    private lateinit var mediaSession: MediaSession
    private lateinit var mediaCallback: MediaCallback
    private lateinit var mediaController: MediaController
    private lateinit var notificationManager: NotificationManager
    private var focusRequest: AudioFocusRequest? = null
    private val channelId = "media_channel"
    var hasAudioFocus = false
    var initialized = false

    companion object {
        @JvmStatic
        var serviceInstance: KeepAliveService? = null
    }

    override fun onCreate() {
        super.onCreate()
        serviceInstance = this
        setupMediaSession()
        setupNotificationChannel()
        createMediaNotification("Unknown Title", "Unknown Artist", 0, 1000, false, null, true)
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        return START_STICKY
    }

    override fun onBind(intent: Intent?): IBinder? = null

    private fun setupMediaSession() {
        mediaSession = MediaSession(this, "RustMediaSession").apply {
            setPlaybackState(
                PlaybackState.Builder()
                    .setActions(
                        PlaybackState.ACTION_PLAY or
                        PlaybackState.ACTION_PAUSE or
                        PlaybackState.ACTION_SKIP_TO_NEXT or
                        PlaybackState.ACTION_SKIP_TO_PREVIOUS or 
                        PlaybackState.ACTION_SEEK_TO
                    )
                    .setState(PlaybackState.STATE_PAUSED, 0L, 1f)
                    .build()
            )
            isActive = true
        }
        mediaController = MediaController(this, mediaSession.sessionToken)
        mediaCallback = MediaCallback()
        MediaHelper.setMediaCallback(mediaSession, mediaCallback)
    }

    private fun setupNotificationChannel() {
        val channelId = "media_channel"
        notificationManager = getSystemService(NotificationManager::class.java)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                channelId,
                "Rust Media Controls",
                NotificationManager.IMPORTANCE_LOW
            )
            notificationManager.createNotificationChannel(channel)
        }
    }

    private fun createMediaNotification(
        title: String,
        artist: String,
        trackLengthMs: Long,
        progressMs: Long,
        isPlaying: Boolean,
        artworkBytes: ByteArray?,
        foreground: Boolean,
    ) {
        val iconId = resources.getIdentifier("ic_notification", "drawable", packageName)

        val bitmap = artworkBytes?.let {
            android.graphics.BitmapFactory.decodeByteArray(it, 0, it.size)
        }

        if (isPlaying) {
            if (!hasAudioFocus) {
                requestAudioFocus()
            }
        } else {
            abandonAudioFocus()
        }

        Log.i("com.example.Music", "requested media notification with state " + isPlaying)

        // Update MediaSession playback state
        val state = android.media.session.PlaybackState.Builder()
            .setState(
                if (isPlaying) PlaybackState.STATE_PLAYING else PlaybackState.STATE_PAUSED,
                progressMs,
                1.0f // playback speed
            )
            .setActions(
                PlaybackState.ACTION_PLAY or
                PlaybackState.ACTION_PAUSE or
                PlaybackState.ACTION_SKIP_TO_NEXT or
                PlaybackState.ACTION_SKIP_TO_PREVIOUS or 
                PlaybackState.ACTION_SEEK_TO
            )
            .build()

        mediaSession.setPlaybackState(state)

        val metadata = MediaMetadata.Builder()
            .putString(MediaMetadata.METADATA_KEY_TITLE, title)
            .putString(MediaMetadata.METADATA_KEY_ARTIST, artist)
            .putLong(android.media.MediaMetadata.METADATA_KEY_DURATION, trackLengthMs)
            .apply {
                if (bitmap != null) {
                    putBitmap(MediaMetadata.METADATA_KEY_ALBUM_ART, bitmap)
                }
            }
            .build()

        mediaSession.setMetadata(metadata)

        val notification: Notification = Notification.Builder(this, "media_channel")
            .setContentTitle(title)
            .setContentText(artist)
            .setLargeIcon(bitmap)
            .setSmallIcon(iconId)
            .setContentIntent(mediaController.sessionActivity)
            .setVisibility(Notification.VISIBILITY_PUBLIC)
            .setStyle(
                MediaStyle()
                    .setMediaSession(mediaSession.sessionToken)
                    .setShowActionsInCompactView(0, 1, 2)
            )
            .addAction(Notification.Action(android.R.drawable.ic_media_previous, "Prev", null))
            .addAction(Notification.Action(android.R.drawable.ic_media_pause, "Pause", null))
            .addAction(Notification.Action(android.R.drawable.ic_media_next, "Next", null))
            .build()
            
        startForeground(1, notification)

        Log.i("com.example.Music", title)
    }

    // JNI-callable
    fun updateMediaNotification(
        title: String,
        artist: String,
        trackLengthMs: Long,
        progressMs: Long,
        isPlaying: Boolean,
        artworkbytes: ByteArray?
    ) {
        createMediaNotification(title, artist, trackLengthMs, progressMs, isPlaying, artworkbytes, false)
    }

    private fun requestAudioFocus(): Boolean {
        val audioManager = getSystemService(Context.AUDIO_SERVICE) as AudioManager

        focusRequest = AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN)
            .setAudioAttributes(
                AudioAttributes.Builder()
                    .setUsage(AudioAttributes.USAGE_MEDIA)
                    .setContentType(AudioAttributes.CONTENT_TYPE_MUSIC)
                    .build()
            )
            .setOnAudioFocusChangeListener { focusChange: Int ->
                when (focusChange) {
                    AudioManager.AUDIOFOCUS_LOSS,
                    AudioManager.AUDIOFOCUS_LOSS_TRANSIENT,
                    AudioManager.AUDIOFOCUS_LOSS_TRANSIENT_CAN_DUCK -> {
                        Log.i("KeepAliveService", "Lost audio focus: $focusChange")
                        nativeOnAudioFocusLost(focusChange)
                        hasAudioFocus = false
                    }
                    AudioManager.AUDIOFOCUS_GAIN -> {
                        Log.i("KeepAliveService", "Gained audio focus")
                        nativeOnAudioFocusGained()
                        hasAudioFocus = true
                    }
                }
            }
            .build()

        val result = audioManager.requestAudioFocus(focusRequest!!)
        hasAudioFocus = true
        return result == AudioManager.AUDIOFOCUS_REQUEST_GRANTED
    }

    private fun abandonAudioFocus() {
        val audioManager = getSystemService(Context.AUDIO_SERVICE) as AudioManager
        focusRequest?.let {
            audioManager.abandonAudioFocusRequest(it)
            focusRequest = null
        }
    }

    external fun nativeOnAudioFocusLost(focusChange: Int)
    external fun nativeOnAudioFocusGained()
}

