//! The page a browser shows at `https://127.0.0.1:<port>/`. Loading it at all
//! proves the browser trusts the local certificate.

/// Renders the status page. Static text only: no scripts, no external assets.
pub(crate) fn status_page(port: u16) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Swakshar is running</title>
<style>
body {{ margin: 0; font: 16px/1.5 -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; background: #f6f7f5; color: #1c2321; }}
main {{ max-width: 34rem; margin: 12vh auto; padding: 2rem; background: #fff; border: 1px solid #e3e6e2; border-radius: 12px; }}
h1 {{ margin: 0 0 .5rem; font-size: 1.35rem; }}
p {{ margin: .5rem 0; color: #4a5451; }}
@media (prefers-color-scheme: dark) {{
  body {{ background: #111514; color: #e8ecea; }}
  main {{ background: #181d1c; border-color: #2a3230; }}
  p {{ color: #aab4b1; }}
}}
</style>
</head>
<body>
<main>
<h1>Swakshar is running</h1>
<p>This browser trusts Swakshar's local certificate on port {port}, so the GST portal can reach it.</p>
<p>You can close this tab.</p>
</main>
</body>
</html>
"#
    )
}
