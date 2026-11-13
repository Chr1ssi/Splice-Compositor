//! The audio panel's content: the default output and microphone with their volume, the others
//! listed below once the default's name is clicked (and hidden again by another click), and the
//! applications that play. A device's row has its name (choosing it, or showing the others), its
//! level (which mutes it) and a slider; each device sits in a card of its own.
use crate::{
    Align, Measure, Node, Rect, Scene, TextStyle, Theme,
    bar::{BUTTON_HEIGHT, button_shape},
    i18n::Language,
    panel::{Content, PanelTarget},
    theme::ROW_RADIUS,
};

/// An output, microphone or application stream.
#[derive(Clone, Debug, PartialEq)]
pub struct NodeView {
    pub id: u32,
    pub label: String,
    /// `(volume, muted)`; `None` until it reported them.
    pub audio: Option<(f32, bool)>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AudioModel {
    pub outputs: Vec<NodeView>,
    pub inputs: Vec<NodeView>,
    pub applications: Vec<NodeView>,
    pub default_output: Option<u32>,
    pub default_input: Option<u32>,
    pub outputs_expanded: bool,
    pub inputs_expanded: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioTarget {
    /// Make the node the default.
    Select(u32),
    Mute(u32),
    /// The node's slider: pressed and dragged, it sets the volume.
    Slider(u32),
    ToggleOutputs,
    ToggleInputs,
}

const SECTION_SPACING: f32 = 16.0;
const SPACING: f32 = 6.0;
const TEXT: TextStyle = TextStyle::new(14.0);
const LEVEL_WIDTH: f32 = 78.0;
const SLIDER_HEIGHT: f32 = 26.0;
/// A device's card: its padding, and the height of the row with the name.
const CARD_PADDING: f32 = 8.0;
const NAME_HEIGHT: f32 = 32.0;
/// Space left of a name.
const NAME_INSET: f32 = 12.0;
/// The width of the mark at the end of a name that shows or hides the other devices.
const MARK_WIDTH: f32 = 24.0;

/// The volume a press at `x` on a slider at `rect` means (where the handle's middle would be).
pub fn slider_value(rect: Rect, x: f32) -> f32 {
    let travel = (rect.w - 14.0).max(1.0);
    ((x - rect.x - 7.0) / travel).clamp(0.0, 1.0)
}

/// A button whose nodes are faded when it is disabled; it reacts only when enabled.
#[allow(clippy::too_many_arguments)]
fn button(
    content: &mut Content,
    theme: &Theme,
    rect: Rect,
    text: &str,
    selected: bool,
    enabled: bool,
    target: PanelTarget,
    hover: Option<&PanelTarget>,
) {
    let mut scene: Scene<()> = Scene::new(rect.w, 0.0);
    button_shape(&mut scene, theme, rect, text, false, selected, false, enabled && hover == Some(&target));
    let fade = if enabled { 1.0 } else { 0.45 };
    content.nodes.extend(scene.nodes.into_iter().map(|node| node.faded(fade)));
    if enabled {
        content.target(rect, target);
    }
}

/// A device's card at `y`; returns its height. A click on the name does `action`; `mark` (if not
/// empty) stands at the name's end.
#[allow(clippy::too_many_arguments)]
fn control(
    content: &mut Content,
    theme: &Theme,
    node: &NodeView,
    y: f32,
    width: f32,
    selected: bool,
    action: Option<AudioTarget>,
    mark: &str,
    hover: Option<&PanelTarget>,
    language: Language,
) -> f32 {
    let height = 2.0 * CARD_PADDING + NAME_HEIGHT + if node.audio.is_some() { 4.0 + SLIDER_HEIGHT } else { 0.0 };
    content.card(Rect::new(0.0, y, width, height), theme.glass_surface());
    let name = if selected { format!("✓ {}", node.label) } else { node.label.clone() };
    let level_rect =
        Rect::new(width - CARD_PADDING - LEVEL_WIDTH, y + CARD_PADDING + (NAME_HEIGHT - BUTTON_HEIGHT) / 2.0, LEVEL_WIDTH, BUTTON_HEIGHT);
    let row = Rect::new(CARD_PADDING, y + CARD_PADDING, level_rect.x - 4.0 - CARD_PADDING, NAME_HEIGHT);
    if let Some(action) = action.map(PanelTarget::Audio) {
        if hover == Some(&action) {
            content.push(Node::Rect { rect: row, radius: ROW_RADIUS, fill: theme.surface, border: None });
        }
        content.target(row, action);
    }
    let mark_width = if mark.is_empty() { 0.0 } else { MARK_WIDTH };
    let label = Rect::new(row.x + NAME_INSET, row.y, (row.w - NAME_INSET - mark_width).max(0.0), row.h);
    if !mark.is_empty() {
        let rect = Rect::new(row.right() - MARK_WIDTH, row.y, MARK_WIDTH, row.h);
        content.push(Node::Label { rect, text: mark.to_owned(), style: TEXT, color: theme.muted, align: Align::Center });
    }
    content.push(Node::Label { rect: label, text: name, style: TEXT.bold(selected), color: theme.text, align: Align::Left });
    let level = match node.audio {
        Some((_, true)) => language.tr("Stumm").to_owned(),
        Some((volume, false)) => format!("{} %", (volume * 100.0).round()),
        None => "…".to_owned(),
    };
    button(content, theme, level_rect, &level, false, node.audio.is_some(), PanelTarget::Audio(AudioTarget::Mute(node.id)), hover);
    let Some((volume, _)) = node.audio else { return height };
    let slider = Rect::new(NAME_INSET, row.bottom() + 4.0, width - 2.0 * NAME_INSET, SLIDER_HEIGHT);
    let position = volume.clamp(0.0, 1.0);
    let track = Rect::new(slider.x, slider.y + (SLIDER_HEIGHT - 6.0) / 2.0, slider.w, 6.0);
    content.push(Node::Rect { rect: track, radius: 3.0, fill: theme.surface, border: None });
    content.push(Node::Rect {
        rect: Rect::new(slider.x, track.y, position * slider.w, 6.0),
        radius: 3.0,
        fill: theme.accent,
        border: None,
    });
    let handle = Rect::new(slider.x + position * (slider.w - 14.0), slider.y + (SLIDER_HEIGHT - 14.0) / 2.0, 14.0, 14.0);
    content.push(Node::Rect { rect: handle, radius: 7.0, fill: theme.accent, border: None });
    content.target(slider, PanelTarget::Audio(AudioTarget::Slider(node.id)));
    height
}

fn text(content: &mut Content, measure: &mut impl Measure, y: f32, width: f32, text: &str, style: TextStyle, color: crate::Color) -> f32 {
    let height = measure.line_height(style);
    content.push(Node::Label { rect: Rect::new(0.0, y, width, height), text: text.to_owned(), style, color, align: Align::Left });
    height
}

pub fn content(
    model: &AudioModel,
    hover: Option<&PanelTarget>,
    theme: &Theme,
    measure: &mut impl Measure,
    width: f32,
    language: Language,
) -> Content {
    let tr = |text| language.tr(text);
    let mut content = Content::default();
    let mut y = 0.0;
    let devices = [
        (tr("Ausgabe"), tr("Kein Ausgabegerät"), &model.outputs, model.default_output, model.outputs_expanded, AudioTarget::ToggleOutputs),
        (tr("Mikrofon"), tr("Kein Eingabegerät"), &model.inputs, model.default_input, model.inputs_expanded, AudioTarget::ToggleInputs),
    ];
    for (title, none, nodes, current, expanded, toggle) in devices {
        y += content.heading(theme, measure, y, width, title) + SPACING;
        let current = current.and_then(|id| nodes.iter().find(|n| n.id == id));
        let others: Vec<&NodeView> = nodes.iter().filter(|n| Some(n.id) != current.map(|c| c.id)).collect();
        match current {
            None => y += text(&mut content, measure, y, width, none, TEXT, theme.muted) + SPACING,
            // The default's name shows and hides the others, where there are any.
            Some(node) => {
                let (action, mark) = match (others.is_empty(), expanded) {
                    (true, _) => (None, ""),
                    (false, false) => (Some(toggle), "▾"),
                    (false, true) => (Some(toggle), "▴"),
                };
                y += control(&mut content, theme, node, y, width, true, action, mark, hover, language) + SPACING;
            }
        }
        // Without a default there is no name to click: the devices are listed right away.
        if expanded || current.is_none() {
            for node in others {
                y += control(&mut content, theme, node, y, width, false, Some(AudioTarget::Select(node.id)), "", hover, language) + SPACING;
            }
        }
        // The column's last spacing gives way to the space between sections.
        y += SECTION_SPACING - SPACING;
    }
    y += content.heading(theme, measure, y, width, tr("Anwendungen")) + SPACING;
    if model.applications.is_empty() {
        y += text(&mut content, measure, y, width, tr("Keine aktiven Anwendungen"), TEXT, theme.muted) + SPACING;
    }
    for node in &model.applications {
        y += control(&mut content, theme, node, y, width, false, None, "", hover, language) + SPACING;
    }
    content.height = y - SPACING;
    content
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, scene::tests::Mono, theme::Palette};

    fn theme() -> Theme {
        let c = |v| Color::rgba(v, v, v, 255);
        Theme::new(Palette { background: c(10), surface: c(20), text: c(200), muted: c(150), accent: c(90), border: c(60) }, None)
    }

    fn node(id: u32, label: &str) -> NodeView {
        NodeView { id, label: label.into(), audio: Some((0.5, false)) }
    }

    fn texts(content: &Content) -> Vec<String> {
        content
            .nodes
            .iter()
            .filter_map(|n| match n {
                Node::Label { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect()
    }

    fn has(content: &Content, target: AudioTarget) -> bool {
        content.targets.iter().any(|(_, t)| *t == PanelTarget::Audio(target))
    }

    #[test]
    fn the_defaults_show_and_the_others_wait_behind_their_name() {
        let model = AudioModel {
            outputs: vec![node(1, "Kopfhörer"), node(2, "Lautsprecher")],
            inputs: vec![node(3, "Mikro")],
            default_output: Some(1),
            default_input: Some(3),
            ..AudioModel::default()
        };
        let content = content(&model, None, &theme(), &mut Mono, 360.0, Language::German);
        let shown = texts(&content);
        assert!(shown.contains(&"✓ Kopfhörer".into()) && shown.contains(&"50 %".into()), "{shown:?}");
        assert!(shown.contains(&"▾".into()) && has(&content, AudioTarget::ToggleOutputs));
        assert!(!shown.iter().any(|t| t.contains("Lautsprecher")));
        // The default cannot be chosen again; its level and slider work.
        assert!(!has(&content, AudioTarget::Select(1)) && has(&content, AudioTarget::Mute(1)) && has(&content, AudioTarget::Slider(1)));
        // A single microphone has no others to show.
        assert!(!has(&content, AudioTarget::ToggleInputs) && shown.iter().filter(|t| *t == "▾").count() == 1);
        assert!(shown.contains(&"Keine aktiven Anwendungen".into()));

        let expanded = content_of(AudioModel { outputs_expanded: true, ..model });
        // The name that showed them hides them again.
        assert!(has(&expanded, AudioTarget::Select(2)) && has(&expanded, AudioTarget::ToggleOutputs));
        assert!(texts(&expanded).contains(&"▴".into()));
    }

    fn content_of(model: AudioModel) -> Content {
        content(&model, None, &theme(), &mut Mono, 360.0, Language::German)
    }

    #[test]
    fn rows_have_their_qml_sizes() {
        let model = AudioModel { outputs: vec![node(1, "A")], default_output: Some(1), ..AudioModel::default() };
        let content = content_of(model);
        let select = content.targets.iter().find(|(_, t)| *t == PanelTarget::Audio(AudioTarget::Mute(1))).unwrap().0;
        // Below the heading (18 high, 6 apart), inside the device's card.
        assert_eq!(select, Rect::new(360.0 - 8.0 - 78.0, 24.0 + 8.0 + 3.0, 78.0, 26.0));
        let slider = content.targets.iter().find(|(_, t)| *t == PanelTarget::Audio(AudioTarget::Slider(1))).unwrap().0;
        assert_eq!(slider, Rect::new(12.0, 24.0 + 8.0 + 32.0 + 4.0, 336.0, 26.0));
        assert_eq!(slider_value(slider, 7.0), 0.0);
        assert_eq!(slider_value(slider, slider.right() - 7.0), 1.0);
        assert_eq!(slider_value(slider, slider.x + slider.w / 2.0), 0.5);
    }

    #[test]
    fn muted_and_unknown_levels() {
        let model = AudioModel {
            outputs: vec![NodeView { id: 1, label: "A".into(), audio: Some((0.5, true)) }],
            applications: vec![NodeView { id: 9, label: "Firefox".into(), audio: None }],
            default_output: Some(1),
            ..AudioModel::default()
        };
        let content = content_of(model);
        let shown = texts(&content);
        assert!(shown.contains(&"Stumm".into()) && shown.contains(&"…".into()) && shown.contains(&"Firefox".into()));
        assert!(!has(&content, AudioTarget::Slider(9)) && !has(&content, AudioTarget::Mute(9)));
    }
}
