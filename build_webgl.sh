cargo build --release --features webgl2 --target wasm32-unknown-unknown  &&
wasm-bindgen --no-typescript --target web --out-dir ./out/ --out-name "life" ./target/wasm32-unknown-unknown/release/life.wasm &&
wasm-opt -Oz --strip-debug -o out/life_compressed.wasm out/life_bg.wasm &&
cp -r assets out/
