# PaperAge

Easy and secure paper backups of (smallish) secrets using the Age format ([age-encryption.org/v1](https://age-encryption.org/v1)).

## Goals

- Take a secret and encrypt it with a passphrase using the [Rust implementaton of Age](https://github.com/str4d/rage).
- Generate a PDF with the cipher text encoded as a QR code
- As the contents of the PDF is fully encrypted, it can be printed on any printer without needing to trust the owner of the printer
- That QR code can then be scanned using any modern smartphone and decrypted using any implementation of Age
