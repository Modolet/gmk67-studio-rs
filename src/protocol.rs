//! @file protocol.rs
//! @brief Pure encoding of the observed GMK67 feature-report sequences.
//! @author modolet <y@xxyx.io>
//! @date 2026-10-12
// Protocol provenance: shtefan4079/gmk67_devicesdriver_open, commit 9cabde0.
// Packet bytes and matrix locations are protocol facts; no upstream code is embedded.

use crate::{layout::KEYS, profile::Profile};
use anyhow::Result;

#[derive(Clone, Debug, PartialEq)]
pub struct Packet {
    pub payload: [u8; 64],
    pub settle_ms: u64,
}

impl Packet {
    fn new(bytes: &[u8]) -> Self {
        let mut payload = [0; 64];
        payload[..bytes.len()].copy_from_slice(bytes);
        Self {
            payload,
            settle_ms: 0,
        }
    }
    pub fn wire(&self) -> [u8; 65] {
        let mut wire = [0; 65];
        wire[1..].copy_from_slice(&self.payload);
        wire
    }
}

fn finish(packets: &mut Vec<Packet>, settle_ms: u64) {
    let mut trailer = Packet::new(&[]);
    trailer.payload[62..].copy_from_slice(&[0xaa, 0x55]);
    trailer.settle_ms = settle_ms;
    packets.push(trailer);
    packets.push(Packet::new(&[4, 2]));
    packets.push(Packet::new(&[4, 0xf0]));
}

pub fn lighting(profile: &Profile) -> Result<Vec<Packet>> {
    profile.validate()?;
    let mut packets = vec![
        Packet::new(&[4, 0x18]),
        Packet::new(&[4, 0x23, 0, 0, 0, 0, 0, 0, 9]),
    ];
    // The observed color block is eight data reports followed by a trailer.
    // Each 4-byte slot stores the matrix index then RGB; padding remains zero.
    let mut matrix = [0u8; 512];
    for key in KEYS {
        let offset = usize::from(key.id) * 4;
        matrix[offset] = key.id;
        matrix[offset + 1..offset + 4]
            .copy_from_slice(profile.colors.get(&key.id).unwrap_or(&[0, 0, 0]));
    }
    for chunk in matrix.chunks_exact(64) {
        packets.push(Packet::new(chunk));
    }
    finish(&mut packets, 50);
    packets.push(Packet::new(&[4, 0x18]));
    packets.push(Packet::new(&[4, 0x13, 0, 0, 0, 0, 0, 0, profile.effect]));
    let first = profile.colors.values().next().copied().unwrap_or([0, 0, 0]);
    let color =
        if profile.colors.len() == KEYS.len() && profile.colors.values().all(|c| *c == first) {
            first
        } else {
            [0, 0, 0]
        };
    packets.push(Packet::new(&[
        0x80,
        color[0],
        color[1],
        color[2],
        0,
        0,
        0,
        0,
        profile.effect,
        15,
        15,
        0,
        0,
        0,
        0xaa,
        0x55,
    ]));
    Ok(packets)
}

pub fn keymap(profile: &Profile) -> Result<Vec<Packet>> {
    profile.validate()?;
    let mut packets = Vec::with_capacity(30);
    // 0x27 is cleared by the reference sequence. Its meaning is undocumented;
    // the UI requires explicit confirmation before sending this replacement.
    for block in [0x11, 0x27] {
        packets.push(Packet::new(&[]));
        packets.push(Packet::new(&[4, 0x18]));
        packets.push(Packet::new(&[4, block, 0, 0, 0, 0, 0, 0, 9]));
        let mut matrix = [0u8; 576];
        if block == 0x11 {
            for (&id, &code) in &profile.keymap {
                let offset = usize::from(id) * 4;
                matrix[offset] = 2;
                matrix[offset + 2] = code;
            }
        }
        for chunk in matrix.chunks_exact(64) {
            packets.push(Packet::new(chunk));
        }
        finish(&mut packets, 0);
    }
    Ok(packets)
}

/// The only transport surface needed by the protocol. A fake verifies failure
/// boundaries without opening a real keyboard.
pub trait FeatureTransport {
    fn exchange(&mut self, report: &[u8; 65]) -> Result<()>;
}

pub fn transmit(
    transport: &mut impl FeatureTransport,
    packets: &[Packet],
    mut wait: impl FnMut(u64),
) -> Result<usize> {
    for (index, packet) in packets.iter().enumerate() {
        use anyhow::Context;
        transport.exchange(&packet.wire()).with_context(|| {
            format!(
                "第 {}/{} 个报文失败；设备配置可能仅部分写入",
                index + 1,
                packets.len()
            )
        })?;
        wait(packet.settle_ms);
    }
    Ok(packets.len())
}
