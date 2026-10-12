//! @file lighting_preview.rs
//! @brief Verify that animated previews change without changing device drafts.
//! @author modolet <y@xxyx.io>
//! @date 2026-10-12

use gmk67_studio_rs::{layout::KEYS, lighting_preview::color, profile::Profile};

#[test]
fn static_preview_keeps_each_draft_color_and_black_stays_off() {
    let profile = Profile::default();
    for key in KEYS {
        let base = profile.colors[&key.id];
        assert_eq!(color(1, base, key, 0., None), base);
        assert_eq!(color(1, base, key, 19., None), base);
        assert_eq!(color(2, [0; 3], key, 0.8, None), [0; 3]);
    }
}

#[test]
fn every_dynamic_effect_changes_the_keyboard_over_time() {
    let profile = Profile::default();
    for effect in 2..=8 {
        let frame = |seconds| {
            KEYS.iter()
                .map(|key| color(effect, profile.colors[&key.id], key, seconds, None))
                .collect::<Vec<_>>()
        };
        assert_ne!(frame(0.4), frame(1.1), "effect {effect} is frozen");
    }
    assert_eq!(
        profile,
        Profile::default(),
        "preview must not alter the draft"
    );
}

#[test]
fn ripple_starts_at_the_clicked_key_and_moves_away() {
    let key = &KEYS[0];
    let origin = Some((key.x + key.width / 2., f32::from(key.row), 4.));
    assert_eq!(color(5, [255, 128, 64], key, 4., origin), [255, 128, 64]);
    assert_eq!(color(5, [255, 128, 64], key, 5., origin), [0; 3]);
}
