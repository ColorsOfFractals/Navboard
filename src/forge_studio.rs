use dioxus::prelude::*;
use uuid::Uuid;
use crate::forge::{Forge, NodeKind, PortDirection};

const NODE_WIDTH: f64 = 220.0;
const BOARD_WIDTH: f64 = 1200.0;
const BOARD_HEIGHT: f64 = 800.0;

#[derive(Clone, Copy, PartialEq)]
struct DragState {
    id: Uuid,
    mouse_x: f64,
    mouse_y: f64,
    node_x: f64,
    node_y: f64,
}

fn kind_label(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Idea => "Idea",
        NodeKind::Resource => "Resource",
        NodeKind::Task => "Task",
        NodeKind::Decision => "Decision",
        NodeKind::Command => "Command",
        NodeKind::Variable => "Variable",
        NodeKind::Condition => "Condition",
        NodeKind::Output => "Output",
    }
}

fn kind_color(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Idea => "#67cbd8",
        NodeKind::Resource => "#a89ce8",
        NodeKind::Task => "#e5a65c",
        NodeKind::Decision => "#ef8181",
        NodeKind::Command => "#67cbd8",
        NodeKind::Variable => "#a89ce8",
        NodeKind::Condition => "#e5a65c",
        NodeKind::Output => "#79c9a1",
    }
}

fn port_position(
    patch: &Forge,
    id: Uuid,
) -> Option<(f64, f64, &'static str)> {
    for node in &patch.nodes {
        if let Some(port) = node.ports.iter().find(|p| p.id == id) {
            let x = match port.direction {
                PortDirection::Input => node.x,
                PortDirection::Output => node.x + NODE_WIDTH,
            };

            return Some((x, node.y + 110.0, kind_color(node.kind)));
        }
    }

    None
}

fn cable_path(
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
) -> String {
    let bend = ((x2 - x1).abs() * 0.5).max(65.0);

    format!(
        "M {x1} {y1} C {} {y1}, {} {y2}, {x2} {y2}",
        x1 + bend,
        x2 - bend
    )
}

#[component]
pub fn ForgeStudio(title: String, initial: Option<Forge>, on_change: EventHandler<crate::model::PieceContent>) -> Element {
    let initial_title = title.clone();
    let mut patch = use_signal(move || initial.unwrap_or_else(|| Forge::new(initial_title)));
    use_effect(move || on_change.call(crate::model::PieceContent::Forge(patch.read().clone())));
    let mut selected_output = use_signal(|| None::<Uuid>);
    let mut dragging = use_signal(|| None::<DragState>);
    let mut palette_open = use_signal(|| false);
    let mut status = use_signal(|| String::from("Ready to patch"));

    let snapshot = patch.read().clone();

    let kinds = [
        NodeKind::Idea,
        NodeKind::Resource,
        NodeKind::Task,
        NodeKind::Decision,
        NodeKind::Command,
        NodeKind::Variable,
        NodeKind::Condition,
        NodeKind::Output,
    ];

    let mut exported = snapshot.clone();
    exported.title = title;
    let preview = exported.export_relationships();

    rsx! {
        div {
            class: "forge-studio",

            onkeydown: move |event| {
                if event.key() == Key::Escape {
                    selected_output.set(None);
                    dragging.set(None);
                    palette_open.set(false);
                    status.set("Ready to patch".into());
                }
            },

            div {
                class: "forge-studio-header",
                div {
                    p { class: "forge-eyebrow", "RELATIONAL COMPOSITION" }
                    h2 { "Forge Studio" }
                    p { "Connect thoughts. Build relationships." }
                }
                span { class: "forge-mode", "PATCH MODE" }
            }

            div {
                class: "forge-toolbar",

                div {
                    class: "forge-palette-anchor",

                    button {
                        class: "forge-add",
                        onclick: move |_| {
                            let next = !palette_open();
                            palette_open.set(next);
                        },
                        "+ Add module"
                    }

                    if palette_open() {
                        div {
                            class: "forge-palette",

                            p { "CHOOSE MODULE TYPE" }

                            for kind in kinds {
                                button {
                                    style: "--node-color: {kind_color(kind)};",
                                    onclick: move |_| {
                                        let count = patch.read().nodes.len();
                                        let x = 40.0 + (count % 4) as f64 * 275.0;
                                        let y = 40.0 + ((count / 4) % 4) as f64 * 185.0;

                                        patch.write().add_node(
                                            kind,
                                            format!("Untitled {}", kind_label(kind)),
                                            x,
                                            y,
                                        );

                                        palette_open.set(false);
                                        status.set("Module added".into());
                                    },

                                    span { class: "forge-palette-dot" }
                                    "{kind_label(kind)}"
                                }
                            }
                        }
                    }
                }

                button {
                    class: "forge-secondary",
                    disabled: selected_output().is_none(),
                    onclick: move |_| {
                        selected_output.set(None);
                        status.set("Connection cancelled".into());
                    },
                    "Cancel patch"
                }

                span {
                    class: "forge-count",
                    "{snapshot.nodes.len()} MODULES / {snapshot.cables.len()} CABLES"
                }
            }

            div {
                class: "forge-status",
                "{status}"
            }

            div {
                class: "forge-viewport",

                div {
                    class: "forge-board",

                    onmousemove: move |event| {
                        if let Some(drag) = dragging() {
                            let point = event.client_coordinates();

                            let x = (drag.node_x + point.x - drag.mouse_x)
                                .clamp(10.0, BOARD_WIDTH - NODE_WIDTH - 10.0);

                            let y = (drag.node_y + point.y - drag.mouse_y)
                                .clamp(10.0, BOARD_HEIGHT - 175.0);

                            patch.write().move_node(drag.id, x, y);
                        }
                    },

                    onmouseup: move |_| {
                        dragging.set(None);
                    },

                    onmouseleave: move |_| {
                        dragging.set(None);
                    },

                    oncontextmenu: move |event| {
                        event.prevent_default();
                        palette_open.set(true);
                    },

                    if snapshot.nodes.is_empty() {
                        div {
                            class: "forge-empty",

                            crate::panel_art::PanelArt {
                                kind: crate::model::PanelKind::Forge
                            }

                            h3 { "Every connection starts somewhere." }
                            p { "Add a module to begin your patch." }
                            p { "Connect output jacks to input jacks." }
                        }
                    }

                    svg {
                        class: "forge-cable-layer",
                        width: "1200",
                        height: "800",

                        for cable in &snapshot.cables {
                            if let (Some((x1, y1, color)), Some((x2, y2, _))) = (
                                port_position(&snapshot, cable.from),
                                port_position(&snapshot, cable.to)
                            ) {
                                {
                                    let path = cable_path(x1, y1, x2, y2);
                                    let cable_id = cable.id;

                                    rsx! {
                                        path {
                                            key: "{cable_id}",
                                            d: "{path}",
                                            fill: "none",
                                            stroke: "{color}",
                                            stroke_width: "3",
                                            stroke_linecap: "round",
                                            class: "forge-cable",

                                            oncontextmenu: move |event| {
                                                event.prevent_default();
                                                event.stop_propagation();

                                                patch.write().disconnect(cable_id);
                                                status.set("Cable disconnected".into());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    for node in &snapshot.nodes {
                        {
                            let id = node.id;
                            let x = node.x;
                            let y = node.y;
                            let color = kind_color(node.kind);
                            let label = kind_label(node.kind);

                            let input = node.ports.iter()
                                .find(|p| p.direction == PortDirection::Input)
                                .map(|p| p.id);

                            let output = node.ports.iter()
                                .find(|p| p.direction == PortDirection::Output)
                                .map(|p| p.id);

                            rsx! {
                                div {
                                    key: "{id}",
                                    class: "forge-node",
                                    style: "left: {x}px; top: {y}px; --node-color: {color};",

                                    oncontextmenu: move |event| {
                                        event.prevent_default();
                                        event.stop_propagation();
                                    },

                                    div {
                                        class: "forge-node-header",

                                        onmousedown: move |event| {
                                            event.prevent_default();

                                            let point = event.client_coordinates();

                                            dragging.set(Some(DragState {
                                                id,
                                                mouse_x: point.x,
                                                mouse_y: point.y,
                                                node_x: x,
                                                node_y: y,
                                            }));
                                        },

                                        span { class: "forge-node-grip", "\u{283F}" }
                                        span { "{label}" }

                                        button {
                                            class: "forge-node-delete",
                                            title: "Remove module",

                                            onmousedown: move |event| {
                                                event.stop_propagation();
                                            },

                                            onclick: move |_| {
                                                patch.write().remove_node(id);
                                                selected_output.set(None);
                                                status.set("Module removed".into());
                                            },

                                            "\u{00D7}"
                                        }
                                    }

                                    input {
                                        class: "forge-node-title",
                                        value: "{node.title}",
                                        placeholder: "Module title",

                                        oninput: move |event| {
                                            if let Some(n) = patch.write()
                                                .nodes.iter_mut()
                                                .find(|n| n.id == id)
                                            {
                                                n.title = event.value();
                                            }
                                        }
                                    }

                                    textarea {
                                        class: "forge-node-content",
                                        value: "{node.content}",
                                        placeholder: "Describe this idea...",

                                        oninput: move |event| {
                                            if let Some(n) = patch.write()
                                                .nodes.iter_mut()
                                                .find(|n| n.id == id)
                                            {
                                                n.content = event.value();
                                            }
                                        }
                                    }

                                    span { class: "forge-input-label", "IN" }
                                    span { class: "forge-output-label", "OUT" }

                                    if let Some(port_id) = input {
                                        button {
                                            class: "forge-jack forge-jack-input",
                                            title: "Input jack",

                                            onclick: move |_| {
                                                if let Some(from) = selected_output() {
                                                    let result = patch.write()
                                                        .connect(from, port_id);

                                                    match result {
                                                        Ok(_) => {
                                                            selected_output.set(None);
                                                            status.set("Patch connected".into());
                                                        }
                                                        Err(error) => {
                                                            status.set(format!(
                                                                "Connection rejected: {error:?}"
                                                            ));
                                                        }
                                                    }
                                                } else {
                                                    status.set(
                                                        "Select an output jack first".into()
                                                    );
                                                }
                                            }
                                        }
                                    }

                                    if let Some(port_id) = output {
                                        button {
                                            class: if selected_output() == Some(port_id) {
                                                "forge-jack forge-jack-output forge-jack-selected"
                                            } else {
                                                "forge-jack forge-jack-output"
                                            },

                                            title: "Output jack",

                                            onclick: move |_| {
                                                selected_output.set(Some(port_id));
                                                status.set(
                                                    "Choose an input jack to complete the patch".into()
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            p {
                class: "forge-help",
                "Drag module headers to move. Output then input to connect. Right-click a cable to disconnect."
            }

            div {
                class: "forge-export",

                p { class: "forge-eyebrow", "LIVE RELATIONSHIP MAP" }

                textarea {
                    class: "forge-preview",
                    readonly: true,
                    value: "{preview}",
                    rows: "7",
                }

                p {
                    class: "forge-lab-note",
                    "Content follows the piece. Unassigned sessions are temporary."
                }
            }
        }
    }
}