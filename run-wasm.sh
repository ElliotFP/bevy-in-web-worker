set -e 

# When running in a worker, there's an issue in debug mode where the first few frames need extended frame intervals
# https://github.com/bevyengine/bevy/issues/13345
cargo build --no-default-features \
--target wasm32-unknown-unknown 

# Generate bindings
for i in target/wasm32-unknown-unknown/debug/*.wasm;
do
    wasm-bindgen --no-typescript --out-dir wasm --web "$i";
    # Scripts loaded in Workers cannot use ES6 modules
    wasm-bindgen --no-typescript --out-dir wasm-no-modules --target no-modules "$i";
done

cp wasm/bevy_in_web_worker.js public/bevy_in_main_thread.js
cp wasm-no-modules/bevy_in_web_worker.js public/bevy_in_web_worker.js
# Both JS files share the same wasm
cp wasm/bevy_in_web_worker_bg.wasm public/bevy_in_web_worker_bg.wasm

basic-http-server public
