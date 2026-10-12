//! @file layout.rs
//! @brief Physical GMK67 matrix locations and supported keyboard HID usages.
//! @author modolet <y@xxyx.io>
//! @date 2026-10-12

#[derive(Clone, Copy, Debug)]
pub struct Key {
    pub id: u8,
    pub label: &'static str,
    pub x: f32,
    pub row: u8,
    pub width: f32,
}

pub const KEYS: &[Key] = &[
    Key {
        id: 1,
        label: "Esc",
        x: 0.0,
        row: 0,
        width: 1.0,
    },
    Key {
        id: 20,
        label: "1",
        x: 1.0,
        row: 0,
        width: 1.0,
    },
    Key {
        id: 21,
        label: "2",
        x: 2.0,
        row: 0,
        width: 1.0,
    },
    Key {
        id: 22,
        label: "3",
        x: 3.0,
        row: 0,
        width: 1.0,
    },
    Key {
        id: 23,
        label: "4",
        x: 4.0,
        row: 0,
        width: 1.0,
    },
    Key {
        id: 24,
        label: "5",
        x: 5.0,
        row: 0,
        width: 1.0,
    },
    Key {
        id: 25,
        label: "6",
        x: 6.0,
        row: 0,
        width: 1.0,
    },
    Key {
        id: 26,
        label: "7",
        x: 7.0,
        row: 0,
        width: 1.0,
    },
    Key {
        id: 27,
        label: "8",
        x: 8.0,
        row: 0,
        width: 1.0,
    },
    Key {
        id: 28,
        label: "9",
        x: 9.0,
        row: 0,
        width: 1.0,
    },
    Key {
        id: 29,
        label: "0",
        x: 10.0,
        row: 0,
        width: 1.0,
    },
    Key {
        id: 30,
        label: "-",
        x: 11.0,
        row: 0,
        width: 1.0,
    },
    Key {
        id: 31,
        label: "=",
        x: 12.0,
        row: 0,
        width: 1.0,
    },
    Key {
        id: 103,
        label: "Bksp",
        x: 13.0,
        row: 0,
        width: 2.0,
    },
    Key {
        id: 119,
        label: "Del",
        x: 15.0,
        row: 1,
        width: 1.0,
    },
    Key {
        id: 37,
        label: "Tab",
        x: 0.0,
        row: 1,
        width: 1.5,
    },
    Key {
        id: 38,
        label: "Q",
        x: 1.5,
        row: 1,
        width: 1.0,
    },
    Key {
        id: 39,
        label: "W",
        x: 2.5,
        row: 1,
        width: 1.0,
    },
    Key {
        id: 40,
        label: "E",
        x: 3.5,
        row: 1,
        width: 1.0,
    },
    Key {
        id: 41,
        label: "R",
        x: 4.5,
        row: 1,
        width: 1.0,
    },
    Key {
        id: 42,
        label: "T",
        x: 5.5,
        row: 1,
        width: 1.0,
    },
    Key {
        id: 43,
        label: "Y",
        x: 6.5,
        row: 1,
        width: 1.0,
    },
    Key {
        id: 44,
        label: "U",
        x: 7.5,
        row: 1,
        width: 1.0,
    },
    Key {
        id: 45,
        label: "I",
        x: 8.5,
        row: 1,
        width: 1.0,
    },
    Key {
        id: 46,
        label: "O",
        x: 9.5,
        row: 1,
        width: 1.0,
    },
    Key {
        id: 47,
        label: "P",
        x: 10.5,
        row: 1,
        width: 1.0,
    },
    Key {
        id: 48,
        label: "[",
        x: 11.5,
        row: 1,
        width: 1.0,
    },
    Key {
        id: 49,
        label: "]",
        x: 12.5,
        row: 1,
        width: 1.0,
    },
    Key {
        id: 67,
        label: "\\",
        x: 13.5,
        row: 1,
        width: 1.5,
    },
    Key {
        id: 118,
        label: "PgUp",
        x: 15.0,
        row: 2,
        width: 1.0,
    },
    Key {
        id: 55,
        label: "Caps",
        x: 0.0,
        row: 2,
        width: 1.75,
    },
    Key {
        id: 56,
        label: "A",
        x: 1.75,
        row: 2,
        width: 1.0,
    },
    Key {
        id: 57,
        label: "S",
        x: 2.75,
        row: 2,
        width: 1.0,
    },
    Key {
        id: 58,
        label: "D",
        x: 3.75,
        row: 2,
        width: 1.0,
    },
    Key {
        id: 59,
        label: "F",
        x: 4.75,
        row: 2,
        width: 1.0,
    },
    Key {
        id: 60,
        label: "G",
        x: 5.75,
        row: 2,
        width: 1.0,
    },
    Key {
        id: 61,
        label: "H",
        x: 6.75,
        row: 2,
        width: 1.0,
    },
    Key {
        id: 62,
        label: "J",
        x: 7.75,
        row: 2,
        width: 1.0,
    },
    Key {
        id: 63,
        label: "K",
        x: 8.75,
        row: 2,
        width: 1.0,
    },
    Key {
        id: 64,
        label: "L",
        x: 9.75,
        row: 2,
        width: 1.0,
    },
    Key {
        id: 65,
        label: ";",
        x: 10.75,
        row: 2,
        width: 1.0,
    },
    Key {
        id: 66,
        label: "'",
        x: 11.75,
        row: 2,
        width: 1.0,
    },
    Key {
        id: 85,
        label: "Enter",
        x: 12.75,
        row: 2,
        width: 2.25,
    },
    Key {
        id: 121,
        label: "PgDn",
        x: 15.0,
        row: 3,
        width: 1.0,
    },
    Key {
        id: 73,
        label: "Shift",
        x: 0.0,
        row: 3,
        width: 2.25,
    },
    Key {
        id: 74,
        label: "Z",
        x: 2.25,
        row: 3,
        width: 1.0,
    },
    Key {
        id: 75,
        label: "X",
        x: 3.25,
        row: 3,
        width: 1.0,
    },
    Key {
        id: 76,
        label: "C",
        x: 4.25,
        row: 3,
        width: 1.0,
    },
    Key {
        id: 77,
        label: "V",
        x: 5.25,
        row: 3,
        width: 1.0,
    },
    Key {
        id: 78,
        label: "B",
        x: 6.25,
        row: 3,
        width: 1.0,
    },
    Key {
        id: 79,
        label: "N",
        x: 7.25,
        row: 3,
        width: 1.0,
    },
    Key {
        id: 80,
        label: "M",
        x: 8.25,
        row: 3,
        width: 1.0,
    },
    Key {
        id: 81,
        label: ",",
        x: 9.25,
        row: 3,
        width: 1.0,
    },
    Key {
        id: 82,
        label: ".",
        x: 10.25,
        row: 3,
        width: 1.0,
    },
    Key {
        id: 83,
        label: "/",
        x: 11.25,
        row: 3,
        width: 1.0,
    },
    Key {
        id: 84,
        label: "Shft",
        x: 12.25,
        row: 3,
        width: 1.75,
    },
    Key {
        id: 101,
        label: "Up",
        x: 14.0,
        row: 3,
        width: 1.0,
    },
    Key {
        id: 91,
        label: "Ctrl",
        x: 0.0,
        row: 4,
        width: 1.25,
    },
    Key {
        id: 92,
        label: "Win",
        x: 1.25,
        row: 4,
        width: 1.25,
    },
    Key {
        id: 93,
        label: "Alt",
        x: 2.5,
        row: 4,
        width: 1.25,
    },
    Key {
        id: 94,
        label: "Space",
        x: 3.75,
        row: 4,
        width: 6.25,
    },
    Key {
        id: 95,
        label: "Alt",
        x: 10.0,
        row: 4,
        width: 1.25,
    },
    Key {
        id: 96,
        label: "Fn",
        x: 11.25,
        row: 4,
        width: 1.25,
    },
    Key {
        id: 99,
        label: "Left",
        x: 13.0,
        row: 4,
        width: 1.0,
    },
    Key {
        id: 100,
        label: "Down",
        x: 14.0,
        row: 4,
        width: 1.0,
    },
    Key {
        id: 102,
        label: "Right",
        x: 15.0,
        row: 4,
        width: 1.0,
    },
];

#[derive(Clone, Copy, Debug)]
pub struct Output {
    pub label: &'static str,
    pub code: u8,
}

pub const OUTPUTS: &[Output] = &[
    Output {
        label: "A",
        code: 0x04,
    },
    Output {
        label: "B",
        code: 0x05,
    },
    Output {
        label: "C",
        code: 0x06,
    },
    Output {
        label: "D",
        code: 0x07,
    },
    Output {
        label: "E",
        code: 0x08,
    },
    Output {
        label: "F",
        code: 0x09,
    },
    Output {
        label: "G",
        code: 0x0A,
    },
    Output {
        label: "H",
        code: 0x0B,
    },
    Output {
        label: "I",
        code: 0x0C,
    },
    Output {
        label: "J",
        code: 0x0D,
    },
    Output {
        label: "K",
        code: 0x0E,
    },
    Output {
        label: "L",
        code: 0x0F,
    },
    Output {
        label: "M",
        code: 0x10,
    },
    Output {
        label: "N",
        code: 0x11,
    },
    Output {
        label: "O",
        code: 0x12,
    },
    Output {
        label: "P",
        code: 0x13,
    },
    Output {
        label: "Q",
        code: 0x14,
    },
    Output {
        label: "R",
        code: 0x15,
    },
    Output {
        label: "S",
        code: 0x16,
    },
    Output {
        label: "T",
        code: 0x17,
    },
    Output {
        label: "U",
        code: 0x18,
    },
    Output {
        label: "V",
        code: 0x19,
    },
    Output {
        label: "W",
        code: 0x1A,
    },
    Output {
        label: "X",
        code: 0x1B,
    },
    Output {
        label: "Y",
        code: 0x1C,
    },
    Output {
        label: "Z",
        code: 0x1D,
    },
    Output {
        label: "1",
        code: 0x1E,
    },
    Output {
        label: "2",
        code: 0x1F,
    },
    Output {
        label: "3",
        code: 0x20,
    },
    Output {
        label: "4",
        code: 0x21,
    },
    Output {
        label: "5",
        code: 0x22,
    },
    Output {
        label: "6",
        code: 0x23,
    },
    Output {
        label: "7",
        code: 0x24,
    },
    Output {
        label: "8",
        code: 0x25,
    },
    Output {
        label: "9",
        code: 0x26,
    },
    Output {
        label: "0",
        code: 0x27,
    },
    Output {
        label: "ENTER",
        code: 0x28,
    },
    Output {
        label: "ESC",
        code: 0x29,
    },
    Output {
        label: "BACKSPACE",
        code: 0x2A,
    },
    Output {
        label: "TAB",
        code: 0x2B,
    },
    Output {
        label: "SPACE",
        code: 0x2C,
    },
    Output {
        label: "MINUS",
        code: 0x2D,
    },
    Output {
        label: "EQUAL",
        code: 0x2E,
    },
    Output {
        label: "LBRACKET",
        code: 0x2F,
    },
    Output {
        label: "RBRACKET",
        code: 0x30,
    },
    Output {
        label: "BACKSLASH",
        code: 0x31,
    },
    Output {
        label: "SEMICOLON",
        code: 0x33,
    },
    Output {
        label: "QUOTE",
        code: 0x34,
    },
    Output {
        label: "TILDE",
        code: 0x35,
    },
    Output {
        label: "COMMA",
        code: 0x36,
    },
    Output {
        label: "PERIOD",
        code: 0x37,
    },
    Output {
        label: "SLASH",
        code: 0x38,
    },
    Output {
        label: "CAPS_LOCK",
        code: 0x39,
    },
    Output {
        label: "F1",
        code: 0x3A,
    },
    Output {
        label: "F2",
        code: 0x3B,
    },
    Output {
        label: "F3",
        code: 0x3C,
    },
    Output {
        label: "F4",
        code: 0x3D,
    },
    Output {
        label: "F5",
        code: 0x3E,
    },
    Output {
        label: "F6",
        code: 0x3F,
    },
    Output {
        label: "F7",
        code: 0x40,
    },
    Output {
        label: "F8",
        code: 0x41,
    },
    Output {
        label: "F9",
        code: 0x42,
    },
    Output {
        label: "F10",
        code: 0x43,
    },
    Output {
        label: "F11",
        code: 0x44,
    },
    Output {
        label: "F12",
        code: 0x45,
    },
    Output {
        label: "PRINTSCREEN",
        code: 0x46,
    },
    Output {
        label: "SCROLLLOCK",
        code: 0x47,
    },
    Output {
        label: "PAUSE",
        code: 0x48,
    },
    Output {
        label: "INSERT",
        code: 0x49,
    },
    Output {
        label: "HOME",
        code: 0x4A,
    },
    Output {
        label: "PAGE_UP",
        code: 0x4B,
    },
    Output {
        label: "DELETE",
        code: 0x4C,
    },
    Output {
        label: "END",
        code: 0x4D,
    },
    Output {
        label: "PAGE_DOWN",
        code: 0x4E,
    },
    Output {
        label: "RIGHT",
        code: 0x4F,
    },
    Output {
        label: "LEFT",
        code: 0x50,
    },
    Output {
        label: "DOWN",
        code: 0x51,
    },
    Output {
        label: "UP",
        code: 0x52,
    },
    Output {
        label: "MUTE",
        code: 0x7F,
    },
    Output {
        label: "VOL_UP",
        code: 0x80,
    },
    Output {
        label: "VOL_DOWN",
        code: 0x81,
    },
    Output {
        label: "POWER",
        code: 0x66,
    },
    Output {
        label: "SLEEP",
        code: 0x74,
    },
    Output {
        label: "WAKE",
        code: 0x75,
    },
];

pub fn output_label(code: u8) -> &'static str {
    OUTPUTS
        .iter()
        .find(|o| o.code == code)
        .map(|o| o.label)
        .unwrap_or("未知")
}
