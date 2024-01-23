# Ragnaroek

Ragnaroek is a tool for interacting with the bootloaders of Samsung devices.

It's meant to serve as a hackable platform for research into various Samsung bootloader protocols, as well as a replacement for the semi-maintained Heimdall flash tool.

In addition to Heimdall's features, this project also strives to support more obscure use cases. These include dumping memory via upload mode or flashing Samsung smartwatches via wireless download mode.

## History

Ragnaroek was started at [spline](https://spline.de/) late one night after a multi-hour long futile attempt to hack wireless download mode support into Heimdall.

## Project status

Some older devices (Galaxy S3, S5) have working flashing. Some newer devices (e.g. Galaxy A40) have working protocol initialization, but not flashing. Yet others (such as some of the Intel-based Samsung tablets) don't even have that.

Testing of new devices is always appreciated! Just remember that bricking is always a possibility.

**Attention:** Currently the GUI and flashing via USB instead of Wifi is not supported.

## Usage

The CLI's arguments may change frequently, as it's under very active development. Therefore, they're not documented in this README. Instead, look at the tool's help output:

```bash
cargo run --bin ragnaroek-cli -- --help
```

To start the GUI (experimental, missing major features):

```bash
cargo run --bin ragnaroek-gui
```

## Compilation

```bash
cargo build --all-features --release
```

## Tests

Test coverage of the codebase is currently quite low. Only the Odin archive and PIT parsers are somewhat covered. A test harness for automatically testing against real devices would be appreciated.

To run the tests:

```bash
cargo test --all --all-features
```

## Resources

* [The original Heimdall project](https://github.com/Benjamin-Dobell/Heimdall)
* [A more maintained fork of Heimdall](https://git.sr.ht/~grimler/Heimdall)
* [Yet another Heimdall fork, seemingly focused on macOS fixes](https://github.com/amo13/Heimdall)
* [Protocol documentation for the download protocol and PIT format](https://samsung-loki.github.io/samsung-docs/)
* [An upload mode implementation](https://github.com/bkerler/sboot_dump)
* [Leaked Odin for Linux](https://forum.xda-developers.com/t/official-samsung-odin-v4-1-2-1-dc05e3ea-for-linux.4453423/)
