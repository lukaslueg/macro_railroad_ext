.DEFAULT_GOAL := test

wasm_src/Cargo.lock:
	cd wasm_src && cargo update

wasm_src/target/deploy/macro_railroad_ext.wasm: wasm_src/Cargo.lock wasm_src/Cargo.toml wasm_src/Web.toml wasm_src/build.rs wasm_src/src/lib.rs wasm_src/src/util.rs
	cd wasm_src && cargo web deploy --release --features webextension

wasm_src/target/deploy/macro_railroad_ext.js: wasm_src/target/deploy/macro_railroad_ext.wasm ;

assets:
	mkdir $@

wasm:
	mkdir $@

wasm/macro_railroad_ext.wasm: wasm_src/target/deploy/macro_railroad_ext.wasm | wasm
ifdef TRAVIS
	cp $< $@
else
	wasm-opt -Oz -o $@ $<
endif

assets/fullscreen.svg: wasm_src/static/fullscreen.svg | assets
	cp $< $@

assets/options.svg: wasm_src/static/options.svg | assets
	cp $< $@

assets/macro_railroad_ext.css: wasm_src/static/macro_railroad_ext.css | assets
	cp $< $@

content_scripts/macro_railroad_ext.js: wasm_src/target/deploy/macro_railroad_ext.js
	sed 's/fetch( "macro_railroad_ext\.wasm"/fetch( chrome.runtime.getURL( "wasm\/macro_railroad_ext.wasm" )/;s/instantiateStreaming === "function"/instantiateStreaming === "Fails_In_Firefox_On_Linux"/;s/\(console\.log( "Finished\)/\/\/\1/' $< > $@

packed.zip: manifest.json assets/fullscreen.svg assets/macro_railroad_ext.css assets/options.svg content_scripts/macro_railroad_ext.js content_scripts/main.js icons/icon48.png icons/icon96.png options/options.html wasm/macro_railroad_ext.wasm
	zip $@ $?

.PHONY: clean
clean:
	-rm packed.zip wasm_src/Cargo.lock integration_test/Cargo.lock wasm/macro_railroad_ext.wasm content_scripts/macro_railroad_ext.js assets/fullscreen.svg assets/options.svg assets/macro_railroad_ext.css
	-rmdir wasm
	-rmdir assets
	cd wasm_src && cargo clean
	cd integration_test && cargo clean

.PHONY: test
test: packed.zip
	cd wasm_src && cargo web test --release
	export RUST_BACKTRACE=1 MACRO_RAILROAD_PACKED="$(abspath $<)"; cargo test --manifest-path integration_test/Cargo.toml

.PHONY: screenshots
screenshots: packed.zip
	export MACRO_RAILROAD_PACKED="$(abspath $<)"; cargo run --manifest-path integration_test/Cargo.toml
