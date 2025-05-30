# Build

## Requirements

```shell
vcpkg install ffmpeg[avcodec,avfilter,avformat,avresample,swresample,mp3lame,vorbis,opus]:x64-windows-static-md --feature-flags=-default-features=false
```

## Build (Windows)

```shell
vcvars64
cargo build --release --target x86_64-pc-windows-msvc
```