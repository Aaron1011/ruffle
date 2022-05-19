use subprocess::{Popen, PopenConfig, Redirection};
use walkdir::WalkDir;

fn main() {
    // Only rerun the build script if the AS files change -
    // we don't want to add the entire directory, because
    // the 'library.swf' timestamp will be updated by
    // 'compile_swc.sh'
    println!("cargo:rerun-if-changed=../compile_swc.sh");
    for entry in WalkDir::new("src/playerglobal") {
        let entry = entry.unwrap();
        if let Some(ext) = entry.path().extension() {
            if ext == "as" {
                println!("cargo:rerun-if-changed={}", entry.path().display());
            }
        }
    }

    // This script expects to be run from the root of the repository
    let mut compile = Popen::create(
        &["bash", "./compile_swc.sh"],
        PopenConfig {
            stdout: Redirection::Pipe,
            stderr: Redirection::Merge,
            cwd: Some("../".into()),
            ..Default::default()
        },
    )
    .expect("Failed to execute 'compile_swc.sh'");

    let (out, _) = compile.communicate(None).unwrap();
    let status = compile.wait().unwrap();
    if !status.success() {
        println!(
            "cargo:warning=Failed to run core/compile_swc.sh - fix the errors before continuing"
        );
        for line in out.unwrap_or_default().lines() {
            println!("cargo:warning={}", line);
        }
    }
}
