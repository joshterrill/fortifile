# FortiFile


### Build and distribute

```bash
cargo build --release
cp target/release/file-manager FortiFile.app/Contents/MacOS/FortiFile
chmod 777 FortiFile.app/Contents/MacOS/*
cp -R FortiFile.app/ /Applications/FortiFile.app/
```