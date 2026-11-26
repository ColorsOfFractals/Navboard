use dioxus::prelude::*;
use crate::model::PanelKind;

#[component]
pub fn PanelMenu(on_select: EventHandler<PanelKind>) -> Element {
    let mut open = use_signal(|| false);

    let options = [
        (PanelKind::Lens, "Collect photographic evidence"),
        (PanelKind::Specs, "Capture structured specifications"),
        (PanelKind::Clock, "Allocate project time"),
        (PanelKind::Text, "Freeform thoughts and writing"),
        (PanelKind::Canvas, "Draw, sketch and annotate"),
        (PanelKind::Stitch, "Compose documents and plans"),
        (PanelKind::Forge, "Construct commands and logic"),
        (PanelKind::Atlas, "Map equipment and infrastructure"),
    ];

    rsx! {
        div {
            class: "nav-panel-menu-anchor",
            onkeydown: move |event| {
                if event.key() == Key::Escape {
                    open.set(false);
                }
            },

            button {
                class: "nav-create nav-add-panel",
                onclick: move |_| {
                    let next = !open();
                    open.set(next);
                },
                span { class: "nav-plus", "+" }
                "Add Panel"
                span { class: "nav-menu-chevron", "v" }
            }

            if open() {
                div {
                    class: "nav-menu-backdrop",
                    onclick: move |_| open.set(false),
                }

                div { class: "nav-type-menu",
                    p { class: "nav-menu-heading", "CHOOSE PANEL TYPE" }

                    for (kind, description) in options {
                        button {
                            key: "{kind.label()}",
                            class: "nav-type-option",
                            style: "--panel-accent: {kind.accent()};",
                            onclick: move |_| {
                                open.set(false);
                                on_select.call(kind);
                            },

                            span {
                                class: "nav-type-symbol",
                                "{kind.symbol()}"
                            }

                            div { class: "nav-type-description",
                                strong { "{kind.label()}" }
                                small { "{description}" }
                            }

                            span { class: "nav-type-arrow", ">" }
                        }
                    }
                }
            }
        }
    }
}