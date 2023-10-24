bitmap
=======

The BMP (Bitmap) image format has a relatively simple structure, and understanding it is important if you want to create or manipulate BMP images in your Rust code. Here's a basic overview of the BMP file structure:

1. **File Header (14 Bytes):**
   - Signature: 2 bytes (BM) - Indicates that it's a BMP file.
   - File Size: 4 bytes - The total size of the BMP file in bytes.
   - Reserved: 4 bytes - Reserved for application-specific use.
   - Data Offset: 4 bytes - The offset to the start of the pixel data (usually 54 bytes).

2. **Bitmap Information Header (40 Bytes):**
   - Header Size: 4 bytes - The size of this header (40 bytes).
   - Image Width: 4 bytes - The width of the image in pixels.
   - Image Height: 4 bytes - The height of the image in pixels.
   - Color Planes: 2 bytes - The number of color planes (always 1).
   - Bits Per Pixel: 2 bytes - The number of bits per pixel (e.g., 24 bits for true color images).
   - Compression: 4 bytes - The compression method used (usually 0 for no compression).
   - Image Data Size: 4 bytes - The size of the raw image data (including padding).
   - Horizontal and Vertical Resolution: 8 bytes - The image's resolution (usually set to 72 DPI).
   - Colors in Palette: 4 bytes - The number of colors in the color palette (usually 0 for true color images).
   - Important Colors: 4 bytes - The number of important colors (usually 0 for true color images).

3. **Color Palette (if Bits Per Pixel <= 8):**
   - A table of RGB color values that define the palette for indexed color images. This section can be variable in size.

4. **Pixel Data (Variable Size):**
   - The actual pixel data where each pixel is represented as a sequence of bytes.

The *image* crate in Rust abstracts much of this detail for you. When you create or read a BMP image using the *image* crate, you primarily work with image objects and don't need to manipulate the raw bytes of the file header and pixel data manually.

