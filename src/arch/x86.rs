use crate::error::Error;
use core::arch::asm;

unsafe fn inb(port: u16) -> u8 {
    let mut value;

    asm!(
        "in al, dx",
        in("dx") port,
        out("al") value,
    );

    value
}

unsafe fn outb(port: u16, value: u8) {
    asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
    );
}

unsafe fn inw(port: u16) -> u16 {
    let mut value;

    asm!(
        "in ax, dx",
        in("dx") port,
        out("ax") value,
    );

    value
}

unsafe fn outw(port: u16, value: u16) {
    asm!(
        "out dx, ax",
        in("dx") port,
        in("ax") value,
    );
}

unsafe fn inl(port: u16) -> u32 {
    let mut value;

    asm!(
        "in eax, dx",
        in("dx") port,
        out("eax") value,
    );

    value
}

unsafe fn outl(port: u16, value: u32) {
    asm!(
        "out dx, eax",
        in("dx") port,
        in("eax") value,
    );
}

#[cfg(target_os = "linux")]
unsafe fn iopl(level: libc::c_int) -> Result<(), Error> {
    let result = unsafe { libc::iopl(level) };

    if result != 0 {
        #[cfg(feature = "std")]
        return Err(nix::errno::from_i32(result))?;
        #[cfg(not(feature = "std"))]
        return Err(Error);
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
unsafe fn iopl(level: libc::c_int) -> Result<(), Error> {
    Ok(())
}

fn compose_address(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let mut address = 0x8000_0000;

    address |= (bus as u32) << 16;
    address |= ((device & 0x1f) as u32) << 11;
    address |= ((function & 0x07) as u32) << 8;
    address |= (offset as u32) & 0xfc;

    address
}

pub fn pci_read8(bus: u8, device: u8, function: u8, offset: u8) -> Result<u8, Error> {
    unsafe { iopl(3)? };
    unsafe { outl(0xcf8, compose_address(bus, device, function, offset)) };
    let value = unsafe { inb(0xcfc + (offset & 3) as u16) };
    unsafe { iopl(0)? };

    Ok(value)
}

pub fn pci_write8(bus: u8, device: u8, function: u8, offset: u8, value: u8) -> Result<(), Error> {
    unsafe { iopl(3)? };
    unsafe { outl(0xcf8, compose_address(bus, device, function, offset)) };
    unsafe { outb(0xcfc + (offset & 3) as u16, value) };
    unsafe { iopl(0)? };

    Ok(())
}

pub fn pci_read16(bus: u8, device: u8, function: u8, offset: u8) -> Result<u16, Error> {
    unsafe { iopl(3)? };
    unsafe { outl(0xcf8, compose_address(bus, device, function, offset)) };
    let value = unsafe { inw(0xcfc + (offset & 3) as u16) };
    unsafe { iopl(0)? };

    Ok(value)
}

pub fn pci_write16(bus: u8, device: u8, function: u8, offset: u8, value: u16) -> Result<(), Error> {
    unsafe { iopl(3)? };
    unsafe { outl(0xcf8, compose_address(bus, device, function, offset)) };
    unsafe { outw(0xcfc + (offset & 3) as u16, value) };
    unsafe { iopl(0)? };

    Ok(())
}

pub fn pci_read32(bus: u8, device: u8, function: u8, offset: u8) -> Result<u32, Error> {
    unsafe { iopl(3)? };
    unsafe { outl(0xcf8, compose_address(bus, device, function, offset)) };
    let value = unsafe { inl(0xcfc + (offset & 3) as u16) };
    unsafe { iopl(0)? };

    Ok(value)
}

pub fn pci_write32(bus: u8, device: u8, function: u8, offset: u8, value: u32) -> Result<(), Error> {
    unsafe { iopl(3)? };
    unsafe { outl(0xcf8, compose_address(bus, device, function, offset)) };
    unsafe { outl(0xcfc + (offset & 3) as u16, value) };
    unsafe { iopl(0)? };

    Ok(())
}

pub fn pci_read64(bus: u8, device: u8, function: u8, offset: u8) -> Result<u64, Error> {
    let lo = pci_read32(bus, device, function, offset)? as u64;
    let hi = pci_read32(bus, device, function, offset + 4)? as u64;

    Ok(hi << 32 | lo)
}

pub fn pci_write64(bus: u8, device: u8, function: u8, offset: u8, value: u64) -> Result<(), Error> {
    pci_write32(bus, device, function, offset, (value & 0xffffffff) as u32)?;
    pci_write32(
        bus,
        device,
        function,
        offset,
        ((value >> 32) & 0xffffffff) as u32,
    )?;

    Ok(())
}

#[derive(Clone, Debug)]
pub struct DeviceIter {
    bus: u8,
    device: u8,
    function: u8,
    exhausted: bool,
}

impl Iterator for DeviceIter {
    type Item = Device;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }

        loop {
            let device = Self::Item {
                bus: self.bus,
                device: self.device,
                function: self.function,
            };

            self.function = (self.function + 1) % 8;

            if self.function == 0 {
                self.device = (self.device + 1) % 32;

                if self.device == 0 {
                    self.bus = self.bus.wrapping_add(1);

                    if self.bus == 0 {
                        self.exhausted = true;
                        return None;
                    }
                }
            }

            if device.vendor_id().ok()? == 0xffff {
                continue;
            }

            if device.device_id().ok()? == 0xffff {
                continue;
            }

            return Some(device);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Device {
    bus: u8,
    device: u8,
    function: u8,
}

impl Device {
    pub fn open(bus: u8, device: u8, function: u8) -> Result<Self, Error> {
        Ok(Self {
            bus,
            device,
            function,
        })
    }

    pub fn enumerate() -> Result<DeviceIter, Error> {
        Ok(DeviceIter {
            bus: 0,
            device: 0,
            function: 0,
            exhausted: false,
        })
    }

    pub fn bus(&self) -> u8 {
        self.bus
    }

    pub fn device(&self) -> u8 {
        self.device
    }

    pub fn function(&self) -> u8 {
        self.function
    }

    pub fn vendor_id(&self) -> Result<u16, Error> {
        self.read16(0x00)
    }

    pub fn device_id(&self) -> Result<u16, Error> {
        self.read16(0x02)
    }

    pub fn read8(&self, offset: u8) -> Result<u8, Error> {
        pci_read8(self.bus, self.device, self.function, offset)
    }

    pub fn write8(&mut self, offset: u8, value: u8) -> Result<(), Error> {
        pci_write8(self.bus, self.device, self.function, offset, value)
    }

    pub fn read16(&self, offset: u8) -> Result<u16, Error> {
        pci_read16(self.bus, self.device, self.function, offset)
    }

    pub fn write16(&mut self, offset: u8, value: u16) -> Result<(), Error> {
        pci_write16(self.bus, self.device, self.function, offset, value)
    }

    pub fn read32(&self, offset: u8) -> Result<u32, Error> {
        pci_read32(self.bus, self.device, self.function, offset)
    }

    pub fn write32(&mut self, offset: u8, value: u32) -> Result<(), Error> {
        pci_write32(self.bus, self.device, self.function, offset, value)
    }

    pub fn read64(&self, offset: u8) -> Result<u64, Error> {
        pci_read64(self.bus, self.device, self.function, offset)
    }

    pub fn write64(&mut self, offset: u8, value: u64) -> Result<(), Error> {
        pci_write64(self.bus, self.device, self.function, offset, value)
    }
}
