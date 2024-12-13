use clap::{arg, Command};
use hidapi::{self, HidApi};
use regex::Regex;
use std::{error::Error, ffi::CString, string::String, vec::Vec};

const REPORT_ID: u8 = 1;

const MIN_BRIGHTNESS: u32 = 400;
const MAX_BRIGHTNESS: u32 = 60000;
const BRIGHTNESS_RANGE: u32 = MAX_BRIGHTNESS - MIN_BRIGHTNESS;

const STUDIO_DISPLAY_PRODUCT_ID: u16 = 0x1114;
const STUDIO_DISPLAY_VENDOR_ID: u16 = 0x05ac;
const STUDIO_DISPLAY_INTERFACE_NR: i32 = 0x7;

fn get_brightness(handle: &mut hidapi::HidDevice) -> Result<u32, Box<dyn Error>> {
    let mut buf = Vec::with_capacity(7); // report id, 4 bytes brightness, 2 bytes unknown
    buf.push(REPORT_ID);
    buf.extend(0_u32.to_le_bytes());
    buf.extend(0_u16.to_le_bytes());
    let size = handle.get_feature_report(&mut buf)?;
    if size != buf.len() {
        Err(format!(
            "Get HID feature report: Expected a size of {}, got {}",
            buf.len(),
            size
        ))?
    }
    let brightness = u32::from_le_bytes(buf[1..5].try_into()?);
    return Ok(brightness);
}

fn get_brightness_percent(handle: &mut hidapi::HidDevice) -> Result<u8, Box<dyn Error>> {
    let value = (get_brightness(handle)? - MIN_BRIGHTNESS) as f32;
    let value_percent = (value / BRIGHTNESS_RANGE as f32 * 100.0) as u8;
    return Ok(value_percent);
}

fn set_brightness(handle: &mut hidapi::HidDevice, brightness: u32) -> Result<(), Box<dyn Error>> {
    let mut buf = Vec::with_capacity(7); // report id, 4 bytes brightness, 2 bytes unknown
    buf.push(REPORT_ID);
    buf.extend(brightness.to_le_bytes());
    buf.extend(0_u16.to_le_bytes());
    handle.send_feature_report(&mut buf)?;
    Ok(())
}

fn set_brightness_percent(
    handle: &mut hidapi::HidDevice,
    brightness: u8,
) -> Result<(), Box<dyn Error>> {
    let nits =
        (((brightness as f32 / 100.0) * BRIGHTNESS_RANGE as f32) + MIN_BRIGHTNESS as f32) as u32;
    let nits = std::cmp::min(nits, MAX_BRIGHTNESS);
    let nits = std::cmp::max(nits, MIN_BRIGHTNESS);
    set_brightness(handle, nits)?;
    Ok(())
}

fn list_displays() -> Result<Vec<String>, Box<dyn Error>> {
    let mut result = Vec::new();
    let re = Regex::new(r"asdbl-[0-9A-F]{8}-[0-9A-F]{16}$")?;
    let entries = std::fs::read_dir("/dev/")?;
    for e in entries {
        let path = e?.path();
        let path_str = path.to_str().unwrap();
        if re.is_match(path_str) {
            result.push(path_str.to_owned())
        }
    }
    return Ok(result);
}

fn list_displays_hapi(hapi: &HidApi) -> Result<Vec<String>, Box<dyn Error>> {
    let mut result = Vec::new();
    for d in hapi.device_list() {
        if d.vendor_id() != STUDIO_DISPLAY_VENDOR_ID {
            continue;
        }
        if d.product_id() != STUDIO_DISPLAY_PRODUCT_ID {
            continue;
        }
        if d.interface_number() != STUDIO_DISPLAY_INTERFACE_NR {
            continue;
        }
        result.push(d.path().to_str()?.to_string())
    }
    return Ok(result);
}

fn cli() -> Command {
    Command::new("asdbctl")
        .about("Tool to get or set the brightness for Apple Studio Displays")
        .subcommand_required(true)
        .subcommand(Command::new("get").about("Get the current brightness in %"))
        .subcommand(
            Command::new("set")
                .about("Set the current brightness in %")
                .arg(
                    arg!(<BRIGHTNESS> "The remote to target")
                        .value_parser(clap::value_parser!(u8).range(0..101)),
                )
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("up")
                .arg(
                    arg!(-s --step <STEP> "Step size in percent")
                        .required(false)
                        .default_value("10")
                        .value_parser(clap::value_parser!(u8).range(1..101)),
                )
                .about("Increase the brightness"),
        )
        .subcommand(
            Command::new("down")
                .arg(
                    arg!(-s --step <STEP> "Step size in percent")
                        .required(false)
                        .default_value("10")
                        .value_parser(clap::value_parser!(u8).range(1..101)),
                )
                .about("Decrease the brightness"),
        )
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = cli().get_matches();
    let mut displays = list_displays()?;
    let hapi = if displays.len() > 0 {
        HidApi::new_without_enumerate()?
    } else {
        HidApi::new()?
    };
    if displays.len() <= 0 {
        displays = list_displays_hapi(&hapi)?;
    }
    if displays.len() <= 0 {
        return Err("No Apple Studio Display found")?;
    }
    let display = displays.first().unwrap().as_str();
    let dev_path = CString::new(display)?;
    let mut handle = hapi.open_path(&dev_path)?;
    match matches.subcommand() {
        Some(("get", _)) => {
            let brightness = get_brightness_percent(&mut handle)?;
            println!("brightness {}", brightness);
        }
        Some(("set", sub_matches)) => {
            let brightness = *sub_matches.get_one::<u8>("BRIGHTNESS").expect("required");
            set_brightness_percent(&mut handle, brightness)?;
        }
        Some(("up", sub_matches)) => {
            let step = *sub_matches.get_one::<u8>("step").expect("required");
            let brightness = get_brightness_percent(&mut handle)?;
            let new_brightness = std::cmp::min(100, brightness + step);
            set_brightness_percent(&mut handle, new_brightness)?;
        }
        Some(("down", sub_matches)) => {
            let step = *sub_matches.get_one::<u8>("step").expect("required");
            let brightness = get_brightness_percent(&mut handle)?;
            let new_brightness = std::cmp::min(100, brightness - step);
            set_brightness_percent(&mut handle, new_brightness)?;
        }
        _ => unreachable!(),
    }
    return Ok(());
}
