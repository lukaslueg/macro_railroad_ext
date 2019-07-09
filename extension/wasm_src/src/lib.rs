#[cfg(cargo_web)]
#[macro_use]
extern crate stdweb;
#[macro_use]

extern crate serde_derive;

#[cfg(cargo_web)]
use stdweb::js_export;

#[allow(dead_code)]
mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[derive(Serialize)]
pub enum DiagramResult {
    Ok {
        name: String,
        width: i64,
        height: i64,
        svg: String,
    },
    Err(String),
}

#[cfg(cargo_web)]
js_serializable!(DiagramResult);

#[cfg_attr(cargo_web, js_export)]
pub fn version_info() -> String {
    format!(
        "macro_railroad_ext built {} on {} using {}",
        built_info::BUILT_TIME_UTC,
        built_info::RUSTC_VERSION,
        built_info::DEPENDENCIES_STR
    )
}

#[cfg_attr(cargo_web, js_export)]
pub fn to_diagram_node(
    src: &str,
    hide_internal: bool,
    keep_groups: bool,
    foldcommontails: bool,
    legend: bool,
) -> DiagramResult {
    match to_diagram(src, hide_internal, keep_groups, foldcommontails, legend) {
        Ok((name, svg)) => {
            use railroad::RailroadNode;
            DiagramResult::Ok {
                name: htmlescape::encode_minimal(&name),
                width: svg.width(),
                height: svg.height(),
                svg: svg.to_string(),
            }
        }
        Err(e) => DiagramResult::Err(e.to_string()),
    }
}

fn to_diagram(
    src: &str,
    hide_internal: bool,
    keep_groups: bool,
    foldcommontails: bool,
    legend: bool,
) -> syn::parse::Result<(String, railroad::Diagram<Box<dyn railroad::RailroadNode>>)> {
    let macro_rules = macro_railroad::parser::parse(&src)?;

    let mut tree = macro_railroad::lowering::MacroRules::from(macro_rules);

    if hide_internal {
        tree.remove_internal();
    }

    if !keep_groups {
        tree.ungroup();
    }

    if foldcommontails {
        tree.foldcommontails();
    }

    tree.normalize();

    let name = tree.name.clone();

    let mut dia = macro_railroad::diagram::into_diagram(tree, legend);

    dia.add_default_css();
    macro_railroad::diagram::add_default_css(&mut dia);

    Ok((name, dia))
}
