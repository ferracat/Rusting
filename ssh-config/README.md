ssh-config
===========

This is meant to be an TUI to manage the *~/.ssh/config*.


### Similar projects
* [sshed](https://github.com/trntv/sshed)


* **build**
```bash
cargo check
cargo build
cargo build --debug
cargo build --production
```

---------------------------------------------------------------------------------------------------

### Notes

> There is a sleep of 10ms on each loop to ease the cpu. This implementation should be changed to event polling with timeout.

Example:

```rust
buse std::time::Duration;
use crossterm::event::{self, Event};

loop {
    // Poll for events with a timeout
    if event::poll(Duration::from_millis(50)).unwrap() {
        // If an event is available within the timeout, read it
        if let Ok(event) = event::read() {
            // Handle the event
        }
    }

    // Perform other work, like updating the terminal UI, outside the event handling if needed
}
```
