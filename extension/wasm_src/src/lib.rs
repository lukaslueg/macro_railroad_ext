#[cfg(cargo_web)]
#[macro_use]
extern crate stdweb;

#[cfg(cargo_web)]
use stdweb::js_export;

use std::{cell::RefCell, rc::Rc};
use stdweb::{traits::*, unstable::TryFrom, web};

mod util;

#[cfg_attr(cargo_web, js_export)]
pub fn load() {
    if let Err(err) = priv_load() {
        stdweb::console!(error, format!("{:?}", err))
    }
}

#[derive(Debug)]
pub enum Error {
    SynParsing(syn::Error),
    Syntax(web::error::SyntaxError),
    InvalidCharacter(web::error::InvalidCharacterError),
    ReplaceChild(String), // ReplaceChildError is private for some reason in stdweb 0.4.20
    NodeNotFound,
    #[cfg(feature = "webextension")]
    UnexpectedType,
}

impl From<syn::Error> for Error {
    fn from(e: syn::Error) -> Self {
        Error::SynParsing(e)
    }
}

impl From<web::error::SyntaxError> for Error {
    fn from(e: web::error::SyntaxError) -> Self {
        Error::Syntax(e)
    }
}

impl From<web::error::InvalidCharacterError> for Error {
    fn from(e: web::error::InvalidCharacterError) -> Self {
        Error::InvalidCharacter(e)
    }
}

type Result<T> = std::result::Result<T, Error>;

struct DiagramOptions {
    hide_internal: bool,
    keep_groups: bool,
    foldcommontails: bool,
    show_legend: bool,
}

impl Default for DiagramOptions {
    fn default() -> Self {
        DiagramOptions {
            hide_internal: true,
            keep_groups: true,
            foldcommontails: true,
            show_legend: true,
        }
    }
}

/// Returns `true` if the document's generator is "rustdoc"
fn is_rustdoc(document: &web::Document) -> Result<bool> {
    let gen = match document.query_selector(r#"head > meta[name="generator"]"#)? {
        Some(e) => e,
        None => return Ok(false),
    };
    Ok(gen.get_attribute("content").as_deref() == Some("rustdoc"))
}

/// Injects the relevant CSS into the document's <head>
fn inject_css(document: &web::Document) -> Result<()> {
    let head = document
        .head()
        .expect("<head> was already checked but now it's gone?");

    let mrext_css = document.create_element("link")?;
    mrext_css.set_attribute("type", "text/css")?;
    mrext_css.set_attribute("rel", "stylesheet")?;
    mrext_css.set_attribute("href", &util::Asset::CSS.url()?)?;

    // Since this CSS is loaded after the fact, the modal may flicker upon
    // page-load. The modal-containers are therefor display:none by default,
    // which is removed once the CSS takes over.
    mrext_css.set_attribute("onload", r#"
for (let n of document.getElementsByClassName("railroad_modal")) {
    n.removeAttribute("style");
}
"#)?;

    head.append_child(&mrext_css);

    let rr_css = document.create_element("style")?;
    rr_css.set_text_content(&railroad::DEFAULT_CSS);
    rr_css.set_attribute("type", "text/css")?;
    head.append_child(&rr_css);

    let mrr_css = document.create_element("style")?;
    mrr_css.set_text_content(&macro_railroad::diagram::CSS);
    mrr_css.set_attribute("type", "text/css")?;
    head.append_child(&mrr_css);

    Ok(())
}

/// The modal to go fullscreen
fn create_modal(document: &web::Document) -> Result<(web::Element, Rc<web::Element>)> {
    let modal_content = document.create_element("div")?;
    modal_content.append_child(&document.create_element("svg")?);
    modal_content.set_attribute("class", "railroad_modal_content")?;

    let modal_container = Rc::new(document.create_element("div")?);
    modal_container.append_child(&modal_content);
    modal_container.set_attribute("class", "railroad_modal")?;

    // See inject_css
    modal_container.set_attribute("style", "display: none")?;

    let modal_container_c = Rc::clone(&modal_container);
    modal_container.add_event_listener(move |_: web::event::ClickEvent| {
        util::classlist_remove(modal_container_c.as_ref(), "railroad_active");
    });

    Ok((modal_content, modal_container))
}

type UpdateFn = Rc<dyn Fn() -> Result<()>>;

/// The update function, called when options are set and to create the initial diagram
fn create_update_fn(
    options: Rc<RefCell<DiagramOptions>>,
    svg_container: Rc<web::Element>,
    modal_content: web::Element,
    macro_src: String,
) -> Result<UpdateFn> {
    Ok(Rc::new(move || {
        let options = options.borrow();
        let (width, svg) = to_diagram(&macro_src, &options)?;
        svg_container
            .replace_child(
                &web::Node::from_html(&svg)?,
                &svg_container.first_child().ok_or(Error::NodeNotFound)?,
            )
            .map_err(|e| Error::ReplaceChild(e.to_string()))?;
        svg_container.set_attribute("style", &format!("max-width: {}px", width))?;
        modal_content
            .replace_child(
                &web::Node::from_html(&svg)?,
                &modal_content.first_child().ok_or(Error::NodeNotFound)?,
            )
            .map_err(|e| Error::ReplaceChild(e.to_string()))?;
        Ok(())
    }))
}

/// The icons in the lower-right corner, including the options-dialog
fn create_icons(
    document: &web::Document,
    modal_container: Rc<web::Element>,
    update_diagram_fn: UpdateFn,
    options: Rc<RefCell<DiagramOptions>>,
) -> Result<web::Element> {
    // The icons in the bottom-right corner
    let icons_container = document.create_element("div")?;
    icons_container.set_attribute("class", "railroad_icons")?;

    // The options-thingy and the options
    let options_container = document.create_element("div")?;
    options_container.set_attribute("style", "position: relative; display: inline")?;

    // The container that holds the options-list
    let dropdown_container = Rc::new(document.create_element("div")?);
    dropdown_container.set_attribute("style", "position: absolute")?;
    dropdown_container.set_attribute("class", "railroad_dropdown_content")?;

    let options_list = document.create_element("ul")?;
    options_list.set_attribute("style", "list-style-type: none; padding: 0px; margin: 0px")?;
    macro_rules! create_option {
            ($key: ident, $label: literal) => {
                {
                    let update_diagram_fn = Rc::clone(&update_diagram_fn);
                    let options = Rc::clone(&options);

                    let list_item = document.create_element("li")?;
                    let input_item = Rc::new(document.create_element("input")?);
                    let input_item_id = util::random_id();
                    input_item.set_attribute("type", "checkbox")?;
                    input_item.set_attribute("id", &input_item_id)?;
                    input_item.set_attribute("checked", &options.borrow().$key.to_string())?;

                    let input_item_c = Rc::clone(&input_item);
                    input_item.add_event_listener(move |_: web::event::ChangeEvent| {
                        let input_item_c = input_item_c.as_ref();
                        options.borrow_mut().$key = util::is_checked(&input_item_c);
                        if let Err(err) = update_diagram_fn() {
                            stdweb::console!(error, format!("{:?}", err));
                        }
                    });

                    list_item.append_child(input_item.as_ref());

                    let label_item = document.create_element("label")?;
                    label_item.set_attribute("style", "padding-left: 8px")?;
                    label_item.set_attribute("for", &input_item_id)?;
                    label_item.set_text_content($label);
                    list_item.append_child(&label_item);

                    options_list.append_child(&list_item);
                    Ok(())
                } as Result<_>
            }
        }
    create_option!(hide_internal, "Hide macro-internal rules")?;
    create_option!(keep_groups, "Keep groups bound")?;
    create_option!(foldcommontails, "Fold common sections")?;
    create_option!(show_legend, "Generate legend")?;
    dropdown_container.append_child(&options_list);

    let options_icon = document.create_element("img")?;
    options_icon.set_attribute("class", "railroad_icon")?;
    options_icon.set_attribute("style", "margin-right: 8px")?;
    options_icon.set_attribute("src", &util::Asset::Options.url()?)?;
    let dropdown_container_c = Rc::clone(&dropdown_container);
    options_icon.add_event_listener(move |_: web::event::ClickEvent| {
        util::classlist_toggle(&dropdown_container_c, "railroad_dropdown_show");
    });
    options_container.append_child(&options_icon);
    options_container.append_child(dropdown_container.as_ref());
    icons_container.append_child(&options_container);

    // The fullscreen-toggle
    let fullscreen_icon = document.create_element("img")?;
    fullscreen_icon.set_attribute("class", "railroad_icon")?;
    fullscreen_icon.set_attribute("src", &util::Asset::Fullscreen.url()?)?;
    fullscreen_icon.add_event_listener(move |_: web::event::ClickEvent| {
        util::classlist_add(&modal_container, "railroad_active");
    });
    icons_container.append_child(&fullscreen_icon);

    Ok(icons_container)
}

fn priv_load() -> Result<()> {
    let document = web::document();

    // If this page was not generated by rustdoc, do nothing at all
    if !is_rustdoc(&document)? {
        return Ok(());
    }

    stdweb::console!(log, util::version_info());

    inject_css(&document)?;

    // Although there is most likely ever going to be exactly one macro-definition
    // per rustdoc-page, let's do it all anyway.
    for n in document.query_selector_all("pre.macro")? {
        let options: Rc<RefCell<DiagramOptions>> = Default::default();

        let macro_src = match web::HtmlElement::try_from(n.clone()) {
            Ok(e) => e.inner_text(),
            Err(e) => {
                stdweb::console!(error, e);
                continue;
            }
        };

        // The div that the `pre.macro` get's moved into, together with the new diagram nodes
        let new_node = document.create_element("div")?;
        new_node.set_attribute("style", "width: 100%;")?;

        let parent_node = match n.parent_node() {
            Some(node) => node,
            None => continue,
        };

        new_node.append_child(&n);

        let (modal_content, modal_container) = create_modal(&document)?;
        new_node.append_child(modal_container.as_ref());

        // The container which holds the inline-svg on the page
        let svg_container = Rc::new(document.create_element("div")?);
        svg_container.set_attribute("class", "railroad_container")?;
        svg_container.append_child(&document.create_element("svg")?);
        new_node.append_child(svg_container.as_ref());

        let update_diagram_fn = create_update_fn(
            Rc::clone(&options),
            Rc::clone(&svg_container),
            modal_content,
            macro_src,
        )?;

        let icons_container = create_icons(
            &document,
            modal_container,
            Rc::clone(&update_diagram_fn),
            options,
        )?;

        svg_container.append_child(&icons_container);

        parent_node.append_child(&new_node);
        update_diagram_fn()?;
    }

    Ok(())
}

/// Parse the given macro_rules!()-source, returns an SVG and it's preferred width
fn to_diagram(src: &str, options: &DiagramOptions) -> syn::parse::Result<(i64, String)> {
    use railroad::RailroadNode;

    let macro_rules = macro_railroad::parser::parse(&src)?;
    let mut tree = macro_railroad::lowering::MacroRules::from(macro_rules);

    if options.hide_internal {
        tree.remove_internal();
    }
    if !options.keep_groups {
        tree.ungroup();
    }
    if options.foldcommontails {
        tree.foldcommontails();
    }
    tree.normalize();

    let dia = macro_railroad::diagram::into_diagram(tree, options.show_legend);
    let mut svg = dia.to_string();
    if svg.ends_with('\n') {
        svg.pop();
    }
    Ok((dia.width(), svg))
}
