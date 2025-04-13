use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::jint;
use jni::objects::JValue;

pub fn set_media_context() { 
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.unwrap();
    let mut env = vm.attach_current_thread().unwrap();
    let class_ctx = env.find_class("android/content/Context").unwrap();

    let media_session_class = env.find_class("android/media/session/MediaSession").unwrap();
    let tag = env.new_string("MyMediaSession").expect("Failed to create Java string");

    let context = unsafe { jni::objects::JObject::from_raw(ctx.context().cast()) };

    let media_session = env.new_object(
        media_session_class,
        "(Landroid/content/Context;Ljava/lang/String;)V",
        &[JValue::Object(&context), JValue::Object(&tag.into())],
    ).expect("Failed to create MediaSession object");

    info!("hi!");

    let flag_handles_media_buttons = 1; // MediaSession.FLAG_HANDLES_MEDIA_BUTTONS
    let flag_handles_transport_controls = 2; // MediaSession.FLAG_HANDLES_TRANSPORT_CONTROLS
    let flags = flag_handles_media_buttons | flag_handles_transport_controls;
    env.call_method(
        &media_session,
        "setFlags",
        "(I)V",
        &[JValue::Int(flags)],
    ).unwrap();

    // Set the session active
    env.call_method(
        &media_session,
        "setActive",
        "(Z)V",
        &[JValue::Bool(1)],
    ).unwrap();

    info!("hi!");
}
