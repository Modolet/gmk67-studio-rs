//! @file main.rs
//! @brief Native GPUI Kit application and read-only command-line inspection.
//! @author modolet <y@xxyx.io>
//! @date 2026-10-12

mod ui;

use anyhow::{Result, bail};
use gmk67_studio_rs::{
    device,
    profile::{Profile, default_path},
    protocol,
};
use gpui_kit::{
    component::{Theme, ThemeMode},
    *,
};
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut profile_path = default_path();
    let mut list = false;
    let mut preview = false;
    let mut demo = false;
    #[cfg(feature = "preview")]
    let mut snapshot = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--list" => list = true,
            "--preview" => preview = true,
            "--demo" => demo = true,
            #[cfg(feature = "preview")]
            "--snapshot" => {
                snapshot =
                    Some(PathBuf::from(args.next().ok_or_else(|| {
                        anyhow::anyhow!("--snapshot 需要 PNG 路径")
                    })?))
            }
            "--profile" => {
                profile_path = PathBuf::from(
                    args.next()
                        .ok_or_else(|| anyhow::anyhow!("--profile 需要文件路径"))?,
                )
            }
            "--help" | "-h" => {
                println!(
                    "GMK67 Studio\n\n--list              只扫描有线设备，不写入\n--preview           输出草稿的报文计划，不访问设备\n--profile <file>    指定本地草稿 JSON\n--demo              离线预览界面，不访问设备\n"
                );
                return Ok(());
            }
            _ => bail!("未知参数 {arg}"),
        }
    }
    #[cfg(feature = "preview")]
    if let Some(path) = snapshot {
        return ui::snapshot(&path);
    }
    if list {
        let keyboards = device::scan()?;
        for keyboard in &keyboards {
            println!(
                "{} · {} · {}",
                keyboard.manufacturer,
                keyboard.product,
                keyboard.path.to_string_lossy()
            );
        }
        if keyboards.is_empty() {
            println!("未发现匹配的有线 GMK67。请接入数据线并切到有线模式。");
        }
        return Ok(());
    }
    if preview {
        let profile = if profile_path.exists() {
            Profile::load(&profile_path)?
        } else {
            Profile::default()
        };
        for (name, packets) in [
            ("lighting", protocol::lighting(&profile)?),
            ("keymap", protocol::keymap(&profile)?),
        ] {
            println!("{name}: {} reports", packets.len());
            for (i, packet) in packets.iter().enumerate() {
                println!(
                    "{:02}: {}",
                    i + 1,
                    packet
                        .wire()
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<Vec<_>>()
                        .join(" ")
                );
            }
        }
        return Ok(());
    }
    application().with_assets(assets::Assets).run(move |cx| {
        init(cx);
        Theme::change(ThemeMode::Dark, None, cx);
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        let bounds = Bounds::centered(None, size(px(1340.), px(900.)), cx);
        open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(1080.), px(740.))),
                titlebar: Some(TitlebarOptions {
                    title: Some("GMK67 Studio".into()),
                    ..Default::default()
                }),
                app_id: Some("io.xxyx.gmk67-studio-rs".into()),
                ..Default::default()
            },
            cx,
            move |window, cx| cx.new(|cx| ui::Studio::new(profile_path, demo, window, cx)),
        )
        .expect("无法打开 GMK67 Studio 窗口");
        cx.activate(true);
    });
    Ok(())
}
