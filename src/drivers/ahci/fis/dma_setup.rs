use crate::drivers::Register;
use super::Type;

#[repr(C)]
pub struct DmaSetup
{
    // DWORD 0
    pub fis_type: Register<Type>,
    /// - Port Multiplier 7:4
    /// - Reserved 3
    /// - Data Transfer Direction 2
    ///   - Value 1 = Device to host
    /// - Interrupt Bit 1
    /// - Auto-activate
    ///   - Specifies if DMA Activates FIS is needed
    pub pmport_dia: Register<u8>,
    
    /// Reserved
    reserved_0: Register<[u8; 2]>,

    // DWORD 1
    /// OSDev Wiki Quote: "DMA Buffer Identifier. Used to Identify DMA buffer in host memory.
    /// 
    /// SATA Spec says host specific and not in Spec. Trying AHCI spec might work."
    pub dma_buffer_id_low: Register<u32>,

    // DWORD 2
    pub dma_buffer_id_high: Register<u32>,
    // Why low & high, when osdev wiki uses a simple 64 bit integer?
    // Alignment: I do not use packed for this struct (or any struct), as I do not need to violate alignment rules and the compiler will enforce them.
    // Therefore I am guaranteed to call functions, which require aligned pointers.
    // A u64 has alignment of 8 (bytes/u8), u32 a alignment of 4, preceeding dma_buffer_id_low is 4 bytes, using u64 will result in a padding of 4 bytes.
    // On the other hand, outside of x86_64 this struct may be useless

    // DWORD 3
    reserved_1: Register<u32>,

    // DWORD 4
    /// Byte offset into buffer. First 2 bits must be 0.
    pub dma_buf_offset: Register<u32>,

    // DWORD 5
    /// Number of bytes to transfer. Bit 0 must be 0
    pub transfer_count: Register<u32>,

    // DWORD 6
    reserved_2: Register<u32>
}

impl DmaSetup
{
    pub const fn default() -> Self
    {
        Self {

            fis_type: Register::new(Type::DMA_SETUP),
            pmport_dia: Register::new(0),
            reserved_0: Register::new([0u8; 2]),
            dma_buffer_id_low: Register::new(0),
            dma_buffer_id_high: Register::new(0),
            reserved_1: Register::new(0),
            dma_buf_offset: Register::new(0),
            transfer_count: Register::new(0),
            reserved_2: Register::new(0)
        }
    }

    /// OSDev Wiki Quote: "DMA Buffer Identifier. Used to Identify DMA buffer in host memory.
    /// 
    /// SATA Spec says host specific and not in Spec. Trying AHCI spec might work."
    pub fn get_dma_buffer(&self) -> u64
    {
        let low = self.dma_buffer_id_low.get();
        let hi = self.dma_buffer_id_high.get();
        ((hi as u64) << 32) | (low as u64)
    }

    /// OSDev Wiki Quote: "DMA Buffer Identifier. Used to Identify DMA buffer in host memory.
    /// 
    /// SATA Spec says host specific and not in Spec. Trying AHCI spec might work."
    pub fn set_dma_buffer(&mut self, value: u64)
    {
        let value_low = value as u32;
        let value_hi = ((value & 0xff_ff_ff_ff_00_00_00_00u64) >> 32) as u32;
        self.dma_buffer_id_low.set(value_low);
        self.dma_buffer_id_high.set(value_hi);
    }
}

impl Default for DmaSetup
{
    fn default() -> Self
    {
        Self::default()
    }
}
