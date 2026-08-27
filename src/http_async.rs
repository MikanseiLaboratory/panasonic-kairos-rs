//! Async HTTP client (`reqwest` + Basic / Digest).

use crate::auth;
use crate::config::HttpConfig;
use crate::error::{Error, Result};
use crate::id::ResourceId;
use crate::transport::{decode_json, decode_one, encode_patch, path_and_query, MERGE_PATCH_JSON};
use crate::types::{
    ActionPatch, Aux, AuxPatch, Input, LayerPatch, Macro, MacroPatch, Multiviewer,
    MultiviewerPatch, Scene, SnapshotPatch,
};
use std::time::Duration;
use url::Url;

/// Async KAIROS REST client.
#[derive(Debug, Clone)]
pub struct Client {
    config: HttpConfig,
    http: reqwest::Client,
}

impl Client {
    /// Connect (lazy — no probe is sent).
    pub fn connect(config: HttpConfig) -> Result<Self> {
        let mut builder = reqwest::Client::builder();
        if config.timeout_ms > 0 {
            builder = builder.timeout(Duration::from_millis(config.timeout_ms));
        }
        Ok(Self {
            config,
            http: builder
                .build()
                .map_err(|e| Error::Transport(e.to_string()))?,
        })
    }

    /// Borrow the config.
    pub fn config(&self) -> &HttpConfig {
        &self.config
    }

    /// `GET /inputs`
    pub async fn list_inputs(&self) -> Result<Vec<Input>> {
        decode_json(&self.get(&["inputs"]).await?)
    }

    /// `GET /inputs/{input}`
    pub async fn get_input(&self, id: impl Into<ResourceId>) -> Result<Input> {
        let id = crate::transport::segment(id);
        decode_one(&self.get(&["inputs", &id]).await?)
    }

    /// `GET /macros`
    pub async fn list_macros(&self) -> Result<Vec<Macro>> {
        decode_json(&self.get(&["macros"]).await?)
    }

    /// `GET /macros/{macro}`
    pub async fn get_macro(&self, id: impl Into<ResourceId>) -> Result<Macro> {
        let id = crate::transport::segment(id);
        decode_one(&self.get(&["macros", &id]).await?)
    }

    /// `PATCH /macros/{macro}`
    pub async fn patch_macro(&self, id: impl Into<ResourceId>, patch: &MacroPatch) -> Result<()> {
        let id = crate::transport::segment(id);
        self.patch(&["macros", &id], patch).await
    }

    /// Fire a macro (`state: play`).
    pub async fn play_macro(&self, id: impl Into<ResourceId>) -> Result<()> {
        self.patch_macro(id, &MacroPatch::play()).await
    }

    /// `GET /aux`
    pub async fn list_aux(&self) -> Result<Vec<Aux>> {
        decode_json(&self.get(&["aux"]).await?)
    }

    /// `GET /aux/{aux}`
    pub async fn get_aux(&self, id: impl Into<ResourceId>) -> Result<Aux> {
        let id = crate::transport::segment(id);
        decode_one(&self.get(&["aux", &id]).await?)
    }

    /// `PATCH /aux/{aux}`
    pub async fn patch_aux(&self, id: impl Into<ResourceId>, patch: &AuxPatch) -> Result<()> {
        let id = crate::transport::segment(id);
        self.patch(&["aux", &id], patch).await
    }

    /// Assign an AUX source (must be listed in [`Aux::sources`]).
    pub async fn set_aux_source(
        &self,
        id: impl Into<ResourceId>,
        source: impl Into<String>,
    ) -> Result<()> {
        self.patch_aux(id, &AuxPatch::source(source)).await
    }

    /// `GET /multiviewers`
    pub async fn list_multiviewers(&self) -> Result<Vec<Multiviewer>> {
        decode_json(&self.get(&["multiviewers"]).await?)
    }

    /// `GET /multiviewers/{multiviewer}`
    pub async fn get_multiviewer(&self, id: impl Into<ResourceId>) -> Result<Multiviewer> {
        let id = crate::transport::segment(id);
        decode_one(&self.get(&["multiviewers", &id]).await?)
    }

    /// `GET /multiviewers/{multiviewer}/sdp` (`application/sdp`).
    pub async fn get_multiviewer_sdp(&self, id: impl Into<ResourceId>) -> Result<String> {
        let id = crate::transport::segment(id);
        let bytes = self.get(&["multiviewers", &id, "sdp"]).await?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    /// `PATCH /multiviewers/{multiviewer}`
    pub async fn patch_multiviewer(
        &self,
        id: impl Into<ResourceId>,
        patch: &MultiviewerPatch,
    ) -> Result<()> {
        let id = crate::transport::segment(id);
        self.patch(&["multiviewers", &id], patch).await
    }

    /// Recall a multiviewer preset by id.
    pub async fn set_multiviewer_preset(
        &self,
        id: impl Into<ResourceId>,
        preset: u32,
    ) -> Result<()> {
        self.patch_multiviewer(id, &MultiviewerPatch::preset(preset))
            .await
    }

    /// `GET /scenes`
    pub async fn list_scenes(&self) -> Result<Vec<Scene>> {
        decode_json(&self.get(&["scenes"]).await?)
    }

    /// `GET /scenes/{scene}`
    pub async fn get_scene(&self, id: impl Into<ResourceId>) -> Result<Scene> {
        let id = crate::transport::segment(id);
        decode_one(&self.get(&["scenes", &id]).await?)
    }

    /// `PATCH /scenes/{scene}/{layer}`
    pub async fn patch_layer(
        &self,
        scene: impl Into<ResourceId>,
        layer: impl Into<ResourceId>,
        patch: &LayerPatch,
    ) -> Result<()> {
        let scene = crate::transport::segment(scene);
        let layer = crate::transport::segment(layer);
        self.patch(&["scenes", &scene, &layer], patch).await
    }

    /// Set a layer `sourceA`.
    pub async fn set_layer_source_a(
        &self,
        scene: impl Into<ResourceId>,
        layer: impl Into<ResourceId>,
        source: impl Into<String>,
    ) -> Result<()> {
        self.patch_layer(scene, layer, &LayerPatch::source_a(source))
            .await
    }

    /// Set a layer `sourceB`.
    pub async fn set_layer_source_b(
        &self,
        scene: impl Into<ResourceId>,
        layer: impl Into<ResourceId>,
        source: impl Into<String>,
    ) -> Result<()> {
        self.patch_layer(scene, layer, &LayerPatch::source_b(source))
            .await
    }

    /// `PATCH /scenes/{scene}/actions/{uuid}`
    pub async fn patch_action(
        &self,
        scene: impl Into<ResourceId>,
        action: impl Into<ResourceId>,
        patch: &ActionPatch,
    ) -> Result<()> {
        let scene = crate::transport::segment(scene);
        let action = crate::transport::segment(action);
        self.patch(&["scenes", &scene, "actions", &action], patch)
            .await
    }

    /// Fire a scene action (`state: play`).
    pub async fn play_action(
        &self,
        scene: impl Into<ResourceId>,
        action: impl Into<ResourceId>,
    ) -> Result<()> {
        self.patch_action(scene, action, &ActionPatch::play()).await
    }

    /// `PATCH /scenes/{scene}/macros/{macro}`
    pub async fn patch_scene_macro(
        &self,
        scene: impl Into<ResourceId>,
        macro_id: impl Into<ResourceId>,
        patch: &MacroPatch,
    ) -> Result<()> {
        let scene = crate::transport::segment(scene);
        let macro_id = crate::transport::segment(macro_id);
        self.patch(&["scenes", &scene, "macros", &macro_id], patch)
            .await
    }

    /// Fire a scene-scoped macro.
    pub async fn play_scene_macro(
        &self,
        scene: impl Into<ResourceId>,
        macro_id: impl Into<ResourceId>,
    ) -> Result<()> {
        self.patch_scene_macro(scene, macro_id, &MacroPatch::play())
            .await
    }

    /// `PATCH /scenes/{scene}/snapshots/{snapshot}`
    pub async fn patch_snapshot(
        &self,
        scene: impl Into<ResourceId>,
        snapshot: impl Into<ResourceId>,
        patch: &SnapshotPatch,
    ) -> Result<()> {
        let scene = crate::transport::segment(scene);
        let snapshot = crate::transport::segment(snapshot);
        self.patch(&["scenes", &scene, "snapshots", &snapshot], patch)
            .await
    }

    /// Recall a scene snapshot.
    pub async fn recall_snapshot(
        &self,
        scene: impl Into<ResourceId>,
        snapshot: impl Into<ResourceId>,
    ) -> Result<()> {
        self.patch_snapshot(scene, snapshot, &SnapshotPatch::recall())
            .await
    }

    async fn get(&self, segments: &[&str]) -> Result<Vec<u8>> {
        let url = self.config.url_for_segments(segments)?;
        self.request_bytes("GET", &url, None).await
    }

    async fn patch<T: serde::Serialize>(&self, segments: &[&str], body: &T) -> Result<()> {
        let url = self.config.url_for_segments(segments)?;
        let payload = encode_patch(body)?;
        self.request_bytes("PATCH", &url, Some(&payload)).await?;
        Ok(())
    }

    async fn request_bytes(&self, method: &str, url: &Url, body: Option<&[u8]>) -> Result<Vec<u8>> {
        let uri_path = path_and_query(url);
        let first = self
            .dispatch(
                method,
                url,
                body,
                Some(&auth::basic_header(&self.config.credentials)),
            )
            .await?;

        match first {
            Dispatch::Ok(bytes) => Ok(bytes),
            Dispatch::Unauthorized(challenge) => {
                if !auth::is_digest_challenge(&challenge) {
                    return Err(Error::Auth(format!("unsupported auth: {challenge}")));
                }
                let authorization = auth::authorization_header(
                    &self.config.credentials,
                    method,
                    &uri_path,
                    &challenge,
                )?;
                match self
                    .dispatch(method, url, body, Some(&authorization))
                    .await?
                {
                    Dispatch::Ok(bytes) => Ok(bytes),
                    Dispatch::Unauthorized(msg) => Err(Error::Auth(msg)),
                }
            }
        }
    }

    async fn dispatch(
        &self,
        method: &str,
        url: &Url,
        body: Option<&[u8]>,
        authorization: Option<&str>,
    ) -> Result<Dispatch> {
        let mut req = self.http.request(
            method
                .parse::<reqwest::Method>()
                .map_err(|e| Error::Protocol(e.to_string()))?,
            url.as_str(),
        );
        if let Some(value) = authorization {
            req = req.header(reqwest::header::AUTHORIZATION, value);
        }
        if let Some(bytes) = body {
            req = req
                .header(reqwest::header::CONTENT_TYPE, MERGE_PATCH_JSON)
                .body(bytes.to_vec());
        }
        let resp = req.send().await?;
        let status = resp.status();
        if status.as_u16() == 401 {
            let challenge = resp
                .headers()
                .get_all(reqwest::header::WWW_AUTHENTICATE)
                .iter()
                .filter_map(|v| v.to_str().ok())
                .find(|v| auth::is_digest_challenge(v))
                .or_else(|| {
                    resp.headers()
                        .get(reqwest::header::WWW_AUTHENTICATE)
                        .and_then(|v| v.to_str().ok())
                })
                .unwrap_or_default()
                .to_string();
            return Ok(Dispatch::Unauthorized(challenge));
        }
        let bytes = resp.bytes().await?.to_vec();
        if !status.is_success() {
            return Err(Error::http(
                status.as_u16(),
                String::from_utf8_lossy(&bytes),
            ));
        }
        Ok(Dispatch::Ok(bytes))
    }
}

enum Dispatch {
    Ok(Vec<u8>),
    Unauthorized(String),
}
