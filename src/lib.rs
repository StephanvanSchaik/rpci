use std::arch::asm;

use libc::iopl;

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

fn compose_address(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let mut address = 0x8000_0000;

    address |= (bus as u32) << 16;
    address |= ((device & 0x1f) as u32) << 11;
    address |= ((function & 0x07) as u32) << 8;
    address |= (offset as u32) & 0xfc;

    address
}

pub  fn pci_read8(
    bus: u8,
    device: u8,
    function: u8,
    offset: u8,
) -> u8 {
    unsafe { iopl(3) };
    unsafe { outl(0xcf8, compose_address(bus, device, function, offset)) };
    let value = unsafe { inb(0xcfc + (offset & 3) as u16) };
    unsafe { iopl(0) };

    value
}

pub fn pci_write8(
    bus: u8,
    device: u8,
    function: u8,
    offset: u8,
    value: u8,
) {
    unsafe { iopl(3) };
    unsafe { outl(0xcf8, compose_address(bus, device, function, offset)) };
    unsafe { outb(0xcfc + (offset & 3) as u16, value) };
    unsafe { iopl(0) };
}

pub fn pci_read16(
    bus: u8,
    device: u8,
    function: u8,
    offset: u8,
) -> u16 {
    unsafe { iopl(3) };
    unsafe { outl(0xcf8, compose_address(bus, device, function, offset)) };
    let value = unsafe { inw(0xcfc + (offset & 3) as u16) };
    unsafe { iopl(0) };

    value
}

pub fn pci_write16(
    bus: u8,
    device: u8,
    function: u8,
    offset: u8,
    value: u16,
) {
    unsafe { iopl(3) };
    unsafe { outl(0xcf8, compose_address(bus, device, function, offset)) };
    unsafe { outw(0xcfc + (offset & 3) as u16, value) };
    unsafe { iopl(0) };
}

pub fn pci_read32(
    bus: u8,
    device: u8,
    function: u8,
    offset: u8,
) -> u32 {
    unsafe { iopl(3) };
    unsafe { outl(0xcf8, compose_address(bus, device, function, offset)) };
    let value = unsafe { inl(0xcfc + (offset & 3) as u16) };
    unsafe { iopl(0) };

    value
}

pub fn pci_write32(
    bus: u8,
    device: u8,
    function: u8,
    offset: u8,
    value: u32,
) {
    unsafe { iopl(3) };
    unsafe { outl(0xcf8, compose_address(bus, device, function, offset)) };
    unsafe { outl(0xcfc + (offset & 3) as u16, value) };
    unsafe { iopl(0) };
}

pub fn pci_read64(
    bus: u8,
    device: u8,
    function: u8,
    offset: u8,
) -> u64 {
    let lo = pci_read32(bus, device, function, offset) as u64;
    let hi = pci_read32(bus, device, function, offset + 4) as u64;

    hi << 32 | lo
}

pub fn pci_write64(
    bus: u8,
    device: u8,
    function: u8,
    offset: u8,
    value: u64,
) {
    pci_write32(bus, device, function, offset, (value & 0xffffffff) as u32);
    pci_write32(bus, device, function, offset, ((value >> 32) & 0xffffffff) as u32);
}
