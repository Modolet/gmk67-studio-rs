//! @file profile.rs
//! @brief Validated local drafts; these are not backups read from the keyboard.
//! @author modolet <y@xxyx.io>
//! @date 2026-10-12

use crate::layout::{KEYS, OUTPUTS};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

pub const EFFECTS: [&str; 8] = [
    "常亮", "呼吸", "光谱", "雨滴", "涟漪", "波浪", "星光", "滚动",
];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub version: u8,
    pub effect: u8,
    pub colors: BTreeMap<u8, [u8; 3]>,
    pub keymap: BTreeMap<u8, u8>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            version: 1,
            effect: 1,
            colors: KEYS.iter().map(|k| (k.id, [94, 219, 193])).collect(),
            keymap: BTreeMap::new(),
        }
    }
}

impl Profile {
    pub fn validate(&self) -> Result<()> {
        ensure!(self.version == 1, "不支持的草稿版本 {}", self.version);
        ensure!((1..=8).contains(&self.effect), "灯效编号必须为 1–8");
        ensure!(
            self.colors.len() == KEYS.len(),
            "草稿必须包含全部 66 个键位的颜色"
        );
        for id in self.colors.keys().chain(self.keymap.keys()) {
            ensure!(KEYS.iter().any(|k| k.id == *id), "未知的键位编号 {id}");
        }
        ensure!(
            !self.keymap.contains_key(&96),
            "Fn 键的改写协议尚未确认，请保留 Fn"
        );
        for code in self.keymap.values() {
            ensure!(
                OUTPUTS.iter().any(|o| o.code == *code),
                "不支持的键值 {code:#04x}"
            );
        }
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self> {
        let mut bytes = Vec::new();
        fs::File::open(path)
            .with_context(|| format!("读取草稿 {}", path.display()))?
            .take(64 * 1024 + 1)
            .read_to_end(&mut bytes)?;
        ensure!(bytes.len() <= 64 * 1024, "草稿文件过大");
        let profile: Self = serde_json::from_slice(&bytes).context("草稿 JSON 格式错误")?;
        profile.validate()?;
        Ok(profile)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        self.validate()?;
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent)?;
        }
        let temp = path.with_extension("json.tmp");
        fs::write(&temp, serde_json::to_vec_pretty(self)?)?;
        fs::rename(&temp, path).context("保存草稿失败")
    }
}

pub fn default_path() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("gmk67-studio-rs/profile.json")
}
