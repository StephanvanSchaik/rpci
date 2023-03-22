use crate::error::Error;
use nom::{
    bytes::complete::{tag, take_while_m_n},
    combinator::map_res,
    IResult,
};
use std::fs::ReadDir;
use std::path::PathBuf;

fn is_hex_digit(c: char) -> bool {
    c.is_digit(16)
}

fn parse_path(input: &str) -> IResult<&str, (u8, u8, u8)> {
    let (input, _) = tag("/sys/bus/pci/devices/0000:")(input)?;

    let (input, bus) = map_res(take_while_m_n(2, 2, is_hex_digit), |input| {
        u8::from_str_radix(input, 16)
    })(input)?;

    let (input, _) = tag(":")(input)?;

    let (input, device) = map_res(take_while_m_n(2, 2, is_hex_digit), |input| {
        u8::from_str_radix(input, 16)
    })(input)?;

    let (input, _) = tag(".")(input)?;

    let (input, function) = map_res(take_while_m_n(1, 1, is_hex_digit), |input| {
        u8::from_str_radix(input, 16)
    })(input)?;

    Ok((input, (bus, device, function)))
}

#[derive(Debug)]
pub struct DeviceIter {
    dir: ReadDir,
}

impl Iterator for DeviceIter {
    type Item = Device;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = match self.dir.next() {
            Some(entry) => entry.ok()?,
            _ => return None,
        };

        Some(Self::Item { path: entry.path() })
    }
}

#[derive(Clone, Debug)]
pub struct Device {
    path: PathBuf,
}

impl Device {
    pub fn open(bus: u8, device: u8, function: u8) -> Result<Self, Error> {
        let path = format!("/sys/bus/pci/devices/0000:{bus:02x}:{device:02x}.{function}");
        let path = PathBuf::from(path);

        Ok(Self { path })
    }

    pub fn enumerate() -> Result<DeviceIter, Error> {
        let dir = std::fs::read_dir("/sys/bus/pci/devices")?;

        Ok(DeviceIter { dir })
    }

    pub fn bus(&self) -> u8 {
        parse_path(&self.path.to_string_lossy())
            .map(|(_, (bus, _, _))| bus)
            .unwrap_or(0)
    }

    pub fn device(&self) -> u8 {
        parse_path(&self.path.to_string_lossy())
            .map(|(_, (_, device, _))| device)
            .unwrap_or(0)
    }

    pub fn function(&self) -> u8 {
        parse_path(&self.path.to_string_lossy())
            .map(|(_, (_, _, function))| function)
            .unwrap_or(0)
    }

    pub fn vendor_id(&self) -> Result<u16, Error> {
        let value = std::fs::read_to_string(self.path.join("vendor"))?;
        let without_prefix = value.trim().trim_start_matches("0x");
        let value = u16::from_str_radix(without_prefix, 16)?;

        Ok(value)
    }

    pub fn device_id(&self) -> Result<u16, Error> {
        let value = std::fs::read_to_string(self.path.join("device"))?;
        let without_prefix = value.trim().trim_start_matches("0x");
        let value = u16::from_str_radix(without_prefix, 16)?;

        Ok(value)
    }
}
