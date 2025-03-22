cargo fmt
rustup default stable
rustup update
rustup target add x86_64-pc-windows-msvc
rustup target add i686-pc-windows-msvc

call build32.cmd
call build64.cmd
