use bytemuck::{NoUninit, Pod, Zeroable};

/// Trait implemented by Embedded Controller commands.
pub trait Command: NoUninit {
    /// The command ID.
    const CMD: Cmd;

    /// Command version.
    ///
    /// Some commands come in multiple versions (although none of the ones supported here).
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
