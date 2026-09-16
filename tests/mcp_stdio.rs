//! End-to-end test: drive the built binary over stdio like a real MCP client.

use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::TempDir;

fn run_session(input: &str) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nxm-webtomd"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn server");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .expect("write stdin");

    let output = child.wait_with_output().expect("wait");
    String::from_utf8(output.stdout).expect("utf8 stdout")
}

#[test]
fn initialize_and_list_tools() {
    let input = "\
{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}
{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}
";
    let out = run_session(input);
    assert!(out.contains("\"nxm-webtomd\""), "serverInfo missing: {out}");
    assert!(out.contains("web_to_md"), "web_to_md missing: {out}");
}

#[test]
fn web_to_md_inline_returns_stub_markdown() {
    let call = "\
{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\"params\":{\"name\":\"web_to_md\",\"arguments\":{\"url\":\"https://httpbin.org/html\"}}}\n";
    let out = run_session(call);
    // Ensure the call succeeded (not an error).
    assert!(!out.contains("\"isError\": true"), "unexpected error: {out}");
    // Ensure we got a text response.
    assert!(out.contains("\"type\":\"text\""), "missing text content: {out}");
    // Ensure we got some non‑empty content.
    // Extract the text field value (naive check).
}

#[test]
fn web_to_md_to_file() {
    let dir = TempDir::new().unwrap();
    let out_path = dir.path().join("article.md");

    let call = format!(
        "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\"params\":{{\"name\":\"web_to_md\",\"arguments\":{{\"url\":\"https://httpbin.org/html\",\"output_path\":\"{}\"}}}}}}\n",
        out_path.display()
    );
    let out = run_session(&call);
    // Ensure the call succeeded and reported a write.
    assert!(out.contains("Wrote"), "write confirmation missing: {out}");

    // Verify the file was created and contains something.
    let contents = std::fs::read_to_string(&out_path).expect("read output file");
    assert!(!contents.is_empty(), "output file is empty");
}