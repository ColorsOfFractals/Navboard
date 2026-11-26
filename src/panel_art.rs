use dioxus::prelude::*;
use crate::model::PanelKind;

const TEXT_ART: Asset = asset!("/assets/panel-art/text.svg");
const CANVAS_ART: Asset = asset!("/assets/panel-art/canvas.svg");
const STITCH_ART: Asset = asset!("/assets/panel-art/stitch.svg");
const FORGE_ART: Asset = asset!("/assets/panel-art/forge.svg");

const CLOCK_ART: Asset = asset!("/assets/panel-art/clock.svg");
const SPECS_ART: Asset = asset!("/assets/panel-art/specs.svg");
const LENS_ART: Asset = asset!("/assets/panel-art/lens.svg");
#[component]
pub fn PanelArt(kind: PanelKind) -> Element {
    let illustration = match kind {
        PanelKind::Text => TEXT_ART,
        PanelKind::Canvas => CANVAS_ART,
        PanelKind::Stitch => STITCH_ART,
        PanelKind::Atlas => asset!("/assets/panel-art/atlas.svg"), PanelKind::Forge => FORGE_ART, PanelKind::Lens => LENS_ART, PanelKind::Echo => asset!("/assets/panel-art/echo.svg"), PanelKind::Specs => SPECS_ART, PanelKind::Clock => CLOCK_ART,
    };

    rsx! {
        div {
            class: "nav-panel-art",
            img {
                src: illustration,
                alt: "{kind.label()} illustration",
                draggable: "false",
            }
        }
    }
}