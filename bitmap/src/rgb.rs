use std::fmt;

#[derive(Clone)]
pub struct Rgb(u8, u8, u8);

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Rgb(r, g, b)
    }

    pub fn to_ne_bytes(&self) -> [u8; 3] {
        [self.0, self.1, self.2]
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }
}

impl fmt::LowerHex for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }
}
