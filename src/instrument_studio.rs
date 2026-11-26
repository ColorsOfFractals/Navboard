use dioxus::prelude::*;

#[component]
pub fn InstrumentStudio(
    instrument: String,
    on_back: EventHandler<()>,
    worlds: Vec<crate::model::Project>,
    writable: bool,
    on_assign: EventHandler<(Option<uuid::Uuid>, String, crate::model::Panel)>,
) -> Element {
    let mut title = use_signal(|| format!("Untitled {}", instrument));
    let mut notes = use_signal(String::new);
    let mut confirm_exit = use_signal(|| false);
    let mut content = use_signal(|| None::<crate::model::PieceContent>);
    let mut assigning = use_signal(|| false);
    let mut export_status = use_signal(String::new);
    let mut world_name = use_signal(String::new);
    let kind = match instrument.as_str() {
        "Text" => Some(crate::model::PanelKind::Text),
        "Canvas" => Some(crate::model::PanelKind::Canvas),
        "Stitch" => Some(crate::model::PanelKind::Stitch),
        "Atlas" => Some(crate::model::PanelKind::Atlas), "Forge" => Some(crate::model::PanelKind::Forge), "Lens" => Some(crate::model::PanelKind::Lens), "Echo" => Some(crate::model::PanelKind::Echo), "Specs" => Some(crate::model::PanelKind::Specs), "Clock" => Some(crate::model::PanelKind::Clock),
        _ => None,
    };
    let submit = move |destination: Option<uuid::Uuid>, name: String| {
        if let Some(kind) = kind {
            let panel = crate::model::Panel {
                id: uuid::Uuid::new_v4(), kind, title: title(), notes: notes(), content: content(),
            };
            on_assign.call((destination, name, panel));
        }
    };

    let (accent, description, available) = match instrument.as_str() {
        "Text" => (
            "#a89ce8",
            "Express ideas through language.",
            true,
        ),
        "Canvas" => (
            "#67cbd8",
            "Sketch, draw and visualize.",
            true,
        ),
        "Stitch" => (
            "#79c9a1",
            "Arrange steps into procedures.",
            true,
        ),
        "Forge" => (
            "#e5a65c",
            "Connect ideas, systems and resources.",
            true,
        ),
        "Clock" => (
            "#8299ec",
            "Explore events, timelines and durations.",
            true,
        ),
        "Specs" => (
            "#d5c57b",
            "Gather structured project specifications.",
            true,
        ),
        "Echo" => (
            "#ef8181",
            "Transcribe microphone and system audio locally.",
            true,
        ),
        "Lens" => (
            "#d9a6d8",
            "Collect photographs and visual evidence.",
            true,
        ),
        "Atlas" => (
            "#8db8a8",
            "Map equipment and infrastructure.",
            true,
        ),
        _ => ("#79c9a1", "Independent instrument.", false),
    };

    let category = match instrument.as_str() {
        "Echo" | "Lens" | "Atlas" => "CAPTURE & COLLECT",
        _ => "CONSTRUCT & EXPRESS",
    };

    rsx! {
        section {
            class: "nb-instrument-shell",
            style: "--instrument-accent:{accent};",

            div {
                class: "nb-instrument-navigation",

                button {
                    class: "nb-orbit-return",
                    onclick: move |_| {
                        if available {
                            confirm_exit.set(true);
                        } else {
                            on_back.call(());
                        }
                    },
                    "Back to Orbit"
                }

                span {
                    class: "nb-instrument-session",
                    "INDEPENDENT SESSION"
                }
            }

            if !matches!(instrument.as_str(), "Canvas" | "Stitch" | "Forge" | "Clock" | "Specs" | "Lens" | "Echo" | "Atlas") {
                div {
                    class: "nb-instrument-heading",

                    span { "{category}" }
                    h1 { "{instrument} Studio" }
                    p { "{description}" }
                }
            }

            if available {
                div {
                    class: "nb-instrument-title",

                    label { "PIECE NAME" }

                    input {
                        value: "{title}",
                        placeholder: "Untitled piece",
                        oninput: move |event| {
                            title.set(event.value());
                        },
                    }
                }
            }

            div {
                class: "nb-instrument-workspace",

                match instrument.as_str() {
                    "Text" => rsx! {
                        textarea {
                            class: "nb-instrument-notes",
                            value: "{notes}",
                            placeholder: "Give your thoughts room to breathe...",
                            oninput: move |event| {
                                notes.set(event.value());
                            },
                        }
                    },
                    "Canvas" => rsx! {
                        crate::canvas_studio::CanvasStudio {
                            title: title(),
                            initial: None,
                            on_change: move |value| content.set(Some(value)),
                        }
                    },
                    "Stitch" => rsx! {
                        crate::stitch_studio::StitchStudio {
                            title: title(),
                            initial: None,
                            on_change: move |value| content.set(Some(value)),
                        }
                    },
                    "Clock" => rsx! {
                        crate::clock_studio::ClockStudio { title: title(), initial: None, on_change: move |value| content.set(Some(value)) }
                    },
                    "Specs" => rsx! {
                        crate::specs_studio::SpecsStudio { title: title(), initial: None, on_change: move |value| content.set(Some(value)) }
                    },
                    "Echo" => rsx! {
                        crate::echo_studio::EchoStudio { initial: None, on_change: move |value| content.set(Some(value)) }
                    },
                    "Lens" => rsx! {
                        crate::lens_studio::LensStudio { title: title(), initial: None, on_change: move |value| content.set(Some(value)) }
                    },
                    "Atlas" => rsx! { crate::atlas_studio::AtlasStudio { title: title(), initial: None, on_change: move |value| content.set(Some(value)) } },
                    "Forge" => rsx! {
                        crate::forge_studio::ForgeStudio {
                            title: title(),
                            initial: None,
                            on_change: move |value| content.set(Some(value)),
                        }
                    },
                    _ => rsx! {
                        div {
                            class: "nb-instrument-placeholder",

                            div {
                                class: "nb-instrument-planet",
                                "{instrument}"
                            }

                            h2 { "Instrument under construction" }
                            p { "{description}" }
                            span { "ENGINE DEVELOPMENT PENDING" }
                        }
                    },
                }
            }

            div {
                class: "nb-instrument-assignment",

                div {
                    class: "nb-instrument-assignment-status",

                    i {}
                    span {
                        if available {
                            "Temporary / Unassigned piece"
                        } else {
                            "Instrument shell"
                        }
                    }
                }

                if available {
                    div { style: "display:flex;flex-direction:column;gap:5px;align-items:flex-end;margin-left:auto;",
                        button {
                            class: "nb-orbit-return",
                            disabled: title().trim().is_empty() || (instrument != "Text" && content().is_none()),
                            onclick: move |_| {
                                if let Some(kind) = kind {
                                    let panel = crate::model::Panel {
                                        id: uuid::Uuid::new_v4(), kind, title: title(), notes: notes(), content: content(),
                                    };
                                    export_status.set(match crate::manifest::export_dialog(panel) {
                                        Ok(Some(path)) => format!("Exported: {}", path.display()),
                                        Ok(None) => "Export canceled".into(),
                                        Err(error) => format!("Export failed: {error}"),
                                    });
                                }
                            },
                            "Export Manifest"
                        }
                        if !export_status().is_empty() {
                            span { style: "font-size:11px;max-width:260px;overflow-wrap:anywhere;", "{export_status}" }
                        }
                    }
                }
                button {
                    class: "nb-instrument-assign",
                    disabled: !available || !writable || title().trim().is_empty() || (instrument != "Text" && content().is_none()),
                    onclick: move |_| assigning.set(true),
                    "Assign to World"
                }
            }

            if assigning() {
                div { class: "nb-instrument-modal",
                    div { class: "nb-instrument-dialog", style: "max-height:80vh;overflow:auto;",
                        h2 { "Assign to World" }
                        p { "Choose an existing World or create a new one." }
                        for world in worlds.iter() {
                            { let id = world.id; rsx! {
                                div { style: "margin:8px 0;",
                                    button { class: "nb-orbit-return", onclick: move |_| submit(Some(id), String::new()), "{world.name}" }
                                }
                            } }
                        }
                        input { class: "nav-panel-title-input", placeholder: "New World name", value: "{world_name}", oninput: move |e| world_name.set(e.value()) }
                        div { class: "nb-instrument-dialog-actions",
                            button { class: "nb-orbit-return", onclick: move |_| assigning.set(false), "Cancel" }
                            button { class: "nb-orbit-return", disabled: world_name().trim().is_empty(), onclick: move |_| submit(None, world_name().trim().to_string()), "Create World & Assign" }
                        }
                    }
                }
            }
            if confirm_exit() {
                div {
                    class: "nb-instrument-modal",

                    div {
                        class: "nb-instrument-dialog",

                        h2 { "Return to Orbit?" }

                        p {
                            "This session is temporary. Returning to Orbit will discard its contents."
                        }

                        div {
                            class: "nb-instrument-dialog-actions",

                            button {
                                class: "nb-orbit-return",
                                onclick: move |_| confirm_exit.set(false),
                                "Keep working"
                            }

                            button {
                                class: "nb-instrument-discard",
                                onclick: move |_| on_back.call(()),
                                "Discard session"
                            }
                        }
                    }
                }
            }
        }
    }
}