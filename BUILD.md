# Navboard Windows Build

## Toolchain

- Windows x64
- Rust MSVC toolchain
- Visual Studio C++ build tools
- Dioxus CLI
- NSIS with the plugins required by the packaging recipe

Release development used Rust 1.99.1 and Dioxus CLI 0.7.10.

## Application

From the repository root:

    cargo test --locked
    dx serve --desktop

## Release staging

    dx bundle --windows --release --package-types nsis

The custom NSIS recipe consumes the generated application
staging directory under:

    target/dx/navboard/bundle/windows/nsis/_staging

It also consumes the Echo worker, setup script, dependency
lock, application license and third-party license notices.

The optional Python environment must not be included in
the custom installer payload.

## Branded installer

Obtain the official Microsoft Edge WebView2 Evergreen
Standalone Installer for x64 from Microsoft.

From the packaging directory, invoke makensis.exe with:

    /DPROJECT_ROOT=<absolute repository path>
    /DWEBVIEW2_INSTALLER=<absolute WebView2 installer path>
    Navboard.nsi

Review the File directives against the generated staging
directory before compiling the installer.

The original release was compiled in a neutral workspace.
Build paths, toolchain differences and timestamps may
prevent byte-for-byte reproduction of the release binary.

The normalized packaging recipe has not yet been
independently rebuilt on a clean machine.

## Verification

    Get-FileHash .\Navboard_0.1.0_x64-setup.exe -Algorithm SHA256

Compare against the published SHA256SUMS.txt.

## Echo

See the optional Echo runtime instructions in README.md.

Python dependencies are provisioned separately from PyPI
using the supplied version and hash lock.