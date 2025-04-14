use jni::objects::JClass;
use jni::objects::JObject;
use jni::objects::JValue;
use jni::sys::jint;
use jni::JNIEnv;
use log::info;

pub fn set_media_context() {
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.unwrap();
    let mut env = vm.attach_current_thread().unwrap();
    let class_ctx = env.find_class("android/content/Context").unwrap();
    let context = unsafe { JObject::from_raw(ctx.context().cast()) };

    let media_session_class = env.find_class("android/media/session/MediaSession").unwrap();

    let tag = env.new_string("MusicService").unwrap();
    let mut media_session = env
        .new_object(
            media_session_class,
            "(Landroid/content/Context;Ljava/lang/String;)V",
            &[JValue::Object(&context), JValue::Object(&tag.into())],
        )
        .unwrap();

    // let callback_class = env.find_class("com/example/myapp/MediaSessionCallback")?;
    // let callback_obj = env.new_object(callback_class, "()V", &[])?;
    //
    // env.call_method(
    //     &media_session,
    //     "setCallback",
    //     "(Landroid/media/session/MediaSession$Callback;)V",
    //     &[JValue::Object(&callback_obj)],
    // )?;

    let flag_handles_media_buttons = 1;
    let flag_handles_transport_controls = 2;
    let flags = flag_handles_media_buttons | flag_handles_transport_controls;

    try_to_get_audio_focus(&mut env, &context);

    env.call_method(&media_session, "setFlags", "(I)V", &[JValue::Int(flags)]);

    let is_active = env.call_method(&media_session, "isActive", "()Z", &[]).unwrap().z().unwrap();

    if !is_active {
        env.call_method(&media_session, "setActive", "(Z)V", &[JValue::Bool(1)]).unwrap();
    }

    update_playback_state(&mut env, &mut media_session, 2, 1, 0).unwrap();

    update_metadata(
        &mut env,
        &media_session,
        "Song Title",
        "Artist Name",
        None,
        "Song Title",
        "Artist",
        None,
    )
    .unwrap();

    show_media_notification(&mut env, &context).unwrap();
}

pub fn show_media_notification(env: &mut JNIEnv, context: &JObject) -> jni::errors::Result<()> {
    let channel_id = env.new_string("media_channel")?;
    let channel_name = env.new_string("Media Controls")?;
    let importance = 3;

    let channel_class = env.find_class("android/app/NotificationChannel")?;
    let channel = env.new_object(
        channel_class,
        "(Ljava/lang/String;Ljava/lang/CharSequence;I)V",
        &[
            JValue::Object(&channel_id),
            JValue::Object(&JObject::from(channel_name)),
            JValue::Int(importance),
        ],
    )?;

    let notif_service_str = env
        .get_static_field("android/content/Context", "NOTIFICATION_SERVICE", "Ljava/lang/String;")?
        .l()?;

    let notification_manager = env
        .call_method(
            &context,
            "getSystemService",
            "(Ljava/lang/String;)Ljava/lang/Object;",
            &[JValue::Object(&notif_service_str)],
        )?
        .l()?;

    env.call_method(
        &notification_manager,
        "createNotificationChannel",
        "(Landroid/app/NotificationChannel;)V",
        &[JValue::Object(&channel)],
    )?;

    let tag = env.new_string("RustMediaSession")?;
    let session_class = env.find_class("android/media/session/MediaSession")?;
    let session = env.new_object(
        session_class,
        "(Landroid/content/Context;Ljava/lang/String;)V",
        &[JValue::Object(&context), JValue::Object(&tag)],
    )?;

    let intent_class = env.find_class("android/content/Intent")?;
    let action_str = env.new_string("ACTION_PLAY")?;
    let play_intent = env.new_object(
        intent_class,
        "(Ljava/lang/String;)V",
        &[JValue::Object(&action_str)],
    )?;

    let pi_class = env.find_class("android/app/PendingIntent")?;
    let pending_intent = env.call_static_method(
        pi_class,
        "getBroadcast",
        "(Landroid/content/Context;ILandroid/content/Intent;I)Landroid/app/PendingIntent;",
        &[
            JValue::Object(&context),
            JValue::Int(0),
            JValue::Object(&play_intent),
            JValue::Int(0x04000000), // FLAG_IMMUTABLE
        ],
    )?.l()?;

    let builder_class = env.find_class("android/app/Notification$Builder")?;
    let builder = env.new_object(
        builder_class,
        "(Landroid/content/Context;Ljava/lang/String;)V",
        &[JValue::Object(&context), JValue::Object(&channel_id)],
    )?;

    let title = env.new_string("Track Title")?;
    let artist = env.new_string("Artist Name")?;
    env.call_method(&builder, "setContentTitle", "(Ljava/lang/CharSequence;)Landroid/app/Notification$Builder;", &[JValue::Object(&JObject::from(title))])?;
    env.call_method(&builder, "setContentText", "(Ljava/lang/CharSequence;)Landroid/app/Notification$Builder;", &[JValue::Object(&JObject::from(artist))])?;
    env.call_method(&builder, "setSmallIcon", "(I)Landroid/app/Notification$Builder;", &[JValue::Int(17301540)])?;

    let action_class = env.find_class("android/app/Notification$Action")?;
    let label = env.new_string("Play")?;
    let play_action = env.new_object(
        action_class,
        "(ILjava/lang/CharSequence;Landroid/app/PendingIntent;)V",
        &[
            JValue::Int(17301540), // icon
            JValue::Object(&JObject::from(label)),
            JValue::Object(&pending_intent),
        ],
    )?;

    env.call_method(
        &builder,
        "addAction",
        "(Landroid/app/Notification$Action;)Landroid/app/Notification$Builder;",
        &[JValue::Object(&play_action)],
    )?;

    let style_class = env.find_class("android/app/Notification$MediaStyle")?;
    let media_style = env.new_object(style_class, "()V", &[])?;

    let session_token = env.call_method(session, "getSessionToken", "()Landroid/media/session/MediaSession$Token;", &[])?.l()?;

    env.call_method(
        &media_style,
        "setMediaSession",
        "(Landroid/media/session/MediaSession$Token;)Landroid/app/Notification$MediaStyle;",
        &[JValue::Object(&session_token)],
    )?;

    env.call_method(
        &builder,
        "setStyle",
        "(Landroid/app/Notification$Style;)Landroid/app/Notification$Builder;",
        &[JValue::Object(&media_style)],
    )?;

    let notification = env.call_method(builder, "build", "()Landroid/app/Notification;", &[])?.l()?;

    env.call_method(
        &notification_manager,
        "notify",
        "(ILandroid/app/Notification;)V",
        &[JValue::Int(1), JValue::Object(&notification)],
    )?;

    Ok(())
}

fn try_to_get_audio_focus(env: &mut JNIEnv, context: &JObject) -> jni::errors::Result<jint> {
    let audio_service_field = env
        .get_static_field("android/content/Context", "AUDIO_SERVICE", "Ljava/lang/String;")?
        .l()?;

    let audio_manager_obj = env
        .call_method(
            &context,
            "getSystemService",
            "(Ljava/lang/String;)Ljava/lang/Object;",
            &[JValue::Object(&audio_service_field)],
        )?
        .l()?;

    let audio_manager = JObject::from(audio_manager_obj);

    let stream_music = 3; // AudioManager.STREAM_MUSIC
    let focus_gain = 1; // AudioManager.AUDIOFOCUS_GAIN

    let result = env
        .call_method(
            &audio_manager,
            "requestAudioFocus",
            "(Landroid/media/AudioManager$OnAudioFocusChangeListener;II)I",
            &[
                JValue::Object(&JObject::null()), // no listener
                JValue::Int(stream_music),
                JValue::Int(focus_gain),
            ],
        )?
        .i()?;

    Ok(result)
}

fn update_playback_state(
    env: &mut JNIEnv,
    media_session: &mut JObject,
    m_state: i32,
    playing_queue_len: usize,
    current_index_on_queue: usize,
) -> jni::errors::Result<()> {
    let builder_class = env.find_class("android/media/session/PlaybackState$Builder")?;
    let builder = env.new_object(builder_class, "()V", &[])?;

    let mut actions = android_media_constants::ACTION_PLAY_PAUSE
        | android_media_constants::ACTION_PLAY_FROM_MEDIA_ID
        | android_media_constants::ACTION_PLAY_FROM_SEARCH;

    if playing_queue_len > 0 {
        if m_state == android_media_constants::STATE_PLAYING {
            actions |= android_media_constants::ACTION_PAUSE;
        } else {
            actions |= android_media_constants::ACTION_PLAY;
        }
        if current_index_on_queue > 0 {
            actions |= android_media_constants::ACTION_SKIP_TO_PREVIOUS;
        }
        if current_index_on_queue < playing_queue_len - 1 {
            actions |= android_media_constants::ACTION_SKIP_TO_NEXT;
        }
    }

    env.call_method(
        &builder,
        "setActions",
        "(J)Landroid/media/session/PlaybackState$Builder;",
        &[JValue::Long(actions as i64)],
    )?;

    let position = -1_i64;
    let playback_speed = 1.0f32;

    env.call_method(
        &builder,
        "setState",
        "(IJF)Landroid/media/session/PlaybackState$Builder;",
        &[JValue::Int(m_state), JValue::Long(position), JValue::Float(playback_speed)],
    )?;

    let playback_state =
        env.call_method(&builder, "build", "()Landroid/media/session/PlaybackState;", &[])?.l()?;

    env.call_method(
        &media_session,
        "setPlaybackState",
        "(Landroid/media/session/PlaybackState;)V",
        &[JValue::Object(&playback_state)],
    )?;

    Ok(())
}

mod android_media_constants {
    pub const STATE_PLAYING: i32 = 3;

    pub const ACTION_PLAY: i64 = 1 << 2;
    pub const ACTION_PAUSE: i64 = 1 << 1;
    pub const ACTION_PLAY_PAUSE: i64 = 1 << 0;
    pub const ACTION_SKIP_TO_PREVIOUS: i64 = 1 << 7;
    pub const ACTION_SKIP_TO_NEXT: i64 = 1 << 8;
    pub const ACTION_PLAY_FROM_MEDIA_ID: i64 = 1 << 4;
    pub const ACTION_PLAY_FROM_SEARCH: i64 = 1 << 5;
}

pub fn update_metadata<'a>(
    env: &mut JNIEnv<'a>,
    session: &JObject<'a>,
    display_title: &str,
    display_subtitle: &str,
    display_icon_uri: Option<&str>, // Optional icon URI
    title: &str,
    artist: &str,
    art_bitmap: Option<JObject<'a>>, // Optional Bitmap
) -> jni::errors::Result<()> {
    let metadata_builder_class = env.find_class("android/media/MediaMetadata$Builder")?;
    let metadata_builder = env.new_object(metadata_builder_class, "()V", &[])?;

    macro_rules! put_string {
        ($key_field:expr, $value:expr) => {{
            let key = env
                .get_static_field("android/media/MediaMetadata", $key_field, "Ljava/lang/String;")?
                .l()?;
            let value = env.new_string($value)?;
            env.call_method(
                &metadata_builder,
                "putString",
                "(Ljava/lang/String;Ljava/lang/String;)Landroid/media/MediaMetadata$Builder;",
                &[JValue::Object(&key), JValue::Object(&value)],
            )?;
        }};
    }

    put_string!("METADATA_KEY_DISPLAY_TITLE", display_title);
    put_string!("METADATA_KEY_DISPLAY_SUBTITLE", display_subtitle);

    if let Some(icon_uri) = display_icon_uri {
        put_string!("METADATA_KEY_DISPLAY_ICON_URI", "android.R.drawable.ic_media_play");
    }

    put_string!("METADATA_KEY_TITLE", title);
    put_string!("METADATA_KEY_ARTIST", artist);

    if let Some(bitmap) = art_bitmap {
        let art_key = env
            .get_static_field(
                "android/media/MediaMetadata",
                "METADATA_KEY_ART",
                "Ljava/lang/String;",
            )?
            .l()?;
        env.call_method(
            &metadata_builder,
            "putBitmap",
            "(Ljava/lang/String;Landroid/graphics/Bitmap;)Landroid/media/MediaMetadata$Builder;",
            &[JValue::Object(&art_key), JValue::Object(&bitmap)],
        )?;
    }

    let metadata =
        env.call_method(&metadata_builder, "build", "()Landroid/media/MediaMetadata;", &[])?.l()?;

    env.call_method(
        &session,
        "setMetadata",
        "(Landroid/media/MediaMetadata;)V",
        &[JValue::Object(&metadata)],
    )?;

    Ok(())
}
