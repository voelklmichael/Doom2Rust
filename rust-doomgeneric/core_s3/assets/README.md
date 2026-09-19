# assets

`main.rs` embeds `doom1.wad` from this directory with `include_bytes!`, so the file must exist
before the firmware builds. It is gitignored.

Use the shareware IWAD (`doom1.wad`, 4,196,020 bytes, E1M1-E1M9 and three demos). It fits the
8 MB app partition in `partitions.csv` together with the code; the full game WAD (about 11 MB)
would need a larger partition.
