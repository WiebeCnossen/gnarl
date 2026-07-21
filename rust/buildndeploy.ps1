cd $PSScriptRoot
$deploy = (get-command gnarl).Source
cargo check &&
  cargo clippy &&
  cargo test &&
  cargo build --release &&
  copy ./target/release/gnarl.exe $deploy -PassThru
