#![allow(dead_code)]

use std::ffi::{c_void, CStr, CString};
use std::os::raw::{c_char, c_int, c_long, c_uchar, c_uint, c_ulong};

type Display = c_void;
type Window = c_ulong;
type Atom = c_ulong;
type XserverRegion = c_ulong;

const SHAPE_INPUT: c_int = 2;
const CLIENT_MESSAGE: c_int = 33;
const PROP_MODE_REPLACE: c_int = 0;
const XA_ATOM: Atom = 4;
const SUBSTRUCTURE_REDIRECT_MASK: c_long = 1 << 20;
const SUBSTRUCTURE_NOTIFY_MASK: c_long = 1 << 19;

type FnXOpenDisplay = unsafe extern "C" fn(*const c_char) -> *mut Display;
type FnXCloseDisplay = unsafe extern "C" fn(*mut Display) -> c_int;
type FnXFlush = unsafe extern "C" fn(*mut Display) -> c_int;
type FnXDefaultRootWindow = unsafe extern "C" fn(*mut Display) -> Window;
type FnXInternAtom = unsafe extern "C" fn(*mut Display, *const c_char, c_int) -> Atom;
type FnXChangeProperty = unsafe extern "C" fn(
    *mut Display,
    Window,
    Atom,
    Atom,
    c_int,
    c_int,
    *const c_uchar,
    c_int,
) -> c_int;
type FnXRaiseWindow = unsafe extern "C" fn(*mut Display, Window) -> c_int;
type FnXQueryTree = unsafe extern "C" fn(
    *mut Display,
    Window,
    *mut Window,
    *mut Window,
    *mut *mut Window,
    *mut c_uint,
) -> c_int;
type FnXFetchName = unsafe extern "C" fn(*mut Display, Window, *mut *mut c_char) -> c_int;
type FnXFree = unsafe extern "C" fn(*mut c_void) -> c_int;
type FnXSendEvent = unsafe extern "C" fn(*mut Display, Window, c_int, c_long, *mut c_void) -> c_int;
type FnXMoveWindow = unsafe extern "C" fn(*mut Display, Window, c_int, c_int) -> c_int;

type FnXFixesCreateRegion = unsafe extern "C" fn(*mut Display, *mut c_void, c_int) -> XserverRegion;
type FnXFixesSetWindowShapeRegion = unsafe extern "C" fn(*mut Display, Window, c_int, c_int, c_int, XserverRegion);
type FnXFixesDestroyRegion = unsafe extern "C" fn(*mut Display, XserverRegion);

#[repr(C)]
struct XClientMessageEvent {
    type_: c_int,
    serial: c_ulong,
    send_event: c_int,
    display: *mut Display,
    window: Window,
    message_type: Atom,
    format: c_int,
    data: [c_long; 5],
}

/// Recursively traverses X11 windows to find a window matching a specific title substring.
unsafe fn find_window_by_title(
    dpy: *mut Display,
    current: Window,
    target: &str,
    x_query_tree: &libloading::Symbol<FnXQueryTree>,
    x_fetch_name: &libloading::Symbol<FnXFetchName>,
    x_free: &libloading::Symbol<FnXFree>,
) -> Option<Window> {
    let mut name_ptr: *mut c_char = std::ptr::null_mut();
    if (x_fetch_name)(dpy, current, &mut name_ptr) != 0 && !name_ptr.is_null() {
        let name = CStr::from_ptr(name_ptr).to_string_lossy();
        let matches = name.contains(target);
        (x_free)(name_ptr as *mut c_void);
        if matches {
            return Some(current);
        }
    }

    let mut root: Window = 0;
    let mut parent: Window = 0;
    let mut children: *mut Window = std::ptr::null_mut();
    let mut nchildren: c_uint = 0;

    if (x_query_tree)(dpy, current, &mut root, &mut parent, &mut children, &mut nchildren) != 0
        && !children.is_null()
    {
        let slice = std::slice::from_raw_parts(children, nchildren as usize);
        let mut found = None;
        for &child in slice {
            if let Some(w) = find_window_by_title(dpy, child, target, x_query_tree, x_fetch_name, x_free) {
                found = Some(w);
                break;
            }
        }
        (x_free)(children as *mut c_void);
        found
    } else {
        None
    }
}

/// Enforces the X11 window to stay above all fullscreen applications, sets the Dock type,
/// optionally repositions it, and applies XFixes click-through region.
pub fn enforce_always_on_top(target_title: &str, click_through: bool, target_pos: Option<(i32, i32)>) -> bool {
    unsafe {
        // Load libX11.so
        let lib_x11 = match libloading::Library::new("libX11.so.6")
            .or_else(|_| libloading::Library::new("libX11.so"))
        {
            Ok(l) => l,
            Err(e) => {
                log::debug!("X11 library not available: {:?}", e);
                return false;
            }
        };

type FnXSetErrorHandler = unsafe extern "C" fn(Option<unsafe extern "C" fn(*mut Display, *mut c_void) -> c_int>) -> Option<unsafe extern "C" fn(*mut Display, *mut c_void) -> c_int>;

unsafe extern "C" fn ignore_x_error(_dpy: *mut Display, _err: *mut c_void) -> c_int {
    0
}

        if let Ok(set_handler) = lib_x11.get::<FnXSetErrorHandler>(b"XSetErrorHandler\0") {
            (set_handler)(Some(ignore_x_error));
        }

        let x_open_display: libloading::Symbol<FnXOpenDisplay> = match lib_x11.get(b"XOpenDisplay\0") {
            Ok(s) => s,
            Err(_) => return false,
        };
        let x_close_display: libloading::Symbol<FnXCloseDisplay> = match lib_x11.get(b"XCloseDisplay\0") {
            Ok(s) => s,
            Err(_) => return false,
        };
        let x_flush: libloading::Symbol<FnXFlush> = match lib_x11.get(b"XFlush\0") {
            Ok(s) => s,
            Err(_) => return false,
        };
        let x_default_root: libloading::Symbol<FnXDefaultRootWindow> = match lib_x11.get(b"XDefaultRootWindow\0") {
            Ok(s) => s,
            Err(_) => return false,
        };
        let x_intern_atom: libloading::Symbol<FnXInternAtom> = match lib_x11.get(b"XInternAtom\0") {
            Ok(s) => s,
            Err(_) => return false,
        };
        let x_change_property: libloading::Symbol<FnXChangeProperty> = match lib_x11.get(b"XChangeProperty\0") {
            Ok(s) => s,
            Err(_) => return false,
        };
        let x_raise_window: libloading::Symbol<FnXRaiseWindow> = match lib_x11.get(b"XRaiseWindow\0") {
            Ok(s) => s,
            Err(_) => return false,
        };
        let x_query_tree: libloading::Symbol<FnXQueryTree> = match lib_x11.get(b"XQueryTree\0") {
            Ok(s) => s,
            Err(_) => return false,
        };
        let x_fetch_name: libloading::Symbol<FnXFetchName> = match lib_x11.get(b"XFetchName\0") {
            Ok(s) => s,
            Err(_) => return false,
        };
        let x_free: libloading::Symbol<FnXFree> = match lib_x11.get(b"XFree\0") {
            Ok(s) => s,
            Err(_) => return false,
        };
        let x_send_event: libloading::Symbol<FnXSendEvent> = match lib_x11.get(b"XSendEvent\0") {
            Ok(s) => s,
            Err(_) => return false,
        };

        let dpy = (x_open_display)(std::ptr::null());
        if dpy.is_null() {
            return false;
        }

        let root = (x_default_root)(dpy);
        let win = match find_window_by_title(dpy, root, target_title, &x_query_tree, &x_fetch_name, &x_free) {
            Some(w) => w,
            None => {
                (x_close_display)(dpy);
                return false;
            }
        };

        // If target position specified, move the window
        if let Some((x, y)) = target_pos {
            if let Ok(x_move_window) = lib_x11.get::<FnXMoveWindow>(b"XMoveWindow\0") {
                (x_move_window)(dpy, win, x, y);
            }
        }

        // Atoms for Keep-Above & Window Type
        let net_wm_state = (x_intern_atom)(dpy, CString::new("_NET_WM_STATE").unwrap().as_ptr(), 0);
        let net_wm_state_above = (x_intern_atom)(dpy, CString::new("_NET_WM_STATE_ABOVE").unwrap().as_ptr(), 0);
        let net_wm_state_stays_on_top = (x_intern_atom)(dpy, CString::new("_NET_WM_STATE_STAYS_ON_TOP").unwrap().as_ptr(), 0);
        let net_wm_state_skip_taskbar = (x_intern_atom)(dpy, CString::new("_NET_WM_STATE_SKIP_TASKBAR").unwrap().as_ptr(), 0);
        let net_wm_state_skip_pager = (x_intern_atom)(dpy, CString::new("_NET_WM_STATE_SKIP_PAGER").unwrap().as_ptr(), 0);

        let net_wm_window_type = (x_intern_atom)(dpy, CString::new("_NET_WM_WINDOW_TYPE").unwrap().as_ptr(), 0);
        let net_wm_window_type_dock = (x_intern_atom)(dpy, CString::new("_NET_WM_WINDOW_TYPE_DOCK").unwrap().as_ptr(), 0);
        let net_wm_window_type_utility = (x_intern_atom)(dpy, CString::new("_NET_WM_WINDOW_TYPE_UTILITY").unwrap().as_ptr(), 0);

        // 1. Set window type to DOCK and UTILITY (compositors keep DOCK above full screen games)
        let window_types = [net_wm_window_type_dock, net_wm_window_type_utility];
        (x_change_property)(
            dpy,
            win,
            net_wm_window_type,
            XA_ATOM,
            32,
            PROP_MODE_REPLACE,
            window_types.as_ptr() as *const c_uchar,
            2,
        );

        // 2. Set _NET_WM_STATE properties directly
        let states = [
            net_wm_state_above,
            net_wm_state_stays_on_top,
            net_wm_state_skip_taskbar,
            net_wm_state_skip_pager,
        ];
        (x_change_property)(
            dpy,
            win,
            net_wm_state,
            XA_ATOM,
            32,
            PROP_MODE_REPLACE,
            states.as_ptr() as *const c_uchar,
            states.len() as c_int,
        );

        // 3. Send EWMH ClientMessage event to the root window (action: 1 = _NET_WM_STATE_ADD)
        let mut msg_event = XClientMessageEvent {
            type_: CLIENT_MESSAGE,
            serial: 0,
            send_event: 1,
            display: dpy,
            window: win,
            message_type: net_wm_state,
            format: 32,
            data: [
                1, // _NET_WM_STATE_ADD
                net_wm_state_above as c_long,
                net_wm_state_stays_on_top as c_long,
                1, // source indication (1 = normal application)
                0,
            ],
        };

        (x_send_event)(
            dpy,
            root,
            0,
            SUBSTRUCTURE_REDIRECT_MASK | SUBSTRUCTURE_NOTIFY_MASK,
            &mut msg_event as *mut _ as *mut c_void,
        );

        // 4. Raise the window
        (x_raise_window)(dpy, win);
        (x_flush)(dpy);

        // 5. Apply XFixes click-through if enabled
        if click_through {
            if let Ok(lib_xfixes) = libloading::Library::new("libXfixes.so.3")
                .or_else(|_| libloading::Library::new("libXfixes.so"))
            {
                if let (Ok(x_fixes_create_region), Ok(x_fixes_set_shape), Ok(x_fixes_destroy_region)) = (
                    lib_xfixes.get::<FnXFixesCreateRegion>(b"XFixesCreateRegion\0"),
                    lib_xfixes.get::<FnXFixesSetWindowShapeRegion>(b"XFixesSetWindowShapeRegion\0"),
                    lib_xfixes.get::<FnXFixesDestroyRegion>(b"XFixesDestroyRegion\0"),
                ) {
                    let region = (x_fixes_create_region)(dpy, std::ptr::null_mut(), 0);
                    (x_fixes_set_shape)(dpy, win, SHAPE_INPUT, 0, 0, region);
                    (x_fixes_destroy_region)(dpy, region);
                    (x_flush)(dpy);
                    log::debug!("Applied XFixes click-through on window {}", win);
                }
            }
        }

        (x_close_display)(dpy);
        log::info!("Enforced Always-On-Top and Dock z-order on window {}", win);
        true
    }
}

/// Sets the Linux X11 window to allow all mouse events to pass directly through to games.
pub fn apply_x11_click_through(win_id: u64) -> bool {
    unsafe {
        let lib_x11 = match libloading::Library::new("libX11.so.6")
            .or_else(|_| libloading::Library::new("libX11.so"))
        {
            Ok(lib) => lib,
            Err(_) => return false,
        };

        let lib_xfixes = match libloading::Library::new("libXfixes.so.3")
            .or_else(|_| libloading::Library::new("libXfixes.so"))
        {
            Ok(lib) => lib,
            Err(_) => return false,
        };

        let x_open_display: libloading::Symbol<FnXOpenDisplay> = match lib_x11.get(b"XOpenDisplay\0") {
            Ok(s) => s,
            Err(_) => return false,
        };
        let x_close_display: libloading::Symbol<FnXCloseDisplay> = match lib_x11.get(b"XCloseDisplay\0") {
            Ok(s) => s,
            Err(_) => return false,
        };
        let x_flush: libloading::Symbol<FnXFlush> = match lib_x11.get(b"XFlush\0") {
            Ok(s) => s,
            Err(_) => return false,
        };
        let x_fixes_create_region: libloading::Symbol<FnXFixesCreateRegion> = match lib_xfixes.get(b"XFixesCreateRegion\0") {
            Ok(s) => s,
            Err(_) => return false,
        };
        let x_fixes_set_shape: libloading::Symbol<FnXFixesSetWindowShapeRegion> = match lib_xfixes.get(b"XFixesSetWindowShapeRegion\0") {
            Ok(s) => s,
            Err(_) => return false,
        };
        let x_fixes_destroy_region: libloading::Symbol<FnXFixesDestroyRegion> = match lib_xfixes.get(b"XFixesDestroyRegion\0") {
            Ok(s) => s,
            Err(_) => return false,
        };

        let dpy = (x_open_display)(std::ptr::null());
        if dpy.is_null() {
            return false;
        }

        let region = (x_fixes_create_region)(dpy, std::ptr::null_mut(), 0);
        (x_fixes_set_shape)(dpy, win_id as Window, SHAPE_INPUT, 0, 0, region);
        (x_fixes_destroy_region)(dpy, region);
        (x_flush)(dpy);
        (x_close_display)(dpy);
        true
    }
}

type FnXDefaultScreen = unsafe extern "C" fn(*mut Display) -> c_int;
type FnXDisplayWidth = unsafe extern "C" fn(*mut Display, c_int) -> c_int;
type FnXDisplayHeight = unsafe extern "C" fn(*mut Display, c_int) -> c_int;

/// Queries the primary Linux X11 display resolution.
pub fn get_screen_resolution() -> (f32, f32) {
    unsafe {
        if let Ok(lib_x11) = libloading::Library::new("libX11.so.6")
            .or_else(|_| libloading::Library::new("libX11.so"))
        {
            if let (Ok(x_open_display), Ok(x_close_display), Ok(x_default_screen), Ok(x_display_width), Ok(x_display_height)) = (
                lib_x11.get::<FnXOpenDisplay>(b"XOpenDisplay\0"),
                lib_x11.get::<FnXCloseDisplay>(b"XCloseDisplay\0"),
                lib_x11.get::<FnXDefaultScreen>(b"XDefaultScreen\0"),
                lib_x11.get::<FnXDisplayWidth>(b"XDisplayWidth\0"),
                lib_x11.get::<FnXDisplayHeight>(b"XDisplayHeight\0"),
            ) {
                let dpy = (x_open_display)(std::ptr::null());
                if !dpy.is_null() {
                    let screen = (x_default_screen)(dpy);
                    let w = (x_display_width)(dpy, screen) as f32;
                    let h = (x_display_height)(dpy, screen) as f32;
                    (x_close_display)(dpy);
                    if w > 0.0 && h > 0.0 {
                        return (w, h);
                    }
                }
            }
        }
    }
    (2560.0, 1440.0)
}

/// Queries the primary Linux X11 active display refresh rate (Hz).
pub fn get_monitor_refresh_rate() -> f32 {
    if let Ok(output) = std::process::Command::new("xrandr").output() {
        if let Ok(text) = String::from_utf8(output.stdout) {
            for line in text.lines() {
                if line.contains('*') {
                    for part in line.split_whitespace() {
                        if part.ends_with('*') || part.ends_with("*+") {
                            let clean = part.trim_end_matches('*').trim_end_matches('+');
                            if let Ok(rate) = clean.parse::<f32>() {
                                if rate > 0.0 {
                                    return rate;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    180.0
}
