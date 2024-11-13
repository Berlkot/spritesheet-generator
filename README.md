# Sritesheet generator
> [!NOTE]
> Right now project is mostly complete but requires some cleanup. Not sure when i will get time for that
## Compling
```bash
cargo build --release
```
## Running
Fill out `output_dir` and `input_dir` with actial values
```bash
# input_dir is expected to contain subdirectories
# Subdirectories are used here to comple multpule animations into one spritesheet
./target/release/spritesheet_generator input_dir output_dir

# you can also run without output_dir. In that case output directory will be pwd
./target/release/spritesheet_generator input_dir
```
## Todo
- [ ] Add option to disable image optimizations
- [ ] Add option to compile spritesheet inline
- [ ] Use hash function in image compare
- [ ] Write down features of generator in README
