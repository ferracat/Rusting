use std::fmt;

// --> File Header (14 bytes)
#[derive(Debug, Default)]
pub struct FileHeader {
    pub file_type:    u16, // 2 bytes
    pub size:         u32, // 4 bytes
    pub reserved1:    u16, // 2 bytes
    pub reserved2:    u16, // 2 bytes
    pub start_offset: u32, // 4 bytes
}

impl fmt::LowerHex for FileHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}{:x}{:x}{:x}{:x}", self.file_type, self.size, self.reserved1, self.reserved2, self.start_offset)
    }
}

impl fmt::UpperHex for FileHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}{:x}{:x}{:x}{:x}", self.file_type, self.size, self.reserved1, self.reserved2, self.start_offset)
    }
}

impl FileHeader {
    pub fn new() -> FileHeader {
        FileHeader {
            // file_type can be some of the following:
            //   "BM" [0x42, 0x4D] - Windows 3.1x, 95, NT, ... etc.
            //   "BA" [0x42, 0x41] - OS/2 struct bitmap array
            //   "CI" [0x43, 0x49] - OS/2 struct color icon
            //   "CP" [0x43, 0x50] - OS/2 const color pointer
            //   "IC" [0x49, 0x43] - OS/2 struct icon
            //   "PT" [0x50, 0x54] - OS/2 pointer
            file_type: 0x4D42,  // 0x42 is 'B and 0x4D is 'M' but because of little endian, it needs to be inverted to have "BM" 
            size: 14,           // The size will be updated with the size of the following header (BitmapInformationHeader)
            reserved1: 0,       // Reserved for application-specific use and if created manually can be 0)
            reserved2: 0,       // Reserved for application-specific use and if created manually can be 0)
            start_offset: 54,   // Typically 54 for BMP headers
        }
    }

    fn convert_u16(x: u16) -> [u8; 2] {
        let b1 = x as u8;
        let b2 = (x >> 8) as u8;
        [b1, b2]
    }

    fn convert_u32(x: u32) -> [u8; 4] {
        let b1 = x as u8;
        let b2 = (x >> 8) as u8;
        let b3 = (x >> 16) as u8;
        let b4 = (x >> 24) as u8;
        [b1, b2, b3, b4]
    }

    pub fn to_ne_bytes(&self) -> [u8; 14] {
        let mut result: [u8; 14] = [0; 14];
        result[0..2].copy_from_slice(&Self::convert_u16(self.file_type));
        result[2..6].copy_from_slice(&Self::convert_u32(self.size));
        result[6..8].copy_from_slice(&Self::convert_u16(self.reserved1));
        result[8..10].copy_from_slice(&Self::convert_u16(self.reserved2));
        result[10..14].copy_from_slice(&Self::convert_u32(self.start_offset));
        result
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }

    // pub fn as_bytes(&self) -> [u8] {
    //     let hex_str = format!("{:x}", self);
    //     hex_str.as_bytes()
    // }
}

// --> Bitmap Information Header (40 bytes)
#[derive(Debug, Default)]
pub struct BitmapInformationHeader {
    pub size:            u32, // 4 bytes
    pub widh:            u32, // 4 bytes
    pub height:          u32, // 4 bytes
    pub color_planes:    u16, // 2 bytes
    pub bits_per_pixel:  u16, // 2 bytes
    pub compression:     u32, // 4 bytes
    pub image_data_size: u32, // 4 bytes
    pub resolution:      u64, // 8 bytes
    pub colors:          u32, // 4 bytes
    pub imp_colors:      u32, // 4 bytes
}

impl fmt::LowerHex for BitmapInformationHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}", self.size, self.widh, self.height, self.color_planes, self.bits_per_pixel, self.compression, self.image_data_size, self.resolution, self.colors, self.imp_colors)
    }
}

impl BitmapInformationHeader {
    pub fn new() -> BitmapInformationHeader {
        BitmapInformationHeader {
            size: 40, // The size of this header
            widh: 100, // The width of the image in pixels
            height: 100, // The height of the image in pixels
            color_planes: 1, // The number of color planes (always 1)
            bits_per_pixel: 24, // The number of bits per pixel (e.g., 24 bits for true color images).
            compression: 0, // The compression method used (usually 0 for no compression)
            image_data_size: 24*10000, // The size of the raw image data (including padding)
            //resolution: 72, // The image's resolution (usually set to 72 DPI)
            resolution: 40,
            colors: 0, // The number of colors in the color palette (usually 0 for true color images)
            imp_colors: 0, // The number of important colors (usually 0 for true color images)
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }

    // pub fn as_bytes(&self) -> String {
    //     format!("{:x}", self)
    // }
}
