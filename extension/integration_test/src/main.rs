const DIAGRAM_CONTAINER: &str =
    "#main > div.docblock.type-decl > div > div > div.railroad_container";
const URL_PANIC: &str = "https://doc.rust-lang.org/std/macro.panic.html";

struct Browser {
    _ext: tempdir::TempDir,
    browser: headless_chrome::browser::Browser,
}

impl Browser {
    fn extract_extension() -> Result<tempdir::TempDir, failure::Error> {
        let packed_path =
            std::env::var_os("MACRO_RAILROAD_PACKED").expect("Archive path not given by env");
        let packed_f = std::fs::File::open(packed_path)?;
        let extract_path = tempdir::TempDir::new("mrtest")?;
        let mut zip_archive = zip::ZipArchive::new(packed_f)?;
        for i in 0..zip_archive.len() {
            let mut f = zip_archive.by_index(i)?;
            let fname = extract_path.path().to_path_buf().join(f.sanitized_name());
            std::fs::create_dir_all(&fname.parent().unwrap())?;
            let mut e = std::fs::File::create(fname)?;
            std::io::copy(&mut f, &mut e)?;
        }
        Ok(extract_path)
    }

    fn new() -> Result<Self, failure::Error> {
        let ext = Self::extract_extension()?;
        let browser = headless_chrome::Browser::new(
            headless_chrome::LaunchOptionsBuilder::default()
                .extensions(vec![ext.path().as_ref()])
                .headless(false)
                .build()
                .unwrap(),
        )?;
        eprintln!("{:?}", browser.get_version()?);
        Ok(Self { _ext: ext, browser })
    }

    fn navigate_to_macro_page(
        &self,
        url: &str,
    ) -> Result<std::sync::Arc<headless_chrome::browser::tab::Tab>, failure::Error> {
        const DECL_TOGGLE: &str = "#main > div.toggle-wrapper.collapsed > a > span.toggle-label";
        let tab = self.wait_for_initial_tab()?;
        tab.navigate_to(url)?;
        tab.wait_for_element(DECL_TOGGLE)?.click()?;
        tab.wait_for_element(DIAGRAM_CONTAINER)?;
        Ok(tab)
    }

    #[cfg(test)]
    fn testable_tab(
        &self,
    ) -> Result<std::sync::Arc<headless_chrome::browser::tab::Tab>, failure::Error> {
        self.navigate_to_macro_page(URL_PANIC)
    }
}

impl std::ops::Deref for Browser {
    type Target = headless_chrome::browser::Browser;

    fn deref(&self) -> &Self::Target {
        &self.browser
    }
}

fn main() -> Result<(), failure::Error> {
    let browser = Browser::new()?;

    let screenshot = |url: &str, fname: &str| -> Result<(), failure::Error> {
        let tab = browser.navigate_to_macro_page(url)?;
        let png_data = tab.capture_screenshot(
            headless_chrome::protocol::page::ScreenshotFormat::PNG,
            None,
            true,
        )?;
        std::fs::write(fname, &png_data)?;
        Ok(())
    };

    screenshot(URL_PANIC, "std_panic.png")?;
    screenshot(
        "https://docs.rs/nom/4.2.2/nom/macro.named_attr.html",
        "nom_named_attr.png",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Browser, DIAGRAM_CONTAINER};
    const MACRO_BLOCK: &str = "#main > div.docblock.type-decl > div > div > pre";
    const MODAL_CONTAINER: &str = "#main > div.docblock.type-decl > div > div > div.railroad_modal";
    const LEGEND: &str =
        "#main > div.docblock.type-decl > div > div > div.railroad_container > svg > g > g.legend";
    const OPTIONS: &str =
        "#main > div.docblock.type-decl > div > div > div.railroad_container > div > div > img";
    const OPT_LEGEND: &str = "#main > div.docblock.type-decl > div > div > div.railroad_container > div > div > div > ul > li:nth-child(4) > label";

    fn init_log() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn loads() -> Result<(), failure::Error> {
        init_log();
        let browser = Browser::new()?;
        let tab = browser.wait_for_initial_tab()?;
        tab.navigate_to("https://doc.rust-lang.org")?;
        // TODO assert the extension loaded
        // TODO assert that the stylesheet get loaded, via document.syleSheets
        tab.navigate_to("https://docs.rs")?;
        // TODO assert the extension loaded
        Ok(())
    }

    #[test]
    fn executes() -> Result<(), failure::Error> {
        init_log();
        let browser = Browser::new()?;
        let tab = browser.testable_tab()?;
        let _ = tab.find_element(MODAL_CONTAINER)?;
        Ok(())
    }

    #[test]
    fn placement() -> Result<(), failure::Error> {
        init_log();
        let browser = Browser::new()?;
        let tab = browser.testable_tab()?;
        let inline_dia = tab.find_element(DIAGRAM_CONTAINER)?;
        let inline_dia_box = inline_dia.get_box_model()?;
        let macro_block_box = tab.find_element(MACRO_BLOCK)?.get_box_model()?;
        assert!(inline_dia_box.content.below(&macro_block_box.margin));
        assert!(inline_dia_box
            .content
            .within_horizontal_bounds_of(&macro_block_box.margin));
        Ok(())
    }

    #[test]
    fn set_options() -> Result<(), failure::Error> {
        init_log();
        let browser = Browser::new()?;
        let tab = browser.testable_tab()?;
        assert!(tab.find_element(LEGEND).is_ok()); // Legend is there?
        tab.find_element(OPTIONS)?.click()?; // Open the options
        tab.wait_for_element(OPT_LEGEND)?.click()?; // Disable legend
        assert!(headless_chrome::util::Wait::default()
            .until(|| dbg!(tab.find_element(LEGEND)).err())
            .is_ok());
        Ok(())
    }
}
