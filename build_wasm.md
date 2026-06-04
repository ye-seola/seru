source ~/SERU/emsdk/emsdk_env.sh
EMCC_CFLAGS='-sEXPORTED_RUNTIME_METHODS=["HEAPU8","HEAPU32","cwrap","ccall"]' cargo +1.88.0 build --release --target wasm32-unknown-emscripten
