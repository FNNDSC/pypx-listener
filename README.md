# pypx-listener

An instance handler for `storescp` which reorganizes incoming DICOM files.
Rust re-write of `px-repack` from [pypx](https://github.com/FNNDSC/pypx).

## Testing

```shell
./get_test_data.sh examples
```

## Benchmarking: v.s. Python

Setup benchmarks:

```shell
cargo build --release
```

`rx-repack` runs ~12x faster than it takes Python 3.11.3 to do _literally nothing._

```shell
hyperfine -N "python -c ''" "target/release/rx-repack --xcrdir $PWD/examples/FNNDSC-SAG-anon-3d6e850 --xcrfile 0002-1.3.12.2.1107.5.2.19.45152.2013030808110261698786039.dcm  --datadir /tmp/dicom"
```

`rx-repack` called per-instance is ~10x faster than `px-repack` called per-series.

```shell
source examples/pypx/bin/activate
export PATH="$PWD/target/release:$PATH"
hyperfine --prepare 'rm -rf /tmp/dicom' \
          "px-repack --xcrdir $PWD/examples/FNNDSC-SAG-anon-3d6e850 --parseAllFilesWithSubStr , --verbosity 0 --datadir /tmp/dicom" \
"fd --exec rx-repack --xcrdir '{//}' --xcrfile '{/}' --verbosity 0 --datadir /tmp/dicom \; --threads=1 --type f '.*\.dcm$' $PWD/examples/FNNDSC-SAG-anon-3d6e850"
```
