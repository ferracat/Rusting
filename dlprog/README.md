dlprog
=======

This is a program to do multiple downloads from the given links as arguments.
It should have a configuration file to define the maximum amount of threads in the thread pool to do the downloads in paralel. Maybe the default destination path can also be defined in this configuration file. A nice file format to do it is TOML.
As a nice thing to have, multiple progress bars should be in the screen and to do so the ncurses library can be used.

## Dependencies
* libclang-dev
* libncursesw5
* libncursesw5-dev

```bash
sudo apt install libclang-dev libncursesw5 libncursesw5-dev
``` 

## Documentation
```bash
cargo doc --open
```