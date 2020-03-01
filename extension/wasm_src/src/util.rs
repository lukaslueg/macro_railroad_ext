use std::borrow::Cow;
use stdweb::web;

#[allow(dead_code)]
mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub fn version_info() -> String {
    format!(
        "macro_railroad_ext built {} on {} using {}",
        built_info::BUILT_TIME_UTC,
        built_info::RUSTC_VERSION,
        built_info::DEPENDENCIES_STR
    )
}

/// A pseudo-random id
pub fn random_id() -> String {
    use rand::Rng;

    let mut id = String::from("railroad_");
    id.extend(
        rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(8),
    );
    id
}

/// Assets that are hosted either locally or from within the webextension
pub enum Asset {
    Options,
    Fullscreen,
    CSS,
}

impl Asset {
    #[cfg(not(feature = "webextension"))]
    pub fn url(&self) -> super::Result<Cow<'static, str>> {
        Ok(Cow::Borrowed(match self {
            Asset::Options => "options.svg",
            Asset::Fullscreen => "fullscreen.svg",
            Asset::CSS => "macro_railroad_ext.css",
        }))
    }

    #[cfg(feature = "webextension")]
    pub fn url(&self) -> super::Result<Cow<'static, str>> {
        let src = match self {
            Asset::Options => "assets/options.svg",
            Asset::Fullscreen => "assets/fullscreen.svg",
            Asset::CSS => "assets/macro_railroad_ext.css",
        };
        match stdweb::js! { return chrome.runtime.getURL(@{src}); } {
            stdweb::Value::String(href) => Ok(Cow::Owned(href)),
            _ => Err(super::Error::UnexpectedType),
        }
    }
}

// Some functions which wrap js! calls, stubbed out to allow compilation
// outside of wasm32-targets

#[cfg(cargo_web)]
pub fn classlist_remove(e: &web::Element, name: &str) {
    stdweb::js! {
        @{e}.classList.remove(@{name});
    }
}

#[cfg(not(cargo_web))]
pub fn classlist_remove(_: &web::Element, _: &str) {}

#[cfg(cargo_web)]
pub fn classlist_add(e: &web::Element, name: &str) {
    stdweb::js! {
        @{e}.classList.add(@{name});
    }
}

#[cfg(not(cargo_web))]
pub fn classlist_add(_: &web::Element, _: &str) {}

#[cfg(cargo_web)]
pub fn classlist_toggle(e: &web::Element, name: &str) {
    stdweb::js! {
        @{e}.classList.toggle(@{name});
    }
}

#[cfg(not(cargo_web))]
pub fn classlist_toggle(_: &web::Element, _: &str) {}

#[cfg(not(cargo_web))]
pub fn is_checked(_: &web::Element) -> bool {
    false
}

#[cfg(cargo_web)]
pub fn is_checked(e: &web::Element) -> bool {
    let checked = stdweb::js! {return @{e}.checked};
    checked == stdweb::Value::Bool(true)
}
