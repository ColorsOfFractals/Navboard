use dioxus::prelude::*;
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq)]
enum DrawingTool {
    Pen,
    Eraser,
}

#[derive(Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
enum Paper {
    Dark,
    White,
    Grid,
}

impl Paper {
    fn class(self) -> &'static str {
        match self {
            Self::Dark => "canvas-paper canvas-dark",
            Self::White => "canvas-paper canvas-white",
            Self::Grid => "canvas-paper canvas-grid",
        }
    }
}

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct Stroke {
    id: Uuid,
    color: String,
    width: f64,
    erase: bool,
    points: Vec<(f64, f64)>,
}

impl Stroke {
    fn path(&self) -> String {
        let mut path = String::new();

        for (index, (x, y)) in self.points.iter().enumerate() {
            if index == 0 {
                path.push_str(&format!("M {x:.2} {y:.2}"));
            } else {
                path.push_str(&format!(" L {x:.2} {y:.2}"));
            }
        }

        if self.points.len() == 1 {
            let (x, y) = self.points[0];
            path.push_str(&format!(" L {:.2} {y:.2}", x + 0.01));
        }

        path
    }
}

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CanvasData {
    strokes: Vec<Stroke>,
    paper: Paper,
}

#[component]
pub fn CanvasStudio(title: String, initial: Option<CanvasData>, on_change: EventHandler<crate::model::PieceContent>) -> Element {
    let initial = initial.unwrap_or(CanvasData { strokes: Vec::new(), paper: Paper::Dark });
    let initial_paper = initial.paper;
    let mut strokes = use_signal(move || initial.strokes);
    let mut redo = use_signal(Vec::<Stroke>::new);
    let mut drawing = use_signal(|| None::<Uuid>);
    let mut tool = use_signal(|| DrawingTool::Pen);
    let mut color = use_signal(|| String::from("#67cbd8"));
    let mut width = use_signal(|| 4.0_f64);
    let mut paper = use_signal(|| initial_paper);
    use_effect(move || on_change.call(crate::model::PieceContent::Canvas(CanvasData {
        strokes: strokes.read().clone(), paper: paper(),
    })));
    let mut confirm_clear = use_signal(|| false);

    let palette = [
        "#e8edf5",
        "#67cbd8",
        "#a89ce8",
        "#79c9a1",
        "#e5a65c",
        "#ef8181",
        "#111923",
    ];

    let snapshot = strokes.read().clone();

    let mut mask_ids = Vec::new();

    for (index, stroke) in snapshot.iter().enumerate() {
        if !stroke.erase {
            mask_ids.push((index, format!("mask-{}", stroke.id)));
        }
    }

    rsx! {
        div {
            class: "canvas-studio",

            onkeydown: move |event| {
                if event.key() == Key::Escape {
                    drawing.set(None);
                    confirm_clear.set(false);
                }
            },

            div {
                class: "canvas-studio-header",

                div {
                    p { class: "canvas-eyebrow", "VISUAL COMPOSITION" }
                    h2 { "Canvas Studio" }
                    p { "Sketch thoughts. Give ideas shape." }
                }

                span { class: "canvas-mode", "DRAW MODE" }
            }

            div {
                class: "canvas-toolbar",

                button {
                    class: if tool() == DrawingTool::Pen {
                        "canvas-tool canvas-active"
                    } else {
                        "canvas-tool"
                    },

                    onclick: move |_| tool.set(DrawingTool::Pen),
                    title: "Pen",

                    svg {
                        width: "18",
                        height: "18",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "1.7",
                        path {
                            d: "M4 16L16 4l4 4L8 20H4v-4z M13 7l4 4"
                        }
                    }

                    "Pen"
                }

                button {
                    class: if tool() == DrawingTool::Eraser {
                        "canvas-tool canvas-active"
                    } else {
                        "canvas-tool"
                    },

                    onclick: move |_| tool.set(DrawingTool::Eraser),
                    title: "Eraser",

                    svg {
                        width: "18",
                        height: "18",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "1.7",
                        path {
                            d: "M3 14L13 4a2 2 0 0 1 3 0l5 5a2 2 0 0 1 0 3l-8 8H9z M8 9l9 9 M13 20h9"
                        }
                    }

                    "Eraser"
                }

                span { class: "canvas-tool-divider" }

                button {
                    class: "canvas-tool",
                    title: "Undo last stroke",
                    disabled: snapshot.is_empty(),

                    onclick: move |_| {
                        drawing.set(None);
                        let last = strokes.write().pop();

                        if let Some(stroke) = last {
                            redo.write().push(stroke);
                        }
                    },

                    "\u{21B6} Undo"
                }

                button {
                    class: "canvas-tool",
                    title: "Redo stroke",
                    disabled: redo.read().is_empty(),

                    onclick: move |_| {
                        drawing.set(None);
                        let last = redo.write().pop();

                        if let Some(stroke) = last {
                            strokes.write().push(stroke);
                        }
                    },

                    "\u{21B7} Redo"
                }

                button {
                    class: "canvas-tool canvas-clear",
                    disabled: snapshot.is_empty(),
                    onclick: move |_| confirm_clear.set(true),

                    svg {
                        width: "17",
                        height: "17",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "1.7",
                        path {
                            d: "M3 6h18 M9 6V3h6v3 M5 6l1 15h12l1-15 M10 10v7 M14 10v7"
                        }
                    }

                    "Clear"
                }
            }

            div {
                class: "canvas-properties",

                div {
                    class: "canvas-palette",

                    for swatch in palette {
                        button {
                            class: if color() == swatch {
                                "canvas-swatch canvas-swatch-selected"
                            } else {
                                "canvas-swatch"
                            },

                            style: "background: {swatch};",
                            title: "{swatch}",

                            onclick: move |_| {
                                color.set(swatch.into());
                                tool.set(DrawingTool::Pen);
                            }
                        }
                    }

                    input {
                        class: "canvas-custom-color",
                        r#type: "color",
                        value: "{color}",
                        title: "Custom brush color",

                        oninput: move |event| {
                            color.set(event.value());
                            tool.set(DrawingTool::Pen);
                        }
                    }
                }

                div {
                    class: "canvas-brush",

                    span { "BRUSH" }

                    input {
                        r#type: "range",
                        min: "1",
                        max: "40",
                        step: "1",
                        value: "{width}",

                        oninput: move |event| {
                            if let Ok(value) = event.value().parse::<f64>() {
                                width.set(value);
                            }
                        }
                    }

                    span { "{width()} px" }
                }
            }

            div {
                class: "canvas-paper-tools",

                span { "PAPER" }

                for (option, label) in [
                    (Paper::Dark, "Dark"),
                    (Paper::White, "White"),
                    (Paper::Grid, "Grid"),
                ] {
                    button {
                        class: if paper() == option {
                            "canvas-tool canvas-active"
                        } else {
                            "canvas-tool"
                        },

                        onclick: move |_| paper.set(option),
                        "{label}"
                    }
                }
            }

            div {
                class: "canvas-viewport",

                svg {
                    class: if tool() == DrawingTool::Eraser {
    format!("{} canvas-eraser-cursor", paper().class())
} else {
    paper().class().to_string()
},
                    width: "1000",
                    height: "600",
                    view_box: "0 0 1000 600",

                    onmousedown: move |event| {
                        event.prevent_default();

                        let point = event.element_coordinates();
                        let id = Uuid::new_v4();

                        let stroke = Stroke {
                            id,
                            color: color(),
                            width: width(),
                            erase: tool() == DrawingTool::Eraser,
                            points: vec![(point.x, point.y)],
                        };

                        strokes.write().push(stroke);
                        redo.set(Vec::new());
                        drawing.set(Some(id));
                    },

                    onmousemove: move |event| {
                        if let Some(id) = drawing() {
                            let point = event.element_coordinates();

                            if let Some(stroke) = strokes.write()
                                .iter_mut()
                                .find(|s| s.id == id)
                            {
                                stroke.points.push((point.x, point.y));
                            }
                        }
                    },

                    onmouseup: move |_| drawing.set(None),
                    onmouseleave: move |_| drawing.set(None),

                    oncontextmenu: move |event| {
                        event.prevent_default();
                        drawing.set(None);
                    },

                    defs {
                        for (index, mask_id) in &mask_ids {
                            mask {
                                id: "{mask_id}",
                                x: "0",
                                y: "0",
                                width: "1000",
                                height: "600",
                                mask_units: "userSpaceOnUse",

                                rect {
                                    x: "0",
                                    y: "0",
                                    width: "1000",
                                    height: "600",
                                    fill: "white",
                                }

                                for eraser in snapshot.iter().skip(*index + 1).filter(|s| s.erase) {
                                    path {
                                        d: "{eraser.path()}",
                                        stroke: "black",
                                        stroke_width: "{eraser.width}",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        fill: "none",
                                    }
                                }
                            }
                        }
                    }

                    for (index, mask_id) in &mask_ids {
                        {
                            let stroke = &snapshot[*index];
                            let mask_url = format!("url(#{mask_id})");

                            rsx! {
                                path {
                                    key: "{stroke.id}",
                                    d: "{stroke.path()}",
                                    stroke: "{stroke.color}",
                                    stroke_width: "{stroke.width}",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    fill: "none",
                                    mask: "{mask_url}",
                                    pointer_events: "none",
                                }
                            }
                        }
                    }
                }
            }

            div {
                class: "canvas-status",

                span { "{snapshot.len()} STROKES" }
                span { "1000 x 600" }

                span {
                    if tool() == DrawingTool::Pen {
                        "PEN ACTIVE"
                    } else {
                        "ERASER ACTIVE"
                    }
                }
            }

            p {
                class: "canvas-lab-note",
                "Content follows the piece. Unassigned sessions are temporary."
            }

            if confirm_clear() {
                div {
                    class: "nav-world-confirm-backdrop",

                    onclick: move |_| confirm_clear.set(false),

                    div {
                        class: "nav-world-confirm",

                        onclick: move |event| {
                            event.stop_propagation();
                        },

                        p {
                            class: "nav-world-danger-label",
                            "CANVAS STUDIO"
                        }

                        h2 { "Clear this drawing?" }

                        p {
                            "All strokes and drawing history will be removed."
                        }

                        div {
                            class: "nav-world-confirm-actions",

                            button {
                                class: "canvas-tool",
                                onclick: move |_| confirm_clear.set(false),
                                "Cancel"
                            }

                            button {
                                class: "nav-world-delete-button",

                                onclick: move |_| {
                                    drawing.set(None);
                                    strokes.set(Vec::new());
                                    redo.set(Vec::new());
                                    confirm_clear.set(false);
                                },

                                "Clear canvas"
                            }
                        }
                    }
                }
            }
        }
    }
}