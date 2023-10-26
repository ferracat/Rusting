use std::fs::File;
use std::io::Write;
use std::mem;

// https://www.programiz.com/rust/file-handling
// https://en.wikipedia.org/wiki/BMP_file_format

mod headers;
mod rgb;

use rgb::Rgb;


const WIDTH: u16 = 100;
const HEIGHT: u16 = 100;


// ------------------------------------------------------------------------------------------------
//                                             MAIN
// ------------------------------------------------------------------------------------------------
fn main() {
    let bmp_filename = "image.bmp";

    // --- Creating the BMP data ------------------------------------------------------------------
    // --> File Header (14 bytes)
    let mut file_header = headers::FileHeader::new();
    println!("[DEBUG] file_header.size = {}", file_header.size);
    println!("[DEBUG] file_header\n{:#?}", file_header);
    println!("[DEBUG] file_header\n{:X}", file_header);

    // --> Bitmap information header (40 bytes)
    let bmp_info_header = headers::BitmapInformationHeader::new();
    file_header.size += mem::size_of::<headers::BitmapInformationHeader>() as u32;
    //println!("[DEBUG] file_header.size = {}", file_header.size);
    println!("[DEBUG] file_header\n{:#?}", file_header);
    println!("[DEBUG] bmp_info_header\n{:#?}", bmp_info_header);

    // --> Image Data for 100x100 pixels
    let white_pixel = Rgb::new(255, 255, 255); // White pixel
    let mut image_data: Vec<u8> = Vec::new();
    for _ in 0..(WIDTH * HEIGHT) {
        let pixel_bytes = white_pixel.to_ne_bytes();
        image_data.extend_from_slice(&pixel_bytes);
    }
    println!("[DEBUG] sizeof(image_data) = {}", image_data.len());
    // Convert image_data to a byte slice
    let image_data_bytes: &[u8] = &image_data;

    file_header.size += image_data.len() as u32;
    println!("[DEBUG] file_header.size = {}", file_header.size);

    // --- Creating the image file ----------------------------------------------------------------
    // Open file
    let mut bmp_file = File::create(bmp_filename).expect("creation failed");

    // Write the file header to the file (14 bytes)
    // let file_header_bytes: &[u8] = file_header.as_bytes();
    let file_header_bytes: &[u8] = &file_header.to_ne_bytes();
    println!("[DEBUG] file_header_bytes\n{:#?}", file_header_bytes);
    bmp_file.write_all(file_header_bytes).expect("Failed to write file header");

    // Write the bitmap information header to the file
    let bmp_info_header_bytes: [u8; 40] = unsafe { mem::transmute(bmp_info_header) };
    println!("[DEBUG] bmp_info_header_files\n{:#?}", bmp_info_header_bytes);
    bmp_file.write_all(&bmp_info_header_bytes).expect("Failed to write bitmap info header");

    // Write the image data to the file
    bmp_file.write_all(&image_data_bytes).expect("Failed to write image data");

    println!("Created a file {}", bmp_filename);
}

