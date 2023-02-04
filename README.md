# PaperAge

Easy and secure paper backups of (smallish) secrets using the Age format ([age-encryption.org/v1](https://age-encryption.org/v1)).

## Features

* Takes plaintext input either from a file or stdin
* Encrypts that input with a passphrase
* Outputs a PDF with a QR code of the encrypted ciphertext
* The passphrase **isn't** rendered on the PDF so you can print it on an untrusted printer (for example at work or at the library)

## Limitations

* The maximum input size is about 1.5KiB as QR codes cannot encode arbitrarily large payloads (1.5KiB results in a payload of about 3KiB when encrypted)
* Only passphrase-based encryption is supported at the moment
* Only the A4 paper size is supported at the moment

## Usage

```
paper-age [OPTIONS] [INPUT]
```

### **Arguments:**

* `<INPUT>` — The path to the file to read, use `-` to read from stdin (max. ~1.5KB)

### **Options:**

* `--title <TITLE>` — Page title (max. 64 characters)

  Default value: `Paper Rage`
* `-o`, `--output <OUTPUT>` — Output file name

  Default value: `out.pdf`
* `-g`, `--grid` — Draw a grid pattern for debugging layout issues
* `--fonts-license` — Print out the license for the embedded fonts
* `-v, --verbose...` — More output per occurrence
* `-q, --quiet...` — Less output per occurrence
* `-h, --help` — Print help
* `-V, --version` — Print version
## License & Credits

PaperAge is released under the MIT License. See the [LICENSE.txt](LICENSE.txt) file for details.

Includes the [IBM Plex Mono](https://www.ibm.com/plex/) font, see [IBMPlexMono-LICENSE.txt](src/assets/fonts/IBMPlexMono-LICENSE.txt) for details.

The encryption uses the Rust implementation of Age from [github.com/str4d/rage](https://github.com/str4d/rage).
