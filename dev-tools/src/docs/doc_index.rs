use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

struct RustdocAssets {
    normalize_css: String,
    rustdoc_css: String,
    main_js: String,
    search_js: String,
    stringdex_js: String,
    storage_js: String,
    favicon_svg: String,
    favicon_png: String,
    noscript_css: String,
}

pub fn generate(target_dir: &str) -> Result<()> {
    let doc_dir = PathBuf::from(target_dir).join("doc");
    let static_dir = doc_dir.join("static.files");
    let assets = discover_assets(&static_dir)?;
    let html = render_html(&assets);
    let output_path = doc_dir.join("index.html");

    fs::write(&output_path, html)
        .with_context(|| format!("Failed to write {}", output_path.display()))?;

    Ok(())
}

fn discover_assets(static_dir: &Path) -> Result<RustdocAssets> {
    let entries = fs::read_dir(static_dir)
        .with_context(|| format!("Failed to read {}", static_dir.display()))?;

    let mut names = Vec::new();
    for entry in entries {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            names.push(name.to_string());
        }
    }
    names.sort();

    Ok(RustdocAssets {
        normalize_css: find_asset(&names, "normalize-", ".css", static_dir)?,
        rustdoc_css: find_asset(&names, "rustdoc-", ".css", static_dir)?,
        main_js: find_asset(&names, "main-", ".js", static_dir)?,
        search_js: find_asset(&names, "search-", ".js", static_dir)?,
        stringdex_js: find_asset(&names, "stringdex-", ".js", static_dir)?,
        storage_js: find_asset(&names, "storage-", ".js", static_dir)?,
        favicon_svg: find_asset(&names, "favicon-", ".svg", static_dir)?,
        favicon_png: find_asset(&names, "favicon-32x32-", ".png", static_dir)?,
        noscript_css: find_asset(&names, "noscript-", ".css", static_dir)?,
    })
}

fn find_asset(names: &[String], prefix: &str, suffix: &str, static_dir: &Path) -> Result<String> {
    names
        .iter()
        .find(|name| name.starts_with(prefix) && name.ends_with(suffix))
        .cloned()
        .with_context(|| {
            format!(
                "Missing rustdoc asset matching pattern {}*{} in {}",
                prefix,
                suffix,
                static_dir.display()
            )
        })
}

fn render_html(assets: &RustdocAssets) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>gig-log - Rust</title>
    <link rel="stylesheet" href="static.files/{normalize_css}">
    <link rel="stylesheet" href="static.files/{rustdoc_css}">
    <meta name="rustdoc-vars"
        data-root-path="./"
        data-static-root-path="static.files/"
        data-current-crate="gig_log"
        data-themes=""
        data-resource-suffix=""
        data-rustdoc-version="1.92.0"
        data-channel="1.92.0"
        data-search-js="{search_js}"
        data-stringdex-js="{stringdex_js}"
        >
    <script src="static.files/{storage_js}"></script>
    <script defer src="static.files/{main_js}"></script>
    <noscript><link rel="stylesheet" href="static.files/{noscript_css}"></noscript>
    <link rel="alternate icon" type="image/png" href="static.files/{favicon_png}">
    <link rel="icon" type="image/svg+xml" href="static.files/{favicon_svg}">
</head>
<body class="rustdoc mod crate">
    <nav class="sidebar">
        <div class="sidebar-crate">
            <h2><a href="#">gig-log</a></h2>
        </div>
        <div class="sidebar-elems">
            <section id="rustdoc-toc">
                <h3><a href="#crates">Workspace Crates</a></h3>
                <ul class="block">
                    <li><a href="#crates" title="Crates">Crates</a></li>
                </ul>
            </section>
        </div>
    </nav>
    <div class="sidebar-resizer" title="Drag to resize sidebar"></div>
    <main>
        <div class="width-limiter">
            <section id="main-content" class="content">
                <div class="main-heading">
                    <h1>Crate <span>gig-log</span></h1>
                    <rustdoc-toolbar></rustdoc-toolbar>
                </div>
                <p>Documentation for the gig-log workspace.</p>
                <h2 id="crates" class="section-header">
                    Crates<a href="#crates" class="anchor">§</a>
                </h2>
                <dl class="item-table">
                    <dt><a class="mod" href="gig_log_api/index.html">gig_log_api</a></dt>
                    <dd>The backend REST API server.</dd>
                    <dt><a class="mod" href="gig_log_common/index.html">gig_log_common</a></dt>
                    <dd>Shared types and utilities.</dd>
                    <dt><a class="mod" href="gig_log_dev_tools/index.html">gig_log_dev_tools</a></dt>
                    <dd>Developer tooling for docs and workspace workflows.</dd>
                    <dt><a class="mod" href="gig_log_frontend/index.html">gig_log_frontend</a></dt>
                    <dd>The Leptos frontend application.</dd>
                </dl>
            </section>
        </div>
    </main>
</body>
</html>
"##,
        normalize_css = assets.normalize_css,
        rustdoc_css = assets.rustdoc_css,
        main_js = assets.main_js,
        search_js = assets.search_js,
        stringdex_js = assets.stringdex_js,
        storage_js = assets.storage_js,
        favicon_svg = assets.favicon_svg,
        favicon_png = assets.favicon_png,
        noscript_css = assets.noscript_css,
    )
}
