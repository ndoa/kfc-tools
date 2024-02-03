# kfc-tools
[![CI](https://github.com/ndoa/kfc-tools/workflows/CI/badge.svg)](https://github.com/ndoa/kfc-tools/actions)

## Usage:

Run the extractor with the following arguments:

```bash
kfc_extractor {path to enshrouded.kfc_dir} {path to enshrouded.kfc_data}
```


Output notes:

* The extractor will create an `output` folder in the current directory containing the extracted files.

* The extracted files are named by hash rather than their original string name.
    * The "original" filenames (pre-hash) are mostly just GUID-strings, so are almost equally as useless for reverse engineering/modding purposes.
      These will likely be associated with "nicer" fake file names in the future.

* The extracted files are currently categorized into two groups:
    * `./output/resource_packages/` - Keen resource packages with the CRPF file magic.
    * `./output/raw/` - Catch-all for all unknown files