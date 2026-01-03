use jni::objects::GlobalRef;
use jni::objects::JClass;
use jni::objects::JObject;
use jni::objects::JString;
use jni::objects::JValue;
use jni::sys::jint;
use jni::sys::jobject;
use jni::JNIEnv;
use log::info;

use once_cell::sync::Lazy;
use std::sync::mpsc::Sender;
use std::sync::Mutex;

pub static MEDIA_MSG_TX: Lazy<Mutex<Option<Sender<MediaMsg>>>> = Lazy::new(|| Mutex::new(None));
pub static NOTIFICATION: Lazy<Mutex<Option<GlobalRef>>> = Lazy::new(|| Mutex::new(None));

use crate::gui::start_controller_thread;

#[derive(Debug)]
pub enum MediaMsg {
    Play,
    Pause,
    Next,
    Previous,
    SeekTo(i64),
}

// Send message from notification callback to control
fn send_media_msg(msg: MediaMsg) {
    if let Some(tx) = MEDIA_MSG_TX.lock().unwrap().as_ref() {
        info!("{msg:?}");
        tx.send(msg).unwrap();
    }
}

#[no_mangle]
pub extern "system" fn
Java_dev_dioxus_main_WryActivity_onFolderPicked(
    mut env: JNIEnv,
    _class: JObject,
    uri: JObject,
) {
    let uri: String = env
        .get_string((&uri).into())
        .expect("Invalid URI")
        .into();

    // Store / send into your Rust app
    info!("Picked folder URI: {uri}");
}

pub fn open_folder_picker() {
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.unwrap();
    let mut env = vm.attach_current_thread().unwrap();
    let context = unsafe { JObject::from_raw(ctx.context().cast()) };

    let class_loader = env
        .call_method(&context, "getClassLoader", "()Ljava/lang/ClassLoader;", &[]).unwrap()
        .l().unwrap();

    let binding = env.new_string("dev.dioxus.main.WryActivity").unwrap();

    let service_class = env
        .call_method(
            &class_loader,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[JValue::Object(&binding)],
        ).unwrap()
        .l().unwrap();

    let service_class = JClass::from(service_class);

    let instance_obj = env
        .call_static_method(
            service_class,
            "getServiceInstance",
            "()Ldev/dioxus/main/WryActivity;",
            &[],
        )
        .unwrap()
        .l()
        .unwrap();

    if instance_obj.is_null() {
        panic!("No WryActivity Instance");
    }

    env.call_method(
        instance_obj,
        "openFolderPicker",
        "()V",
        &[],
    ).unwrap();
}

// Runs background management through foreground service ran function
#[no_mangle]
pub extern "C" fn Java_dev_dioxus_main_KeepAliveService_startRustBackground(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
) {
    start_controller_thread();
}

/// Updates media notification through foreground service connected function
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
    let context = unsafe { JObject::from_raw(ctx.context().cast()) };

    let class_loader = env
        .call_method(&context, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])?
        .l()?;

    let binding = env.new_string("dev.dioxus.main.KeepAliveService")?;

    let service_class = env
        .call_method(
            &class_loader,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[JValue::Object(&binding)],
        )?
        .l()?;

    let service_class = JClass::from(service_class);

    let instance_obj = env
        .call_static_method(
            service_class,
            "getServiceInstance",
            "()Ldev/dioxus/main/KeepAliveService;",
            &[],
        )
        .unwrap()
        .l()
        .unwrap();

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
    )?;

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
        env.throw_new(
            "java/lang/IllegalStateException",
            "Notification not initialized",
        )
        .unwrap();
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
