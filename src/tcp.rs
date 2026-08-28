//! Blocking Simple Control Protocol client (`std::net::TcpStream`).

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

use crate::config::TcpConfig;
use crate::error::{Error, Result};
use crate::simple;

/// Blocking KAIROS Simple Control client.
#[derive(Debug)]
pub struct Client {
    stream: BufReader<TcpStream>,
}

impl Client {
    /// Connect to Kairos on [`TcpConfig::addr`].
    pub fn connect(config: TcpConfig) -> Result<Self> {
        let addr = config.addr();
        let stream = TcpStream::connect(&addr)?;
        stream.set_nodelay(true)?;
        if config.timeout_ms > 0 {
            let timeout = Duration::from_millis(config.timeout_ms);
            stream.set_read_timeout(Some(timeout))?;
            stream.set_write_timeout(Some(timeout))?;
        }
        Ok(Self {
            stream: BufReader::new(stream),
        })
    }

    /// Send a keep-alive (empty line). No reply is expected (Kairos v1.2+).
    pub fn keep_alive(&mut self) -> Result<()> {
        self.write_raw(b"\r\n")
    }

    /// Send a raw command (without trailing newline) and expect `OK`.
    pub fn exec(&mut self, command: &str) -> Result<()> {
        self.write_line(command)?;
        simple::check_ok(&self.read_line()?)
    }

    /// Query an attribute (`path` with no `=`). Returns the response line.
    pub fn query(&mut self, path: &str) -> Result<String> {
        self.write_line(path)?;
        self.read_line()
    }

    /// `list:` / `list:{path}` — rows until a blank line.
    pub fn list(&mut self, path: &str) -> Result<Vec<String>> {
        self.write_line(&simple::list(path))?;
        self.read_block()
    }

    /// `info:{path}` — attribute names until a blank line.
    pub fn info(&mut self, path: &str) -> Result<Vec<String>> {
        self.write_line(&simple::info(path))?;
        self.read_block()
    }

    /// `subscribe:{path}`
    pub fn subscribe(&mut self, path: &str) -> Result<()> {
        self.exec(&simple::subscribe(path))
    }

    /// `unsubscribe:{path}`
    pub fn unsubscribe(&mut self, path: &str) -> Result<()> {
        self.exec(&simple::unsubscribe(path))
    }

    /// Recursively list `MEDIA.stills` clips (`.rr` objects).
    pub fn list_media_stills(&mut self) -> Result<Vec<String>> {
        let mut stills = Vec::new();
        let mut queue = vec!["MEDIA.stills".to_string()];
        while let Some(path) = queue.pop() {
            for entry in self.list(&path)? {
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
    pub fn force_source(
        &mut self,
        scene: &str,
        layer: &str,
        bus: &str,
        source: &str,
    ) -> Result<()> {
        self.exec(&simple::force_source(scene, layer, bus, source))
    }

    /// Assign a media still to a layer bus.
    pub fn set_media_still(
        &mut self,
        scene: &str,
        layer: &str,
        bus: &str,
        still: &str,
    ) -> Result<()> {
        self.exec(&simple::set_media_still(scene, layer, bus, still))
    }

    /// Layer `transition_cut` / `transition_auto`.
    pub fn layer_transition(&mut self, scene: &str, layer: &str, auto: bool) -> Result<()> {
        self.exec(&simple::layer_transition(scene, layer, auto))
    }

    /// Scene master cut.
    pub fn scene_cut(&mut self, scene: &str) -> Result<()> {
        self.exec(&simple::scene_cut(scene))
    }

    /// Scene master auto.
    pub fn scene_auto(&mut self, scene: &str) -> Result<()> {
        self.exec(&simple::scene_auto(scene))
    }

    /// RAM / clip player function or assignment (`play`, `repeat=1`, …).
    pub fn player(&mut self, player: &str, op: &str) -> Result<()> {
        self.exec(&simple::player(player, op))
    }

    /// Mute (`1`) or unmute (`0`) the master mixer or a named channel.
    pub fn set_audio_mute(&mut self, channel: Option<&str>, mute: u8) -> Result<()> {
        self.exec(&simple::audio_mute(channel, mute))
    }

    fn write_line(&mut self, command: &str) -> Result<()> {
        self.write_raw(format!("{command}\r\n").as_bytes())
    }

    fn write_raw(&mut self, bytes: &[u8]) -> Result<()> {
        self.stream.get_mut().write_all(bytes)?;
        self.stream.get_mut().flush()?;
        Ok(())
    }

    fn read_line(&mut self) -> Result<String> {
        let mut buf = String::new();
        let n = self.stream.read_line(&mut buf)?;
        if n == 0 {
            return Err(Error::Protocol("TCP closed".into()));
        }
        Ok(trim_crlf(buf))
    }

    fn read_block(&mut self) -> Result<Vec<String>> {
        let mut items = Vec::new();
        loop {
            let line = self.read_line()?;
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

fn trim_crlf(buf: String) -> String {
    buf.trim_end_matches(['\r', '\n']).to_string()
}
