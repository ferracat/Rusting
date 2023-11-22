Rusting
============
Rust language learning projects

There is a command line Rust book that has some interesting code examples: 
> https://github.com/kyclark/command-line-rust

## Installation
The easyest way to install the rust build environment is by running the following command:
```bash
curl --proto '=https' --tlsv1.3 https://sh.rustup.rs -sSf | sh
```

The previous command should install the rust-analyzer in the *\~/.cargo/bin* folder but in order for it to be detectable by the editors, it must be in the path. To do so, the following lines can be added to the *\~/.profile*:
```bash
# set PATH so it includes the Rust cargo binaries if they exist
if [ -d "$HOME/.cargo/bin" ] && ! grep -Eq "(^|:)$HOME/.cargo/bin($|:)" <<<$PATH ; then
    PATH="$HOME/.cargo/bin:$PATH"
fi
```

#### Install debugger
```bash
sudo apt install lldb
cd $(dirname $(which lldb))
sudo ln -s lldb-vscode-* lldb-vscode
```

> Check health of helix editor --> `hx --health rust`

---

The links will be added automatically but in any case the following can be done.

After doing so it is advisable to have the rust-analyser.
```bash
rustup component add rust-analyzer
```

To keep it updated
```bash
rustup update
```
