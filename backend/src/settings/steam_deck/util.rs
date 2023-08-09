//! Rough Rust port of some BatCtrl functionality
//! Original: /usr/share/jupiter_controller_fw_updater/RA_bootloader_updater/linux_host_tools/BatCtrl
//! I do not have access to the source code, so this is my own interpretation of what it does.
//!
//! But also Quanta is based in a place with some questionable copyright practices, so...
#![allow(dead_code)]

use std::fs::OpenOptions;
use std::io::{Error, Read, Seek, SeekFrom, Write};

pub const JUPITER_HWMON_NAME: &'static str = "jupiter";
pub const STEAMDECK_HWMON_NAME: &'static str = "steamdeck_hwmon";
pub const GPU_HWMON_NAME: &'static str = "amdgpu";

pub fn card_also_has(card: &dyn sysfuss::SysEntity, extensions: &'static [&'static str]) -> bool {
    extensions.iter()
        .all(|ext| card.as_ref().join(ext).exists())
}

#[inline]
fn write2(p0: u8, p1: u8) -> Result<usize, Error> {
    write_to(0x6c, 0x81)?;
    wait_ready_for_write()?;
    let count0 = write_to(0x68, p0)?;
    wait_ready_for_write()?;
    let count1 = write_to(0x68, p1)?;
    Ok(count0 + count1)
}

fn write_read(p0: u8) -> Result<u8, Error> {
    write_to(0x6c, 0x81)?;
    wait_ready_for_write()?;
    write_to(0x68, p0)?;
    wait_ready_for_read()?;
    read_from(0x68)
}

fn write_to(location: u64, value: u8) -> Result<usize, Error> {
    let mut file = OpenOptions::new().write(true).open("/dev/port")?;
    file.seek(SeekFrom::Start(location))?;
    file.write(&[value])
}

fn read_from(location: u64) -> Result<u8, Error> {
    let mut file = OpenOptions::new().read(true).open("/dev/port")?;
    file.seek(SeekFrom::Start(location))?;
    let mut buffer = [0];
    file.read(&mut buffer)?;
    Ok(buffer[0])
}

fn wait_ready_for_write() -> Result<(), Error> {
    let mut count = 0;
    while count < 0x1ffff && (read_from(0x6c)? & 2) != 0 {
        count += 1;
    }
    Ok(())
}

fn wait_ready_for_read() -> Result<(), Error> {
    let mut count = 0;
    while count < 0x1ffff && (read_from(0x6c)? & 1) == 0 {
        count += 1;
    }
    Ok(())
}

pub fn set_led(red_unused: bool, green_aka_white: bool, blue_unused: bool) -> Result<usize, Error> {
    let payload: u8 = 0x80
        | (red_unused as u8 & 1)
        | ((green_aka_white as u8 & 1) << 1)
        | ((blue_unused as u8 & 1) << 2);
    //log::info!("Payload: {:b}", payload);
    write2(Setting::LEDStatus as _, payload)
}

const THINGS: &[u8] = &[
    1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0,
    0, 0, 1, 1, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0, 1, 0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1,
    1, 0, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 0, 1, 0,
    0, 0, 1, 1, 1, 0, 1, 0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0,
];

const TIME_UNIT: std::time::Duration = std::time::Duration::from_millis(200);

pub fn flash_led() {
    let old_led_state = write_read(Setting::LEDStatus as _)
        .map_err(|e| log::error!("Failed to read LED status: {}", e));
    for &code in THINGS {
        let on = code != 0;
        if let Err(e) = set_led(on, on, false) {
            log::error!("Thing err: {}", e);
        }
        std::thread::sleep(TIME_UNIT);
    }
    if let Ok(old_led_state) = old_led_state {
        log::debug!("Restoring LED state to {:#02b}", old_led_state);
        write2(Setting::LEDStatus as _, old_led_state)
            .map_err(|e| log::error!("Failed to restore LED status: {}", e))
            .unwrap();
    }
}

pub fn set(setting: Setting, mode: u8) -> Result<usize, Error> {
    write2(setting as u8, mode)
}

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Setting {
    CycleCount = 0x32,
    ControlBoard = 0x6C,
    Charge = 0xA6,
    ChargeMode = 0x76,
    LEDStatus = 199,
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum ControlBoard {
    Enable = 0xAA,
    Disable = 0xAB,
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum ChargeMode {
    Normal = 0,
    Discharge = 0x42,
    Idle = 0x45,
}

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Charge {
    Enable = 0,
    Disable = 4,
}
