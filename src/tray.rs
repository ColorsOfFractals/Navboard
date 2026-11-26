use dioxus::prelude::*;
use tray_icon::{
    TrayIcon, TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState,
};

pub fn create() -> Result<TrayIcon, String> {
    let rgba = image::load_from_memory_with_format(
        include_bytes!("../assets/Navboard_Icon.png"),
        image::ImageFormat::Png,
    )
    .map_err(|e| e.to_string())?
    .to_rgba8();

    let (width, height) = rgba.dimensions();

    let icon = tray_icon::Icon::from_rgba(
        rgba.into_raw(), width, height
    ).map_err(|e| e.to_string())?;

    TrayIconBuilder::new()
        .with_tooltip("Navboard")
        .with_icon(icon)
        .build()
        .map_err(|e| e.to_string())
}

use std::sync::{Mutex, atomic::{AtomicBool, Ordering}};
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::desktop::tao::dpi::{LogicalSize, PhysicalPosition};
use dioxus::desktop::tao::platform::windows::WindowBuilderExtWindows;

static OPEN: AtomicBool = AtomicBool::new(false);
static BUSY: AtomicBool = AtomicBool::new(false);
static COMMAND: Mutex<Option<String>> = Mutex::new(None);

pub fn use_tray(
    mut orbit: Signal<bool>,
    mut instrument: Signal<Option<String>>,
    mut project: Signal<Option<usize>>,
    mut panel: Signal<Option<usize>>,
) {
    let window = dioxus::desktop::use_window();
    let _tray = use_hook(|| create().map_err(|e| eprintln!("[TRAY] {e}")).ok());
    let parent = window.clone();
    let _handler = dioxus::desktop::use_tray_icon_event_handler(move |event| {
        if let TrayIconEvent::Click { button, button_state, position, .. } = event {
            if *button_state != MouseButtonState::Up { return; }
            match button {
                MouseButton::Right => {
                    if OPEN.swap(true, Ordering::SeqCst) { return; }
                    BUSY.store(instrument.peek().is_some() || panel.peek().is_some(), Ordering::SeqCst);
                    let scale = parent.scale_factor();
                    let builder = WindowBuilder::new()
                        .with_title("Navboard Quick Launch")
                        .with_inner_size(LogicalSize::new(370.0, 460.0))
                        .with_position(PhysicalPosition::new(
                            (position.x - 370.0 * scale).max(0.0) as i32,
                            (position.y - 460.0 * scale - 12.0).max(0.0) as i32,
                        ))
                        .with_decorations(false)
                        .with_resizable(false)
                        .with_always_on_top(true)
                        .with_skip_taskbar(true);
                    parent.new_window(VirtualDom::new(TrayLauncher), Config::new().with_window(builder).with_menu(None));
                }
                MouseButton::Left => {
                    parent.set_visible(true);
                    parent.set_minimized(false);
                    parent.set_focus();
                }
                _ => {}
            }
        }
    });
    use_future(move || {
        let window = window.clone();
        async move {
            loop {
                let command = COMMAND.lock().ok().and_then(|mut c| c.take());
                if let Some(command) = command {
                    let busy = instrument.peek().is_some() || panel.peek().is_some();
                    if command == "restore" || !busy {
                        if command == "exit" { window.close(); return; }
                        window.set_visible(true);
                        window.set_minimized(false);
                        window.set_focus();
                        match command.as_str() {
                            "restore" => {},
                            "orbit" => { project.set(None); panel.set(None); orbit.set(true); },
                            "worlds" => { project.set(None); panel.set(None); orbit.set(false); },
                            _ => { project.set(None); panel.set(None); orbit.set(true); instrument.set(Some(command)); }
                        }
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
        }
    });
}

#[component]
pub fn TrayLauncher() -> Element {
    let window = dioxus::desktop::use_window();
    use_drop(|| OPEN.store(false, Ordering::SeqCst));
    let busy = BUSY.load(Ordering::SeqCst);
    let navigate = Callback::new(move |command: String| {
        if command != "dismiss" {
            if let Ok(mut slot) = COMMAND.lock() { *slot = Some(command); }
        }
        window.close();
    });
    rsx! {
        style { {CSS} }

        div {
            class: "tray-launcher",

            div {
                class: "tray-launcher-header",
                img {
                    src: crate::NAVBOARD_ICON,
                    alt: ""
                }
                div {
                    strong { "NAVBOARD" }
                    small { "Your world, in orbit." }
                }
                button {
                    class: "tray-dismiss",
                    onclick: move |_| navigate.call("dismiss".into()),
                    "X"
                }
            }

            div {
                class: "tray-launcher-navigation",
                button {
                    onclick: move |_| navigate.call("restore".into()),
                    "Show Navboard"
                }
                button {
                    disabled: busy,
                    onclick: move |_| navigate.call("orbit".into()),
                    "Open Orbit"
                }
                button {
                    disabled: busy,
                    onclick: move |_| navigate.call("worlds".into()),
                    "Enter Worlds"
                }
            }

            div {
                class: "tray-launcher-label",
                "INSTRUMENTS"
            }

            div {
                class: "tray-launcher-grid",
                for (name, color) in [
                    ("Text", "#a89ce8"),
                    ("Canvas", "#67cbd8"),
                    ("Stitch", "#79c9a1"),
                    ("Forge", "#e5a65c"),
                    ("Clock", "#8299ec"),
                    ("Specs", "#d5c57b"),
                    ("Echo", "#ef8181"),
                    ("Lens", "#d9a6d8"),
                    ("Atlas", "#8db8a8"),
                ] {
                    button {
                        disabled: busy,
                        style: "border-color: {color}; color: {color};",
                        onclick: move |_| navigate.call(name.to_string()),
                        "{name}"
                    }
                }
            }

            if busy {
                div {
                    class: "tray-launcher-notice",
                    "Active session protected. Return to Orbit first."
                }
            }

            button {
                class: "tray-launcher-exit",
                disabled: busy,
                onclick: move |_| navigate.call("exit".into()),
                "Exit Navboard"
            }
        }
    }
}
const CSS: &str = r#"

/* RING 1.16.9 | CUSTOM TRAY LAUNCHER */

.tray-launcher-backdrop {
    position: fixed;
    inset: 0;
    z-index: 9000;
    background: rgba(5, 12, 16, .35);
}

.tray-launcher {
    position: fixed;
    right: 22px;
    bottom: 22px;
    width: 340px;
    max-width: calc(100vw - 44px);
    max-height: calc(100vh - 44px);
    overflow: hidden;
    box-sizing: border-box;
    z-index: 9001;
    padding: 18px;
    border: 1px solid #467e6e;
    border-radius: 16px;
    background: linear-gradient(145deg, #203d36, #10191e);
    box-shadow: 0 18px 60px #0009;
    color: #dcece6;
}

.tray-launcher-header {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 18px;
}

.tray-launcher-header img {
    width: 54px;
    height: 54px;
    object-fit: contain;
}

.tray-launcher-header strong {
    display: block;
    letter-spacing: 3px;
    color: #9bdcc1;
}

.tray-launcher-header small {
    display: block;
    color: #8db8a8;
    font-size: 11px;
}

.tray-launcher button {
    padding: 10px;
    border: 1px solid #467e6e;
    border-radius: 8px;
    background: #152622;
    color: #dcece6;
    cursor: pointer;
    transition: background .15s, transform .15s;
}

.tray-launcher button:hover:not(:disabled) {
    background: #356455;
    transform: translateY(-1px);
}

.tray-launcher button:disabled {
    opacity: .35;
    cursor: default;
}

.tray-launcher .tray-dismiss {
    margin-left: auto;
    padding: 5px 10px;
}

.tray-launcher-navigation {
    display: grid;
    gap: 7px;
}

.tray-launcher-label {
    margin: 18px 0 10px;
    color: #8db8a8;
    font-size: 10px;
    letter-spacing: 2px;
}

.tray-launcher-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
}

.tray-launcher-notice {
    margin-top: 12px;
    font-size: 11px;
    color: #d5c57b;
}

.tray-launcher .tray-launcher-exit {
    width: 100%;
    margin-top: 18px;
    border-color: #815254;
    color: #ef8181;
}
html, body { overflow: hidden; margin: 0; background: #10191e; font-family: Segoe UI, sans-serif; }
.tray-launcher { inset: 0; width: 100%; max-width: none; max-height: 100vh; border-radius: 0; box-shadow: none; }
"#;
