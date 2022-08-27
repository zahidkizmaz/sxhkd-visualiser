# sxhkd-visualiser

Show SXHKD keybindings in a small GTK application.

![image](https://user-images.githubusercontent.com/15658403/187045758-02c3ff01-a39f-48b4-976c-53be793df0b8.png)


## How to install

### Build from source

#### Dependencies
This application uses GTK 3 thus, it should be installed prior to build: https://www.gtk.org/docs/installations/


```shell
curl https://sh.rustup.rs -sSf | sh -s  # If you don't have Rust installed
git clone https://github.com/zahidkizmaz/sxhkd-visualiser.git
cd sxhkd-visualiser
cargo build --release
cargo run --release
```
