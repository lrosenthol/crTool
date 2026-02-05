//! macOS security-scoped resource access for files opened via the system
//! (e.g. drop on app icon, "Open With"). Must call startAccessingSecurityScopedResource
//! before reading and stopAccessingSecurityScopedResource when done.

use std::path::Path;

#[cfg(target_os = "macos")]
mod mac {
    use super::*;
    use std::ffi::CString;

    use objc::{class, msg_send, sel, sel_impl};

    /// Run `f` while holding a macOS security-scoped access token for `path`.
    ///
    /// Returns Ok(f()) if access was granted, otherwise an Err explaining the failure.
    pub fn with_security_scoped_access<T, F>(path: &Path, f: F) -> Result<T, String>
    where
        F: FnOnce() -> Result<T, String>,
    {
        let path_str = path
            .to_str()
            .ok_or_else(|| "Path is not valid UTF-8 (cannot build NSURL)".to_string())?;

        let c_path = CString::new(path_str).map_err(|_| "Path contains null byte".to_string())?;

        unsafe {
            let cls = class!(NSString);
            let ns_string: *mut objc::runtime::Object =
                msg_send![cls, stringWithUTF8String: c_path.as_ptr()];

            let cls = class!(NSURL);
            let url: *mut objc::runtime::Object = msg_send![cls, fileURLWithPath: ns_string];

            if url.is_null() {
                return Err("Failed to create NSURL from path".into());
            }

            let ok: bool = msg_send![url, startAccessingSecurityScopedResource];
            if !ok {
                return Err(
                    "macOS did not grant security-scoped access. Check sandbox entitlements."
                        .into(),
                );
            }

            // Keep URL alive and ensure we always stop access, even if f() panics or errors.
            let _: () = msg_send![url, retain];
            struct Stopper(*mut objc::runtime::Object);
            impl Drop for Stopper {
                fn drop(&mut self) {
                    unsafe {
                        let _: () = msg_send![self.0, stopAccessingSecurityScopedResource];
                        let _: () = msg_send![self.0, release];
                    }
                }
            }
            let _stopper = Stopper(url);

            f()
        }
    }
}

#[cfg(target_os = "macos")]
pub use mac::with_security_scoped_access;

#[cfg(not(target_os = "macos"))]
/// No-op on non-macOS; runs `f` directly.
pub fn with_security_scoped_access<T, F>(_path: &Path, f: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String>,
{
    f()
}
