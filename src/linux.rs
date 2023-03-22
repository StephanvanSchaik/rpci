use crate::error::Error;
use nix::sys::uio::{pread, pwrite};
use nom::{
    bytes::complete::{tag, take_while_m_n},
    combinator::map_res,
    IResult,
};
use std::fs::{OpenOptions, ReadDir};
use std::os::unix::io::AsRawFd;
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

    pub fn read_u8(&self, offset: u8) -> Result<u8, Error> {
        let file = OpenOptions::new().read(true).open(self.path.join("config"))?;
        let mut bytes = [0u8; 1];
        pread(file.as_raw_fd(), &mut bytes, offset as _)?;

        Ok(bytes[0])
    }

    pub fn write_u8(&self, offset: u8, value: u8) -> Result<(), Error> {
        let file = OpenOptions::new().write(true).open(self.path.join("config"))?;
        let mut bytes = [value];
        pwrite(file.as_raw_fd(), &mut bytes, offset as _)?;

        Ok(())
    }

    pub fn read_u16(&self, offset: u8) -> Result<u16, Error> {
        let file = OpenOptions::new().read(true).open(self.path.join("config"))?;
        let mut bytes = [0u8; 2];
        pread(file.as_raw_fd(), &mut bytes, offset as _)?;

        Ok(u16::from_ne_bytes(bytes))
    }

    pub fn write_u16(&self, offset: u8, value: u16) -> Result<(), Error> {
        let file = OpenOptions::new().write(true).open(self.path.join("config"))?;
        let mut bytes = u16::to_ne_bytes(value);
        pwrite(file.as_raw_fd(), &mut bytes, offset as _)?;

        Ok(())
    }

    pub fn read_u32(&self, offset: u8) -> Result<u32, Error> {
        let file = OpenOptions::new().read(true).open(self.path.join("config"))?;
        let mut bytes = [0u8; 4];
        pread(file.as_raw_fd(), &mut bytes, offset as _)?;

        Ok(u32::from_ne_bytes(bytes))
    }

    pub fn write_u32(&self, offset: u8, value: u32) -> Result<(), Error> {
        let file = OpenOptions::new().write(true).open(self.path.join("config"))?;
        let mut bytes = u32::to_ne_bytes(value);
        pwrite(file.as_raw_fd(), &mut bytes, offset as _)?;

        Ok(())
    }

    pub fn read_u64(&self, offset: u8) -> Result<u64, Error> {
        let file = OpenOptions::new().read(true).open(self.path.join("config"))?;
        let mut bytes = [0u8; 8];
        pread(file.as_raw_fd(), &mut bytes, offset as _)?;

        Ok(u64::from_ne_bytes(bytes))
    }

    pub fn write_u64(&self, offset: u8, value: u64) -> Result<(), Error> {
        let file = OpenOptions::new().write(true).open(self.path.join("config"))?;
        let mut bytes = u64::to_ne_bytes(value);
        pwrite(file.as_raw_fd(), &mut bytes, offset as _)?;

        Ok(())
    }
}
