// NEW

use crate::drivers::Register;
use super::Type;

// May I regret not adding packed (here or anywhere else)? On x86 I doubt that, as sizes and alignment of types fit perfectly, but on other platforms?

// FLAW: this things will be copied into memory, therefore they don't need this Register<u8> thing.
#[repr(C)]
pub struct RegH2D
{
    // DWORD 0
    pub fis_type: Register<Type>,

    /// pmport 7:4, reserved 3:1, 0: (1 = Command) (0 = Control)
    /// 
    /// WARNING: This comment (and the other fis comments for this port) may be wrong.
    /// 
    /// Command/Control may be bit 7, pmport 3:0 (rest reserved)
    // but thanks to redox-os for bringing light for this possible error now I HAVE AN INTERRUPT REQUEST!
    pub pmport_cc: Register<u8>,

    /// Command Register
    pub command: Register<u8>,

    /// Feature Register 7:0
    pub featurel: Register<u8>,

    // DWORD 1
    /// LBA low Register, 7:0
    pub lba0: Register<u8>,

    /// LBA mid Register, 15:8
    pub lba1: Register<u8>,

    /// LBA high Register, 23:16
    pub lba2: Register<u8>,

    /// Device Register
    pub device: Register<u8>,

    // DWORD 2
    /// LBA Register, 31:24
    pub lba3: Register<u8>,

    /// LBA Register, 39:32
    pub lba4: Register<u8>,

    /// LBA Register, 47:40
    pub lba5: Register<u8>,

    /// Feature Register, Bits 15:8
    pub featureh: Register<u8>,

    // DWORD 3
    /// Count Register: Bits 7:0
    pub countl: Register<u8>,

    /// Count Register: Bits 15:8
    pub counth: Register<u8>,

    /// Isochronous command completion
    pub icc: Register<u8>,

    /// Control Register
    pub control: Register<u8>,

    // DWORD 4
    /// Reserved
    reserved: Register<[u8; 4]>
}

impl RegH2D
{
    pub const fn default() -> Self
    {
        Self {
            fis_type:  Register::new(Type::REG_H2D),
            pmport_cc: Register::new(0),
            command:   Register::new(0),
            featurel:  Register::new(0),
            lba0:      Register::new(0),
            lba1:      Register::new(0),
            lba2:      Register::new(0),
            device:    Register::new(0),
            lba3:      Register::new(0),
            lba4:      Register::new(0),
            lba5:      Register::new(0),
            featureh:  Register::new(0),
            countl:    Register::new(0),
            counth:    Register::new(0),
            icc:       Register::new(0),
            control:   Register::new(0),
            reserved:  Register::new([0u8; 4]),
        }
    }
}

impl Default for RegH2D
{
    fn default() -> Self
    {
        Self::default()
    }
}

impl super::Fis for RegH2D
{
    fn get_type(&self) -> Type
    {
        self.fis_type.get()
    }

    fn copy_into(&self, dst: &mut [Register<u8>; 64])
    {
        unsafe {

            let src_ptr = self as *const _ as *const u8;
            let dst_ptr = dst as *mut _ as *mut u8;
            
            // Zero out all 64 bytes
            core::ptr::write_bytes(dst_ptr, 0, 64);
            
            // Copy FIS into dst
            core::ptr::copy_nonoverlapping(src_ptr, dst_ptr, core::mem::size_of::<Self>());
        }
    }
}

// impl Into<[Register<u8>; 64]> for RegH2D
// {
//     fn into(self) -> [Register<u8>; 64]
//     {
//         /*use core::mem::MaybeUninit;
//         
//         let mut it = MaybeUninit::uninit_array();
// 
//         it[0].write(Register::new(self.fis_type.get() as u8));
//         it[1].write(self.pmport_cc.clone());
//         it[2].write(self.command.clone());
//         it[3].write(self.featurel.clone());
// 
//         it[4].write(self.lba0.clone());
//         it[5].write(self.lba1.clone());
//         it[6].write(self.lba2.clone());
//         it[7].write(self.device.clone());
// 
//         it[8].write(self.lba3.clone());
//         it[9].write(self.lba4.clone());
//         it[10].write(self.lba5.clone());
//         it[11].write(self.featureh.clone());
// 
//         it[12].write(self.countl.clone());
//         it[13].write(self.counth.clone());
//         it[14].write(self.icc.clone());
//         it[15].write(self.control.clone());
// 
//         for i in 16..64
//         {
//             it[i].write(Register::new(0));
//         }
// 
//         return MaybeUninit::array_assume_init(it);*/
//     }
// }
