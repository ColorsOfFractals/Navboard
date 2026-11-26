use dioxus::prelude::*;
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq)]
pub struct MenuTarget {
    pub project_id: Uuid,
    pub panel_id: Uuid,
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, PartialEq)]
pub enum MenuAction {
    Open,
    Rename,
    Duplicate,
    Delete,
}

#[derive(Clone, PartialEq)]
pub struct MenuItem {
    pub label: &'static str,
    pub symbol: &'static str,
    pub action: MenuAction,
    pub enabled: bool,
    pub danger: bool,
}

pub fn panel_items() -> Vec<MenuItem> {
    vec![
        MenuItem {
            label: "Open panel",
            symbol: "\u{2197}",
            action: MenuAction::Open,
            enabled: true,
            danger: false,
        },
        MenuItem {
            label: "Rename panel",
            symbol: "\u{270E}",
            action: MenuAction::Rename,
            enabled: true,
            danger: false,
        },
        MenuItem {
            label: "Duplicate panel",
            symbol: "\u{25A3}",
            action: MenuAction::Duplicate,
            enabled: true,
            danger: false,
        },
        MenuItem {
            label: "Delete panel",
            symbol: "\u{00D7}",
            action: MenuAction::Delete,
            enabled: false,
            danger: true,
        },
    ]
}

#[component]
pub fn ContextMenu(
    target: MenuTarget,
    heading: String,
    items: Vec<MenuItem>,
    on_action: EventHandler<MenuAction>,
    on_dismiss: EventHandler<()>,
) -> Element {
    let position = format!(
        "left: clamp(8px, {}px, calc(100vw - 260px)); \
         top: clamp(8px, {}px, calc(100vh - 240px));",
        target.x,
        target.y
    );

    rsx! {
        div {
            class: "nav-context-layer",

            oncontextmenu: move |event| {
                event.prevent_default();
                on_dismiss.call(());
            },

            onkeydown: move |event| {
                if event.key() == Key::Escape {
                    on_dismiss.call(());
                }
            },

            div {
                class: "nav-context-backdrop",
                onclick: move |_| on_dismiss.call(()),
            }

            div {
                class: "nav-context-menu",
                style: "{position}",
                role: "menu",

                p {
                    class: "nav-context-heading",
                    "{heading}"
                }

                for item in items {
                    button {
                        class: if item.danger {
                            "nav-context-item nav-context-danger"
                        } else {
                            "nav-context-item"
                        },
                        role: "menuitem",
                        disabled: !item.enabled,

                        onclick: move |_| {
                            on_action.call(item.action);
                        },

                        span {
                            class: "nav-context-symbol",
                            "{item.symbol}"
                        }

                        span { "{item.label}" }
                    }
                }
            }
        }
    }
}