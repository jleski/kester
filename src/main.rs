mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use iced::{widget::{Button, Column, Row, Text, Container, Scrollable}, Length, Alignment, Color, Application, Theme, Command, Element, window};
use iced::theme;
use iced::event::{self, Event};
use windows::{
    Win32::Foundation::*,
    Win32::UI::WindowsAndMessaging::*
};
use windows::Win32::System::ProcessStatus::*;
use windows::Win32::System::Threading::*;
use std::path::PathBuf;
use windows::Win32::Graphics::Gdi::{RedrawWindow, RDW_FRAME, RDW_INVALIDATE, RDW_UPDATENOW};
use std::sync::{Mutex};
use iced::widget::{Checkbox, Slider};
use iced::window::{Id, Mode};
use once_cell::sync::Lazy;
use tray_item::TrayItem;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};

mod config;

use config::{Config, load_config};

static WINDOW_INFO_BUFFER: Lazy<Mutex<Vec<WindowInfo>>> = Lazy::new(|| Mutex::new(Vec::new()));

fn main() -> iced::Result {
    let settings = iced::Settings {
        window: window::Settings {
            size: iced::Size::new(700.0, 900.0),
            resizable: false,
            decorations: true,
            exit_on_close_request: true,
            ..Default::default()
        },
        ..Default::default()
    };

    WindowManager::run(settings)
}

struct WindowManager {
    config: Config,
    windows: Vec<WindowInfo>,
    selected_window: Option<usize>,
    current_transparency: u8,
    persist_setting: bool,
    default_opacity: Option<u8>,
    use_default_opacity: bool,
    tray: Option<Arc<TrayItem>>,
    window_visible: bool,
    _tx: mpsc::Sender<Message>,
    _rx: Arc<Mutex<Receiver<Message>>>,
}

#[derive(Debug, Clone)]
struct WindowInfo {
    title: String,
    exe_name: String,
    transparency: String,
    hwnd: HWND,
}

#[derive(Debug, Clone)]
enum Message {
    RefreshWindows,
    SelectWindow(usize),
    UpdateTransparency(u8),
    TogglePersist(bool),
    UpdateDefaultOpacity(u8),
    ToggleDefaultOpacity(bool),
    MinimizeToTray,
    ShowWindow,
    CloseRequested,
    Ignore
}

unsafe impl Send for WindowInfo {}
unsafe impl Sync for WindowInfo {}

impl Application for WindowManager {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();
    fn new(_flags: ()) -> (Self, Command<Message>) {
        let config = load_config("config.yaml").unwrap_or_default();
        let default_opacity = config.default_opacity;

        let (tx, rx) = mpsc::channel();
        let rx = Arc::new(Mutex::new(rx));

        let mut tray = TrayItem::new("Transparency Manager", "tray_icon")
            .expect("Failed to create tray icon");

        let tx_show = tx.clone();
        tray.add_menu_item("Show Window", move || {
            tx_show.send(Message::ShowWindow).expect("Failed to send show message");
        }).expect("Failed to add Show menu item");

        let tx_exit = tx.clone();
        tray.add_menu_item("Exit", move || {
            tx_exit.send(Message::CloseRequested).expect("Failed to send exit message");
        }).expect("Failed to add Exit menu item");

        (
            WindowManager {
                config,
                windows: Vec::new(),
                selected_window: None,
                current_transparency: 0,
                persist_setting: false,
                default_opacity,
                use_default_opacity: default_opacity.is_some(),
                tray: Some(Arc::new(tray)),
                window_visible: true,
                _tx: tx,
                _rx: rx,
            },
            Command::perform(async {}, |_| Message::RefreshWindows),
        )
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        let rx = self._rx.clone();
        iced::Subscription::batch([
            event::listen().map(|event| {
                match event {
                    Event::Window(Id::MAIN, window::Event::CloseRequested) => Message::CloseRequested,
                    Event::Window(Id::MAIN, window::Event::Resized { width, height }) => {
                        if width == 0 && height == 0 {
                            Message::MinimizeToTray
                        } else {
                            Message::Ignore
                        }
                    },
                    _ => Message::Ignore
                }
            }),
            iced::subscription::unfold("tray_events", (), move |_| {
                let rx = rx.clone();
                async move {
                    match rx.lock().unwrap().recv() {
                        Ok(message) => (message, ()),
                        Err(_) => (Message::Ignore, ()),
                    }
                }
            })
        ])
    }

    fn title(&self) -> String {
        format!("Transparency Manager v{}", built_info::PKG_VERSION)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Ignore => {
                /* Do nothing */
            }
            Message::CloseRequested => std::process::exit(0),
            Message::MinimizeToTray => {
                self.window_visible = false;
                return Command::batch(vec![
                    window::change_mode(window::Id::MAIN, Mode::Hidden),
                    window::minimize(window::Id::MAIN, true),
                ])
            }
            Message::ShowWindow => {
                self.window_visible = true;
                return Command::batch(vec![
                    window::change_mode(window::Id::MAIN, Mode::Windowed),
                    window::minimize(window::Id::MAIN, false),
                    window::gain_focus(window::Id::MAIN),
                ])
            }
            Message::UpdateDefaultOpacity(value) => {
                self.default_opacity = Some(value);
                self.config.default_opacity = Some(value);

                // Apply new default opacity to all windows without explicit settings
                for window in &self.windows {
                    let has_explicit_setting = self.config.specific_windows.iter().any(|w|
                        w.title.as_ref().map_or(false, |t| t == &window.title) ||
                            w.executable.as_ref().map_or(false, |e| e == &window.exe_name)
                    );

                    if !has_explicit_setting {
                        set_window_transparency(window.hwnd, value)
                            .unwrap_or_else(|_| println!("Failed to set transparency for: {}", window.title));
                    }
                }

                config::save_config(&self.config, "config.yaml").expect("Config saved successfully");
                // self.default_opacity = Some(value);
                // self.config.default_opacity = Some(value);
                // config::save_config(&self.config, "config.yaml").expect("Config saved successfully");
            }
            Message::ToggleDefaultOpacity(value) => {
                self.use_default_opacity = value;
                if !value {
                    self.default_opacity = None;
                    self.config.default_opacity = None;
                } else {
                    let default_value = 100;
                    self.default_opacity = Some(default_value);
                    self.config.default_opacity = Some(default_value);
                }
                config::save_config(&self.config, "config.yaml").expect("Config saved successfully");
            }
            Message::RefreshWindows => {
                // Clear existing windows list
                self.windows.clear();

                // Clear the global buffer
                WINDOW_INFO_BUFFER.lock().unwrap().clear();

                // Enumerate windows and collect info
                unsafe {
                    EnumWindows(Some(enum_window), LPARAM(&self.config as *const _ as isize)).expect("TODO: panic message");
                }

                // Move windows from buffer to our state
                self.windows = WINDOW_INFO_BUFFER.lock().unwrap().drain(..).collect();

                // Reset selection
                self.selected_window = None;
            }
            Message::SelectWindow(index) => {
                self.selected_window = Some(index);
                let window = &self.windows[index];
                self.current_transparency = window.transparency
                    .trim_end_matches('%')
                    .parse()
                    .unwrap_or(100);
                self.persist_setting = self.config.specific_windows.iter().any(|w|
                    w.title.as_ref().map_or(false, |t| t == &window.title) ||
                        w.executable.as_ref().map_or(false, |e| e == &window.exe_name)
                );
            }
            Message::UpdateTransparency(value) => {
                self.current_transparency = value;
                if let Some(index) = self.selected_window {
                    let window = &self.windows[index];
                    set_window_transparency(window.hwnd, value).unwrap_or_else(|_| println!("Failed to set transparency"));

                    // Save to config if persist is checked
                    if self.persist_setting {
                        // Remove any existing config for this window
                        self.config.specific_windows.retain(|w| {
                            w.title.as_ref().map_or(true, |t| t != &window.title) &&
                                w.executable.as_ref().map_or(true, |e| e != &window.exe_name)
                        });

                        // Add new config
                        self.config.specific_windows.push(config::WindowConfig {
                            title: Some(window.title.clone()),
                            executable: Some(window.exe_name.clone()),
                            opacity: value,
                        });

                        // Save config to file
                        config::save_config(&self.config, "config.yaml").expect("Config saved successfully");
                    }
                }
            }
            Message::TogglePersist(value) => {
                self.persist_setting = value;
                if let Some(index) = self.selected_window {
                    let window = &self.windows[index];
                    if !value {
                        // Remove config for this window
                        self.config.specific_windows.retain(|w| {
                            w.title.as_ref().map_or(true, |t| t != &window.title) &&
                                w.executable.as_ref().map_or(true, |e| e != &window.exe_name)
                        });
                    } else {
                        // Add new config entry with current transparency
                        self.config.specific_windows.push(config::WindowConfig {
                            title: Some(window.title.clone()),
                            executable: Some(window.exe_name.clone()),
                            opacity: self.current_transparency,
                        });
                    }
                    // Save config to file
                    config::save_config(&self.config, "config.yaml").expect("Config saved successfully");
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let title = Text::new("Transparency Manager")
            .size(40)
            .style(Color::from([0.5, 0.5, 0.5]));

        let refresh_button = Button::new(Text::new("Refresh"))
            .on_press(Message::RefreshWindows)
            .padding(10);

        let header = Row::new()
            .push(title)
            .push(refresh_button)
            .align_items(Alignment::Center)
            .spacing(20);

        let default_opacity_section = if self.use_default_opacity {
            Row::new()
                .push(Text::new("Default Opacity:").size(14))
                .push(Slider::new(
                    0..=100,
                    self.default_opacity.unwrap_or(100),
                    Message::UpdateDefaultOpacity,
                ))
                .push(Text::new(format!("{}%", self.default_opacity.unwrap_or(100))).size(14))
                .spacing(20)
        } else {
            Row::new()
        };

        let selected_info = if let Some(index) = self.selected_window {
            let window = &self.windows[index];
            format!("{} ({})", window.title, window.exe_name)
        } else {
            "No window selected".to_string()
        };

        let selected_info_text = Text::new(selected_info).size(16);

        let transparency_section = if let Some(_) = self.selected_window {
            Row::new()
                .push(Slider::new(
                    0..=100,
                    self.current_transparency,
                    Message::UpdateTransparency,
                ))
                .push(Text::new(format!("{}%", self.current_transparency)).size(14))
                .push(Checkbox::new(
                    "Persist",
                    self.persist_setting,
                ).on_toggle(Message::TogglePersist))
                .spacing(20)
        } else {
            Row::new()
        };

        let windows_list = self.windows.iter().enumerate().fold(
            Column::new().spacing(10),
            |column, (index, window)| {
                column.push(
                    Button::new(
                        Container::new(
                            Column::new()
                                .push(Text::new(&window.title).size(18))
                                .push(Text::new(format!("Executable: {}", window.exe_name)).size(12))
                                .push(Text::new(format!("Transparency: {}", window.transparency)).size(12))
                        )
                            .style(theme::Container::Box)
                            .padding(10)
                    )
                        .on_press(Message::SelectWindow(index))
                        .style(if Some(index) == self.selected_window {
                            theme::Button::Primary
                        } else {
                            theme::Button::Secondary
                        })
                )
            }
        );

        let content = Scrollable::new(windows_list)
            .height(Length::Fill)
            .width(Length::Fill);

        Container::new(
            Column::new()
                .push(header)
                .push(Checkbox::new(
                    "Use Default Opacity",
                    self.use_default_opacity,
                ).on_toggle(Message::ToggleDefaultOpacity))
                .push(default_opacity_section)
                .push(selected_info_text)
                .push(transparency_section)
                .push(content)
                .spacing(20)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .into()
    }
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
                let exe_name = get_window_exe_name(window)
                    .unwrap_or_else(|| "Unknown".to_string())
                    .chars()
                    .filter(|c| c.is_ascii_graphic())
                    .collect::<String>();
                if let Some(opacity) = determine_opacity(&title, &exe_name, config) {
                    set_window_transparency(window, opacity).unwrap_or_else(|_| println!("Failed to set transparency for: {}", title));
                }
                let transparency = get_window_transparency(window).map_or("N/A".to_string(), |a| format!("{}%", (a as f32 / 255.0 * 100.0) as u8));

                let window_info = WindowInfo {
                    title,
                    exe_name,
                    transparency,
                    hwnd: window
                };

                WINDOW_INFO_BUFFER.lock().unwrap().push(window_info);
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