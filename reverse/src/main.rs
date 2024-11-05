use std::io;

#[cfg(not(feature = "grapheme"))]
pub fn reverse(input: &str) -> String {
    input.chars().rev().collect::<String>()
}

#[cfg(feature = "grapheme")]
use unicode_segmentation::UnicodeSegmentation;
#[cfg(feature = "grapheme")]
pub fn reverse(input: &str) -> String {
    input.graphemes(true).rev().collect::<String>()
}

fn main() {
    println!("Type something so that it can be shown in the reverse order");

    // Create a new instance of `std::io::stdin`
    let mut input = String::new();

    // Read a line of text from the user
    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            // Input was successfully read
            println!("You entered: {}", input);
            println!("The reverse: {}", reverse(&input));
        }
        Err(error) => {
            // An error occurred while reading input
            println!("Error: {}", error);
        }
    }
}
