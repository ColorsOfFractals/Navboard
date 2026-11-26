use dioxus::prelude::*;
use uuid::Uuid;
use crate::stitch::Stitch;

#[component]
pub fn StitchStudio(title: String, initial: Option<Stitch>, on_change: EventHandler<crate::model::PieceContent>) -> Element {
    let initial_title = title.clone();
    let mut process = use_signal(move || initial.unwrap_or_else(|| Stitch::new(initial_title)));
    use_effect(move || on_change.call(crate::model::PieceContent::Stitch(process.read().clone())));
    let mut draft = use_signal(String::new);
    let mut dragging = use_signal(|| None::<Uuid>);
    let mut checklist = use_signal(|| false);

    let snapshot = process.read().clone();

    let mut export = snapshot.clone();
    export.title = title;

    let preview = if checklist() {
        export.export_checklist()
    } else {
        export.export_markdown()
    };

    rsx! {
        div {
            class: "stitch-studio",

            div {
                class: "stitch-studio-header",
                div {
                    p { class: "stitch-eyebrow", "SEQUENTIAL COMPOSITION" }
                    h2 { "Stitch Studio" }
                    p { "Capture thoughts. Arrange steps. Build a process." }
                }
                span { class: "stitch-mode", "DESIGN MODE" }
            }

            div {
                class: "stitch-section-heading",
                span { "PROCESS SEQUENCE" }
                span { "{snapshot.steps.len()} STEPS" }
            }

            if snapshot.steps.is_empty() {
                div {
                    class: "stitch-empty",
                    crate::panel_art::PanelArt {
                        kind: crate::model::PanelKind::Stitch
                    }
                    h3 { "Every process begins with a step." }
                    p { "Capture ideas in any order. Arrange them later." }
                }
            }

            div {
                class: "stitch-sequence",

                for (index, step) in snapshot.steps.iter().enumerate() {
                    {
                        let id = step.id;
                        let total = snapshot.steps.len();
                        let number = format!("{:02}", index + 1);

                        rsx! {
                            div {
                                key: "{id}",
                                class: if dragging() == Some(id) {
                                    "stitch-step stitch-dragging"
                                } else {
                                    "stitch-step"
                                },

                                ondragover: move |event| {
                                    event.prevent_default();
                                },

                                ondrop: move |event| {
                                    event.prevent_default();

                                    if let Some(source) = dragging() {
                                        process.write().move_to(source, index);
                                    }

                                    dragging.set(None);
                                },

                                span {
                                    class: "stitch-grip",
                                    draggable: "true",
                                    title: "Drag to rearrange",
                                    ondragstart: move |_| {
                                        dragging.set(Some(id));
                                    },
                                    ondragend: move |_| {
                                        dragging.set(None);
                                    },
                                    "\u{283F}"
                                }

                                span {
                                    class: "stitch-number",
                                    "{number}"
                                }

                                textarea {
                                    class: "stitch-step-input",
                                    rows: "2",
                                    value: "{step.text}",
                                    placeholder: "Describe this step...",
                                    oninput: move |event| {
                                        // Allow temporary empty text while editing.
                                        if let Some(item) = process.write()
                                            .steps.iter_mut()
                                            .find(|s| s.id == id)
                                        {
                                            item.text = event.value();
                                        }
                                    }
                                }

                                div {
                                    class: "stitch-step-actions",

                                    button {
                                        title: "Move up",
                                        disabled: index == 0,
                                        onclick: move |_| {
                                            if index > 0 {
                                                process.write().move_to(id, index - 1);
                                            }
                                        },
                                        "\u{2191}"
                                    }

                                    button {
                                        title: "Move down",
                                        disabled: index + 1 >= total,
                                        onclick: move |_| {
                                            process.write().move_to(id, index + 1);
                                        },
                                        "\u{2193}"
                                    }

                                    button {
                                        title: "Duplicate step",
                                        onclick: move |_| {
                                            process.write().duplicate(id);
                                        },
                                        "\u{25A3}"
                                    }

                                    button {
                                        class: "stitch-remove",
                                        title: "Remove step",
                                        onclick: move |_| {
                                            process.write().remove(id);
                                        },
                                        "\u{00D7}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            form {
                class: "stitch-capture",

                onsubmit: move |event| {
                    event.prevent_default();

                    let text = draft().trim().to_string();

                    if process.write().add(text).is_some() {
                        draft.set(String::new());
                    }
                },

                input {
                    class: "stitch-capture-input",
                    placeholder: "Jot down another step...",
                    value: "{draft}",
                    oninput: move |event| {
                        draft.set(event.value());
                    }
                }

                button {
                    class: "stitch-add",
                    r#type: "submit",
                    disabled: draft().trim().is_empty(),
                    "+ Add step"
                }
            }

            div {
                class: "stitch-export",

                div {
                    class: "stitch-section-heading",
                    span { "LIVE TEMPLATE PREVIEW" }

                    div {
                        class: "stitch-format-switch",

                        button {
                            class: if !checklist() { "selected" } else { "" },
                            onclick: move |_| checklist.set(false),
                            "Numbered"
                        }

                        button {
                            class: if checklist() { "selected" } else { "" },
                            onclick: move |_| checklist.set(true),
                            "Checklist"
                        }
                    }
                }

                textarea {
                    class: "stitch-preview",
                    readonly: true,
                    value: "{preview}",
                    rows: "8",
                }

                p {
                    class: "stitch-lab-note",
                    "Content follows the piece. Unassigned sessions are temporary."
                }
            }
        }
    }
}