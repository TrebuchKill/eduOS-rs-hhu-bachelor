use crate::drivers::Register;
use super::Type;

// Actual size: 28 bytes
// Thanks to u64 padded to 32 bytes
#[repr(C)]
pub struct DmaSetup
{
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

    /// "DMA Buffer Identifier. Used to Identify DMA buffer in host memory.
    /// 
    /// SATA Spec says host specific and not in Spec. Trying AHCI spec might work."
    pub dma_buffer_id: Register<u64>,

    reserved_1: Register<u32>,

    /// Byte offset into buffer. First 2 bits must be 0.
    pub dma_buf_offset: Register<u32>,

    /// Number of bytes to transfer. Bit 0 must be 0
    pub transfer_count: Register<u32>,

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
            dma_buffer_id: Register::new(0),
            reserved_1: Register::new(0),
            dma_buf_offset: Register::new(0),
            transfer_count: Register::new(0),
            reserved_2: Register::new(0)
        }
    }
}

impl Default for DmaSetup
{
    fn default() -> Self
    {
        Self::default()
    }
}
