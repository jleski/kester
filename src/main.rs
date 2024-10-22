use windows::{
    Win32::Foundation::*,
    Win32::UI::WindowsAndMessaging::*,
    Win32::Graphics::Dwm::*,
};

use windows::Win32::System::ProcessStatus::*;
use windows::Win32::System::Threading::*;
use std::path::PathBuf;
use windows::Win32::Graphics::Gdi::{RedrawWindow, RDW_FRAME, RDW_INVALIDATE, RDW_UPDATENOW};

mod config;

use config::{Config, load_config};

fn main() {
    let config = load_config("config.yaml").expect("Failed to load configuration");
    println!("Open Visible Windows:");
    unsafe { EnumWindows(Some(enum_window), LPARAM(&config as *const Config as isize)) }
        .expect("Failed to enumerate windows");
}

extern "system" fn enum_window(window: HWND, lparam: LPARAM) -> BOOL {
    let config = unsafe { &*(lparam.0 as *const Config) };
    unsafe {
        if !is_window_visible_and_normal(window) {
            return true.into();
        }

        let mut text: [u16; 512] = [0; 512];
        let len = GetWindowTextW(window, &mut text);
        if len > 0 {
            let title = String::from_utf16_lossy(&text[..len as usize]);
            if !title.is_empty() {
                let exe_name = get_window_exe_name(window).unwrap_or_else(|| "Unknown".to_string());
                if let Some(opacity) = determine_opacity(&title, &exe_name, config) {
                    set_window_transparency(window, opacity).unwrap_or_else(|_| println!("Failed to set transparency for: {}", title));
                }
                let class_name = get_window_class(window);
                let (width, height) = get_window_size(window);
                let is_cloaked = is_window_cloaked(window);
                //let transparency = get_window_transparency(window);

                println!("- Title: {}", title);
                println!("  Executable: {}", exe_name);
                println!("  Class: {}", class_name);
                println!("  Size: {}x{}", width, height);
                println!("  Cloaked: {}", is_cloaked);
                println!("  Transparency: {}", get_window_transparency(window).map_or("N/A".to_string(), |a| format!("{}%", (a as f32 / 255.0 * 100.0) as u8)));
                println!();
            }
        }
    }
    true.into()
}

fn is_window_visible_and_normal(window: HWND) -> bool {
    unsafe {
        IsWindowVisible(window).as_bool() &&
        !IsIconic(window).as_bool() &&
        GetAncestor(window, GA_ROOT) == window &&
        GetWindowLongW(window, GWL_STYLE) & (WS_POPUP.0 | WS_CHILD.0) as i32 == 0
    }
}

fn get_window_class(window: HWND) -> String {
    unsafe {
        let mut class_name: [u16; 256] = [0; 256];
        let len = GetClassNameW(window, &mut class_name);
        String::from_utf16_lossy(&class_name[..len as usize])
    }
}

fn get_window_size(window: HWND) -> (i32, i32) {
    unsafe {
        let mut rect = RECT::default();
        GetWindowRect(window, &mut rect).expect("TODO: panic message");
        (rect.right - rect.left, rect.bottom - rect.top)
    }
}

fn is_window_cloaked(window: HWND) -> bool {
    unsafe {
        let mut cloaked: u32 = 0;
        DwmGetWindowAttribute(
            window,
            DWMWA_CLOAKED,
            &mut cloaked as *mut u32 as *mut _,
            size_of::<u32>() as u32,
        ).is_ok() && cloaked != 0
    }
}

fn get_window_transparency(window: HWND) -> Option<u8> {
    unsafe {
        let style = GetWindowLongW(window, GWL_EXSTYLE);
        if (style as u32 & WS_EX_LAYERED.0) != 0 {
            let mut alpha: u8 = 0;
            let mut _color: COLORREF = COLORREF(0);
            let mut _flags: LAYERED_WINDOW_ATTRIBUTES_FLAGS = LAYERED_WINDOW_ATTRIBUTES_FLAGS(0);
            if GetLayeredWindowAttributes(window, Some(&mut _color), Some(&mut alpha), Some(&mut _flags)).is_ok() {
                return Some(alpha);
            }
        }
        None
    }
}

fn get_window_exe_name(window: HWND) -> Option<String> {
    unsafe {
        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(window, Some(&mut process_id));

        let process_handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id);
        if let Ok(handle) = process_handle {
            let mut buffer = [0u16; MAX_PATH as usize];
            if K32GetModuleFileNameExW(handle, None, &mut buffer) != 0 {
                let path = PathBuf::from(String::from_utf16_lossy(&buffer));
                return path.file_name().and_then(|name| name.to_str()).map(String::from);
            }
        }
        None
    }
}
fn set_window_transparency(window: HWND, percentage: u8) -> Result<(), windows::core::Error> {
    unsafe {
        let mut style = GetWindowLongW(window, GWL_EXSTYLE);

        if percentage == 100 {
            // Remove the layered window style to make it fully opaque
            style &= !WS_EX_LAYERED.0 as i32;
            SetWindowLongW(window, GWL_EXSTYLE, style);
            let _ = RedrawWindow(window, None, None, RDW_FRAME | RDW_INVALIDATE | RDW_UPDATENOW);
            Ok(())
        } else {
            // Set or ensure the layered window style
            style |= WS_EX_LAYERED.0 as i32;
            SetWindowLongW(window, GWL_EXSTYLE, style);

            let alpha = (percentage as f32 * 255.0 / 100.0) as u8;
            SetLayeredWindowAttributes(window, COLORREF(0), alpha, LWA_ALPHA)
        }
    }
}
fn determine_opacity(title: &str, exe_name: &str, config: &Config) -> Option<u8> {
    for window_config in &config.specific_windows {
        if window_config.title.as_ref().map_or(false, |t| title.contains(t)) ||
            window_config.executable.as_ref().map_or(false, |e| exe_name.contains(e))
        {
            return Some(window_config.opacity);
        }
    }
    config.default_opacity
}