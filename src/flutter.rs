use zed_extension_api::{
    self as zed, SlashCommand, SlashCommandArgumentCompletion, SlashCommandOutput,
    SlashCommandOutputSection, Worktree,
};

struct FlutterExtension;

fn handle_pubspec(worktree: &Worktree) -> Result<SlashCommandOutput, String> {
    let content = worktree
        .read_text_file("pubspec.yaml")
        .map_err(|_| "No pubspec.yaml found in project root. Is this a Flutter/Dart project?".to_string())?;

    let text = format!("# pubspec.yaml\n\n```yaml\n{content}\n```");

    Ok(SlashCommandOutput {
        sections: vec![SlashCommandOutputSection {
            range: (0..text.len()).into(),
            label: "pubspec.yaml".to_string(),
        }],
        text,
    })
}

impl zed::Extension for FlutterExtension {
    fn new() -> Self {
        Self
    }

    fn complete_slash_command_argument(
        &self,
        _command: SlashCommand,
        _args: Vec<String>,
    ) -> Result<Vec<SlashCommandArgumentCompletion>, String> {
        Ok(vec![])
    }

    fn run_slash_command(
        &self,
        command: SlashCommand,
        _args: Vec<String>,
        worktree: Option<&Worktree>,
    ) -> Result<SlashCommandOutput, String> {
        match command.name.as_str() {
            "flutter-pubspec" => {
                let worktree = worktree.ok_or("This command requires an open project")?;
                handle_pubspec(worktree)
            }
            _ => Err(format!("Unknown command: {}", command.name)),
        }
    }
}

zed::register_extension!(FlutterExtension);
