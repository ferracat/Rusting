fn convert_u16(x: u16) -> [u8; 2] {
    let b1 = x as u8;
    let b2 = (x >> 8) as u8;
    [b1, b2]
}

// ------------------------------------------------------------------------------------------------
//                                             MAIN
// ------------------------------------------------------------------------------------------------
fn main() {
    let num: u16 = 1000;

    let converted: [u8; 2] = convert_u16(num);

    println!("[DEBUG] converted = {:#?}\n", converted);

}

