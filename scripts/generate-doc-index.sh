#!/usr/bin/env bash
set -euo pipefail

DOC_DIR="target/doc"
STATIC_DIR="$DOC_DIR/static.files"

# Discover hashed filenames for rustdoc assets
normalize_css=$(basename "$STATIC_DIR"/normalize-*.css)
rustdoc_css=$(basename "$STATIC_DIR"/rustdoc-*.css)
main_js=$(basename "$STATIC_DIR"/main-*.js)
search_js=$(basename "$STATIC_DIR"/search-*.js)
stringdex_js=$(basename "$STATIC_DIR"/stringdex-*.js)
storage_js=$(basename "$STATIC_DIR"/storage-*.js)
favicon_svg=$(basename "$STATIC_DIR"/favicon-*.svg)
favicon_png=$(basename "$STATIC_DIR"/favicon-32x32-*.png)
noscript_css=$(basename "$STATIC_DIR"/noscript-*.css)

cat > "$DOC_DIR/index.html" << ENDOFHTML
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>gig-log - Rust</title>
    <link rel="stylesheet" href="static.files/${normalize_css}">
    <link rel="stylesheet" href="static.files/${rustdoc_css}">
    <meta name="rustdoc-vars"
        data-root-path="./"
        data-static-root-path="static.files/"
        data-current-crate="gig_log"
        data-themes=""
        data-resource-suffix=""
        data-rustdoc-version="1.92.0"
        data-channel="1.92.0"
        data-search-js="${search_js}"
        data-stringdex-js="${stringdex_js}"
        >
    <script src="static.files/${storage_js}"></script>
    <script defer src="static.files/${main_js}"></script>
    <noscript><link rel="stylesheet" href="static.files/${noscript_css}"></noscript>
    <link rel="alternate icon" type="image/png" href="static.files/${favicon_png}">
    <link rel="icon" type="image/svg+xml" href="static.files/${favicon_svg}">
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
                    <dt><a class="mod" href="gig_log_frontend/index.html">gig_log_frontend</a></dt>
                    <dd>The Leptos frontend application.</dd>
                </dl>
            </section>
        </div>
    </main>
</body>
</html>
ENDOFHTML
