//! @file protocol.rs
//! @brief Golden-packet parity and failures that must never reach a keyboard.
//! @author modolet <y@xxyx.io>
//! @date 2026-10-12

use anyhow::{Result, bail};
use gmk67_studio_rs::{
    device::is_wired,
    layout::KEYS,
    profile::Profile,
    protocol::{self, FeatureTransport},
};

#[test]
fn red_static_packets_match_reference_capture_template() {
    let mut profile = Profile::default();
    profile
        .colors
        .values_mut()
        .for_each(|rgb| *rgb = [255, 0, 0]);
    let expected: Vec<Vec<u8>> = include_str!("golden-lighting.txt")
        .lines()
        .map(|line| {
            line.split_whitespace()
                .map(|hex| u8::from_str_radix(hex, 16).unwrap())
                .collect()
        })
        .collect();
    let packets = protocol::lighting(&profile).unwrap();
    assert_eq!(packets.len(), expected.len());
    for (index, (packet, expected)) in packets.iter().zip(expected).enumerate() {
        assert_eq!(
            packet.wire().as_slice(),
            expected,
            "reference packet {index}"
        );
    }
}

#[test]
fn recoloring_escape_does_not_mutate_command_headers_or_padding() {
    let mut profile = Profile::default();
    profile.colors.insert(1, [4, 19, 128]);
    profile.effect = 6;
    let packets = protocol::lighting(&profile).unwrap();
    assert_eq!(&packets[2].payload[4..8], &[1, 4, 19, 128]);
    assert_eq!(&packets[0].payload[..2], &[4, 0x18]);
    assert_eq!(&packets[14].payload[..9], &[4, 0x13, 0, 0, 0, 0, 0, 0, 6]);
    assert_eq!(&packets[2].payload[..4], &[0; 4]);
}

#[test]
fn keymap_preserves_reference_block_order_and_trailers() {
    let mut profile = Profile::default();
    profile.keymap.insert(119, 0x3a); // Del -> F1, near the end of the matrix.
    let packets = protocol::keymap(&profile).unwrap();
    assert_eq!(packets.len(), 30);
    assert_eq!(packets[2].payload[1], 0x11);
    assert_eq!(packets[17].payload[1], 0x27);
    let matrix: Vec<u8> = packets[3..12].iter().flat_map(|p| p.payload).collect();
    assert_eq!(&matrix[119 * 4..119 * 4 + 4], &[2, 0, 0x3a, 0]);
    assert!(packets[18..27].iter().all(|p| p.payload == [0; 64]));
    for offset in [12, 27] {
        assert_eq!(&packets[offset].payload[62..], &[0xaa, 0x55]);
    }
}

#[test]
fn invalid_drafts_are_rejected_before_encoding() {
    for profile in [
        Profile {
            colors: Default::default(),
            ..Profile::default()
        },
        Profile {
            version: 2,
            ..Profile::default()
        },
        Profile {
            effect: 0,
            ..Profile::default()
        },
        Profile {
            keymap: [(96, 4)].into(),
            ..Profile::default()
        },
        Profile {
            keymap: [(255, 4)].into(),
            ..Profile::default()
        },
        Profile {
            keymap: [(1, 255)].into(),
            ..Profile::default()
        },
    ] {
        assert!(protocol::lighting(&profile).is_err());
        assert!(protocol::keymap(&profile).is_err());
    }
}

#[test]
fn wireless_receiver_and_apple_id_collision_are_excluded() {
    assert!(is_wired("hfd.cn", "USB DEVICE"));
    assert!(!is_wired("HS6209", "USB Dongle"));
    assert!(!is_wired("Apple, Inc.", "Aluminium Keyboard"));
    assert!(!is_wired("", "USB DEVICE"));
}

#[test]
fn first_io_error_stops_the_batch_without_retrying_the_rest() {
    struct Fake {
        calls: usize,
    }
    impl FeatureTransport for Fake {
        fn exchange(&mut self, _: &[u8; 65]) -> Result<()> {
            self.calls += 1;
            if self.calls == 3 {
                bail!("disconnected");
            }
            Ok(())
        }
    }
    let mut fake = Fake { calls: 0 };
    let error = protocol::transmit(
        &mut fake,
        &protocol::lighting(&Profile::default()).unwrap(),
        |_| {},
    )
    .unwrap_err();
    assert_eq!(fake.calls, 3);
    assert!(format!("{error:#}").contains("部分写入"));
}

#[test]
fn layout_contains_exactly_66_unique_keys_inside_color_matrix() {
    let ids: std::collections::BTreeSet<_> = KEYS.iter().map(|k| k.id).collect();
    assert_eq!(ids.len(), 66);
    assert_eq!(KEYS.len(), 66);
    assert!(ids.iter().all(|id| *id < 128));
}

#[test]
fn only_lighting_trailer_has_extra_reference_settle_delay() {
    let profile = Profile::default();
    let lighting = protocol::lighting(&profile).unwrap();
    assert_eq!(lighting[10].settle_ms, 50);
    assert!(
        lighting
            .iter()
            .enumerate()
            .all(|(i, p)| i == 10 || p.settle_ms == 0)
    );
    assert!(
        protocol::keymap(&profile)
            .unwrap()
            .iter()
            .all(|p| p.settle_ms == 0)
    );
}
