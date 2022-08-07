.DEFAULT_GOAL := test

wasm_src/Cargo.lock:
	cd wasm_src && cargo update

wasm_src/pkg/macro_railroad_ext.wasm: wasm_src/Cargo.lock wasm_src/Cargo.toml wasm_src/build.rs wasm_src/src/lib.rs
	cd wasm_src && wasm-pack build -t no-modules && cp pkg/macro_railroad_ext_bg.wasm pkg/macro_railroad_ext.wasm

wasm_src/pkg/macro_railroad_ext.js: wasm_src/pkg/macro_railroad_ext.wasm

wasm:
	mkdir $@

wasm/macro_railroad_ext.wasm: wasm_src/pkg/macro_railroad_ext.wasm | wasm
	cp $< $@

content_scripts/macro_railroad_ext.js: wasm_src/pkg/macro_railroad_ext.js
	cp $< $@

packed.zip: manifest.json assets/fullscreen.svg assets/macro_railroad_ext.css assets/options.svg content_scripts/macro_railroad_ext.js content_scripts/main.js icons/icon48.png icons/icon96.png options/options.html wasm/macro_railroad_ext.wasm
	zip $@ $?

.PHONY: clean
clean:
	-rm packed.zip wasm_src/Cargo.lock integration_test/Cargo.lock wasm/macro_railroad_ext.wasm content_scripts/macro_railroad_ext.js assets/fullscreen.svg assets/options.svg assets/macro_railroad_ext.css
	-rm -r wasm_src/pkg
	-rmdir wasm
	cd wasm_src && cargo clean
	cd integration_test && cargo clean

.PHONY: test
test: packed.zip
	cd wasm_src && cargo test --release
	export RUST_BACKTRACE=1 MACRO_RAILROAD_PACKED="$(abspath $<)"; cargo test --manifest-path integration_test/Cargo.toml

.PHONY: screenshots
screenshots: packed.zip
	export MACRO_RAILROAD_PACKED="$(abspath $<)"; cargo run --manifest-path integration_test/Cargo.toml
