.DEFAULT_GOAL := all
ALL_GOALS := wasm/macro_railroad_ext.wasm content_scripts/macro_railroad_ext.js
wasm_src/Cargo.lock:
	cd wasm_src && cargo update

wasm_src/target/deploy/macro_railroad_ext.js wasm_src/target/deploy/macro_railroad_ext.wasm: wasm_src/Cargo.lock wasm_src/Cargo.toml wasm_src/Web.toml wasm_src/build.rs wasm_src/src/lib.rs
	cd wasm_src && cargo web deploy

wasm:
	mkdir $@

wasm/macro_railroad_ext.wasm: wasm_src/target/deploy/macro_railroad_ext.wasm | wasm
	cp $< $@

content_scripts/macro_railroad_ext.js: wasm_src/target/deploy/macro_railroad_ext.js
	sed 's/fetch( "macro_railroad_ext\.wasm"/fetch( chrome.runtime.getURL( "wasm\/macro_railroad_ext.wasm" )/;s/instantiateStreaming === "function"/instantiateStreaming === "Fails_In_Firefox_On_Linux"/;s/\(console\.log( "Finished\)/\/\/\1/' $< > $@

packed.zip: manifest.json assets/fullscreen.svg assets/macro_railroad_ext.css assets/options.svg content_scripts/macro_railroad_ext.js content_scripts/main.js icons/icon48.png icons/icon96.png options/options.html wasm/macro_railroad_ext.wasm
	zip $@ $?

.PHONY: all
all: $(ALL_GOALS)

.PHONY: clean
clean:
	rm $(ALL_GOALS)
