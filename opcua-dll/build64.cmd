cargo fmt

SET LIB=opcua

REM //==========================================================================
REM // 64-bit build
REM //
SET CARGO_CFG_TARGET_ARCH=x86_64
SET ARCH=x86_64-pc-windows-msvc
SET CARGO_TARGET_DIR=%TEMP%\targets\LIB
SET RELEASE_DIR=%CARGO_TARGET_DIR%\%ARCH%\release
cargo build --target=%ARCH% --release
copy %RELEASE_DIR%\%LIB%.dll ..\opcua-lvlib\%LIB%64.dll 
REM cargo build --target=x86_64-pc-windows-msvc --release
