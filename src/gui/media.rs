use jni::objects::GlobalRef;
use jni::objects::JClass;
use jni::objects::JObject;
use jni::objects::JValue;
use jni::objects::JString;
use jni::sys::jint;
use jni::AttachGuard;
use jni::JNIEnv;
use jni::JavaVM;
use log::info;
use jni::sys::jobject;

use once_cell::sync::Lazy;
use std::sync::Mutex;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub static MEDIA_MSG_TX: Lazy<Mutex<Option<UnboundedSender<MediaMsg>>>> =
    Lazy::new(|| Mutex::new(None));

pub static NOTIFICATION: Lazy<Mutex<Option<GlobalRef>>> =
    Lazy::new(|| Mutex::new(None));

#[derive(Debug)]
pub enum MediaMsg {
    Play,
    Pause,
    Next,
    Previous,
    SeekTo(i64),
}

fn send_media_msg(msg: MediaMsg) {
    if let Some(tx) = MEDIA_MSG_TX.lock().unwrap().as_ref() {
        info!("{msg:?}");
        tx.send(msg).unwrap();
    }
}


#[no_mangle]
pub extern "C" fn Java_dev_dioxus_main_KeepAliveService_startRustBackground(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
) {
    std::thread::spawn(|| {
        let mut i = 0;
        loop {
            log::info!("Rust background loop tick {i}");
            i += 1;
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
}

pub fn update_media_notification(
    title: &str,
    artist: &str,
    track_len: i64,
    track_progress: i64,
    playing: bool,
    artwork_bytes: Option<Vec<u8>>,
) -> jni::errors::Result<()> {
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.unwrap();
    let mut env = vm.attach_current_thread().unwrap();
    let class_ctx = env.find_class("android/content/Context").unwrap();
    let context = unsafe { JObject::from_raw(ctx.context().cast()) };

    let class_loader = env
        .call_method(&context, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])
        .unwrap()
        .l()
        .unwrap();

    let binding = env.new_string("dev.dioxus.main.KeepAliveService").unwrap();

    let service_class = env
        .call_method(
            &class_loader,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[JValue::Object(&binding)],
        )
        .unwrap()
        .l()
        .unwrap();

    let service_class = JClass::from(service_class);

    let instance_obj = env
        .call_static_method(
            service_class,
            "getServiceInstance", 
            "()Ldev/dioxus/main/KeepAliveService;", 
            &[],
        ).unwrap()
        .l().unwrap(); 

    if instance_obj.is_null() {
        panic!("No KeepAliveService Instance");
    }

    let j_title: JString = env.new_string(title)?.into();
    let j_artist: JString = env.new_string(artist)?.into();

    let j_bytes = if let Some(bytes) = artwork_bytes {
        let array = env.byte_array_from_slice(&bytes)?;
        JObject::from(array)
    } else {
        JObject::null()
    };

    env.call_method(
        instance_obj,
        "updateMediaNotification",
        "(Ljava/lang/String;Ljava/lang/String;JJZ[B)V",
        &[
            JValue::from(&j_title),
            JValue::from(&j_artist),
            JValue::Long(track_len),
            JValue::Long(track_progress),
            JValue::from(playing),
            JValue::from(&j_bytes),
        ],
    ).unwrap();

    Ok(())
}

#[no_mangle]
pub extern "system" fn Java_dev_dioxus_main_MediaCallbackKt_nativeOnPlay(
    _env: JNIEnv,
    _class: JClass,
) {
    log::info!("Rust received Play");
    send_media_msg(MediaMsg::Play)
}

#[no_mangle]
pub extern "system" fn Java_dev_dioxus_main_MediaCallbackKt_nativeOnPause(
    _env: JNIEnv,
    _class: JClass,
) {
    log::info!("Rust received Pause");
    send_media_msg(MediaMsg::Pause)
}

#[no_mangle]
pub extern "system" fn Java_dev_dioxus_main_MediaCallbackKt_nativeOnNext(
    _env: JNIEnv,
    _class: JClass,
) {
    log::info!("Rust received Next");
    send_media_msg(MediaMsg::Next)
}

#[no_mangle]
pub extern "system" fn Java_dev_dioxus_main_MediaCallbackKt_nativeOnPrevious(
    _env: JNIEnv,
    _class: JClass,
) {
    log::info!("Rust received Previous");
    send_media_msg(MediaMsg::Previous)
}

#[no_mangle]
pub extern "system" fn Java_dev_dioxus_main_MediaCallbackKt_nativeOnSeekTo(
    _env: JNIEnv,
    _class: JClass,
    pos: jint,
) {
    log::info!("Rust received Seek To {:?}", pos);
    send_media_msg(MediaMsg::SeekTo(pos.into()));
}
use std::thread;

#[no_mangle]
pub extern "system" fn Java_dev_dioxus_main_MediaCallbackKt_getNotification<'a>(
    mut env: JNIEnv,
    _class: JClass,
    _context: JObject,
) -> jobject {
    info!("notification accessed");
    let lock = NOTIFICATION.lock().unwrap();

    if let Some(global_ref) = &*lock {
        info!("{global_ref:?}");
        **global_ref.as_obj()
    } else {
        info!("failed to get notification");
        env.throw_new("java/lang/IllegalStateException", "Notification not initialized").unwrap();
        *JObject::null()
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_dioxus_main_KeepAliveService_nativeOnAudioFocusLost(
    _env: JNIEnv,
    _class: JClass,
    focus_change: jint,
) {
    info!("Audio focus lost: {focus_change}");
    send_media_msg(MediaMsg::Pause)
}

#[no_mangle]
pub extern "system" fn Java_dev_dioxus_main_KeepAliveService_nativeOnAudioFocusGained(
    _env: JNIEnv,
    _class: JClass,
) {
    // TODO: check if previously playing
    info!("Audio focus gained");
}


pub fn start_audio_service(env: &mut JNIEnv, context: &JObject) -> jni::errors::Result<()> {
    let intent_class = env.find_class("android/content/Intent")?;
    let intent = env.new_object(intent_class, "()V", &[])?;

    let class_loader = env
        .call_method(&context, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])
        .unwrap()
        .l()
        .unwrap();

    let binding = env.new_string("dev.dioxus.main.MediaCallback").unwrap();

    let callback_class = env
        .call_method(
            &class_loader,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[JValue::Object(&binding)],
        )
        .unwrap()
        .l()
        .unwrap();

    let callback_class = JClass::from(callback_class);

    let callback = env.new_object(callback_class, "()V", &[]).unwrap();

    Ok(())
}

// pub fn show_media_notification(
//     env: &mut JNIEnv,
//     context: &JObject,
//     session: &JObject,
//     bitmap: Option<&JObject>,
//     creating: bool,
// ) -> jni::errors::Result<()> {
//     let channel_id = env.new_string("media_channel")?;
//     let channel_name = env.new_string("Media Controls")?;
//     let importance = 3;
//
//     let channel_class = env.find_class("android/app/NotificationChannel")?;
//     let channel = env.new_object(
//         channel_class,
//         "(Ljava/lang/String;Ljava/lang/CharSequence;I)V",
//         &[
//             JValue::Object(&channel_id),
//             JValue::Object(&JObject::from(channel_name)),
//             JValue::Int(importance),
//         ],
//     )?;
//
//     let notif_service_str = env
//         .get_static_field(
//             "android/content/Context",
//             "NOTIFICATION_SERVICE",
//             "Ljava/lang/String;",
//         )?
//         .l()?;
//
//     let notification_manager = env
//         .call_method(
//             &context,
//             "getSystemService",
//             "(Ljava/lang/String;)Ljava/lang/Object;",
//             &[JValue::Object(&notif_service_str)],
//         )?
//         .l()?;
//
//     env.call_method(
//         &notification_manager,
//         "createNotificationChannel",
//         "(Landroid/app/NotificationChannel;)V",
//         &[JValue::Object(&channel)],
//     )?;
//
//     let intent_class = env.find_class("android/content/Intent")?;
//     let action_str = env.new_string("ACTION_PLAY")?;
//     let play_intent = env.new_object(
//         intent_class,
//         "(Ljava/lang/String;)V",
//         &[JValue::Object(&action_str)],
//     )?;
//
//     let pi_class = env.find_class("android/app/PendingIntent")?;
//     let pending_intent = env
//         .call_static_method(
//             pi_class,
//             "getBroadcast",
//             "(Landroid/content/Context;ILandroid/content/Intent;I)Landroid/app/PendingIntent;",
//             &[
//                 JValue::Object(&context),
//                 JValue::Int(0),
//                 JValue::Object(&play_intent),
//                 JValue::Int(0x04000000), // FLAG_IMMUTABLE
//             ],
//         )?
//         .l()?;
//
//     let builder_class = env.find_class("android/app/Notification$Builder")?;
//     let builder = env.new_object(
//         builder_class,
//         "(Landroid/content/Context;Ljava/lang/String;)V",
//         &[JValue::Object(&context), JValue::Object(&channel_id)],
//     )?;
//
//     let title = env.new_string("Track Title")?;
//     let artist = env.new_string("Artist Name")?;
//
//     env.call_method(
//         &builder,
//         "setContentTitle",
//         "(Ljava/lang/CharSequence;)Landroid/app/Notification$Builder;",
//         &[JValue::Object(&JObject::from(title))],
//     )?;
//     env.call_method(
//         &builder,
//         "setContentText",
//         "(Ljava/lang/CharSequence;)Landroid/app/Notification$Builder;",
//         &[JValue::Object(&JObject::from(artist))],
//     )?;
//
//     let icon_name = env.new_string("ic_notification")?; // your drawable name
//     let def_type = env.new_string("drawable")?;
//     let def_package = env.new_string("com.example.Trackfish")?; // your actual package
//
//     let resources = env
//         .call_method(
//             &context,
//             "getResources",
//             "()Landroid/content/res/Resources;",
//             &[],
//         )?
//         .l()?;
//
//     // int id = getResources().getIdentifier("ic_notification", "drawable", "com.example.Trackfish");
//     let icon_id = env
//         .call_method(
//             &resources,
//             "getIdentifier",
//             "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)I",
//             &[
//                 JValue::Object(&JObject::from(icon_name)),
//                 JValue::Object(&JObject::from(def_type)),
//                 JValue::Object(&JObject::from(def_package)),
//             ],
//         )?
//         .i()?;
//
//     env.call_method(
//         &builder,
//         "setSmallIcon",
//         "(I)Landroid/app/Notification$Builder;",
//         &[JValue::Int(icon_id)],
//     )?;
//
//     if let Some(bitmap) = bitmap {
//         env.call_method(
//             &builder,
//             "setLargeIcon",
//             "(Landroid/graphics/Bitmap;)Landroid/app/Notification$Builder;",
//             &[JValue::Object(&bitmap)],
//         ).unwrap();
//     }
//
//     let action_class = env.find_class("android/app/Notification$Action")?;
//     let label = env.new_string("Play")?;
//     let play_action = env.new_object(
//         action_class,
//         "(ILjava/lang/CharSequence;Landroid/app/PendingIntent;)V",
//         &[
//             JValue::Int(17301540), // icon
//             JValue::Object(&JObject::from(label)),
//             JValue::Object(&pending_intent),
//         ],
//     ).unwrap();
//
//     let style_class = env.find_class("android/app/Notification$MediaStyle")?;
//     let media_style = env.new_object(style_class, "()V", &[])?;
//     let session_token = env
//         .call_method(
//             session,
//             "getSessionToken",
//             "()Landroid/media/session/MediaSession$Token;",
//             &[],
//         )?
//         .l().unwrap();
//
//     env.call_method(
//         &media_style,
//         "setMediaSession",
//         "(Landroid/media/session/MediaSession$Token;)Landroid/app/Notification$MediaStyle;",
//         &[JValue::Object(&session_token)],
//     ).unwrap();
//
//     env.call_method(
//         &builder,
//         "setStyle",
//         "(Landroid/app/Notification$Style;)Landroid/app/Notification$Builder;",
//         &[JValue::Object(&media_style)],
//     ).unwrap();
//
//     let notification = env
//         .call_method(builder, "build", "()Landroid/app/Notification;", &[])?
//         .l()?;
//
//     *NOTIFICATION.lock().unwrap() = Some(env.new_global_ref(&notification)?);
//     let notification_global = NOTIFICATION.lock().unwrap();
//
//     
//     if let Some(ref global_ref) = *notification_global {
//         if let Err(e) = env.call_method(
//             &notification_manager,
//             "notify",
//             "(ILandroid/app/Notification;)V",
//             &[JValue::Int(1), JValue::Object(global_ref.as_obj())],
//         ) {
//             eprintln!("notify() failed: {:?}", e);
//             if let Ok(exception) = env.exception_occurred() {
//                 if !exception.is_null() {
//                     env.exception_describe().unwrap();
//                     env.exception_clear().unwrap();
//                 }
//             }
//         }
//     }
//
//
//     if creating {
//         start_audio_service(env, context);
//     }
//
//     Ok(())
// }
//
// fn try_to_get_audio_focus(env: &mut JNIEnv, context: &JObject) -> jni::errors::Result<jint> {
//     let audio_service_field = env
//         .get_static_field(
//             "android/content/Context",
//             "AUDIO_SERVICE",
//             "Ljava/lang/String;",
//         )?
//         .l()?;
//
//     let audio_manager_obj = env
//         .call_method(
//             &context,
//             "getSystemService",
//             "(Ljava/lang/String;)Ljava/lang/Object;",
//             &[JValue::Object(&audio_service_field)],
//         )?
//         .l()?;
//
//     let audio_manager = JObject::from(audio_manager_obj);
//
//     let stream_music = 3; // AudioManager.STREAM_MUSIC
//     let focus_gain = 1; // AudioManager.AUDIOFOCUS_GAIN
//
//     let result = env
//         .call_method(
//             &audio_manager,
//             "requestAudioFocus",
//             "(Landroid/media/AudioManager$OnAudioFocusChangeListener;II)I",
//             &[
//                 JValue::Object(&JObject::null()), // no listener
//                 JValue::Int(stream_music),
//                 JValue::Int(focus_gain),
//             ],
//         )?
//         .i()?;
//
//     Ok(result)
// }
//
// fn update_playback_state(
//     env: &mut JNIEnv,
//     media_session: &JObject,
//     m_state: i32,
//     position: i64,
// ) -> jni::errors::Result<()> {
//     let builder_class = env.find_class("android/media/session/PlaybackState$Builder")?;
//     let builder = env.new_object(builder_class, "()V", &[])?;
//
//     let mut actions = android_media_constants::ACTION_PLAY_PAUSE
//         | android_media_constants::ACTION_PLAY_FROM_MEDIA_ID
//         | android_media_constants::ACTION_PLAY_FROM_SEARCH;
//
//     actions |= android_media_constants::ACTION_PAUSE;
//     actions |= android_media_constants::ACTION_PLAY;
//
//     actions |= android_media_constants::ACTION_SKIP_TO_PREVIOUS;
//     actions |= android_media_constants::ACTION_SKIP_TO_NEXT;
//
//     env.call_method(
//         &builder,
//         "setActions",
//         "(J)Landroid/media/session/PlaybackState$Builder;",
//         &[JValue::Long(actions as i64)],
//     )?;
//
//     let playback_speed = 1.0f32;
//
//     env.call_method(
//         &builder,
//         "setState",
//         "(IJF)Landroid/media/session/PlaybackState$Builder;",
//         &[
//             JValue::Int(m_state),
//             JValue::Long(position),
//             JValue::Float(playback_speed),
//         ],
//     )?;
//
//     let playback_state = env
//         .call_method(
//             &builder,
//             "build",
//             "()Landroid/media/session/PlaybackState;",
//             &[],
//         )?
//         .l()?;
//
//     env.call_method(
//         &media_session,
//         "setPlaybackState",
//         "(Landroid/media/session/PlaybackState;)V",
//         &[JValue::Object(&playback_state)],
//     )?;
//
//     Ok(())
// }
//
// mod android_media_constants {
//     pub const STATE_NONE: i32 = 0;
//     pub const STATE_PLAYING: i32 = 2;
//     pub const STATE_PAUSED: i32 = 3;
//
//     pub const ACTION_PLAY: i64 = 1 << 2;
//     pub const ACTION_PAUSE: i64 = 1 << 1;
//     pub const ACTION_PLAY_PAUSE: i64 = 1 << 0;
//     pub const ACTION_SKIP_TO_PREVIOUS: i64 = 1 << 7;
//     pub const ACTION_SKIP_TO_NEXT: i64 = 1 << 8;
//     pub const ACTION_PLAY_FROM_MEDIA_ID: i64 = 1 << 4;
//     pub const ACTION_PLAY_FROM_SEARCH: i64 = 1 << 5;
// }
//
// pub fn update_metadata<'a>(
//     env: &mut JNIEnv<'a>,
//     session: &JObject<'a>,
//     display_icon_uri: Option<&str>, // Optional icon URI
//     title: &str,
//     artist: &str,
//     length: i64,
//     art_bitmap: Option<&JObject<'a>>, // Optional Bitmap
// ) -> jni::errors::Result<()> {
//     let metadata_builder_class = env.find_class("android/media/MediaMetadata$Builder")?;
//     let metadata_builder = env.new_object(metadata_builder_class, "()V", &[])?;
//
//     macro_rules! put_string {
//         ($key_field:expr, $value:expr) => {{
//             let key = env
//                 .get_static_field(
//                     "android/media/MediaMetadata",
//                     $key_field,
//                     "Ljava/lang/String;",
//                 )?
//                 .l()?;
//             let value = env.new_string($value)?;
//             env.call_method(
//                 &metadata_builder,
//                 "putString",
//                 "(Ljava/lang/String;Ljava/lang/String;)Landroid/media/MediaMetadata$Builder;",
//                 &[JValue::Object(&key), JValue::Object(&value)],
//             )?;
//         }};
//     }
//
//     put_string!("METADATA_KEY_DISPLAY_TITLE", title);
//     put_string!("METADATA_KEY_DISPLAY_SUBTITLE", artist);
//
//     if let Some(icon_uri) = display_icon_uri {
//         put_string!(
//             "METADATA_KEY_DISPLAY_ICON_URI",
//             "android.R.drawable.ic_media_play"
//         );
//     }
//
//     put_string!("METADATA_KEY_TITLE", title);
//     put_string!("METADATA_KEY_ARTIST", artist);
//
//     env.call_method(
//         &metadata_builder,
//         "putLong",
//         "(Ljava/lang/String;J)Landroid/media/MediaMetadata$Builder;",
//         &[
//             JValue::Object(&env.new_string("android.media.metadata.DURATION")?.into()),
//             JValue::Long(length), // 3 minutes
//         ],
//     )?;
//
//     if let Some(bitmap) = art_bitmap {
//         let art_key = env
//             .get_static_field(
//                 "android/media/MediaMetadata",
//                 "METADATA_KEY_ART",
//                 "Ljava/lang/String;",
//             )?
//             .l()?;
//         env.call_method(
//             &metadata_builder,
//             "putBitmap",
//             "(Ljava/lang/String;Landroid/graphics/Bitmap;)Landroid/media/MediaMetadata$Builder;",
//             &[JValue::Object(&art_key), JValue::Object(&bitmap)],
//         )?;
//     }
//
//     let metadata = env
//         .call_method(
//             &metadata_builder,
//             "build",
//             "()Landroid/media/MediaMetadata;",
//             &[],
//         )?
//         .l()?;
//
//     env.call_method(
//         &session,
//         "setMetadata",
//         "(Landroid/media/MediaMetadata;)V",
//         &[JValue::Object(&metadata)],
//     )?;
//
//     Ok(())
// }
