# hce-tuner

This tool is a HCE tuner based on [jw1912](https://github.com/jw1912/hce-tuner)'s implementation. This implementation has been adapted to work specifically with `byte-knight`.

## Commands

### Tune

Tune the engine's HCE values. Tuning will automatically stop if there is no improvement detected, so generally I run at least 20,000 epochs for a new tune.

### Interleave

This command interleaves data sets to create a new dataset to be used for tuning. Data is read and normalized to be white-relative and then output to the target file.
