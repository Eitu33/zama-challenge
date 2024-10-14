# Zama Challenge | Thomas Plisson

## Usage

Install [Rust](https://www.rust-lang.org/tools/install) if you haven't already.

Launch the server:
```
cargo run --bin server
```

Send files from the client (check post command help for options):
```
cargo run --bin client 127.0.0.1:3000 post data/client
```

Request the i-th file and its corresponding merkle proof:
```
cargo run --bin client 127.0.0.1:3000 get 2
```