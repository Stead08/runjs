use deno_core::error::AnyError;
use deno_core::op;
use deno_core::Extension;
use deno_core::Op;
use std::rc::Rc;

#[op]
async fn op_read_file(path: String) -> Result<String, AnyError> {
    let contents = tokio::fs::read_to_string(path).await?;
    Ok(contents)
}

#[op]
async fn op_write_file(path: String, contents: String) -> Result<(), AnyError> {
    tokio::fs::write(path, contents).await?;
    Ok(())
}

#[op]
fn op_remove_file(path: String) -> Result<(), AnyError> {
    std::fs::remove_file(path)?;
    Ok(())
}

async fn run_js(file_path: &str) -> Result<(), AnyError> {
    let current_dirctory = std::env::current_dir()?;
    let main_module = deno_core::resolve_path(file_path, current_dirctory.as_path())?;

    let runjs_extension = Extension::builder("fs")
        .ops(vec![
            op_read_file::DECL,
            op_write_file::DECL,
            op_remove_file::DECL,
        ])
        .build();
    let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
        extensions: vec![runjs_extension],
        ..Default::default()
    });
    js_runtime.execute_script(
        "[runjs:runtime.js]",
        deno_core::FastString::StaticAscii(include_str!("./runtime.js")),
    )?;

    let mod_id = js_runtime.load_main_module(&main_module, None).await?;
    let result = js_runtime.mod_evaluate(mod_id);
    js_runtime.run_event_loop(false).await?;
    result.await?
}
fn main() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    if let Err(error) = runtime.block_on(run_js("./example.js")) {
        eprintln!("error {}", error);
    }
}
