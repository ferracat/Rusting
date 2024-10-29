use signal_hook::consts::SIGINT;
use signal_hook::iterator::Signals;
use std::process;
use std::thread;
use std::time::Duration;

fn main() {
    // Set up signal handling for SIGINT (Ctrl+C)
    let mut signals = Signals::new(&[SIGINT]).expect("Failed to set up signals");

    // Spawn a thread to handle the signal so it doesn’t block the main thread
    thread::spawn(move || {
        for signal in &mut signals {
            if signal == SIGINT {
                println!("Ctrl+C detected! Exiting gracefully...");
                process::exit(1); // Exit with code 1
            }
        }
    });

    // Simulate main program work
    println!("Running... Press Ctrl+C to exit.");
    loop {
        thread::sleep(Duration::from_secs(1));
        println!("Working...");
    }
}
