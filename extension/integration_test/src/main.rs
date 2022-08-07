const DIAGRAM_CONTAINER: &str =
    "#main-content > div.docblock.item-decl > div > div > div.railroad_container, #main > div.docblock.type-decl > div > div > div.railroad_container";
const OPTIONS: &str =
    "#main-content > div.docblock.item-decl > div > div > div.railroad_container > div > div > img, #main > div.docblock.type-decl > div > div > div.railroad_container";
const OPT_FULLSCREEN: &str =
    "#main-content > div.docblock.item-decl > div > div > div.railroad_container > div > img, #main > div.docblock.type-decl > div > div > div.railroad_container";
const URL_NAMED: &str = "https://docs.rs/nom/4.2.2/nom/macro.named_attr.html";
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
            let fname = extract_path.path().to_path_buf().join(f.enclosed_name().unwrap());
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
                .path(Some(
                    headless_chrome::browser::default_executable().unwrap(),
                ))
                .headless(false)
                .build()
                .unwrap(),
        )?;
        log::info!("Browser version {:?}", browser.get_version()?);
        Ok(Self { _ext: ext, browser })
    }

    fn navigate_to_macro_page(
        &self,
        url: &str,
    ) -> Result<std::sync::Arc<headless_chrome::browser::tab::Tab>, failure::Error> {
        let tab = self.wait_for_initial_tab()?;
        log::trace!("Navigating to `{}`", &url);
        tab.navigate_to(url)?;
        log::trace!("Waiting for decl-element");
        // Ignore if the selector is not there, might be uncollapsed already...
        if let Ok(elem) =
            tab.wait_for_element("#main > div.toggle-wrapper.collapsed > a > span.toggle-label")
        {
            elem.click()?;
        }
        log::trace!("Waiting for diagram");
        tab.wait_for_element(DIAGRAM_CONTAINER)?;
        log::trace!("Successfully navigated");
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
    env_logger::init();

    let browser = Browser::new()?;

    let screenshot =
        |tab: std::sync::Arc<headless_chrome::Tab>, fname: &str| -> Result<(), failure::Error> {
            let png_data = tab.capture_screenshot(
                headless_chrome::protocol::page::ScreenshotFormat::PNG,
                None,
                true,
            )?;
            std::fs::write(fname, &png_data)?;
            log::info!("Successfully screenshotted `{}`", &fname);
            Ok(())
        };

    let screenshot_fs =
        |tab: std::sync::Arc<headless_chrome::Tab>, fname: &str| -> Result<(), failure::Error> {
            log::trace!("Waiting for Options...");
            tab.find_element(OPTIONS)?.click()?;
            log::trace!("Waiting for Fullscreen...");
            tab.wait_for_element(OPT_FULLSCREEN)?.click()?;
            std::thread::sleep(std::time::Duration::from_secs(2)); // Wait for the gfx
            screenshot(tab, fname)
        };

    screenshot(browser.navigate_to_macro_page(URL_PANIC)?, "std_panic.png")?;

    screenshot(
        browser.navigate_to_macro_page(URL_NAMED)?,
        "nom_named_attr.png",
    )?;

    screenshot_fs(
        browser.navigate_to_macro_page(URL_PANIC)?,
        "std_panic_fs.png",
    )?;
    screenshot_fs(
        browser.navigate_to_macro_page(URL_NAMED)?,
        "nom_named_attr_fs.png",
    )?;

    let tab = browser.navigate_to_macro_page(URL_PANIC)?;
    tab.find_element(OPTIONS)?.click()?;
    screenshot(tab, "std_panic_options.png")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const LEGEND: &str =
        "#main-content > div.docblock.item-decl > div > div > div.railroad_container > svg > g > g.legend, #main > div.docblock.type-decl > div > div > div.railroad_container > svg > g > g.legend";
    const MAIN: &str = "#main-content, #main";
    const MODAL_CONTAINER: &str = "#main-content > div.docblock.item-decl > div > div > div.railroad_modal, #main > div.docblock.type-decl > div > div > div.railroad_modal";
    const OPT_LEGEND: &str = "#main-content > div.docblock.item-decl > div > div > div.railroad_container > div > div > div > ul > li:nth-child(4) > label, #main > div.docblock.type-decl > div > div > div.railroad_container > div > div > div > ul > li:nth-child(4) > label";
    const URL_BITFLAGS: &str = "https://docs.rs/bitflags/1.1.0/bitflags/macro.bitflags.html";
    const MACRO_BLOCK: &str = "#main-content > div.docblock.item-decl > div > div > pre, #main > div.docblock.type-decl > div > div > pre";

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
        tab.find_element(MODAL_CONTAINER).map(|_| ())
    }

    fn test_placement(browser: &Browser, url: &str) -> Result<(), failure::Error> {
        let tab = browser.navigate_to_macro_page(url)?;
        log::trace!("Looking for main-box");
        let main_box = tab.find_element(MAIN)?.get_box_model()?;
        log::trace!("Looking for macro-box");
        let macro_block_box = tab.find_element(MACRO_BLOCK)?.get_box_model()?;
        assert!(macro_block_box.content.within_bounds_of(&main_box.margin));

        log::trace!("Looking for diagram-box");
        let inline_dia_box = tab.find_element(DIAGRAM_CONTAINER)?.get_box_model()?;
        assert!(inline_dia_box.content.within_bounds_of(&main_box.margin));
        assert!(inline_dia_box.content.above(&macro_block_box.margin));
        assert!(inline_dia_box
            .content
            .within_horizontal_bounds_of(&macro_block_box.margin));
        Ok(())
    }

    #[test]
    fn placement() -> Result<(), failure::Error> {
        init_log();
        let browser = Browser::new()?;
        test_placement(&browser, URL_PANIC)?;
        test_placement(&browser, URL_BITFLAGS)?;
        test_placement(&browser, URL_NAMED)?;
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
