use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanelKind {
    Text,
    Canvas,
    Stitch,
    Forge,
    Clock,
    Specs,
    Lens,
    Echo,
    Atlas,
}

impl PanelKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Text => "Text",
            Self::Canvas => "Canvas",
            Self::Stitch => "Stitch",
            Self::Forge => "Forge",
            Self::Lens => "Lens", Self::Echo => "Echo", Self::Atlas => "Atlas",
            Self::Specs => "Specs",
            Self::Clock => "Clock",
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Panel {
    pub id: Uuid,
    pub title: String,
    pub notes: String,
    pub kind: PanelKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<PieceContent>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub panels: Vec<Panel>,
}
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum PieceContent {
    Canvas(crate::canvas_studio::CanvasData),
    Stitch(crate::stitch::Stitch),
    Forge(crate::forge::Forge),
    Clock(crate::clock::Clock),
    Specs(crate::specs::Specs),
    Lens(crate::lens::Lens),
    Echo(crate::echo::Echo),
    Atlas(crate::atlas::Atlas),
}

impl PanelKind {
    pub fn accent(self) -> &'static str {
        match self {
            Self::Text => "#a89ce8",
            Self::Canvas => "#67cbd8",
            Self::Stitch => "#79c9a1",
            Self::Forge => "#e5a65c",
            Self::Lens => "#d9a6d8", Self::Echo => "#ef8181", Self::Atlas => "#8db8a8",
            Self::Specs => "#d5c57b",
            Self::Clock => "#8299ec",
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Self::Text => "\u{00B6}",
            Self::Canvas => "\u{270E}",
            Self::Stitch => "\u{2261}",
            Self::Forge => "\u{2318}", Self::Clock => "\u{25F7}", Self::Specs => "\u{25A4}", Self::Lens => "\u{25CE}", Self::Echo => "\u{266B}", Self::Atlas => "\u{25C7}",
        }
    }
}