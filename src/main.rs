use std::{
    io,
    sync::{Arc, Condvar, Mutex},
    thread,
    time::{Duration, Instant},
};

use argh::FromArgs;
use command::{GetKeyboardBacklight, SetKeyboardBacklight};
use ec::EmbeddedController;

use crate::command::{LedBrightnesses, LedControl, LedFlags, LedId};

mod command;
mod ec;

/// keylightd - automatic keyboard backlight daemon for Framework laptops
#[derive(Debug, FromArgs)]
struct Args {
    /// brightness level when active (0-100) [default=30]
    #[argh(option, default = "30", from_str_fn(parse_brightness))]
    brightness: u8,

    /// activity timeout in seconds [default=10]
    #[argh(option, default = "10")]
    timeout: u32,

    /// also control the power LED in the fingerprint module
    #[argh(switch)]
    power: bool,
}

fn parse_brightness(s: &str) -> Result<u8, String> {
    let brightness = s.parse::<u8>().map_err(|e| e.to_string())?;
    if brightness > 100 {
        return Err("invalid brightness value {brightness} (valid range: 0-100)".into());
    }
    Ok(brightness)
}

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_module(
            env!("CARGO_PKG_NAME"),
            if cfg!(debug_assertions) {
                log::LevelFilter::Debug
            } else {
                log::LevelFilter::Info
            },
        )
        .init();

    let args: Args = argh::from_env();
    log::debug!("args={:?}", args);

    let ec = EmbeddedController::open()?;
    let fade_to = |target: u8| -> io::Result<()> {
        let resp = ec.command(GetKeyboardBacklight)?;
        let mut cur = if resp.enabled != 0 { resp.percent } else { 0 };
        while cur != target {
            if cur > target {
                cur -= 1;
            } else {
                cur += 1;
            }

            if args.power {
                // The power LED cannot be faded from software (although the beta BIOS apparently
                // has a switch for dimming it, so maybe it'll work with the next BIOS update).
                // So instead, we treat 0 as off and set it back to auto for any non-zero value.
                if cur == 0 {
                    ec.command(LedControl {
                        led_id: LedId::POWER,
                        flags: LedFlags::NONE,
                        brightness: LedBrightnesses::default(),
                    })?;
                } else if cur == 1 {
                    ec.command(LedControl {
                        led_id: LedId::POWER,
                        flags: LedFlags::AUTO,
                        brightness: LedBrightnesses::default(),
                    })?;
                }
            }

            ec.command(SetKeyboardBacklight { percent: cur })?;

            thread::sleep(Duration::from_millis(3));
        }
        Ok(())
    };

    let act = Arc::new(ActivityState {
        last_activity: Mutex::new(Instant::now()),
        condvar: Condvar::new(),
    });

    for (path, mut device) in evdev::enumerate() {
        // Filter devices so that only the Framework's builtin touchpad and keyboard are listened
        // to. Since we don't support hotplug, listening on USB devices wouldn't work reliably.
	//log::info!("checking device {:#?}", device.name());
        match device.name() {
            Some("PIXA3854:00 093A:0274 Touchpad" | "AT Translated Set 2 keyboard" | "keyd virtual keyboard" | "kanata" ) => {
                let act = act.clone();
                thread::spawn(move || -> io::Result<()> {
                    let name = device.name();
                    let name = name.as_deref().unwrap_or("<unknown>").to_string();
                    log::info!("starting listener on {}: {name}", path.display());
                    loop {
                        if let Err(e) = device.fetch_events() {
                            log::warn!(
                                "error while fetching events for device '{name}': {e}; closing"
                            );
                            return Err(e);
                        }
                        *act.last_activity.lock().unwrap() = Instant::now();
                        act.condvar.notify_one();

                        // Delay a bit, to avoid busy looping.
                        thread::sleep(Duration::from_millis(500));
                    }
                });
            }
            _ => {}
        }
    }

    log::info!("idle timeout: {} seconds", args.timeout);
    log::info!("brightness level: {}%", args.brightness);

    let mut state = None;
    loop {
        let guard = act.last_activity.lock().unwrap();
        let last = *guard;
        let (unneeded, result) = act
            .condvar
            .wait_timeout_while(guard, Duration::from_secs(args.timeout.into()), |instant| {
                *instant == last
            })
            .unwrap();
	drop(unneeded);
        let new_state = !result.timed_out();
        if state != Some(new_state) {
            log::info!("activity state changed: {state:?} -> {new_state}");
            if new_state {
                // Fade in
                fade_to(args.brightness)?;
            } else {
                // Fade out
                fade_to(0)?;
            }
            state = Some(new_state);
        }
    }
}

struct ActivityState {
    last_activity: Mutex<Instant>,
    condvar: Condvar,
}
