# Ubuntu Library Mapper

Simple utility that generates mappings between x86-64 shared library files (`.so`) and the Ubuntu packages that provide them.

```json
{
  "libgdk-3.so.0": "libgtk-3-0",
  "libgomp.so.1": "libgomp1",
  "libcrypto.so.1.1": "libssl1.1",
  ..
}
```

## Usage

Build and run the project with cargo:

```bash
cargo build --release
```

The tool will generate the following files:
- `library_mapping_focal.json` (Ubuntu 20.04 LTS)
- `library_mapping_jammy.json` (Ubuntu 22.04 LTS)

### ⚠️ Resource Usage Warning

Please note:

* Each `Contents-amd64.gz` file is around 180MB, and one is downloaded for each Ubuntu version processed
* When decompressed, each file is approximately 4GB in size
* Due to gzip streaming, this binary uses less than 10MB of RAM at any given moment
* The final generated JSON mapping files are less than 1MB each
