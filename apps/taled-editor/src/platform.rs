use ply_engine::prelude::macroquad;

/// Open a URL in the system browser.
pub(crate) fn open_url(url: &str) {
    #[cfg(target_os = "android")]
    android_open_url(url);

    #[cfg(not(target_os = "android"))]
    {
        macroquad::prelude::info!("open_url: {url}");
    }
}

/// Android JNI: calls `activity.startActivity(new Intent(ACTION_VIEW, Uri.parse(url)))`.
#[cfg(target_os = "android")]
fn android_open_url(url: &str) {
    use std::ffi::CString;

    let Ok(url_cstr) = CString::new(url) else {
        return;
    };
    let Ok(action_cstr) = CString::new("android.intent.action.VIEW") else {
        return;
    };

    // SAFETY: All JNI pointers come from miniquad's verified Android runtime.
    unsafe {
        use macroquad::miniquad::native::android::{ACTIVITY, attach_jni_env};

        let env = attach_jni_env();
        if env.is_null() || ACTIVITY.is_null() {
            return;
        }

        let new_string = (**env).NewStringUTF.unwrap();
        let find_class = (**env).FindClass.unwrap();
        let get_static_method = (**env).GetStaticMethodID.unwrap();
        let call_static_obj = (**env).CallStaticObjectMethod.unwrap();

        // Uri uri = Uri.parse(url)
        let uri_cls = find_class(env, c"android/net/Uri".as_ptr());
        if uri_cls.is_null() {
            return;
        }
        let parse_mid = get_static_method(
            env,
            uri_cls,
            c"parse".as_ptr(),
            c"(Ljava/lang/String;)Landroid/net/Uri;".as_ptr(),
        );
        if parse_mid.is_null() {
            return;
        }
        let jurl = new_string(env, url_cstr.as_ptr());
        let uri = call_static_obj(env, uri_cls, parse_mid, jurl);
        if uri.is_null() {
            return;
        }

        // Intent intent = new Intent(ACTION_VIEW, uri)
        let jaction = new_string(env, action_cstr.as_ptr());
        let intent = macroquad::miniquad::new_object!(
            env,
            "android/content/Intent",
            "(Ljava/lang/String;Landroid/net/Uri;)V",
            jaction,
            uri
        );
        if intent.is_null() {
            return;
        }

        // activity.startActivity(intent)
        macroquad::miniquad::call_void_method!(
            env,
            ACTIVITY,
            "startActivity",
            "(Landroid/content/Intent;)V",
            intent
        );
    }
}

/// Return the app-private files directory (for logging, etc.).
#[cfg(target_os = "android")]
pub(crate) fn files_dir() -> Option<String> {
    // SAFETY: All JNI pointers come from miniquad's verified Android runtime.
    unsafe {
        use macroquad::miniquad::native::android::{ACTIVITY, attach_jni_env};

        let env = attach_jni_env();
        if env.is_null() || ACTIVITY.is_null() {
            return None;
        }

        let get_object_class = (**env).GetObjectClass.unwrap();
        let get_method = (**env).GetMethodID.unwrap();
        let call_obj = (**env).CallObjectMethod.unwrap();
        let get_utf = (**env).GetStringUTFChars.unwrap();
        let release_utf = (**env).ReleaseStringUTFChars.unwrap();

        // activity.getExternalFilesDir(null) → File
        let cls = get_object_class(env, ACTIVITY);
        let mid = get_method(
            env,
            cls,
            c"getExternalFilesDir".as_ptr(),
            c"(Ljava/lang/String;)Ljava/io/File;".as_ptr(),
        );
        if mid.is_null() {
            return None;
        }
        let file = call_obj(env, ACTIVITY, mid, std::ptr::null_mut::<()>());
        if file.is_null() {
            return None;
        }

        // File.getAbsolutePath() → String
        let fcls = get_object_class(env, file);
        let abs = get_method(
            env,
            fcls,
            c"getAbsolutePath".as_ptr(),
            c"()Ljava/lang/String;".as_ptr(),
        );
        if abs.is_null() {
            return None;
        }
        let jpath = call_obj(env, file, abs);
        if jpath.is_null() {
            return None;
        }

        let mut copy = 0u8;
        let chars = get_utf(env, jpath as _, &mut copy);
        if chars.is_null() {
            return None;
        }
        let path = std::ffi::CStr::from_ptr(chars)
            .to_string_lossy()
            .into_owned();
        release_utf(env, jpath as _, chars);
        Some(path)
    }
}

#[cfg(not(target_os = "android"))]
pub(crate) fn files_dir() -> Option<String> {
    Some(".".to_owned())
}

/// Handle the Android back button (or Escape on desktop).
pub(crate) fn is_back_pressed() -> bool {
    macroquad::prelude::is_key_pressed(macroquad::prelude::KeyCode::Back)
        || macroquad::prelude::is_key_pressed(macroquad::prelude::KeyCode::Escape)
}
