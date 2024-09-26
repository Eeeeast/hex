# Console disassembler
## About the programme
* Accepts as input records that are ASCII strings consisting of several pairs of 16-character digits. Entries must begin with a colon character. The Intel HEX record format is used.
* Use `--help` to get help.
* The `-a` argument adds field values to each record.
* The `-o` argument replaces common commands with private commands, if any.
* Example:
`hex :100060000C943E000C943E0011241FBECFEFD8E04C :10007000DEBFCDBF0E9440000C9452000C940000E3`
## Installation
* Install the Rust and Cargo compiler.
* Clone the repository.
* `cd` into the repository and run the build command with the release flag.
* After compilation, the executable will be available in `target\release`.
