<picture style="width: 76px; height: 96px" width="76" height="96">
  <source media="(prefers-color-scheme: dark)" srcset="https://user-images.githubusercontent.com/43314/216831744-e17e8282-669d-4716-b728-1ba31edda3f0.svg">
  <source media="(prefers-color-scheme: light)" srcset="https://user-images.githubusercontent.com/43314/216831743-2afcda16-c2e4-406d-9183-ebfcd2d50537.svg">
  <img style="width: 76px; height: 96px" width="76" height="96" alt="" src="https://user-images.githubusercontent.com/43314/216831743-2afcda16-c2e4-406d-9183-ebfcd2d50537.svg">
</picture>

# PaperAge

Easy and secure paper backups of (smallish) secrets using the Age format ([age-encryption.org/v1](https://age-encryption.org/v1)).

## Features

* Takes plaintext input either from a file or stdin
* Encrypts that input with a passphrase
* Outputs a PDF with a QR code of the encrypted ciphertext
* The error correction level of the QR code is optimised (less data → more error correction)
* The passphrase **isn't** rendered on the PDF so you can print it on an untrusted printer (for example at work or at the library)

## Limitations

* The maximum input size is about 1.5KiB as QR codes cannot encode arbitrarily large payloads (1.5KiB results in a payload of about 3KiB when encrypted)
* Only passphrase-based encryption is supported at the moment
* Only the A4 paper size is supported at the moment

## Usage

```
paper-age [OPTIONS] [INPUT]
```

### **Arguments**

* `<INPUT>` — The path to the file to read, use `-` to read from stdin (max. ~1.5KB)

### **Options**

* `-t`, `--title <TITLE>` — Page title (max. 64 characters)

  Default value: `PaperAge`
* `-o`, `--output <OUTPUT>` — Output file name

  Default value: `out.pdf`
* `-f`, `--force` — Overwrite the output file if it already exists
* `-g`, `--grid` — Draw a grid pattern for debugging layout issues
* `--fonts-license` — Print out the license for the embedded fonts
* `-v`, `--verbose...` — More output per occurrence
* `-q`, `--quiet...` — Less output per occurrence
* `-h`, `--help` — Print help
* `-V`, `--version` — Print version

## Development

Run the latest from git locally, assuming you have already [installed Rust](https://www.rust-lang.org/learn/get-started):

1. Pull this repo
2. Run the tests: `cargo test`
3. Get help: `cargo run -- -h`
4. Encrypt from stdin: `echo "Hello World" | cargo run -- --title="secrets from stdin" --out="stdin.pdf"`
5. Run with maximum verbosity:  `echo "Hello World" | cargo run -- -vvvv`

### Releases

Releases are compiled and released on GitHub when new versions are tagged in git.

Use [cargo release](https://github.com/crate-ci/cargo-release) to tag and publish a new version, for example:

```sh
cargo release 1.2.3
```

⚠️ Append `--execute` to the command to actually execute the release.

## License & Credits

PaperAge is released under the MIT License. See [LICENSE.txt](LICENSE.txt) for details.

Includes the SIL Open Font Licensed [IBM Plex Mono](https://www.ibm.com/plex/) font. See [IBMPlexMono-LICENSE.txt](src/assets/fonts/IBMPlexMono-LICENSE.txt).

Uses the Rust implementation of Age from [github.com/str4d/rage](https://github.com/str4d/rage) and the [printpdf](https://github.com/fschutt/printpdf) library.

Thanks to [Ariel Salminen](https://arie.ls) for the PaperAge icon.
