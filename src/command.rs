//! Commands for the Embedded Controller.
//!
//! Reference: https://github.com/FrameworkComputer/EmbeddedController/blob/hx20-hx30/include/ec_commands.h
//!
//! (command IDs begin with `EC_CMD_`)

#![allow(dead_code)]

use bytemuck::{NoUninit, Pod, Zeroable};

/// Trait implemented by Embedded Controller commands.
pub trait Command: NoUninit {
    /// The command ID.
    const CMD: Cmd;

    /// Command version.
    ///
    /// Some commands come in multiple versions.
    const VERSION: u32 = 0;

    /// The associated response type.
    type Response: Pod;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cmd {
    #[allow(unused)] // no longer used by cros-ec
    ProtoVersion = 0x0000,
    Hello = 0x0001,
    GetVersion = 0x0002,
    // ...
    GetKeyboardBacklight = 0x0022,
    SetKeyboardBacklight = 0x0023,
    LedControl = 0x0029,
}

//////////////////////////////////
// Hello
//////////////////////////////////

#[derive(Clone, Copy, NoUninit)]
#[repr(C)]
pub struct Hello {
    pub in_data: u32,
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct HelloResponse {
    pub out_data: u32,
}

impl Command for Hello {
    const CMD: Cmd = Cmd::Hello;
    type Response = HelloResponse;
}

//////////////////////////////////
// GetVersion
//////////////////////////////////

#[derive(Clone, Copy, NoUninit)]
#[repr(C)]
pub struct GetVersion;

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct GetVersionResponse {
    version_string_ro: [u8; 32],
    version_string_rw: [u8; 32],
    reserved: [u8; 32],
    current_image: u32,
}

impl Command for GetVersion {
    const CMD: Cmd = Cmd::GetVersion;
    type Response = GetVersionResponse;
}

//////////////////////////////////
// GetKeyboardBacklight
//////////////////////////////////

#[derive(Clone, Copy, NoUninit)]
#[repr(C)]
pub struct GetKeyboardBacklight;

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct GetKeyboardBacklightResponse {
    pub percent: u8,
    pub enabled: u8,
}

impl Command for GetKeyboardBacklight {
    const CMD: Cmd = Cmd::GetKeyboardBacklight;
    type Response = GetKeyboardBacklightResponse;
}

//////////////////////////////////
// SetKeyboardBacklight
//////////////////////////////////

#[derive(Clone, Copy, NoUninit)]
#[repr(C)]
pub struct SetKeyboardBacklight {
    pub percent: u8,
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct SetKeyboardBacklightResponse;

impl Command for SetKeyboardBacklight {
    const CMD: Cmd = Cmd::SetKeyboardBacklight;
    type Response = SetKeyboardBacklightResponse;
}

//////////////////////////////////
// LedControl
//////////////////////////////////

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct LedControl {
    pub led_id: LedId,
    pub flags: LedFlags,
    pub brightness: LedBrightnesses,
}

impl Command for LedControl {
    const CMD: Cmd = Cmd::LedControl;
    // ectool always uses version 1 for this command, version 0 does not work and returns unexpected
    // data.
    const VERSION: u32 = 1;
    type Response = LedControlResponse;
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(transparent)]
pub struct LedId(u8);

impl LedId {
    pub const BATTERY: Self = Self(0);
    pub const POWER: Self = Self(1);
    pub const ADAPTER: Self = Self(2);
    pub const LEFT: Self = Self(3);
    pub const RIGHT: Self = Self(4);
    pub const RECOVERY_HW_REINIT: Self = Self(5);
    pub const SYSRQ_DEBUG: Self = Self(6);
}

#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
#[repr(transparent)]
pub struct LedFlags(u8);

impl LedFlags {
    pub const NONE: Self = Self(0);
    pub const QUERY: Self = Self(1 << 0);
    pub const AUTO: Self = Self(1 << 1);
}

pub struct LedColor(u8);

impl LedColor {
    pub const RED: Self = Self(0);
    pub const GREEN: Self = Self(1);
    pub const BLUE: Self = Self(2);
    pub const YELLOW: Self = Self(3);
    pub const WHITE: Self = Self(4);
    pub const AMBER: Self = Self(5);
    pub const COUNT: usize = 6;
}

#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
#[repr(transparent)]
pub struct LedBrightnesses {
    raw: [u8; LedColor::COUNT],
}

impl LedBrightnesses {
    pub fn single(color: LedColor, brightness: u8) -> Self {
        Self::default().set(color, brightness)
    }

    pub fn set(mut self, color: LedColor, brightness: u8) -> Self {
        self.raw[usize::from(color.0)] = brightness;
        self
    }
}

#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
#[repr(transparent)]
pub struct LedControlResponse {
    brightness: LedBrightnesses,
}
