//! JSON types matching the KAIROS REST API object model.

use serde::{Deserialize, Serialize};

/// Video input (IP / SDI / NDI / Stream, …).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Input {
    /// Input index.
    pub index: u32,
    /// Input name (`IP1`, `SDI1`, …).
    pub name: String,
    /// Tally state as reported by the device.
    pub tally: u32,
    /// Object UUID.
    pub uuid: String,
}

/// Global or scene-scoped macro.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Macro {
    /// Display color, e.g. `rgb(255,255,255)`.
    pub color: String,
    /// Macro name.
    pub name: String,
    /// Folder path (`""` for the root).
    #[serde(default)]
    pub path: String,
    /// Always `null` on GET. Write [`MacroState::Play`] via PATCH.
    #[serde(default)]
    pub state: Option<String>,
    /// Object UUID.
    pub uuid: String,
}

/// PATCH body for a macro.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MacroPatch {
    /// Macro action. The REST spec documents `play`; Kairos also accepts
    /// `stop`, `record`, and `stop_record`.
    pub state: MacroState,
}

/// Writable macro / action state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MacroState {
    /// Play / fire the macro or action.
    Play,
    /// Stop a running macro.
    Stop,
    /// Start recording a macro.
    Record,
    /// Stop recording a macro.
    StopRecord,
}

/// AUX bus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aux {
    /// AUX index.
    pub index: u32,
    /// AUX name.
    pub name: String,
    /// Currently selected source.
    pub source: String,
    /// Sources that may be assigned.
    #[serde(default)]
    pub sources: Vec<String>,
    /// Object UUID.
    pub uuid: String,
}

/// PATCH body for an AUX (`source` must be in [`Aux::sources`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuxPatch {
    /// Source name to assign.
    pub source: String,
}

impl AuxPatch {
    /// Create a source assignment patch.
    pub fn source(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
        }
    }
}

/// Multiviewer layout preset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultiviewerPreset {
    /// Preset id.
    pub id: u32,
    /// Preset name.
    pub name: String,
    /// `true` when the preset is user-defined.
    pub usr: bool,
}

/// Multiviewer output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Multiviewer {
    /// Multiviewer index (`0..=3` on AT-KC200 / AT-KC2000).
    pub index: u32,
    /// Multiviewer name.
    pub name: String,
    /// Always `null` on GET. Write a preset id via PATCH.
    #[serde(default)]
    pub preset: Option<u32>,
    /// Available presets.
    #[serde(default)]
    pub presets: Vec<MultiviewerPreset>,
    /// Embedded SDP (also available as `application/sdp` from `/sdp`).
    #[serde(default)]
    pub sdp: String,
    /// Object UUID.
    pub uuid: String,
}

/// PATCH body for a multiviewer (`preset` must exist in [`Multiviewer::presets`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultiviewerPatch {
    /// Preset id to recall.
    pub preset: u32,
}

impl MultiviewerPatch {
    /// Create a preset recall patch.
    pub fn preset(preset: u32) -> Self {
        Self { preset }
    }
}

/// Scene action (cut / auto / show_layer / recall, …).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action {
    /// Action name (`Main:cut`, `Background:show_layer`, …).
    pub name: String,
    /// Always `null` on GET. Write [`MacroState::Play`] via PATCH.
    #[serde(default)]
    pub state: Option<String>,
    /// Object UUID.
    pub uuid: String,
}

/// PATCH body for a scene action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionPatch {
    /// Action trigger. The spec documents `play`.
    pub state: MacroState,
}

impl ActionPatch {
    /// Fire the action.
    pub fn play() -> Self {
        Self {
            state: MacroState::Play,
        }
    }
}

impl MacroPatch {
    /// Fire the macro.
    pub fn play() -> Self {
        Self::with_state(MacroState::Play)
    }

    /// Build a PATCH body for any documented macro state.
    pub fn with_state(state: MacroState) -> Self {
        Self { state }
    }
}

/// Scene layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layer {
    /// Layer name (`Background`, `Layer-1`, …).
    pub name: String,
    /// Folder path.
    #[serde(default)]
    pub path: String,
    /// Optional program/source A.
    #[serde(default, rename = "sourceA")]
    pub source_a: Option<String>,
    /// Optional preview/source B.
    #[serde(default, rename = "sourceB")]
    pub source_b: Option<String>,
    /// Sources that may be assigned.
    #[serde(default)]
    pub sources: Vec<String>,
    /// Object UUID, when provided.
    #[serde(default)]
    pub uuid: Option<String>,
}

/// PATCH body for a layer. At least one of `sourceA` / `sourceB` is required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LayerPatch {
    /// New source A.
    #[serde(skip_serializing_if = "Option::is_none", rename = "sourceA")]
    pub source_a: Option<String>,
    /// New source B.
    #[serde(skip_serializing_if = "Option::is_none", rename = "sourceB")]
    pub source_b: Option<String>,
}

impl LayerPatch {
    /// Patch only `sourceA`.
    pub fn source_a(source: impl Into<String>) -> Self {
        Self {
            source_a: Some(source.into()),
            source_b: None,
        }
    }

    /// Patch only `sourceB`.
    pub fn source_b(source: impl Into<String>) -> Self {
        Self {
            source_a: None,
            source_b: Some(source.into()),
        }
    }
}

/// Scene snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    /// Snapshot name.
    pub name: String,
    /// Always `null` on GET. Write [`SnapshotState::Recall`] via PATCH.
    #[serde(default)]
    pub state: Option<String>,
    /// Object UUID.
    pub uuid: String,
}

/// Writable snapshot state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SnapshotState {
    /// Recall the snapshot.
    Recall,
}

/// PATCH body for a snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotPatch {
    /// Snapshot action. The spec documents `recall`.
    pub state: SnapshotState,
}

impl SnapshotPatch {
    /// Recall the snapshot.
    pub fn recall() -> Self {
        Self {
            state: SnapshotState::Recall,
        }
    }
}

/// Scene (ME).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    /// Scene actions.
    #[serde(default)]
    pub actions: Vec<Action>,
    /// Layers.
    #[serde(default)]
    pub layers: Vec<Layer>,
    /// Scene-scoped macros.
    #[serde(default)]
    pub macros: Vec<Macro>,
    /// Scene name.
    pub name: String,
    /// Folder path.
    #[serde(default)]
    pub path: String,
    /// Snapshots.
    #[serde(default)]
    pub snapshots: Vec<Snapshot>,
    /// Tally state.
    pub tally: u32,
    /// Object UUID.
    pub uuid: String,
}

/// Error payload returned for some `400` responses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorBody {
    /// HTTP-style code.
    pub code: u16,
    /// Human-readable parser / validation message.
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn input_from_spec_example() {
        let json = r#"{
            "index": 0,
            "name": "IP1",
            "tally": 1,
            "uuid": "e53210f7-2235-5ae3-9c02-4f58d67bf8b8"
        }"#;
        let input: Input = serde_json::from_str(json).unwrap();
        assert_eq!(input.name, "IP1");
        assert_eq!(input.tally, 1);
    }

    #[test]
    fn layer_patch_omits_unset_fields() {
        let body = serde_json::to_value(LayerPatch::source_a("Black")).unwrap();
        assert_eq!(body, serde_json::json!({"sourceA": "Black"}));
    }

    #[test]
    fn serde_macro_patch_states() {
        assert_eq!(
            serde_json::to_value(MacroPatch::play()).unwrap(),
            serde_json::json!({"state": "play"})
        );
        assert_eq!(
            serde_json::to_value(MacroPatch::with_state(MacroState::StopRecord)).unwrap(),
            serde_json::json!({"state": "stop_record"})
        );
    }

    #[test]
    fn merge_patch_content_roundtrip() {
        let patch = AuxPatch::source("Black");
        assert_eq!(
            serde_json::to_string(&patch).unwrap(),
            r#"{"source":"Black"}"#
        );
    }
}
