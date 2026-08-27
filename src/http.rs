//! Blocking HTTP client (`ureq` + Basic / Digest).

use crate::auth;
use crate::config::HttpConfig;
use crate::error::{Error, Result};
use crate::id::ResourceId;
use crate::transport::{decode_json, decode_one, encode_patch, path_and_query, MERGE_PATCH_JSON};
use crate::types::{
    ActionPatch, Aux, AuxPatch, Input, LayerPatch, Macro, MacroPatch, Multiviewer,
    MultiviewerPatch, Scene, SnapshotPatch,
};
use std::io::Read;
use std::time::Duration;
use url::Url;

/// Blocking KAIROS REST client.
#[derive(Debug, Clone)]
pub struct Client {
    config: HttpConfig,
    agent: ureq::Agent,
}

impl Client {
    /// Connect (lazy — no probe is sent).
    pub fn connect(config: HttpConfig) -> Result<Self> {
        let mut builder = ureq::AgentBuilder::new();
        if config.timeout_ms > 0 {
            let d = Duration::from_millis(config.timeout_ms);
            builder = builder.timeout_connect(d).timeout_read(d).timeout_write(d);
        }
        Ok(Self {
            config,
            agent: builder.build(),
        })
    }

    /// Borrow the config.
    pub fn config(&self) -> &HttpConfig {
        &self.config
    }

    /// `GET /inputs`
    pub fn list_inputs(&self) -> Result<Vec<Input>> {
        decode_json(&self.get(&["inputs"])?)
    }

    /// `GET /inputs/{input}`
    pub fn get_input(&self, id: impl Into<ResourceId>) -> Result<Input> {
        let id = crate::transport::segment(id);
        decode_one(&self.get(&["inputs", &id])?)
    }

    /// `GET /macros`
    pub fn list_macros(&self) -> Result<Vec<Macro>> {
        decode_json(&self.get(&["macros"])?)
    }

    /// `GET /macros/{macro}`
    pub fn get_macro(&self, id: impl Into<ResourceId>) -> Result<Macro> {
        let id = crate::transport::segment(id);
        decode_one(&self.get(&["macros", &id])?)
    }

    /// `PATCH /macros/{macro}`
    pub fn patch_macro(&self, id: impl Into<ResourceId>, patch: &MacroPatch) -> Result<()> {
        let id = crate::transport::segment(id);
        self.patch(&["macros", &id], patch)
    }

    /// Fire a macro (`state: play`).
    pub fn play_macro(&self, id: impl Into<ResourceId>) -> Result<()> {
        self.patch_macro(id, &MacroPatch::play())
    }

    /// `GET /aux`
    pub fn list_aux(&self) -> Result<Vec<Aux>> {
        decode_json(&self.get(&["aux"])?)
    }

    /// `GET /aux/{aux}`
    pub fn get_aux(&self, id: impl Into<ResourceId>) -> Result<Aux> {
        let id = crate::transport::segment(id);
        decode_one(&self.get(&["aux", &id])?)
    }

    /// `PATCH /aux/{aux}`
    pub fn patch_aux(&self, id: impl Into<ResourceId>, patch: &AuxPatch) -> Result<()> {
        let id = crate::transport::segment(id);
        self.patch(&["aux", &id], patch)
    }

    /// Assign an AUX source (must be listed in [`Aux::sources`]).
    pub fn set_aux_source(
        &self,
        id: impl Into<ResourceId>,
        source: impl Into<String>,
    ) -> Result<()> {
        self.patch_aux(id, &AuxPatch::source(source))
    }

    /// `GET /multiviewers`
    pub fn list_multiviewers(&self) -> Result<Vec<Multiviewer>> {
        decode_json(&self.get(&["multiviewers"])?)
    }

    /// `GET /multiviewers/{multiviewer}`
    pub fn get_multiviewer(&self, id: impl Into<ResourceId>) -> Result<Multiviewer> {
        let id = crate::transport::segment(id);
        decode_one(&self.get(&["multiviewers", &id])?)
    }

    /// `GET /multiviewers/{multiviewer}/sdp` (`application/sdp`).
    pub fn get_multiviewer_sdp(&self, id: impl Into<ResourceId>) -> Result<String> {
        let id = crate::transport::segment(id);
        let bytes = self.get(&["multiviewers", &id, "sdp"])?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    /// `PATCH /multiviewers/{multiviewer}`
    pub fn patch_multiviewer(
        &self,
        id: impl Into<ResourceId>,
        patch: &MultiviewerPatch,
    ) -> Result<()> {
        let id = crate::transport::segment(id);
        self.patch(&["multiviewers", &id], patch)
    }

    /// Recall a multiviewer preset by id.
    pub fn set_multiviewer_preset(&self, id: impl Into<ResourceId>, preset: u32) -> Result<()> {
        self.patch_multiviewer(id, &MultiviewerPatch::preset(preset))
    }

    /// `GET /scenes`
    pub fn list_scenes(&self) -> Result<Vec<Scene>> {
        decode_json(&self.get(&["scenes"])?)
    }

    /// `GET /scenes/{scene}`
    pub fn get_scene(&self, id: impl Into<ResourceId>) -> Result<Scene> {
        let id = crate::transport::segment(id);
        decode_one(&self.get(&["scenes", &id])?)
    }

    /// `PATCH /scenes/{scene}/{layer}`
    pub fn patch_layer(
        &self,
        scene: impl Into<ResourceId>,
        layer: impl Into<ResourceId>,
        patch: &LayerPatch,
    ) -> Result<()> {
        let scene = crate::transport::segment(scene);
        let layer = crate::transport::segment(layer);
        self.patch(&["scenes", &scene, &layer], patch)
    }

    /// Set a layer `sourceA`.
    pub fn set_layer_source_a(
        &self,
        scene: impl Into<ResourceId>,
        layer: impl Into<ResourceId>,
        source: impl Into<String>,
    ) -> Result<()> {
        self.patch_layer(scene, layer, &LayerPatch::source_a(source))
    }

    /// Set a layer `sourceB`.
    pub fn set_layer_source_b(
        &self,
        scene: impl Into<ResourceId>,
        layer: impl Into<ResourceId>,
        source: impl Into<String>,
    ) -> Result<()> {
        self.patch_layer(scene, layer, &LayerPatch::source_b(source))
    }

    /// `PATCH /scenes/{scene}/actions/{uuid}`
    pub fn patch_action(
        &self,
        scene: impl Into<ResourceId>,
        action: impl Into<ResourceId>,
        patch: &ActionPatch,
    ) -> Result<()> {
        let scene = crate::transport::segment(scene);
        let action = crate::transport::segment(action);
        self.patch(&["scenes", &scene, "actions", &action], patch)
    }

    /// Fire a scene action (`state: play`).
    pub fn play_action(
        &self,
        scene: impl Into<ResourceId>,
        action: impl Into<ResourceId>,
    ) -> Result<()> {
        self.patch_action(scene, action, &ActionPatch::play())
    }

    /// `PATCH /scenes/{scene}/macros/{macro}`
    pub fn patch_scene_macro(
        &self,
        scene: impl Into<ResourceId>,
        macro_id: impl Into<ResourceId>,
        patch: &MacroPatch,
    ) -> Result<()> {
        let scene = crate::transport::segment(scene);
        let macro_id = crate::transport::segment(macro_id);
        self.patch(&["scenes", &scene, "macros", &macro_id], patch)
    }

    /// Fire a scene-scoped macro.
    pub fn play_scene_macro(
        &self,
        scene: impl Into<ResourceId>,
        macro_id: impl Into<ResourceId>,
    ) -> Result<()> {
        self.patch_scene_macro(scene, macro_id, &MacroPatch::play())
    }

    /// `PATCH /scenes/{scene}/snapshots/{snapshot}`
    pub fn patch_snapshot(
        &self,
        scene: impl Into<ResourceId>,
        snapshot: impl Into<ResourceId>,
        patch: &SnapshotPatch,
    ) -> Result<()> {
        let scene = crate::transport::segment(scene);
        let snapshot = crate::transport::segment(snapshot);
        self.patch(&["scenes", &scene, "snapshots", &snapshot], patch)
    }

    /// Recall a scene snapshot.
    pub fn recall_snapshot(
        &self,
        scene: impl Into<ResourceId>,
        snapshot: impl Into<ResourceId>,
    ) -> Result<()> {
        self.patch_snapshot(scene, snapshot, &SnapshotPatch::recall())
    }

    fn get(&self, segments: &[&str]) -> Result<Vec<u8>> {
        let url = self.config.url_for_segments(segments)?;
        self.request_bytes("GET", &url, None)
    }

    fn patch<T: serde::Serialize>(&self, segments: &[&str], body: &T) -> Result<()> {
        let url = self.config.url_for_segments(segments)?;
        let payload = encode_patch(body)?;
        self.request_bytes("PATCH", &url, Some(&payload))?;
        Ok(())
    }

    fn request_bytes(&self, method: &str, url: &Url, body: Option<&[u8]>) -> Result<Vec<u8>> {
        let uri_path = path_and_query(url);
        match self.dispatch(
            method,
            url,
            body,
            Some(&auth::basic_header(&self.config.credentials)),
        ) {
            Ok(bytes) => Ok(bytes),
            Err(DispatchErr::Unauthorized(challenge)) => {
                if !auth::is_digest_challenge(&challenge) {
                    return Err(Error::Auth(format!("unsupported auth: {challenge}")));
                }
                let authorization = auth::authorization_header(
                    &self.config.credentials,
                    method,
                    &uri_path,
                    &challenge,
                )?;
                match self.dispatch(method, url, body, Some(&authorization)) {
                    Ok(bytes) => Ok(bytes),
                    Err(DispatchErr::Unauthorized(msg)) => Err(Error::Auth(msg)),
                    Err(DispatchErr::Other(e)) => Err(e),
                }
            }
            Err(DispatchErr::Other(e)) => Err(e),
        }
    }

    fn dispatch(
        &self,
        method: &str,
        url: &Url,
        body: Option<&[u8]>,
        authorization: Option<&str>,
    ) -> std::result::Result<Vec<u8>, DispatchErr> {
        let mut req = self.agent.request(method, url.as_str());
        if let Some(value) = authorization {
            req = req.set("Authorization", value);
        }
        if body.is_some() {
            req = req.set("Content-Type", MERGE_PATCH_JSON);
        }
        let result = match body {
            Some(b) => req.send_bytes(b),
            None => req.call(),
        };
        match result {
            Ok(resp) => read_response(resp).map_err(DispatchErr::Other),
            Err(ureq::Error::Status(401, resp)) => {
                let challenge = resp
                    .header("www-authenticate")
                    .or_else(|| resp.header("WWW-Authenticate"))
                    .unwrap_or_default()
                    .to_string();
                Err(DispatchErr::Unauthorized(challenge))
            }
            Err(e) => Err(DispatchErr::Other(e.into())),
        }
    }
}

enum DispatchErr {
    Unauthorized(String),
    Other(Error),
}

fn read_response(resp: ureq::Response) -> Result<Vec<u8>> {
    let status = resp.status();
    let mut buf = Vec::new();
    resp.into_reader().read_to_end(&mut buf)?;
    if !(200..300).contains(&status) {
        let msg = String::from_utf8_lossy(&buf).into_owned();
        if status == 401 {
            return Err(Error::Auth(msg));
        }
        return Err(Error::http(status, msg));
    }
    Ok(buf)
}
