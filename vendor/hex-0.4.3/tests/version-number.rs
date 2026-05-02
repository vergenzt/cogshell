#![allow(non_fmt_panic)]


fn test_readme_deps() {
    version_sync::assert_markdown_deps_updated!("README.md");
}


fn test_html_root_url() {
    version_sync::assert_html_root_url_updated!("src/lib.rs");
}
