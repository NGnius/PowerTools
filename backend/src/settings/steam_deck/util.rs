//! Rough Rust port of some BatCtrl functionality
//! I do not have access to the source code, so this is my own interpretation of what it does.
//!
//! But also Quanta is based in a place with some questionable copyright practices, so...
#![allow(dead_code)]

use std::fs::OpenOptions;
use std::io::{Error, Seek, SeekFrom, Read, Write};

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
    let mut file = OpenOptions::new()
        .write(true)
        .open("/dev/port")?;
    file.seek(SeekFrom::Start(location))?;
    file.write(&[value])
}

fn read_from(location: u64) -> Result<u8, Error> {
    let mut file = OpenOptions::new()
        .read(true)
        .open("/dev/port")?;
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
    let payload: u8 = 0x80 + (red_unused as u8 & 1) + ((green_aka_white as u8 & 1) << 1) + ((blue_unused as u8 & 1) << 2);
    //log::info!("Payload: {:b}", payload);
    write2(Setting::LEDStatus as _, payload)
}

pub fn set(setting: Setting, mode: u8) -> Result<usize, Error> {
    write2(setting as u8, mode)
}

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Setting {
    Charge = 0xA6,
    ChargeMode = 0x76,
    LEDStatus = 199,
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
