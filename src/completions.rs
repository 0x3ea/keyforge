use clap::Command;
use clap_complete::{generate, Shell};

pub fn generate_completion(shell: Shell, cmd: &mut Command) -> Result<String, String> {
    let mut output = Vec::new();
    let bin_name = cmd.get_name().to_string();

    generate(shell, cmd, bin_name, &mut output);

    String::from_utf8(output).map_err(|e| format!("completion output is not valid UTF-8: {e}"))
}
