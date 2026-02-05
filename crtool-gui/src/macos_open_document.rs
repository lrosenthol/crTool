//! Handle macOS "Open Document" Apple Event (e.g. drop file on app icon, "Open With").
//! Uses both a Cocoa NSAppleEventManager handler (so the system sees success) and a
//! Carbon fallback; queues file paths for the app to open.

use std::ffi::CStr;
use std::path::PathBuf;
use std::sync::Mutex;

static PENDING_FILES: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

/// Called from Cocoa handler (open_document_handler.m) for each file URL.
#[no_mangle]
pub extern "C" fn crtool_macos_push_pending_file(path: *const std::ffi::c_char) {
    if path.is_null() {
        return;
    }
    let Ok(s) = (unsafe { CStr::from_ptr(path).to_str() }) else {
        return;
    };
    let path_buf = PathBuf::from(s);
    if let Ok(mut pending) = PENDING_FILES.lock() {
        pending.push(path_buf);
    }
}

/// Call from app creator (after NSApp is ready) so Cocoa delivers open-document to us (e.g. Dock drop).
pub fn install_cocoa_handler() {
    extern "C" {
        fn crtool_macos_install_open_document_handler();
    }
    unsafe {
        crtool_macos_install_open_document_handler();
    }
}

/// Call from main() before eframe::run_native (Carbon fallback).
pub fn install_handler() {
    unsafe {
        extern "C" {
            fn AEInstallEventHandler(
                event_class: u32,
                event_id: u32,
                handler: *const std::ffi::c_void,
                refcon: i64,
                is_system_handler: u8,
            ) -> i32;
        }
        let handler = open_documents_handler as *const std::ffi::c_void;
        // kCoreEventClass = 'aevt', kAEOpenDocuments = 'odoc'
        let event_class = u32::from_be_bytes(*b"aevt");
        let event_id = u32::from_be_bytes(*b"odoc");
        let result = AEInstallEventHandler(event_class, event_id, handler, 0, 0);
        if result != 0 {
            eprintln!("crTool-gui: AEInstallEventHandler failed ({})", result);
        }
    }
}

/// Take one pending file path if any (from drop-on-icon / Open With). Call at startup.
pub fn take_pending_file() -> Option<PathBuf> {
    PENDING_FILES.lock().ok().and_then(|mut g| {
        if g.is_empty() {
            None
        } else {
            Some(g.remove(0))
        }
    })
}

/// Called from update() to drain any files opened while the app was running (e.g. second drop).
pub fn drain_pending_files() -> Vec<PathBuf> {
    PENDING_FILES
        .lock()
        .ok()
        .map_or_else(Vec::new, |mut g| g.drain(..).rev().collect())
}

extern "C" fn open_documents_handler(
    event: *mut std::ffi::c_void,
    _reply: *mut std::ffi::c_void,
    _refcon: i64,
) -> i32 {
    unsafe {
        extern "C" {
            fn AEGetParamDesc(
                event: *const std::ffi::c_void,
                key: u32,
                desired_type: u32,
                result: *mut std::ffi::c_void,
            ) -> i32;
            fn AECountItems(desc: *const std::ffi::c_void, result: *mut i32) -> i32;
            fn AEGetNthDesc(
                desc: *const std::ffi::c_void,
                index: i32,
                desired_type: u32,
                descriptor: *mut std::ffi::c_void,
                result: *mut std::ffi::c_void,
            ) -> i32;
            fn AEGetDescDataSize(desc: *const std::ffi::c_void, result: *mut u32) -> i32;
            fn AEGetDescData(
                desc: *const std::ffi::c_void,
                offset: usize,
                ptr: *mut std::ffi::c_void,
                size: u32,
            ) -> i32;
            fn AEDisposeDesc(desc: *mut std::ffi::c_void) -> i32;
        }
        // AEDesc is 8 bytes on 64-bit: type (4) + dataHandle (4)
        #[repr(C)]
        struct AEDesc {
            descriptor_type: u32,
            data_handle: u32,
        }
        const KEY_DIRECT_OBJECT: u32 = u32::from_be_bytes(*b"----");
        const TYPE_AE_LIST: u32 = u32::from_be_bytes(*b"list");
        const TYPE_FILE_URL: u32 = u32::from_be_bytes(*b"furl");

        let mut list_desc = std::mem::zeroed::<AEDesc>();
        let mut result = AEGetParamDesc(
            event,
            KEY_DIRECT_OBJECT,
            TYPE_AE_LIST,
            &mut list_desc as *mut _ as *mut _,
        );
        if result != 0 {
            return result;
        }

        let mut count = 0i32;
        result = AECountItems(&list_desc as *const _ as *const _, &mut count);
        if result != 0 {
            let _ = AEDisposeDesc(&mut list_desc as *mut _ as *mut _);
            return result;
        }

        let mut paths = Vec::new();
        for i in 1..=count {
            let mut url_desc = std::mem::zeroed::<AEDesc>();
            let mut junk = std::mem::zeroed::<AEDesc>();
            result = AEGetNthDesc(
                &list_desc as *const _ as *const _,
                i,
                TYPE_FILE_URL,
                &mut url_desc as *mut _ as *mut _,
                &mut junk as *mut _ as *mut _,
            );
            if result != 0 {
                continue;
            }

            let mut size = 0u32;
            result = AEGetDescDataSize(&url_desc as *const _ as *const _, &mut size);
            if result != 0 || size == 0 || size > 65536 {
                let _ = AEDisposeDesc(&mut url_desc as *mut _ as *mut _);
                continue;
            }

            let mut buf = vec![0u8; size as usize];
            result = AEGetDescData(
                &url_desc as *const _ as *const _,
                0,
                buf.as_mut_ptr() as *mut _,
                size,
            );
            let _ = AEDisposeDesc(&mut url_desc as *mut _ as *mut _);
            if result != 0 {
                continue;
            }

            // typeFileURL is UTF-8 file URL, e.g. "file:///path/to/file"
            if let Ok(s) = std::str::from_utf8(&buf) {
                let s = s.trim_matches('\0');
                if let Some(path_str) = s.strip_prefix("file://") {
                    let path_str = path_str.trim_start_matches('/');
                    let decoded = urlencoding::decode(path_str)
                        .unwrap_or(std::borrow::Cow::Borrowed(path_str));
                    paths.push(PathBuf::from(decoded.as_ref()));
                } else {
                    paths.push(PathBuf::from(s));
                }
            }
        }

        let _ = AEDisposeDesc(&mut list_desc as *mut _ as *mut _);

        if let Ok(mut pending) = PENDING_FILES.lock() {
            pending.extend(paths);
        }
        0i32 // noErr
    }
}
