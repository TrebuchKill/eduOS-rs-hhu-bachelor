// NEW

#[repr(u8)]
pub enum Color
{
    Black = 0,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White = 7
}

impl Color
{
    pub const fn as_foreground_u16(self) -> u16
    {
        u16::from_le_bytes([0, self as u8])
    }

    pub const fn as_background_u16(self) -> u16
    {
        u16::from_le_bytes([0, (self as u8) << 4])
    }

    pub const fn as_foreground_u8(self) -> u8
    {
        self as u8
    }

    pub const fn as_background_u8(self) -> u8
    {
        (self as u8) << 4
    }

    pub const fn from_color_3bit(value: u8) -> Self
    {
        use Color::*;
        match value
        {
            1 => Red,
            2 => Green,
            3 => Yellow,
            4 => Blue,
            5 => Magenta,
            6 => Cyan,
            7 => White,
            _ => Black, // 0 & default
        }
    }
}

// 7 .. 0 7 .. 0
// Blink(1) Background(3) Brigh(1) Forground(3) Value(8)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Cell(u16);

impl Cell
{
    const MASK_BLINK:     u16 = 0x80_00;
    const MASK_BG_COLOR:  u16 = 0x70_00;
    const MASK_FG_BRIGHT: u16 = 0x08_00;
    const MASK_FG_COLOR:  u16 = 0x07_00;
    #[allow(dead_code)]
    const MASK_CHAR:      u16 = 0x00_ff;

    const EMPTY: Self = Self::new(false, Color::Black, false, Color::White, b' ');

    pub const fn new(blink: bool, bg: Color, bright_fg: bool, fg: Color, value: u8) -> Self
    {
        let mut out = Self(0);
        out.set_blink(blink)
            .set_bg_color(bg)
            .set_bright(bright_fg)
            .set_fg_color(fg)
            .set_value(value);
        out
    }

    // Seems to behave like background brightness
    pub const fn is_blink(&self) -> bool
    {
        self.0 & Self::MASK_BLINK == Self::MASK_BLINK
    }

    pub const fn set_blink(&mut self, value: bool) -> &mut Self
    {
        self.0 = if value
        {
            self.0 | Self::MASK_BLINK
        }
        else
        {
            self.0 & !Self::MASK_BLINK
        };
        self
    }

    pub const fn is_bright(&self) -> bool
    {
        self.0 & Self::MASK_FG_BRIGHT == Self::MASK_FG_BRIGHT
    }

    pub const fn set_bright(&mut self, value: bool) -> &mut Self
    {
        self.0 =
            if value
            {
                self.0 | Self::MASK_FG_BRIGHT
            }
            else
            {
                self.0 & !Self::MASK_FG_BRIGHT
            };
        self
    }

    pub const fn get_bg_color(&self) -> Color
    {
        Color::from_color_3bit((self.0 & Self::MASK_BG_COLOR).to_ne_bytes()[0])
    }

    pub const fn set_bg_color(&mut self, color: Color) -> &mut Self
    {
        self.0 = self.0 & !Self::MASK_BG_COLOR | color.as_background_u16();
        self
    }

    pub const fn get_fg_color(&self) -> Color
    {
        Color::from_color_3bit((self.0 & Self::MASK_FG_COLOR).to_ne_bytes()[0] >> 4)
    }

    pub const fn set_fg_color(&mut self, color: Color) -> &mut Self
    {
        self.0 = self.0 & !Self::MASK_FG_COLOR | color.as_foreground_u16();
        self
    }

    pub const fn get_value(&self) -> u8
    {
        let tmp = self.0.to_le_bytes();
        tmp[0]
    }

    pub const fn set_value(&mut self, value: u8) -> &mut Self
    {
        let tmp = self.0.to_le_bytes();
        self.0 = u16::from_le_bytes([value, tmp[1]]);
        self
    }
}

pub struct Buffer
{
    data: [Cell; 80 * 25]
}

impl Buffer
{
    pub const WIDHT: usize = 80;
    pub const HEIGHT: usize = 25;
    pub const AREA: usize = Self::WIDHT * Self::HEIGHT;

    pub const WIDHT_RANGE: core::ops::Range<usize> = 0..Self::WIDHT;
    pub const HEIGHT_RANGE: core::ops::Range<usize> = 0..Self::HEIGHT;

    pub const fn new() -> Self
    {
        Self{ data: [Cell::EMPTY; Self::AREA] }
    }

    fn init(&mut self)
    {
        // self.data = [Cell::EMPTY; Self::AREA];
        Self::init_ptr(self);
    }

    fn init_ptr(this: *mut Self)
    {
        unsafe { this.write_volatile(Self::new()) }
    }

    fn write_cell(&mut self, x: usize, y: usize, value: Cell)
    {
        assert!(Self::WIDHT_RANGE.contains(&x));
        assert!(Self::HEIGHT_RANGE.contains(&y));
        let index = x + (y * 80);
        let ptr: *mut Cell = &mut self.data[index];
        unsafe { ptr.write_volatile(value) }
    }

    fn read_cell(&mut self, x: usize, y: usize) -> Cell
    {
        assert!(Self::WIDHT_RANGE.contains(&x));
        assert!(Self::HEIGHT_RANGE.contains(&y));
        let index = x + (y * 80);
        let ptr: *mut Cell = &mut self.data[index];
        unsafe { ptr.read_volatile() }
    }
}

/*impl core::ops::Index<(usize, usize)> for Buffer
{
    type Output = Cell;

    fn index(&self, index: (usize, usize)) -> &Self::Output
    {
        let (x, y) = index;
        assert!(Self::WIDHT_RANGE.contains(&x));
        assert!(Self::HEIGHT_RANGE.contains(&y));
        &self.data[x + (y * 80)]
    }
}

impl core::ops::IndexMut<(usize, usize)> for Buffer
{
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output
    {
        let (x, y) = index;
        assert!(Self::WIDHT_RANGE.contains(&x));
        assert!(Self::HEIGHT_RANGE.contains(&y));
        &mut self.data[x + (y * 80)]
    }
}*/

pub struct VgaTextOutput
{
    x: i16,
    y: i16,
}

impl VgaTextOutput
{
    const VGA_BUFFER: usize = 0x00_0B_80_00usize;

    fn new() -> Self
    {
        let buf = unsafe { &mut *(Self::VGA_BUFFER as *mut Buffer) };
        buf.init();
        Self { x: 0, y: 0 }
    }

    pub fn pre_inc(&mut self) -> (usize, usize)
    {
        let x = self.x;
        let y = self.y;
        self.x += 1;
        if self.x >= (Buffer::WIDHT as i16)
        {
            self.new_line();
        }
        (x as usize, y as usize)
    }

    pub fn new_line(&mut self)
    {
        self.x = 0;
        self.y += 1;
        if self.y >= (Buffer::HEIGHT as i16)
        {
            self.y = (Buffer::HEIGHT as i16) - 1;
            let buffer = unsafe { &mut *(Self::VGA_BUFFER as *mut Buffer) };
            for y in Buffer::HEIGHT_RANGE.skip(1)
            {
                for x in Buffer::WIDHT_RANGE
                {
                    let cell = buffer.read_cell(x, y);
                    buffer.write_cell(x, y - 1, cell);
                }
            }
            for x in Buffer::WIDHT_RANGE
            {
                buffer.write_cell(x, 24, Cell::EMPTY);
            }
        }
    }
}

impl core::fmt::Write for VgaTextOutput
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result
    {
        let buffer = unsafe { &mut *(Self::VGA_BUFFER as *mut Buffer) };
        for byte in s.as_bytes()
        {
            match byte
            {
                b'\n' => self.new_line(),
                it => {
                    let (x, y) = self.pre_inc();
                    let mut cell = buffer.read_cell(x, y);
                    cell.set_value(*it); // .set_fg_color(Color::White).set_bg_color(Color::Black);
                    buffer.write_cell(x, y, cell);
                }
            }
        }
        Ok(())
    }
}

use crate::synch::spinlock::{
    SpinlockIrqSave,
    SpinlockIrqSaveGuard
};

pub fn get_buffer() -> SpinlockIrqSaveGuard<'static, Option<VgaTextOutput>>
{
    static BUFFER: SpinlockIrqSave<Option<VgaTextOutput>> = SpinlockIrqSave::new(None);
    let mut buf = BUFFER.lock();
    if let None = *buf
    {
        *buf = Some(VgaTextOutput::new());
    }
    buf
}
