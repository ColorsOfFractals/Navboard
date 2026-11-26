use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
struct Instrument {
    name: &'static str,
    description: &'static str,
    color: &'static str,
    angle: f64,
    radius: f64,
    icon: &'static str,
    built: bool,
}

const INSTRUMENTS: [Instrument; 9] = [
    Instrument {
        name: "Text",
        description: "Express ideas through language.",
        color: "#a89ce8",
        angle: -90.0,
        radius: 112.0,
        icon: "M-8 -16H5L9 -12V5H-8Z M-4 -9H4 M-4 -4H4 M-4 1H1",
        built: true,
    },
    Instrument {
        name: "Canvas",
        description: "Sketch, draw and visualize.",
        color: "#67cbd8",
        angle: -30.0,
        radius: 112.0,
        icon: "M-10 5L-7 -3L6 -16L12 -10L-1 3Z M3 -13L9 -7",
        built: true,
    },
    Instrument {
        name: "Stitch",
        description: "Arrange steps into procedures.",
        color: "#79c9a1",
        angle: 30.0,
        radius: 112.0,
        icon: "M-3 -13H11 M-3 -5H11 M-3 3H11 M-10 -13H-8 M-10 -5H-8 M-10 3H-8",
        built: true,
    },
    Instrument {
        name: "Forge",
        description: "Connect ideas, systems and resources.",
        color: "#e5a65c",
        angle: 90.0,
        radius: 112.0,
        icon: "M-12 -15H-1V-4H-12Z M1 -1H12V10H1Z M-7 -4V4H1",
        built: true,
    },
    Instrument {
        name: "Clock",
        description: "Explore events, timelines and durations.",
        color: "#8299ec",
        angle: 150.0,
        radius: 112.0,
        icon: "M0 -16A11 11 0 1 0 0 6A11 11 0 1 0 0 -16 M0 -13V-5L-5 0",
        built: true,
    },
    Instrument {
        name: "Specs",
        description: "Gather structured project specifications.",
        color: "#d5c57b",
        angle: 210.0,
        radius: 112.0,
        icon: "M-10 -15H10V6H-10Z M-10 -7H10 M-2 -15V6 M-10 0H10",
        built: true,
    },
    Instrument {
        name: "Echo",
        description: "Transcribe microphone and system audio locally.",
        color: "#ef8181",
        angle: -90.0,
        radius: 207.0,
        icon: "M-12 -8V-2 M-7 -12V2 M-2 -17V7 M3 -11V1 M8 -14V4 M13 -8V-2",
        built: true,
    },
    Instrument {
        name: "Lens",
        description: "Collect photographs and visual evidence.",
        color: "#d9a6d8",
        angle: 30.0,
        radius: 207.0,
        icon: "M-13 -12H-6L-3 -16H4L7 -12H13V6H-13Z M0 -8A5 5 0 1 0 0 2A5 5 0 1 0 0 -8",
        built: true,
    },
    Instrument {
        name: "Atlas",
        description: "Map equipment and infrastructure.",
        color: "#8db8a8",
        angle: 150.0,
        radius: 207.0,
        icon: "M-10 -15V6 M-4 -13V6 M2 -15V6 M8 -14L14 6",
        built: true,
    },
];

#[component]
pub fn Orbit(
    on_enter: EventHandler<()>,
    on_instrument: EventHandler<String>,
) -> Element {
    let mut selected = use_signal(|| None::<usize>);
    let active = selected().map(|index| INSTRUMENTS[index]);

    let heading = active.map(|item| item.name)
        .unwrap_or("Navboard");

    let description = active.map(|item| item.description)
        .unwrap_or("Enter your Worlds. Give your ideas room to breathe.");

    let accent = active.map(|item| item.color)
        .unwrap_or("#79c9a1");

    let category = match active {
        Some(item) if item.radius > 150.0 => "CAPTURE & COLLECT",
        Some(_) => "CONSTRUCT & EXPRESS",
        None => "YOUR CREATIVE WORKSHOP",
    };

    rsx! {
        section {
            class: "nb-orbit-shell",

            div {
                class: "nb-orbit-heading",
                span { "N A V B O A R D" }
                h1 { "Orbit" }
                p { "Every idea begins somewhere." }
            }

            div {
                class: "nb-orbit-map",

                svg {
                    class: "nb-orbit-space",
                    view_box: "0 0 520 520",

                    defs {
                        radialGradient {
                            id: "nb-orbit-background",
                            stop {
                                offset: "0%",
                                stop_color: "#203c34",
                            }
                            stop {
                                offset: "100%",
                                stop_color: "#0b1119",
                            }
                        }
                    }

                    circle {
                        cx: "260",
                        cy: "260",
                        r: "250",
                        fill: "url(#nb-orbit-background)",
                    }

                    for i in 0..100 {
                        circle {
                            cx: "{12.0 + (i as f64 * 137.51) % 496.0}",
                            cy: "{12.0 + (i as f64 * 91.73) % 496.0}",
                            r: if i % 7 == 0 { "1.4" } else { "0.65" },
                            fill: "#b3d8cf",
                            opacity: "0.27",
                        }
                    }

                    circle {
                        cx: "260",
                        cy: "260",
                        r: "112",
                        fill: "none",
                        stroke: "#79c9a1",
                        stroke_width: "1.2",
                        opacity: "0.5",
                    }

                    circle {
                        class: "nb-orbit-outer-track",
                        cx: "260",
                        cy: "260",
                        r: "207",
                        fill: "none",
                        stroke: "#8db8a8",
                        stroke_width: "1",
                        stroke_dasharray: "4 7",
                        opacity: "0.5",
                    }

                    path {
                        d: "M260 53 L439.27 363.5 L80.73 363.5 Z",
                        stroke: "#8db8a8",
                        opacity: "0.16",
                        fill: "none",
                    }

                    for item in INSTRUMENTS {
                        line {
                            x1: "260",
                            y1: "260",
                            x2: "{260.0 + item.angle.to_radians().cos() * item.radius}",
                            y2: "{260.0 + item.angle.to_radians().sin() * item.radius}",
                            stroke: item.color,
                            opacity: "0.12",
                        }
                    }
                }

                button {
                    class: "nb-orbit-core",
                    title: "Enter Worlds",
                    aria_label: "Enter Navboard Worlds",
                    onclick: move |_| on_enter.call(()),

                    img {
                        src: crate::NAVBOARD_ICON,
                        alt: "",
                        draggable: "false",
                    }

                    strong { "Navboard" }
                    span { "WORLDS" }
                }

                for (index, item) in INSTRUMENTS.into_iter().enumerate() {
                    button {
                        key: "{item.name}",
                        class: if selected() == Some(index) {
                            "nb-orbit-node nb-orbit-selected"
                        } else {
                            "nb-orbit-node"
                        },

                        style: format!(
                            "--orb-color:{};left:{}%;top:{}%;width:{}%;",
                            item.color,
                            50.0 + item.angle.to_radians().cos()
                                * item.radius / 5.2,
                            50.0 + item.angle.to_radians().sin()
                                * item.radius / 5.2,
                            if item.radius > 150.0 { 12.0 } else { 10.8 },
                        ),

                        title: item.description,
                        aria_label: item.name,

                        onclick: move |_| {
                            selected.set(Some(index));
                            on_instrument.call(item.name.to_string());
                        },

                        svg {
                            view_box: "-18 -20 36 32",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "1.8",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",

                            path { d: item.icon }
                        }

                        span { "{item.name}" }
                    }
                }
            }

            div {
                class: "nb-orbit-legend",

                span {
                    i { class: "nb-orbit-inner-dot" }
                    "Construct & Express"
                }

                span {
                    i { class: "nb-orbit-outer-dot" }
                    "Capture & Collect"
                }
            }

            div {
                class: "nb-orbit-inspector",
                style: "--orb-color:{accent};",

                span {
                    class: "nb-orbit-category",
                    "{category}"
                }

                h2 { "{heading}" }
                p { "{description}" }

                if let Some(item) = active {
                    span {
                        class: "nb-orbit-status",

                        if item.built {
                            "Available inside Worlds"
                        } else {
                            "Instrument under construction"
                        }
                    }
                } else {
                    button {
                        class: "nb-orbit-enter",
                        onclick: move |_| on_enter.call(()),
                        "Enter Worlds"
                    }
                }
            }
        }
    }
}