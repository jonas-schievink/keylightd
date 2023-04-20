use std::{
    fs::File,
    io,
    mem::{size_of, size_of_val, MaybeUninit},
    os::fd::AsRawFd,
};

use nix::{errno::Errno, libc::ioctl, request_code_readwrite};

use crate::command::{self, Hello};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IoctlVersion {
    V1,
    V2,
}

/// A handle to the system's ChromiumOS Embedded Controller.
///
/// This uses the ioctl interface of `/dev/cros_ec` to issue commands.
pub struct EmbeddedController {
    fd: File,
    version: IoctlVersion,
}

impl EmbeddedController {
    pub fn open() -> io::Result<Self> {
        let mut this = Self {
            fd: File::options()
                .read(true)
                .write(true)
                .open("/dev/cros_ec")?,
            version: IoctlVersion::V1,
        };

        // The framework EC uses ioctl interface version 2, but this mirrors the logic in ectool
        // just to make sure it doesn't do something nonsensical on non-Framework machines.
        this.version = match this.cmd_v1(Hello {
            in_data: 0xa0b0c0d0,
        }) {
            Err(Errno::ENOTTY) => IoctlVersion::V2,
            _ => IoctlVersion::V1,
        };

        log::debug!("ioctl version {:?}", this.version);

        // Test communication by issuing a `Hello` command and reading back the result.
        let magic = 0xaa55dead;
        let resp = this.command(Hello { in_data: magic })?;
        let expected = magic + 0x01020304;
        if resp.out_data != expected {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "failed to connect to EC: invalid response to hello command (received {:010x}, expected {:010x})",
                    resp.out_data, expected,
                ),
            ));
        }

        log::info!("connected to embedded controller");

        Ok(this)
    }

    pub fn command<C: command::Command>(&self, cmd: C) -> io::Result<C::Response> {
        match self.version {
            IoctlVersion::V1 => self.cmd_v1(cmd),
            IoctlVersion::V2 => self.cmd_v2(cmd),
        }
        .map_err(Into::into)
    }

    fn cmd_v1<C: command::Command>(&self, cmd: C) -> nix::Result<C::Response> {
        let mut resp = MaybeUninit::<C::Response>::uninit();
        let mut cmd = CommandV1 {
            version: C::VERSION,
            command: C::CMD as u32,
            outdata: bytemuck::bytes_of(&cmd).as_ptr() as *mut _,
            outsize: size_of_val(&cmd).try_into().unwrap(),
            indata: resp.as_mut_ptr().cast(),
            insize: size_of_val(&resp).try_into().unwrap(),
            result: 0xff,
        };
        unsafe {
            let ret = ioctl(
                self.fd.as_raw_fd(),
                request_code_readwrite!(':', 0, size_of::<CommandV1>()),
                &mut cmd,
            );
            Errno::result(ret)?;
            Ok(resp.assume_init())
        }
    }

    fn cmd_v2<C: command::Command>(&self, cmd: C) -> nix::Result<C::Response> {
        let mut cmd = CommandV2 {
            header: CommandV2Header {
                version: C::VERSION,
                command: C::CMD as u32,
                outsize: size_of::<C>().try_into().unwrap(),
                insize: size_of::<C::Response>().try_into().unwrap(),
                result: 0xff,
            },
            data: CommandV2Union { req: cmd },
        };

        unsafe {
            let ret = ioctl(
                self.fd.as_raw_fd(),
                request_code_readwrite!(0xEC, 0, size_of::<CommandV2Header>()),
                &mut cmd,
            );
            Errno::result(ret)?;
            Ok(cmd.data.resp)
        }
    }
}

#[repr(C)]
struct CommandV1 {
    version: u32,
    command: u32,
    outdata: *mut u8,
    outsize: u32,
    indata: *mut u8,
    insize: u32,
    result: u32,
}

#[repr(C)]
struct CommandV2<C: command::Command> {
    header: CommandV2Header,
    // Request and response are stored in a `union` rather than using an empty trailing array like
    // the C code does. I believe this is ABI-equivalent, so it shouldn't cause problems.
    data: CommandV2Union<C>,
}

#[repr(C)]
struct CommandV2Header {
    version: u32,
    command: u32,
    outsize: u32,
    insize: u32,
    result: u32,
}

#[repr(C)]
union CommandV2Union<C: command::Command> {
    req: C,
    resp: C::Response,
}
