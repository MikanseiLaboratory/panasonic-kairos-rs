//! Simple Panasonic Kairos Control Protocol (TCP, default port 3005).
//!
//! Commands are newline-terminated (`\r\n`). Assignments use `path=value`.
//! Functions with no argument may be sent as `path` or `path=`.
//! `list:` / `info:` replies are terminated by a blank line.

use crate::error::{Error, Result};

/// Source A (program) attribute name.
pub const SOURCE_A: &str = "sourceA";
/// Source B (preview) attribute name.
pub const SOURCE_B: &str = "sourceB";

/// Acknowledge an assignment / function reply (`OK` vs `Error`).
pub fn check_ok(line: &str) -> Result<()> {
    if line.eq_ignore_ascii_case("ok") {
        Ok(())
    } else if line.is_empty() {
        Err(Error::Protocol("empty TCP response".into()))
    } else {
        Err(Error::Protocol(line.to_string()))
    }
}

/// `SCENES.{scene}.Layers.{layer}.{sourceA|sourceB}={source}` (force / still assign).
pub fn force_source(scene: &str, layer: &str, bus: &str, source: &str) -> String {
    format!("SCENES.{scene}.Layers.{layer}.{bus}={source}")
}

/// Same wire command as [`force_source`]; used when the right-hand side is a still.
pub fn set_media_still(scene: &str, layer: &str, bus: &str, still: &str) -> String {
    force_source(scene, layer, bus, still)
}

/// `SCENES.{scene}.{layer}.transition_cut` or `transition_auto`.
pub fn layer_transition(scene: &str, layer: &str, auto: bool) -> String {
    let kind = if auto {
        "transition_auto"
    } else {
        "transition_cut"
    };
    format!("SCENES.{scene}.{layer}.{kind}")
}

/// `SCENES.{scene}.cut`
pub fn scene_cut(scene: &str) -> String {
    format!("SCENES.{scene}.cut")
}

/// `SCENES.{scene}.auto`
pub fn scene_auto(scene: &str) -> String {
    format!("SCENES.{scene}.auto")
}

/// `{player}.{op}` e.g. `RR1.play`, `RR1.repeat=1`.
pub fn player(player: &str, op: &str) -> String {
    format!("{player}.{op}")
}

/// Master or channel mute. `channel` `None` / `master` targets the mixer.
pub fn audio_mute(channel: Option<&str>, mute: u8) -> String {
    match channel {
        None | Some("") | Some("master") => format!("AUDIOMIXER.mute={mute}"),
        Some(ch) => format!("AUDIOMIXER.{ch}.mute={mute}"),
    }
}

/// `subscribe:{path}`
pub fn subscribe(path: &str) -> String {
    format!("subscribe:{path}")
}

/// `unsubscribe:{path}`
pub fn unsubscribe(path: &str) -> String {
    format!("unsubscribe:{path}")
}

/// `list:` or `list:{path}`
pub fn list(path: &str) -> String {
    if path.is_empty() {
        "list:".into()
    } else {
        format!("list:{path}")
    }
}

/// `info:{path}`
pub fn info(path: &str) -> String {
    format!("info:{path}")
}

/// Display label for a `MEDIA.stills…` object path.
pub fn media_still_label(path: &str) -> String {
    path.strip_prefix("MEDIA.stills.")
        .unwrap_or(path)
        .trim_end_matches(".rr")
        .to_string()
}

/// Whether a `list:MEDIA.stills` row is a clip rather than a folder.
pub fn is_media_still_clip(path: &str) -> bool {
    path.contains(".rr")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn force_source_matches_companion() {
        assert_eq!(
            force_source("Main", "Background", SOURCE_A, "IP1"),
            "SCENES.Main.Layers.Background.sourceA=IP1"
        );
    }

    #[test]
    fn layer_auto_matches_companion() {
        assert_eq!(
            layer_transition("Main", "Background", true),
            "SCENES.Main.Background.transition_auto"
        );
    }

    #[test]
    fn audio_master_and_channel() {
        assert_eq!(audio_mute(None, 1), "AUDIOMIXER.mute=1");
        assert_eq!(
            audio_mute(Some("Channel 1"), 0),
            "AUDIOMIXER.Channel 1.mute=0"
        );
    }

    #[test]
    fn player_repeat_assignment() {
        assert_eq!(player("RR1", "repeat=1"), "RR1.repeat=1");
    }

    #[test]
    fn check_ok_accepts_ok() {
        check_ok("OK").unwrap();
        check_ok("ok").unwrap();
        assert!(check_ok("Error").is_err());
    }
}
