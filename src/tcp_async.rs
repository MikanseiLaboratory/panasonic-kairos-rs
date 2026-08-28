//! Async Simple Control Protocol client (`tokio::net::TcpStream`).

use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::config::TcpConfig;
use crate::error::{Error, Result};
use crate::simple;

/// Async KAIROS Simple Control client.
#[derive(Debug)]
pub struct Client {
    stream: BufReader<TcpStream>,
    timeout: Option<Duration>,
}

impl Client {
    /// Connect to Kairos on [`TcpConfig::addr`].
    pub async fn connect(config: TcpConfig) -> Result<Self> {
        let addr = config.addr();
        let timeout_d = (config.timeout_ms > 0).then(|| Duration::from_millis(config.timeout_ms));
        let stream = timed(timeout_d, async { Ok(TcpStream::connect(&addr).await?) }).await?;
        stream.set_nodelay(true)?;
        Ok(Self {
            stream: BufReader::new(stream),
            timeout: timeout_d,
        })
    }

    /// Send a keep-alive (empty line). No reply is expected (Kairos v1.2+).
    pub async fn keep_alive(&mut self) -> Result<()> {
        self.write_raw(b"\r\n").await
    }

    /// Send a raw command (without trailing newline) and expect `OK`.
    pub async fn exec(&mut self, command: &str) -> Result<()> {
        self.write_line(command).await?;
        simple::check_ok(&self.read_line().await?)
    }

    /// Query an attribute (`path` with no `=`). Returns the response line.
    pub async fn query(&mut self, path: &str) -> Result<String> {
        self.write_line(path).await?;
        self.read_line().await
    }

    /// `list:` / `list:{path}` — rows until a blank line.
    pub async fn list(&mut self, path: &str) -> Result<Vec<String>> {
        self.write_line(&simple::list(path)).await?;
        self.read_block().await
    }

    /// `info:{path}` — attribute names until a blank line.
    pub async fn info(&mut self, path: &str) -> Result<Vec<String>> {
        self.write_line(&simple::info(path)).await?;
        self.read_block().await
    }

    /// `subscribe:{path}`
    pub async fn subscribe(&mut self, path: &str) -> Result<()> {
        self.exec(&simple::subscribe(path)).await
    }

    /// `unsubscribe:{path}`
    pub async fn unsubscribe(&mut self, path: &str) -> Result<()> {
        self.exec(&simple::unsubscribe(path)).await
    }

    /// Recursively list `MEDIA.stills` clips (`.rr` objects).
    pub async fn list_media_stills(&mut self) -> Result<Vec<String>> {
        let mut stills = Vec::new();
        let mut queue = vec!["MEDIA.stills".to_string()];
        while let Some(path) = queue.pop() {
            for entry in self.list(&path).await? {
                if entry.eq_ignore_ascii_case("ok") {
                    continue;
                }
                if simple::is_media_still_clip(&entry) {
                    stills.push(entry);
                } else {
                    queue.push(entry);
                }
            }
        }
        stills.sort();
        Ok(stills)
    }

    /// Force a layer source (bypasses the next transition).
    pub async fn force_source(
        &mut self,
        scene: &str,
        layer: &str,
        bus: &str,
        source: &str,
    ) -> Result<()> {
        self.exec(&simple::force_source(scene, layer, bus, source))
            .await
    }

    /// Assign a media still to a layer bus.
    pub async fn set_media_still(
        &mut self,
        scene: &str,
        layer: &str,
        bus: &str,
        still: &str,
    ) -> Result<()> {
        self.exec(&simple::set_media_still(scene, layer, bus, still))
            .await
    }

    /// Layer `transition_cut` / `transition_auto`.
    pub async fn layer_transition(&mut self, scene: &str, layer: &str, auto: bool) -> Result<()> {
        self.exec(&simple::layer_transition(scene, layer, auto))
            .await
    }

    /// Scene master cut.
    pub async fn scene_cut(&mut self, scene: &str) -> Result<()> {
        self.exec(&simple::scene_cut(scene)).await
    }

    /// Scene master auto.
    pub async fn scene_auto(&mut self, scene: &str) -> Result<()> {
        self.exec(&simple::scene_auto(scene)).await
    }

    /// RAM / clip player function or assignment (`play`, `repeat=1`, …).
    pub async fn player(&mut self, player: &str, op: &str) -> Result<()> {
        self.exec(&simple::player(player, op)).await
    }

    /// Mute (`1`) or unmute (`0`) the master mixer or a named channel.
    pub async fn set_audio_mute(&mut self, channel: Option<&str>, mute: u8) -> Result<()> {
        self.exec(&simple::audio_mute(channel, mute)).await
    }

    async fn write_line(&mut self, command: &str) -> Result<()> {
        self.write_raw(format!("{command}\r\n").as_bytes()).await
    }

    async fn write_raw(&mut self, bytes: &[u8]) -> Result<()> {
        let timeout_d = self.timeout;
        timed(timeout_d, async {
            self.stream.get_mut().write_all(bytes).await?;
            self.stream.get_mut().flush().await?;
            Ok(())
        })
        .await
    }

    async fn read_line(&mut self) -> Result<String> {
        let timeout_d = self.timeout;
        timed(timeout_d, async {
            let mut buf = String::new();
            let n = self.stream.read_line(&mut buf).await?;
            if n == 0 {
                return Err(Error::Protocol("TCP closed".into()));
            }
            Ok(buf.trim_end_matches(['\r', '\n']).to_string())
        })
        .await
    }

    async fn read_block(&mut self) -> Result<Vec<String>> {
        let mut items = Vec::new();
        loop {
            let line = self.read_line().await?;
            if line.is_empty() {
                break;
            }
            if line.eq_ignore_ascii_case("error") {
                return Err(Error::Protocol("TCP list/info failed".into()));
            }
            items.push(line);
        }
        Ok(items)
    }
}

async fn timed<T, F>(timeout_d: Option<Duration>, fut: F) -> Result<T>
where
    F: std::future::Future<Output = Result<T>>,
{
    match timeout_d {
        None => fut.await,
        Some(d) => timeout(d, fut)
            .await
            .map_err(|_| Error::Transport("TCP timeout".into()))?,
    }
}
