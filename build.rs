use slint_build::CompilerConfiguration;



fn main() {
    slint_build::compile_with_config(
        "ui/appwindow.slint",
        CompilerConfiguration::new()
        .with_scale_factor(1.5)
        .with_style("material".to_string())
    ).unwrap();
}
