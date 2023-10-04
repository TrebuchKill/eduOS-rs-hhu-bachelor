// NEW, 4.2.2

use crate::drivers::Register;

// the chapter and the graphic title call it "Command List Structure"
// Inside the graphic it is called the "command header", and the struct in C code is called "HBA_CMD_HEADER"

#[repr(C)]
pub struct CommandListStructure
{
    pub data: [Register<u32>; 8],
}

impl CommandListStructure
{
    pub fn get_prdtl(&self) -> u16
    {
        (self.data[0].get() & 0xff_ff_00_00 >> 16) as u16
    }

    pub fn set_prdtl(&mut self, value: u16)
    {
        let register = &mut self.data[0];
        register.set((register.get() & 0x00_00_ff_ff) | ((value as u32) << 16));
    }

    pub fn get_pmp(&self) -> u8
    {
        (self.data[0].get() & 0x00_00_f0_00 >> 12) as u8
    }

    pub fn set_pmp(&mut self, value: u8)
    {
        // TODO: check value in range
        let register = &mut self.data[0];
        register.set(
            (register.get() & 0xff_ff_0f_ff) | ((value as u32) << 12)
        );
    }

    /// Clear Busy upon R_OK
    pub fn get_clear_busy(&self) -> bool
    {
        const MASK: u32 = 1u32 << 10;
        self.data[0].get() & MASK != 0
    }

    /// Clear Busy upon R_OK
    pub fn set_clear_busy(&mut self, value: bool)
    {
        const MASK: u32 = 1u32 << 10;
        let register = &mut self.data[0];
        register.set(if value { register.get() | MASK } else { register.get() & !MASK });
    }

    pub fn get_bist(&self) -> bool
    {
        const MASK: u32 = 1u32 << 9;
        self.data[0].get() & MASK != 0
    }

    pub fn set_bist(&mut self, value: bool)
    {
        const MASK: u32 = 1u32 << 9;
        let register = &mut self.data[0];
        register.set(if value { register.get() | MASK } else { register.get() & !MASK });
    }

    pub fn get_reset(&self) -> bool
    {
        const MASK: u32 = 1u32 << 8;
        self.data[0].get() & MASK != 0
    }

    pub fn set_reset(&mut self, value: bool)
    {
        const MASK: u32 = 1u32 << 8;
        let register = &mut self.data[0];
        register.set(if value { register.get() | MASK } else { register.get() & !MASK });
    }

    pub fn get_prefetchable(&self) -> bool
    {
        const MASK: u32 = 1u32 << 7;
        self.data[0].get() & MASK != 0
    }

    pub fn set_prefetchable(&mut self, value: bool)
    {
        const MASK: u32 = 1u32 << 7;
        let register = &mut self.data[0];
        register.set(if value { register.get() | MASK } else { register.get() & !MASK });
    }

    pub fn get_write(&self) -> bool
    {
        const MASK: u32 = 1u32 << 6;
        self.data[0].get() & MASK != 0
    }

    pub fn set_write(&mut self, value: bool)
    {
        const MASK: u32 = 1u32 << 6;
        let register = &mut self.data[0];
        register.set(if value { register.get() | MASK } else { register.get() & !MASK });
    }

    /// ATAPI
    pub fn get_atapi(&self) -> bool
    {
        const MASK: u32 = 1u32 << 5;
        self.data[0].get() & MASK != 0
    }

    /// ATAPI
    pub fn set_atapi(&mut self, value: bool)
    {
        const MASK: u32 = 1u32 << 5;
        let register = &mut self.data[0];
        register.set(if value { register.get() | MASK } else { register.get() & !MASK });
    }

    pub fn get_cfl(&self) -> u8
    {
        (self.data[0].get() & 0x00_00_00_1f) as u8
    }

    pub fn set_cfl(&mut self, value: u8)
    {
        assert_eq!(value & 0x1f, value);
        let register = &mut self.data[0];
        register.set((register.get() & 0xff_ff_ff_e0) | (value as u32))
    }

    /// Physical Region Descriptor Byte Count
    pub fn get_prdbc(&self) -> u32
    {
        self.data[1].get()
    }

    /// Physical Region Descriptor Byte Count
    pub fn set_prdbc(&mut self, value: u32)
    {
        self.data[1].set(value);
    }

    pub fn get_ctba(&self) -> u32
    {
        self.data[2].get()
    }

    pub fn set_ctba(&mut self, value: u32)
    {
        assert_eq!(value & 0xff_ff_ff_80, value, "value must be 128-byte cache line aligned.");
        self.data[2].set(value);
    }

    /// Unsafe Note: Only valid for a HBA which has S64A set.
    pub unsafe fn get_ctbau(&self) -> u32
    {
        self.data[3].get()
    }

    /// Unsafe Note: Only valid for a HBA which has S64A set.
    pub unsafe fn set_ctbau(&mut self, value: u32)
    {
        self.data[3].set(value);
    }
}


