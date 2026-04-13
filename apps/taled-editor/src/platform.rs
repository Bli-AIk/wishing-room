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

/// Return the safe area top inset in physical pixels (for camera cutouts / notches).
///
/// Uses the JNI chain: `activity.getWindow().getDecorView().getRootWindowInsets()
/// .getDisplayCutout().getSafeInsetTop()`.
/// Returns 0 when the API is unavailable, the window has no insets, or no cutout exists.
#[cfg(target_os = "android")]
pub(crate) fn safe_inset_top() -> i32 {
    // SAFETY: All JNI pointers come from miniquad's verified Android runtime.
    unsafe {
        use macroquad::miniquad::native::android::{ACTIVITY, attach_jni_env};

        let env = attach_jni_env();
        if env.is_null() || ACTIVITY.is_null() {
            return 0;
        }

        let get_object_class = (**env).GetObjectClass.unwrap();
        let get_method = (**env).GetMethodID.unwrap();
        let call_obj = (**env).CallObjectMethod.unwrap();
        let call_int = (**env).CallIntMethod.unwrap();

        // Helper: look up an instance method, return null method ID on failure.
        macro_rules! method {
            ($cls:expr, $name:expr, $sig:expr) => {
                get_method(env, $cls, $name, $sig)
            };
        }

        // activity.getWindow() → Window
        let act_cls = get_object_class(env, ACTIVITY);
        let m = method!(act_cls, c"getWindow".as_ptr(), c"()Landroid/view/Window;".as_ptr());
        if m.is_null() { return 0; }
        let window = call_obj(env, ACTIVITY, m);
        if window.is_null() { return 0; }

        // window.getDecorView() → View
        let win_cls = get_object_class(env, window);
        let m = method!(win_cls, c"getDecorView".as_ptr(), c"()Landroid/view/View;".as_ptr());
        if m.is_null() { return 0; }
        let decor = call_obj(env, window, m);
        if decor.is_null() { return 0; }

        // view.getRootWindowInsets() → WindowInsets  (API 23+)
        let v_cls = get_object_class(env, decor);
        let m = method!(
            v_cls,
            c"getRootWindowInsets".as_ptr(),
            c"()Landroid/view/WindowInsets;".as_ptr()
        );
        if m.is_null() { return 0; }
        let insets = call_obj(env, decor, m);
        if insets.is_null() { return 0; }

        // insets.getDisplayCutout() → DisplayCutout  (API 28+)
        let i_cls = get_object_class(env, insets);
        let m = method!(
            i_cls,
            c"getDisplayCutout".as_ptr(),
            c"()Landroid/view/DisplayCutout;".as_ptr()
        );
        if m.is_null() { return 0; }
        let cutout = call_obj(env, insets, m);
        if cutout.is_null() { return 0; }

        // cutout.getSafeInsetTop() → int
        let c_cls = get_object_class(env, cutout);
        let m = method!(c_cls, c"getSafeInsetTop".as_ptr(), c"()I".as_ptr());
        if m.is_null() { return 0; }
        call_int(env, cutout, m)
    }
}

#[cfg(not(target_os = "android"))]
pub(crate) fn safe_inset_top() -> i32 {
    0
}

/// Handle the Android back button (or Escape on desktop).
pub(crate) fn is_back_pressed() -> bool {
    macroquad::prelude::is_key_pressed(macroquad::prelude::KeyCode::Back)
        || macroquad::prelude::is_key_pressed(macroquad::prelude::KeyCode::Escape)
}

/// Launch the Android native directory picker (SAF).
#[cfg(target_os = "android")]
pub(crate) fn launch_directory_picker(mode: &str) {
    use std::ffi::CString;

    let Ok(mode_cstr) = CString::new(mode) else {
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
        let get_object_class = (**env).GetObjectClass.unwrap();
        let get_method = (**env).GetMethodID.unwrap();
        let call_void = (**env).CallVoidMethod.unwrap();

        let cls = get_object_class(env, ACTIVITY);
        let mid = get_method(
            env,
            cls,
            c"launchDirectoryPicker".as_ptr(),
            c"(Ljava/lang/String;)V".as_ptr(),
        );
        if mid.is_null() {
            macroquad::prelude::warn!("launchDirectoryPicker method not found");
            return;
        }
        let jmode = new_string(env, mode_cstr.as_ptr());
        call_void(env, ACTIVITY, mid, jmode);
    }
}

#[cfg(not(target_os = "android"))]
pub(crate) fn launch_directory_picker(_mode: &str) {
    macroquad::prelude::info!("launch_directory_picker: not supported on this platform");
}

/// Poll for a completed directory import (returns the local filesystem path).
#[cfg(target_os = "android")]
pub(crate) fn poll_import_result() -> Option<String> {
    // SAFETY: All JNI pointers come from miniquad's verified Android runtime.
    unsafe {
        use macroquad::miniquad::native::android::{ACTIVITY, attach_jni_env};

        let env = attach_jni_env();
        if env.is_null() || ACTIVITY.is_null() {
            return None;
        }

        let get_object_class = (**env).GetObjectClass.unwrap();
        let get_static_method = (**env).GetStaticMethodID.unwrap();
        let call_static_obj = (**env).CallStaticObjectMethod.unwrap();
        let get_utf = (**env).GetStringUTFChars.unwrap();
        let release_utf = (**env).ReleaseStringUTFChars.unwrap();

        let cls = get_object_class(env, ACTIVITY);
        let mid = get_static_method(
            env,
            cls,
            c"pollImportResult".as_ptr(),
            c"()Ljava/lang/String;".as_ptr(),
        );
        if mid.is_null() {
            return None;
        }
        let result = call_static_obj(env, cls, mid);
        if result.is_null() {
            return None;
        }

        let mut copy = 0u8;
        let chars = get_utf(env, result as _, &mut copy);
        if chars.is_null() {
            return None;
        }
        let path = std::ffi::CStr::from_ptr(chars)
            .to_string_lossy()
            .into_owned();
        release_utf(env, result as _, chars);
        Some(path)
    }
}

#[cfg(not(target_os = "android"))]
pub(crate) fn poll_import_result() -> Option<String> {
    None
}
