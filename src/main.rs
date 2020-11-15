use std::borrow::Cow;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use bendy::value::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct FastResume<'a> {
    save_path: String,
    completed_time: i64,
    paused: i64,

    #[serde(borrow)]
    #[serde(flatten)]
    extra: HashMap<String, Value<'a>>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let deluge_dir = PathBuf::from(env::var("APPDATA")?).join("deluge");
    let deluge_state = fs::read(deluge_dir.join("state/torrents.fastresume"))?;
    let deluge_fastresume: HashMap<String, &[u8]> = bendy::serde::from_bytes(&deluge_state)?;

    let qbittorrent_dir = PathBuf::from(env::var("LOCALAPPDATA")?).join("qBittorrent");
    let qbittorrent_backup = qbittorrent_dir.join("BT_backup");

    for entry in fs::read_dir(deluge_dir.join("state"))? {
        let path = entry?.path();

        if path.extension() == Some("torrent".as_ref()) {
            let file_name = path.file_name().unwrap();
            fs::copy(&path, qbittorrent_backup.join(file_name))?;
        }
    }

    for (hash, state_bytes) in deluge_fastresume {
        let mut fastresume: FastResume2 = bendy::serde::from_bytes(&state_bytes)?;
        fastresume.paused = 1;
        fastresume.extra.insert(
            String::from("qBt-savePath"),
            Value::Bytes(Cow::Borrowed(fastresume.save_path.as_bytes())),
        );

        println!("{} {}", hash, fastresume.save_path);

        fs::write(
            qbittorrent_backup.join(format!("{}.fastresume", hash)),
            bendy::serde::to_bytes(&fastresume)?,
        )?;
    }

    Ok(())
}
