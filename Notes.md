To watch src files and rebuild with wasm-pack on change:
`cargo watch -w src/ -x check -s "wasm-pack build"`

To run the backend use `cargo run --bin server`.

To run the frontend use `npm run start` from the `www` directory.