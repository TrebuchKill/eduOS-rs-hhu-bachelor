// NEW

mod interfacespeed;
pub use interfacespeed::*;

mod capabilities;
pub use capabilities::*;

// AHCI Spec 3.1
#[repr(C)]
pub struct GenericHostControl
{
    /// host CAPabilities
    pub cap: Capabilities,
    /// Global Host Control
    pub ghc: u32,
    /// Interrupt Status
    pub is: u32,
    /// Ports Implemented
    /// 
    /// Bitmask: 0x04 says, the 3rd Port (Port 2) is the only available port
    /// 
    /// 0x05 says, the first and thrid Port (Port 0 & 2) are the only available ports
    pub pi: u32,
    /// VerSion
    pub vs: u32,
    /// Command Completion Coalescing ConTroL
    pub ccc_ctl: u32,
    /// Command Completion Coalescing PORTS
    pub ccc_ports: u32,
    /// Enclosure Management LOCation
    pub em_loc: u32,
    /// Enclosure Management ConTroL
    pub em_ctl: u32,
    /// host CAPabilities extended
    pub cap2: u32,
    /// Bios/Os Handoff Control & status
    pub bohc: u32
}
