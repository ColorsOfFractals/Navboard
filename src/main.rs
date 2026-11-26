mod tray;
use dioxus::prelude::*;

const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const NAVBOARD_ICON: Asset = asset!("/assets/Navboard_Icon.png");

mod model;
mod manifest;
mod specs;
mod specs_studio;
mod echo;
mod echo_studio;
mod lens;
mod lens_studio;
mod clock;
mod clock_studio;
mod orbit;
mod instrument_studio;
mod window_chrome;
use window_chrome::WindowChrome;
mod panel_art;
use panel_art::PanelArt;
mod stitch;
mod stitch_studio;
mod forge;
mod forge_studio;
mod atlas;
mod atlas_studio;
mod canvas_studio;
mod storage;
mod panel_menu;
mod context_menu;
use context_menu::{ContextMenu, MenuTarget, MenuAction};
use panel_menu::PanelMenu;
use model::{Panel, PanelKind, Project};

fn main() {
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_window(
                    dioxus::desktop::WindowBuilder::new()
                        .with_title("Navboard")
                .with_window_icon({
                    let rgba = image::load_from_memory_with_format(
                        include_bytes!("../assets/Navboard_Icon.png"),
                        image::ImageFormat::Png,
                    ).expect("Invalid Navboard icon").to_rgba8();

                    let (width, height) = rgba.dimensions();

                    dioxus::desktop::tao::window::Icon::from_rgba(
                        rgba.into_raw(), width, height
                    ).ok()
                })
                        .with_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(1280.0, 960.0))
                        .with_decorations(false)
                        .with_resizable(true)
                )
                .with_menu(None)
        )
        .launch(App);
}


#[derive(Clone, PartialEq)]
struct StartupState {
    projects: Vec<Project>,
    writable: bool,
    status: String,
    tone: &'static str,
}

fn initialize_worlds() -> StartupState {
    match storage::Vault::local().and_then(|vault| vault.load()) {
        Ok(loaded) => {
            let (status, tone) = match loaded.origin {
                storage::LoadOrigin::Empty => (
                    "Local storage ready",
                    "nav-storage-ready",
                ),
                storage::LoadOrigin::Primary => (
                    "All changes saved",
                    "nav-storage-ready",
                ),
                storage::LoadOrigin::Backup => (
                    "Recovered from backup",
                    "nav-storage-warning",
                ),
            };

            StartupState {
                projects: loaded.projects,
                writable: true,
                status: status.into(),
                tone,
            }
        }
        Err(error) => {
            eprintln!("Navboard startup storage error: {error}");

            StartupState {
                projects: Vec::new(),
                writable: false,
                status: format!("Storage locked: {error}"),
                tone: "nav-storage-error",
            }
        }
    }
}

#[component]
fn App() -> Element {
    let startup = use_hook(initialize_worlds);
    let writable = startup.writable;

    let initial_projects = startup.projects.clone();
    let initial_snapshot = startup.projects.clone();

    let mut projects = use_signal(move || initial_projects);
    let mut last_saved = use_signal(move || initial_snapshot);

    let initial_status = startup.status.clone();
    let initial_tone = startup.tone;

    let mut storage_status = use_signal(move || initial_status);
    let mut storage_tone = use_signal(move || initial_tone);

    use_effect(move || {
        if !writable {
            return;
        }

        let snapshot = projects.read().clone();

        // Peek avoids subscribing this effect to its own saved state.
        let unchanged = {
            let saved = last_saved.peek();
            *saved == snapshot
        };

        if unchanged {
            return;
        }

        match storage::Vault::local()
            .and_then(|vault| vault.save(&snapshot))
        {
            Ok(()) => {
                last_saved.set(snapshot);
                storage_status.set("All changes saved".into());
                storage_tone.set("nav-storage-ready");
            }
            Err(error) => {
                eprintln!("Navboard autosave error: {error}");

                storage_status.set(
                    format!("Not saved: {error}")
                );

                storage_tone.set("nav-storage-error");
            }
        }
    });
    let mut active_project = use_signal(|| None::<usize>);
    let mut active_panel = use_signal(|| None::<usize>);

    let mut world_selection_mode = use_signal(|| false);
    let mut selected_worlds = use_signal(Vec::<uuid::Uuid>::new);
    let mut confirm_world_delete = use_signal(|| false);

    let mut context_target = use_signal(|| None::<MenuTarget>);
    let mut orbit_open = use_signal(|| true);
    let mut instrument_open = use_signal(|| None::<String>);

    tray::use_tray(
        orbit_open,
        instrument_open,
        active_project,
        active_panel,
    );
    let current_project = active_project();
    let current_panel = active_panel();

    if orbit_open() {
        return rsx! {
            document::Stylesheet { href: MAIN_CSS }
            document::Stylesheet { href: TAILWIND_CSS }

            div {
                class: "nb-orbit-app",
                WindowChrome {} 

                if let Some(instrument) = instrument_open() {
                    instrument_studio::InstrumentStudio {
                        key: "{instrument}",
                        instrument: instrument.clone(),
                        on_back: move |_| instrument_open.set(None),
                        worlds: projects(),
                        writable,
                        on_assign: move |(destination, name, panel): (Option<uuid::Uuid>, String, Panel)| {
                            if !writable { return; }
                            let mut next = projects.read().clone();
                            let index = if let Some(id) = destination {
                                let Some(index) = next.iter().position(|p| p.id == id) else { return; };
                                index
                            } else {
                                next.push(Project { id: uuid::Uuid::new_v4(), name, panels: Vec::new() });
                                next.len() - 1
                            };
                            let panel_index = next[index].panels.len();
                            next[index].panels.push(panel);
                            match storage::Vault::local().and_then(|vault| vault.save(&next)) {
                                Ok(()) => {
                                    last_saved.set(next.clone());
                                    projects.set(next);
                                    storage_status.set("All changes saved".into());
                                    storage_tone.set("nav-storage-ready");
                                    active_project.set(Some(index));
                                    active_panel.set(Some(panel_index));
                                    instrument_open.set(None);
                                    orbit_open.set(false);
                                }
                                Err(error) => {
                                    storage_status.set(format!("Assignment failed: {error}"));
                                    storage_tone.set("nav-storage-error");
                                }
                            }
                        },
                    }
                } else {
                    orbit::Orbit {
                        on_enter: move |_| orbit_open.set(false),
                        on_instrument: move |name: String| {
                            instrument_open.set(Some(name));
                        },
                    }
                }

                footer {
                    class: "nb-orbit-footer",
                    "NAVBOARD / ORBIT / RING 1.8 | {storage_status}"
                }
            }
        };
    }

    rsx! {
        document::Title { "Navboard" }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        div {
            class: "nav-app flex flex-col min-h-screen",
            onkeydown: move |event| {
                if event.key() == Key::Escape {
                    context_target.set(None);
                }
            },
            WindowChrome {} 

            header { class: "nav-header flex items-center justify-between",
                if current_project.is_none() {
                    button {
                        class: "nb-orbit-return",
                        title: "Return to Orbit",
                        onclick: move |_| {
                            orbit_open.set(true);
                        },
                        "Orbit"
                    }
                }
                div { class: "flex items-center gap-3",
                    div { class: "nav-logo",
                        img {
                            src: NAVBOARD_ICON,
                            alt: "Navboard cosmic cat icon",
                            class: "nav-logo-image",
                        }
                    }
                    div {
                        h1 { class: "nav-brand", "NAVBOARD" }
                        p { class: "nav-brand-caption", "A SPACE FOR IDEAS" }
                    }
                }
                div { class: "nav-version", "EARLY ORBIT  /  0.1.0" }
            }

            main { class: "nav-main flex-1",
                if !writable {
                    section { class: "nav-storage-lock",
                        p { class: "nav-eyebrow", "VAULT PROTECTION" }
                        h2 { "Your worlds are protected." }
                        p {
                            "Navboard could not safely open its local storage."
                        }
                        p { class: "nav-storage-lock-detail",
                            "{storage_status}"
                        }
                        p {
                            "Existing files have not been overwritten. Close Navboard and inspect the Vault before continuing."
                        }
                    }
                } else
                if let Some(project_index) = current_project {
                    if let Some(project) = projects().get(project_index) {
                        if let Some(panel_index) = current_panel {
                            if let Some(panel) = project.panels.get(panel_index) {
                                section { class: "nav-panel-editor",
                                    button {
                                        class: "nav-back",
                                        onclick: move |_| active_panel.set(None),
                                        "← Back to visionboard"
                                    }

                                    p { class: "nav-eyebrow", "PANEL EDITOR" }

                                    input {
                                        class: "nav-panel-title-input",
                                        value: "{panel.title}",
                                        oninput: move |event| {
                                            if let Some(project) = projects.write().get_mut(project_index) {
                                                if let Some(panel) = project.panels.get_mut(panel_index) {
                                                    panel.title = event.value();
                                                }
                                            }
                                        }
                                    }

                                    div { class: "nav-editor-surface",
                                        div { class: "nav-editor-toolbar",
                                            span { "✦  YOUR CREATIVE SPACE" }
                                            span { "{panel.kind.label()} PANEL  /  RING 1.7.5" }
                                        }

                                        if panel.kind == PanelKind::Stitch {
                                            stitch_studio::StitchStudio {
                                                key: "{panel.id}",
                                                title: panel.title.clone(),
                                                initial: match &panel.content { Some(model::PieceContent::Stitch(data)) => Some(data.clone()), _ => None },
                                                on_change: move |content| {
                                                    let changed = projects.peek().get(project_index).and_then(|p| p.panels.get(panel_index)).map(|p| p.content.as_ref() != Some(&content)).unwrap_or(false);
                                                    if !changed { return; }
                                                    if let Some(panel) = projects.write().get_mut(project_index).and_then(|p| p.panels.get_mut(panel_index)) {
                                                        if panel.content.as_ref() != Some(&content) { panel.content = Some(content); }
                                                    }
                                                },
                                            }
                                        } else if panel.kind == PanelKind::Clock {
                                            clock_studio::ClockStudio {
                                                key: "{panel.id}",
                                                title: panel.title.clone(),
                                                initial: match &panel.content { Some(model::PieceContent::Clock(data)) => Some(data.clone()), _ => None },
                                                on_change: move |content| {
                                                    let changed = projects.peek().get(project_index).and_then(|p| p.panels.get(panel_index)).map(|p| p.content.as_ref() != Some(&content)).unwrap_or(false);
                                                    if changed {
                                                        if let Some(panel) = projects.write().get_mut(project_index).and_then(|p| p.panels.get_mut(panel_index)) { panel.content = Some(content); }
                                                    }
                                                },
                                            }
                                        } else if panel.kind == PanelKind::Specs {
                                            specs_studio::SpecsStudio {
                                                key: "{panel.id}",
                                                title: panel.title.clone(),
                                                initial: match &panel.content { Some(model::PieceContent::Specs(data)) => Some(data.clone()), _ => None },
                                                on_change: move |content| {
                                                    let changed = projects.peek().get(project_index).and_then(|p| p.panels.get(panel_index)).map(|p| p.content.as_ref() != Some(&content)).unwrap_or(false);
                                                    if changed {
                                                        if let Some(panel) = projects.write().get_mut(project_index).and_then(|p| p.panels.get_mut(panel_index)) { panel.content = Some(content); }
                                                    }
                                                },
                                            }
                                        } else if panel.kind == PanelKind::Echo {
                                            echo_studio::EchoStudio {
                                                key: "{panel.id}",
                                                initial: match &panel.content { Some(model::PieceContent::Echo(data)) => Some(data.clone()), _ => None },
                                                on_change: move |content| {
                                                    if let Some(panel) = projects.write().get_mut(project_index).and_then(|p| p.panels.get_mut(panel_index)) { panel.content = Some(content); }
                                                },
                                            }
                                        } else if panel.kind == PanelKind::Lens {
                                            lens_studio::LensStudio {
                                                key: "{panel.id}",
                                                title: panel.title.clone(),
                                                initial: match &panel.content { Some(model::PieceContent::Lens(data)) => Some(data.clone()), _ => None },
                                                on_change: move |content| {
                                                    let changed = projects.peek().get(project_index).and_then(|p| p.panels.get(panel_index)).map(|p| p.content.as_ref() != Some(&content)).unwrap_or(false);
                                                    if changed {
                                                        if let Some(panel) = projects.write().get_mut(project_index).and_then(|p| p.panels.get_mut(panel_index)) { panel.content = Some(content); }
                                                    }
                                                },
                                            }
                                         } else if panel.kind == PanelKind::Atlas {
 atlas_studio::AtlasStudio {
 key: "{panel.id}", title: panel.title.clone(),
 initial: match &panel.content { Some(model::PieceContent::Atlas(data)) => Some(data.clone()), _ => None },
 on_change: move |content| {
 let changed = projects.peek().get(project_index).and_then(|p| p.panels.get(panel_index)).map(|p| p.content.as_ref() != Some(&content)).unwrap_or(false);
 if changed { if let Some(panel) = projects.write().get_mut(project_index).and_then(|p| p.panels.get_mut(panel_index)) { panel.content = Some(content); } }
 },
 }
 } else if panel.kind == PanelKind::Forge {
                                            forge_studio::ForgeStudio {
                                                key: "{panel.id}",
                                                title: panel.title.clone(),
                                                initial: match &panel.content { Some(model::PieceContent::Forge(data)) => Some(data.clone()), _ => None },
                                                on_change: move |content| {
                                                    let changed = projects.peek().get(project_index).and_then(|p| p.panels.get(panel_index)).map(|p| p.content.as_ref() != Some(&content)).unwrap_or(false);
                                                    if !changed { return; }
                                                    if let Some(panel) = projects.write().get_mut(project_index).and_then(|p| p.panels.get_mut(panel_index)) {
                                                        if panel.content.as_ref() != Some(&content) { panel.content = Some(content); }
                                                    }
                                                },
                                            }
                                        } else if panel.kind == PanelKind::Canvas {
                                            canvas_studio::CanvasStudio {
                                                key: "{panel.id}",
                                                title: panel.title.clone(),
                                                initial: match &panel.content { Some(model::PieceContent::Canvas(data)) => Some(data.clone()), _ => None },
                                                on_change: move |content| {
                                                    let changed = projects.peek().get(project_index).and_then(|p| p.panels.get(panel_index)).map(|p| p.content.as_ref() != Some(&content)).unwrap_or(false);
                                                    if !changed { return; }
                                                    if let Some(panel) = projects.write().get_mut(project_index).and_then(|p| p.panels.get_mut(panel_index)) {
                                                        if panel.content.as_ref() != Some(&content) { panel.content = Some(content); }
                                                    }
                                                },
                                            }
                                        } else {
textarea {
                                            class: "nav-editor-notes",
                                            placeholder: "Start writing, sketching out an idea, or planning your next creation...",
                                            value: "{panel.notes}",
                                            oninput: move |event| {
                                                if let Some(project) = projects.write().get_mut(project_index) {
                                                    if let Some(panel) = project.panels.get_mut(panel_index) {
                                                        panel.notes = event.value();
                                                    }
                                                }
                                            }
                                        }
}
                                    }

                                    p { class: "nav-editor-hint",
                                        if panel.kind == PanelKind::Stitch {
                                            "Changes are saved automatically to your local Vault."
                                        } else if panel.kind == PanelKind::Forge {
                                            "Changes are saved automatically to your local Vault."
                                        } else if panel.kind == PanelKind::Canvas {
                                            "Changes are saved automatically to your local Vault."
                                        } else {
                                            "Changes are saved automatically to your local Vault."
                                        }
                                    }
                                }
                            }
                        } else {
                            section { class: "nav-workspace",
                                button {
                                    class: "nav-back",
                                    onclick: move |_| active_project.set(None),
                                    "← All projects"
                                }

                                div { class: "nav-board-heading",
                                    div {
                                        p { class: "nav-eyebrow", "PROJECT VISIONBOARD" }
                                        h2 { "{project.name}" }
                                        p { class: "nav-board-subtitle",
                                            "A universe for your ideas, one panel at a time."
                                        }
                                    }

                                    PanelMenu {
                                        on_select: move |kind: PanelKind| {
                                            if let Some(project) = projects.write().get_mut(project_index) {
                                                let number = project.panels.len() + 1;
                                                project.panels.push(Panel {
                                                    id: uuid::Uuid::new_v4(),
                                                    kind,
                                                    title: format!("Untitled {} {}", kind.label(), number),
                                                    notes: String::new(),
                                                     content: None,
                                                });
                                            }
                                        }
                                    }
                                }

                                if project.panels.is_empty() {
                                    div { class: "nav-empty-board",
                                        div { class: "nav-board-orbit", "✦" }
                                        h3 { "An empty universe." }
                                        p { "Your first panel will live here." }
                                        span { "PRESS ADD PANEL TO BEGIN" }
                                    }
                                } else {
                                    div { class: "nav-panel-grid",
                                        for (panel_index, panel) in project.panels.iter().enumerate() {
                                            button {
                                                key: "{panel.id}",
                                                class: "nav-panel-card",
                                                style: "--panel-accent: {panel.kind.accent()};",
                                                onclick: move |_| active_panel.set(Some(panel_index)),
                                                oncontextmenu: {
                                                    let project_id = project.id;
                                                    let panel_id = panel.id;

                                                    move |event: MouseEvent| {
                                                        event.prevent_default();
                                                        event.stop_propagation();

                                                        let point = event.client_coordinates();

                                                        context_target.set(Some(MenuTarget {
                                                            project_id,
                                                            panel_id,
                                                            x: point.x,
                                                            y: point.y,
                                                        }));
                                                    }
                                                },
                                                div { class: "nav-panel-card-top",
                                                    span { "{panel.kind.symbol()}" }
                                                    span { "{panel.kind.label()} / {panel_index + 1}" }
                                                }
                                                PanelArt { kind: panel.kind }
                                                h3 { "{panel.title}" }
                                                p {
                                                    if panel.notes.trim().is_empty() {
                                                        "An idea waiting to happen."
                                                    } else {
                                                        "{panel.notes}"
                                                    }
                                                }
                                                span { class: "nav-panel-open", "Open panel →" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    section { class: "nav-welcome",
                        div { class: "nav-orbit", "✦" }
                        p { class: "nav-eyebrow", "WELCOME TO YOUR CREATIVE SPACE" }
                        h2 { "Give your ideas\nroom to breathe." }
                        p { class: "nav-intro",
                            "Draw, explore, connect, and present. Every great idea starts somewhere."
                        }
                        button {
                            class: "nav-create",
                            onclick: move |_| {
                                let index = projects.read().len();
                                projects.write().push(Project {
                                    id: uuid::Uuid::new_v4(),
                                    name: format!("Untitled World {}", index + 1),
                                    panels: Vec::new(),
                                });
                                active_panel.set(None);
                                active_project.set(Some(index));
                            },
                            span { class: "nav-plus", "+" }
                            "Create Project"
                        }
                    }

                    section { class: "nav-library",

    if confirm_world_delete() {
        div {
            class: "nav-world-confirm-backdrop",

            onclick: move |_| {
                confirm_world_delete.set(false);
            },

            div {
                class: "nav-world-confirm",

                onclick: move |event| {
                    event.stop_propagation();
                },

                p {
                    class: "nav-world-danger-label",
                    "WORLD MANAGEMENT"
                }

                h2 {
                    "Delete {selected_worlds.read().len()} world(s)?"
                }

                p {
                    "All panels inside the selected worlds will be removed from your library."
                }

                p {
                    "This action cannot be undone from the application."
                }

                div {
                    class: "nav-world-confirm-actions",

                    button {
                        class: "nav-world-tool",
                        onclick: move |_| {
                            confirm_world_delete.set(false);
                        },
                        "Cancel"
                    }

                    button {
                        class: "nav-world-delete-button",

                        onclick: move |_| {
                            let ids = selected_worlds.read().clone();

                            active_panel.set(None);
                            active_project.set(None);

                            projects.write().retain(
                                |project| !ids.contains(&project.id)
                            );

                            selected_worlds.set(Vec::new());
                            confirm_world_delete.set(false);
                            world_selection_mode.set(false);
                        },

                        "Delete worlds"
                    }
                }
            }
        }
    }

    div {
        class: "nav-archive-placeholder",

        svg {
            width: "22",
            height: "22",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.5",

            rect {
                x: "3", y: "3",
                width: "18", height: "5",
                rx: "1",
            }

            path { d: "M5 8v12h14V8 M10 12h4" }
        }

        div {
            h4 { "Archived worlds" }
            p { "A resting place for completed projects." }
        }

        span { "COMING LATER" }
    }

                        div { class: "nav-section-heading flex items-center justify-between",
                            div {
    class: "nav-world-heading",
    h3 { "YOUR WORLDS" }

    div {
        class: "nav-world-tools",

        button {
            class: "nav-world-tool",
            onclick: move |_| {
                let next = !world_selection_mode();
                world_selection_mode.set(next);
                selected_worlds.set(Vec::new());
                confirm_world_delete.set(false);
            },
            if world_selection_mode() {
                "Done"
            } else {
                "Select"
            }
        }

        button {
            class: "nav-world-tool",
            disabled: true,
            title: "Archive functionality coming later",

            svg {
                width: "16",
                height: "16",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.6",
                rect {
                    x: "3", y: "3",
                    width: "18", height: "5",
                    rx: "1",
                }
                path { d: "M5 8v12h14V8 M10 12h4" }
            }

            "Archive"
        }

        if world_selection_mode() {
            span {
                class: "nav-world-selected-count",
                "{selected_worlds.read().len()} selected"
            }

            button {
                class: "nav-world-tool nav-world-trash",
                title: "Delete selected worlds",
                disabled: selected_worlds.read().is_empty(),

                onclick: move |_| {
                    confirm_world_delete.set(true);
                },

                svg {
                    width: "18",
                    height: "18",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "1.7",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",

                    path {
                        d: "M3 6h18 M9 6V3h6v3 M5 6l1 15h12l1-15 M10 10v7 M14 10v7"
                    }
                }
            }
        }
    }
}
                            span { "{projects().len()} PROJECTS" }
                        }

                        if projects().is_empty() {
                            div { class: "nav-empty-library",
                                div { class: "nav-empty-symbol", "◇" }
                                p { "No projects yet." }
                                span { "Your first canvas awaits." }
                            }
                        } else {
                            div { class: "nav-project-grid",
                                for (index, project) in projects().iter().enumerate() {
                                    button {
                                        key: "{project.id}",
                                        class: if selected_worlds.read().contains(&project.id) {
    "nav-project-card nav-world-selected"
} else {
    "nav-project-card"
},
                                        onclick: {
    let world_id = project.id;

    move |_| {
        if world_selection_mode() {
            let mut selection = selected_worlds.write();

            if selection.contains(&world_id) {
                selection.retain(|id| *id != world_id);
            } else {
                selection.push(world_id);
            }

            confirm_world_delete.set(false);
        } else {
            active_panel.set(None);
            active_project.set(Some(index));
        }
    }
},
                                        if world_selection_mode() {
    span {
        class: if selected_worlds.read().contains(&project.id) {
            "nav-world-check nav-world-checked"
        } else {
            "nav-world-check"
        },

        if selected_worlds.read().contains(&project.id) {
            "\u{2713}"
        }
    }
}

div { class: "nav-project-icon", "✦" }
                                        h4 { "{project.name}" }
                                        p { "{project.panels.len()} panels" }
                                        span { "Open visionboard →" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(target) = context_target() {
                ContextMenu {
                    target,
                    heading: "PANEL ACTIONS".to_string(),
                    items: context_menu::panel_items(),

                    on_dismiss: move |_| {
                        context_target.set(None);
                    },

                    on_action: move |action: MenuAction| {
                        context_target.set(None);

                        let location = {
                            let worlds = projects.read();

                            worlds.iter().enumerate().find_map(
                                |(project_index, project)| {
                                    if project.id != target.project_id {
                                        return None;
                                    }

                                    project.panels.iter().position(
                                        |panel| panel.id == target.panel_id
                                    ).map(|panel_index| {
                                        (project_index, panel_index)
                                    })
                                }
                            )
                        };

                        if let Some((project_index, panel_index)) = location {
                            match action {
                                MenuAction::Open | MenuAction::Rename => {
                                    active_project.set(Some(project_index));
                                    active_panel.set(Some(panel_index));
                                }

                                MenuAction::Duplicate => {
                                    let mut worlds = projects.write();

                                    if let Some(project) = worlds.get_mut(project_index) {
                                        if let Some(original) = project.panels.get(panel_index) {
                                            let mut duplicate = original.clone();

                                            duplicate.id = uuid::Uuid::new_v4();
                                            duplicate.title = format!(
                                                "{} - Copy",
                                                duplicate.title
                                            );

                                            project.panels.push(duplicate);
                                        }
                                    }
                                }

                                MenuAction::Delete => {
                                    // Confirmation workflow comes next.
                                }
                            }
                        }
                    }
                }
            }

            footer { class: "nav-footer flex items-center justify-between",
                div { class: "nav-storage-status {storage_tone}",
                    span { class: "nav-storage-dot" }
                    span { "{storage_status}" }
                }
                span { "NAVBOARD  /  RING 1.7.5" }
            }
        }
    }
}