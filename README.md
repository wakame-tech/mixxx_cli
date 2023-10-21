# mixxx_cli
WIP

- list playlist's tracks

```bash
export MIXXX_DB_PATH=/path/to/mixxxdb.sqlite
cargo run -- playlist --id {{id}}
```

- convert track_locations

```bash
cargo run -- convert --in /path/to/mixxxdb.sqlite --out ./copy.sqlite --directory "F:/"
02:56:45 [DEBUG] (1) mixxx_cli: 3037 tracks converted
```