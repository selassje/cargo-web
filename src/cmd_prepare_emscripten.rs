use emscripten::setup_emscripten;
use emscripten::EmscriptenPackage;
use error::Error;

pub fn command_prepare_emscripten<'a>() -> Result<(), Error> {
    let emscripten = EmscriptenPackage {
        git: "https://github.com/emscripten-core/emsdk.git".to_owned(),
        rev: "main".to_owned(),
        sdk_version: "2.0.9".to_owned(),
    };
    setup_emscripten(
        &emscripten,
        &std::path::Path::new("F:\\Repos\\emsdk_my_cargo_web"),
    )?;
    Ok(())
}
