//! @file ui.rs
//! @brief Keyboard canvas, retained editors and asynchronous explicit apply flows.
//! @author modolet <y@xxyx.io>
//! @date 2026-10-12

use anyhow::Result;
use gmk67_studio_rs::{
    device::{self, Keyboard},
    layout::{KEYS, OUTPUTS, output_label},
    lighting_preview,
    profile::{EFFECTS, Profile},
    protocol,
};
use gpui_kit::{
    component::{
        Disableable, Sizable, WindowExt,
        button::{Button, ButtonVariants},
        color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState},
        dialog::{DialogButtonProps, DialogFooter},
        select::{Select, SelectEvent, SelectState},
    },
    prelude::FluentBuilder as _,
    *,
};
use std::{collections::BTreeSet, path::PathBuf, time::Instant};

const BG: u32 = 0x101516;
const PANEL: u32 = 0x192022;
const EDGE: u32 = 0x2b3638;
const INK: u32 = 0xe9f0ef;
const MUTED: u32 = 0x90a4a2;
const ACCENT: u32 = 0x5edbc1;

pub struct Studio {
    profile: Profile,
    path: PathBuf,
    devices: Vec<Keyboard>,
    selected_device: Option<usize>,
    selected_keys: BTreeSet<u8>,
    color: [u8; 3],
    picker: Entity<ColorPickerState>,
    output: Entity<SelectState<Vec<SharedString>>>,
    output_code: Option<u8>,
    tab: usize,
    busy: bool,
    demo: bool,
    status: String,
    error: bool,
    dirty: bool,
    subscriptions: Vec<Subscription>,
    preview_seconds: f32,
    preview_last_tick: Instant,
    preview_playing: bool,
    ripple: Option<(f32, f32, f32)>,
}

enum Job {
    Scan,
    Apply(Keyboard, Profile, bool),
    Save(PathBuf, Profile),
    Load(PathBuf),
}

enum Outcome {
    Devices(Vec<Keyboard>),
    Written(usize),
    Saved,
    Loaded(Profile),
}

impl Job {
    fn run(self) -> Result<Outcome> {
        match self {
            Self::Scan => Ok(Outcome::Devices(device::scan()?)),
            Self::Apply(device, profile, keymap) => {
                let packets = if keymap {
                    protocol::keymap(&profile)?
                } else {
                    protocol::lighting(&profile)?
                };
                Ok(Outcome::Written(device::write(&device, &packets)?))
            }
            Self::Save(path, profile) => {
                profile.save(&path)?;
                Ok(Outcome::Saved)
            }
            Self::Load(path) => Ok(Outcome::Loaded(Profile::load(&path)?)),
        }
    }
}

impl Studio {
    pub fn new(path: PathBuf, demo: bool, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let picker = cx.new(|cx| ColorPickerState::new(window, cx).default_value(rgb(ACCENT)));
        let options: Vec<SharedString> = OUTPUTS.iter().map(|o| o.label.into()).collect();
        let output = cx.new(|cx| SelectState::new(options, None, window, cx).searchable(true));
        let mut subscriptions = Vec::new();
        subscriptions.push(
            cx.subscribe(&picker, |this, _, event: &ColorPickerEvent, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let c = Rgba::from(*color);
                    this.color = [
                        (c.r * 255.).round() as u8,
                        (c.g * 255.).round() as u8,
                        (c.b * 255.).round() as u8,
                    ];
                    cx.notify();
                }
            }),
        );
        subscriptions.push(cx.subscribe(
            &output,
            |this, _, event: &SelectEvent<Vec<SharedString>>, cx| {
                let SelectEvent::Confirm(value) = event;
                this.output_code = value.as_ref().and_then(|s| {
                    OUTPUTS
                        .iter()
                        .find(|o| o.label == s.as_ref())
                        .map(|o| o.code)
                });
                cx.notify();
            },
        ));
        let (profile, status, error) = if path.exists() {
            match Profile::load(&path) {
                Ok(profile) => (profile, "已载入本地草稿；未读取键盘当前设置".into(), false),
                Err(e) => (Profile::default(), format!("草稿载入失败：{e:#}"), true),
            }
        } else {
            (
                Profile::default(),
                "本地预览已就绪 · 尚未向键盘写入".into(),
                false,
            )
        };
        let mut this = Self {
            profile,
            path,
            devices: Vec::new(),
            selected_device: None,
            selected_keys: BTreeSet::new(),
            color: [94, 219, 193],
            picker,
            output,
            output_code: None,
            tab: 0,
            busy: false,
            demo,
            status,
            error,
            dirty: false,
            subscriptions,
            preview_seconds: 0.,
            preview_last_tick: Instant::now(),
            preview_playing: true,
            ripple: None,
        };
        if !demo {
            this.start(Job::Scan, cx);
        }
        this
    }

    fn start(&mut self, job: Job, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.busy = true;
        self.error = false;
        self.status = match &job {
            Job::Scan => "正在扫描有线键盘…",
            Job::Apply(..) => "正在写入，请保持键盘连接…",
            Job::Save(..) => "正在保存本地草稿…",
            Job::Load(..) => "正在载入本地草稿…",
        }
        .into();
        let task = cx.background_executor().spawn(async move { job.run() });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.busy = false;
                match result {
                    Ok(Outcome::Devices(devices)) => {
                        let old = this
                            .selected_device
                            .and_then(|i| this.devices.get(i))
                            .map(|d| d.path.clone());
                        this.selected_device = old
                            .and_then(|path| devices.iter().position(|d| d.path == path))
                            .or_else(|| (devices.len() == 1).then_some(0));
                        this.status = if devices.is_empty() {
                            "未发现有线键盘 · 请检查数据线和模式开关".into()
                        } else {
                            format!("发现 {} 个有线键盘 · 未向设备写入", devices.len())
                        };
                        this.devices = devices;
                    }
                    Ok(Outcome::Written(count)) => {
                        this.status = format!(
                            "已发送 {count} 个报文 · 请检查键盘实际效果，协议不支持配置读回"
                        )
                    }
                    Ok(Outcome::Saved) => {
                        this.status = "草稿已保存到本机；这不是键盘配置备份".into();
                        this.dirty = false;
                    }
                    Ok(Outcome::Loaded(profile)) => {
                        this.profile = profile;
                        this.dirty = false;
                        this.status = "草稿已载入 · 点击应用后才写入键盘".into();
                    }
                    Err(e) => {
                        this.error = true;
                        this.status = format!("{e:#}");
                    }
                }
                eprintln!("[GMK67] {}", this.status);
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    fn confirm_apply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(device) = self
            .selected_device
            .and_then(|i| self.devices.get(i))
            .cloned()
        else {
            return;
        };
        if self.busy || self.demo {
            return;
        }
        let profile = self.profile.clone();
        let keymap = self.tab == 1;
        let this = cx.entity().downgrade();
        let description = if keymap {
            "将用本地草稿替换键盘已有改键。参考协议同时清空一个含义未确认的附加配置块，可能影响其他自定义功能；无法读回或备份原设置。Fn 键保留。"
        } else {
            "将把预览中的颜色和灯效写入所选有线键盘，覆盖已有灯光设置。无法读回原设置；请在写入后检查实际效果。"
        };
        window.open_dialog(cx, move |dialog, _, _| {
            let this = this.clone();
            let device = device.clone();
            let profile = profile.clone();
            let target_path = device.path.to_string_lossy().into_owned();
            let apply = std::rc::Rc::new(move |cx: &mut App| {
                let _ = this.update(cx, |this, cx| {
                    this.start(Job::Apply(device.clone(), profile.clone(), keymap), cx)
                });
            });
            let apply_button = apply.clone();
            dialog
                .title(if keymap {
                    "替换键盘改键配置？"
                } else {
                    "应用灯光到键盘？"
                })
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(description)
                        .child(div().text_sm().child(format!("目标：{target_path}"))),
                )
                .button_props(
                    DialogButtonProps::default()
                        .ok_text("确认写入")
                        .cancel_text("取消"),
                )
                .footer(
                    DialogFooter::new()
                        .child(
                            Button::new("cancel-write")
                                .label("取消")
                                .outline()
                                .on_click(|_, window, cx| {
                                    window.close_dialog(cx);
                                }),
                        )
                        .child(
                            Button::new("confirm-write")
                                .label("确认写入")
                                .primary()
                                .on_click(move |_, window, cx| {
                                    apply_button(cx);
                                    window.close_dialog(cx);
                                }),
                        ),
                )
                .on_ok(move |_, _, cx| {
                    apply(cx);
                    true
                })
        });
    }

    fn select_key(&mut self, id: u8, cx: &mut Context<Self>) {
        if let Some(key) = KEYS.iter().find(|key| key.id == id) {
            self.ripple = Some((
                key.x + key.width / 2.,
                f32::from(key.row),
                self.preview_seconds,
            ));
        }
        if self.tab == 1 {
            self.selected_keys.clear();
            self.selected_keys.insert(id);
        } else if !self.selected_keys.remove(&id) {
            self.selected_keys.insert(id);
        }
        cx.notify();
    }

    fn paint(&mut self, all: bool, cx: &mut Context<Self>) {
        let keys: Vec<_> = if all {
            KEYS.iter().map(|k| k.id).collect()
        } else {
            self.selected_keys.iter().copied().collect()
        };
        for id in keys {
            self.profile.colors.insert(id, self.color);
        }
        self.dirty = true;
        cx.notify();
    }

    fn heading(title: &str, subtitle: &str) -> Div {
        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(title.to_owned()),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(MUTED))
                    .child(subtitle.to_owned()),
            )
    }

    fn device_card(&self, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .bg(rgb(PANEL))
            .border_1()
            .border_color(rgb(EDGE))
            .rounded_xl()
            .child(div().text_xs().text_color(rgb(MUTED)).child("当前设备"))
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("ZUOYA GMK67"),
            )
            .child(div().text_sm().text_color(rgb(ACCENT)).child(if self.demo {
                "离线演示"
            } else if self.selected_device.is_some() {
                "● 有线连接"
            } else {
                "○ 等待连接"
            }))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(MUTED))
                    .child("原版 · hfd.cn · 05AC:024F"),
            )
            .children(self.devices.iter().enumerate().map(|(index, device)| {
                let active = self.selected_device == Some(index);
                Button::new(("device", index))
                    .label(device.path.to_string_lossy().into_owned())
                    .small()
                    .outline()
                    .when(active, |b| b.bg(rgb(EDGE)))
                    .disabled(self.busy)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.selected_device = Some(index);
                        cx.notify();
                    }))
            }))
            .child(
                Button::new("scan")
                    .label("刷新设备")
                    .outline()
                    .disabled(self.busy || self.demo)
                    .on_click(cx.listener(|this, _, _, cx| this.start(Job::Scan, cx))),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(MUTED))
                    .child("无线接收器不会作为写入目标。"),
            )
    }

    fn keyboard(&self, unit: f32, cx: &mut Context<Self>) -> Div {
        let height = 52.;
        div()
            .relative()
            .w_full()
            .h(px(height * 5. + 38.))
            .rounded_xl()
            .bg(rgb(0x0c1112))
            .border_1()
            .border_color(rgb(EDGE))
            .children(KEYS.iter().map(|key| {
                let id = key.id;
                let selected = self.selected_keys.contains(&id);
                let modified = self.tab == 1 && self.profile.keymap.contains_key(&id);
                let color = self.profile.colors.get(&id).copied().unwrap_or([0, 0, 0]);
                let color = if self.tab == 0 {
                    lighting_preview::color(
                        self.profile.effect,
                        color,
                        key,
                        self.preview_seconds,
                        self.ripple,
                    )
                } else {
                    color
                };
                let tint =
                    (u32::from(color[0]) << 16) | (u32::from(color[1]) << 8) | u32::from(color[2]);
                let glow = ((u32::from(color[0]) / 5 + 12) << 16)
                    | ((u32::from(color[1]) / 5 + 17) << 8)
                    | (u32::from(color[2]) / 5 + 18);
                let caption = if self.tab == 1 {
                    self.profile
                        .keymap
                        .get(&id)
                        .map(|c| output_label(*c))
                        .unwrap_or(key.label)
                } else {
                    key.label
                };
                let key = div()
                    .id(("key", usize::from(id)))
                    .absolute()
                    .left(px(14. + key.x * unit))
                    .top(px(14. + f32::from(key.row) * height))
                    .w(px(key.width * unit - 5.))
                    .h(px(height - 6.))
                    .rounded_md()
                    .border_1()
                    .border_color(rgb(if selected {
                        ACCENT
                    } else if modified {
                        0xa491ef
                    } else {
                        0x38474a
                    }))
                    .bg(rgb(if self.tab == 0 {
                        glow
                    } else if selected {
                        0x2a4441
                    } else {
                        0x20292b
                    }))
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(0x304044)))
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_1()
                    .child(div().text_xs().child(caption.to_owned()))
                    .child(
                        div()
                            .h(px(2.))
                            .w(px((key.width * unit - 16.).max(8.)))
                            .rounded_full()
                            .bg(rgb(if self.tab == 0 {
                                tint
                            } else if modified {
                                0xa491ef
                            } else {
                                EDGE
                            })),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| this.select_key(id, cx)));
                #[cfg(feature = "preview")]
                let key = {
                    use gpui_kit::test::TestSupportExt;
                    key.test_support()
                };
                key
            }))
            .child(
                div()
                    .absolute()
                    .left(px(14. + 15. * unit))
                    .top(px(14.))
                    .w(px(unit - 5.))
                    .h(px(height - 6.))
                    .rounded_full()
                    .border_2()
                    .border_color(rgb(EDGE))
                    .bg(rgb(PANEL))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_xs()
                    .text_color(rgb(MUTED))
                    .child("旋钮"),
            )
            .child(
                div()
                    .absolute()
                    .right(px(18.))
                    .bottom(px(9.))
                    .text_xs()
                    .text_color(rgb(MUTED))
                    .child("GMK67 / 66 KEYS + KNOB"),
            )
    }

    fn lighting_controls(&self, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .gap_3()
                    .p_4()
                    .rounded_xl()
                    .border_1()
                    .border_color(rgb(EDGE))
                    .bg(rgb(PANEL))
                    .child(Self::heading("灯效", "动态示意 · 实际效果以键盘为准"))
                    .child(
                        Button::new("toggle-preview")
                            .ghost()
                            .small()
                            .label(if self.preview_playing {
                                "暂停预览"
                            } else {
                                "播放预览"
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.preview_playing = !this.preview_playing;
                                this.preview_last_tick = Instant::now();
                                cx.notify();
                            })),
                    )
                    .child(div().flex().flex_wrap().gap_2().children(
                        EFFECTS.iter().enumerate().map(|(i, label)| {
                            let active = usize::from(self.profile.effect) == i + 1;
                            Button::new(("effect", i))
                                .label(*label)
                                .small()
                                .outline()
                                .when(active, |b| b.bg(rgb(0x2a4441)).text_color(rgb(ACCENT)))
                                .disabled(self.busy)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.profile.effect = (i + 1) as u8;
                                    this.preview_seconds = 0.;
                                    this.preview_last_tick = Instant::now();
                                    this.ripple = None;
                                    this.dirty = true;
                                    cx.notify();
                                }))
                        }),
                    )),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .gap_3()
                    .p_4()
                    .rounded_xl()
                    .border_1()
                    .border_color(rgb(EDGE))
                    .bg(rgb(PANEL))
                    .child(Self::heading(
                        "逐键颜色",
                        &format!(
                            "已选 {} 个键 · 再点一次可取消选择",
                            self.selected_keys.len()
                        ),
                    ))
                    .child(ColorPicker::new(&self.picker).label("颜色"))
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(
                                Button::new("paint-selected")
                                    .label("涂到选中键")
                                    .outline()
                                    .disabled(self.busy || self.selected_keys.is_empty())
                                    .on_click(cx.listener(|this, _, _, cx| this.paint(false, cx))),
                            )
                            .child(
                                Button::new("paint-all")
                                    .label("涂到全部键")
                                    .outline()
                                    .disabled(self.busy)
                                    .on_click(cx.listener(|this, _, _, cx| this.paint(true, cx))),
                            ),
                    ),
            )
    }

    fn keymap_controls(&self, cx: &mut Context<Self>) -> Div {
        let selected = self.selected_keys.first().copied();
        let key = selected.and_then(|id| KEYS.iter().find(|k| k.id == id));
        let eligible = selected.is_some_and(|id| id != 96);
        div()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .rounded_xl()
            .border_1()
            .border_color(rgb(EDGE))
            .bg(rgb(PANEL))
            .child(Self::heading(
                "按键功能",
                &key.map(|k| format!("选中：{} · 选择一个功能并加入草稿", k.label))
                    .unwrap_or_else(|| "在键盘上选择一个按键".into()),
            ))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        Select::new(&self.output)
                            .placeholder("搜索按键 / F1–F12 / 导航 / 媒体功能")
                            .w(px(360.)),
                    )
                    .child(
                        Button::new("assign")
                            .label("加入草稿")
                            .outline()
                            .disabled(self.busy || !eligible || self.output_code.is_none())
                            .on_click(cx.listener(|this, _, _, cx| {
                                if let (Some(&id), Some(code)) =
                                    (this.selected_keys.first(), this.output_code)
                                    && id != 96
                                {
                                    this.profile.keymap.insert(id, code);
                                    this.dirty = true;
                                }
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("reset-key")
                            .label("恢复默认")
                            .ghost()
                            .disabled(self.busy || !eligible)
                            .on_click(cx.listener(|this, _, _, cx| {
                                if let Some(&id) = this.selected_keys.first() {
                                    this.profile.keymap.remove(&id);
                                    this.dirty = true;
                                }
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(MUTED))
                    .child(if selected == Some(96) {
                        "Fn 功能尚无可靠编码，保留默认设置。"
                    } else {
                        "紫色表示草稿中的改键。应用会覆盖原有自定义映射；暂不支持宏或 Fn 层编辑。"
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .children(self.profile.keymap.iter().map(|(&id, &code)| {
                        let label = KEYS
                            .iter()
                            .find(|k| k.id == id)
                            .map(|k| k.label)
                            .unwrap_or("?");
                        div()
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .bg(rgb(0x302a43))
                            .text_sm()
                            .child(format!("{label} → {}", output_label(code)))
                    })),
            )
    }
}

impl Render for Studio {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Retain subscriptions for the editors for the entire view lifetime.
        let _ = &self.subscriptions;
        let now = Instant::now();
        if self.tab == 0 && self.preview_playing && self.profile.effect != 1 {
            self.preview_seconds += now
                .duration_since(self.preview_last_tick)
                .as_secs_f32()
                .min(0.1);
            let view = cx.entity().downgrade();
            window.on_next_frame(move |_, cx| {
                let _ = view.update(cx, |_, cx| cx.notify());
            });
            window.request_animation_frame();
        }
        self.preview_last_tick = now;
        let unit = ((f32::from(window.viewport_size().width) - 370.) / 16.).clamp(38., 74.);
        div().size_full().flex().flex_col().bg(rgb(BG)).text_color(rgb(INK)).text_sm()
            .child(div().flex().items_center().justify_between().px_6().py_5().border_b_1().border_color(rgb(EDGE))
                .child(div().flex().items_center().gap_3()
                    .child(div().text_2xl().font_weight(FontWeight::BOLD).child("GMK67"))
                    .child(div().text_xs().text_color(rgb(ACCENT)).child("S T U D I O"))
                    .child(div().ml_3().text_color(rgb(MUTED)).child("键盘，按你的方式。")))
                .child(div().text_xs().text_color(rgb(MUTED)).child("ZUOYA  /  65%")))
            .child(div().flex().flex_1().min_h_0().gap_5().p_5()
                .child(div().w(px(250.)).flex_shrink_0().flex().flex_col().gap_4()
                    .child(self.device_card(cx))
                    .child(div().flex().flex_col().gap_3().p_4().rounded_xl().bg(rgb(PANEL)).border_1().border_color(rgb(EDGE))
                        .child(Self::heading("本地草稿", if self.dirty { "有未保存的更改" } else { "修改只发生在本地" }))
                        .child(Button::new("save").label("保存草稿").outline().disabled(self.busy).on_click(cx.listener(|this, _, _, cx| this.start(Job::Save(this.path.clone(), this.profile.clone()), cx))))
                        .child(Button::new("load").label("载入草稿").ghost().disabled(self.busy).on_click(cx.listener(|this, _, _, cx| this.start(Job::Load(this.path.clone()), cx))))
                        .child(div().text_xs().text_color(rgb(MUTED)).child(self.path.display().to_string())))
                    .child(div().text_xs().text_color(rgb(MUTED)).px_2().child("支持原版 GMK67 有线模式。设备效果需要实际检查；本地预览不代表设备当前配置。")))
                .child(div().id("workspace").flex().flex_col().flex_1().min_w_0().gap_4().overflow_y_scroll()
                    .child(div().flex().items_center().justify_between()
                        .child(div().flex().gap_2().children(["灯光", "改键"].into_iter().enumerate().map(|(tab, label)| {
                            Button::new(("tab", tab)).label(label).outline().when(self.tab == tab, |b| b.bg(rgb(0x2a4441)).text_color(rgb(ACCENT)))
                                .on_click(cx.listener(move |this, _, _, cx| { this.tab = tab; this.selected_keys.clear(); cx.notify(); }))
                        })))
                        .child(div().text_xs().text_color(rgb(MUTED)).child(if self.tab == 0 { "点击键帽选择颜色区域" } else { "点击键帽选择改键目标" })))
                    .child(self.keyboard(unit, cx))
                    .child(if self.tab == 0 { self.lighting_controls(cx) } else { self.keymap_controls(cx) })
                    .child(div().flex().items_center().justify_between().gap_4().p_4().rounded_xl().bg(rgb(PANEL)).border_1().border_color(rgb(EDGE))
                        .child(Self::heading("准备好后，应用到键盘", if self.tab == 0 { "将写入当前灯效和全部颜色" } else { "将替换键盘的自定义改键配置" }))
                        .child(Button::new("apply").primary().label(if self.busy { "处理中…" } else if self.tab == 0 { "应用灯光" } else { "应用改键" })
                            .disabled(self.busy || self.demo || self.selected_device.is_none()).on_click(cx.listener(|this, _, window, cx| this.confirm_apply(window, cx)))))))
            .child(div().px_6().py_3().border_t_1().border_color(rgb(EDGE)).text_xs().text_color(rgb(if self.error { 0xffad99 } else { MUTED })).child(self.status.clone()))
    }
}

/// Render the real GPUI views and exercise draft edits without opening HID devices.
#[cfg(feature = "preview")]
pub fn snapshot(path: &std::path::Path) -> Result<()> {
    use gpui_kit::test::TestWindowExt;
    use std::sync::Arc;
    let mut app = HeadlessAppContext::with_platform(
        platform::current_platform(true).text_system(),
        Arc::new(assets::Assets),
        platform::current_headless_renderer,
    );
    app.update(|cx| {
        init(cx);
        component::Theme::change(component::ThemeMode::Dark, None, cx);
        // Freeze overlay transitions so recorded bounds stay valid during clicks.
        cx.set_reduce_motion(true);
    });
    let mut studio = None;
    let window = app.open_window(size(px(1340.), px(900.)), |window, cx| {
        let view = cx.new(|cx| {
            Studio::new(
                PathBuf::from("/tmp/gmk67-studio-preview.json"),
                true,
                window,
                cx,
            )
        });
        studio = Some(view.clone());
        cx.new(|cx| component::Root::new(view, window, cx))
    })?;
    let studio = studio.expect("studio was constructed");
    app.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("apply", cx);
        assert!(!window.has_active_dialog(cx));
        studio.read_with(cx, |view, _| assert!(!view.busy));
        window.click(("key", 1usize), cx);
        studio.update(cx, |view, cx| {
            view.picker.update(cx, |picker, cx| {
                picker.select_color(rgb(0xef9966).into(), window, cx)
            });
        });
    })?;
    app.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("paint-selected", cx);
        studio.read_with(cx, |view, _| {
            assert!(view.selected_keys.contains(&1));
            assert_eq!(view.profile.colors[&1], [239, 153, 102]);
        });
        window.render_frame(cx);
    })?;
    app.capture_screenshot(window.into())?.save(path)?;
    app.update_window(window.into(), |_, window, cx| {
        window.click(("effect", 5usize), cx);
    })?;
    let before = app.capture_screenshot(window.into())?;
    std::thread::sleep(std::time::Duration::from_millis(80));
    app.update_window(window.into(), |_, window, cx| window.render_frame(cx))?;
    let after = app.capture_screenshot(window.into())?;
    assert_ne!(
        before.as_raw(),
        after.as_raw(),
        "wave preview did not animate"
    );
    after.save(path.with_file_name("wave.png"))?;
    app.update_window(window.into(), |_, window, cx| {
        window.click("toggle-preview", cx)
    })?;
    let paused = app.update(|cx| studio.read(cx).preview_seconds);
    std::thread::sleep(std::time::Duration::from_millis(40));
    app.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        studio.read_with(cx, |view, _| {
            assert_eq!(view.preview_seconds, paused, "paused preview advanced")
        });
        window.click(("effect", 0usize), cx);
    })?;
    let keymap_path = path.with_file_name("keymap.png");
    app.update_window(window.into(), |_, window, cx| {
        window.click(("tab", 1usize), cx);
        window.click(("key", 119usize), cx);
        studio.update(cx, |view, cx| {
            view.output.update(cx, |output, cx| {
                output.set_selected_value(&SharedString::from("F1"), window, cx);
                cx.emit(SelectEvent::Confirm(Some(SharedString::from("F1"))));
            });
            cx.notify();
        });
    })?;
    app.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("assign", cx);
        studio.read_with(cx, |view, _| {
            assert_eq!(view.profile.keymap.get(&119), Some(&0x3a))
        });
        window.click(("key", 96usize), cx);
        window.render_frame(cx);
        window.click("assign", cx);
        studio.read_with(cx, |view, _| {
            assert!(!view.profile.keymap.contains_key(&96))
        });
        window.click(("key", 119usize), cx);
        window.render_frame(cx);
    })?;
    app.capture_screenshot(window.into())?.save(&keymap_path)?;
    for tab in [0, 1] {
        app.update_window(window.into(), |_, window, cx| {
            studio.update(cx, |view, cx| {
                view.demo = false;
                view.preview_playing = false;
                view.tab = tab;
                view.devices = vec![Keyboard {
                    path: std::ffi::CString::new("/gmk67-test-no-device").unwrap(),
                    manufacturer: "hfd.cn".into(),
                    product: "USB DEVICE".into(),
                    serial: String::new(),
                }];
                view.selected_device = Some(0);
                cx.notify();
            });
            window.render_frame(cx);
            window.click("apply", cx);
        })?;
        app.update_window(window.into(), |_, window, cx| {
            window.render_frame(cx);
            assert!(window.within("dialog").find("confirm-write").visible());
            assert!(window.within("dialog").find("cancel-write").visible());
        })?;
        if tab == 0 {
            app.capture_screenshot(window.into())?
                .save(path.with_file_name("confirmation.png"))?;
        }
        app.update_window(window.into(), |_, window, cx| {
            window.within("dialog").click("cancel-write", cx);
        })?;
        app.run_until_parked();
        app.update_window(window.into(), |_, window, cx| {
            assert!(!window.has_active_dialog(cx));
            studio.read_with(cx, |view, _| {
                assert!(!view.busy, "cancel started an I/O job")
            });
        })?;
    }
    app.update_window(window.into(), |_, window, cx| {
        studio.update(cx, |view, cx| {
            view.tab = 0;
            cx.notify();
        });
        window.click("apply", cx);
    })?;
    app.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        window.within("dialog").click("confirm-write", cx);
    })?;
    app.run_until_parked();
    app.update_window(window.into(), |_, window, cx| {
        assert!(!window.has_active_dialog(cx));
        studio.read_with(cx, |view, _| {
            assert!(!view.busy);
            assert!(
                view.error && view.status.contains("设备已断开"),
                "confirm did not run the write job against the nonexistent test path: {}",
                view.status
            );
        });
    })?;
    println!(
        "UI interactions and confirm/cancel passed; screenshots: {}, {}",
        path.display(),
        keymap_path.display()
    );
    Ok(())
}
