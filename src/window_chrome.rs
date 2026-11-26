use dioxus::prelude::*;
use dioxus::desktop::use_window;
use dioxus::html::input_data::MouseButton;

#[component]
pub fn WindowChrome() -> Element {
    let window = use_window();
    let minimize_window = window.clone();
    let maximize_window = window.clone();
    let close_window = window.clone();
    let drag_window = window.clone();
    let double_click_window = window.clone();

    rsx! {
        div {
            class: "nav-window-chrome",

            div {
                class: "nav-window-drag",

                onmousedown: move |event| {
                    if event.trigger_button() == Some(MouseButton::Primary) {
                        drag_window.drag();
                    }
                },

                ondoubleclick: move |_| {
                    double_click_window.set_maximized(
                        !double_click_window.is_maximized()
                    );
                },

                img {
                    class: "nav-window-icon",
                    src: crate::NAVBOARD_ICON,
                    draggable: "false",
                    alt: ""
                }

                span {
                    class: "nav-window-title",
                    "Navboard"
                }
            }

            div {
                class: "nav-window-controls",

                button {
                    class: "nav-window-button",
                    title: "Minimize",
                    aria_label: "Minimize window",
                    onclick: move |_| {
                        minimize_window.set_minimized(true);
                    },

                    svg {
                        view_box: "0 0 16 16",
                        path { d: "M3 8 H13" }
                    }
                }

                button {
                    class: "nav-window-button",
                    title: "Maximize / Restore",
                    aria_label: "Maximize or restore window",
                    onclick: move |_| {
                        maximize_window.set_maximized(
                            !maximize_window.is_maximized()
                        );
                    },

                    svg {
                        view_box: "0 0 16 16",
                        rect {
                            x: "3",
                            y: "3",
                            width: "10",
                            height: "10"
                        }
                    }
                }

                button {
                    class: "nav-window-button nav-window-close",
                    title: "Close",
                    aria_label: "Close window",
                    onclick: move |_| {
                        close_window.close();
                    },

                    svg {
                        view_box: "0 0 16 16",
                        path { d: "M3 3 L13 13 M13 3 L3 13" }
                    }
                }
            }
        }
    }
}