use zed_extension_api::{
    self as zed, SlashCommand, SlashCommandArgumentCompletion, SlashCommandOutput,
    SlashCommandOutputSection, Worktree,
};

struct FlutterExtension;

fn file_exists(worktree: &Worktree, path: &str) -> bool {
    worktree.read_text_file(path).is_ok()
}

fn extract_yaml_value<'a>(content: &'a str, key: &str) -> Option<&'a str> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(key) {
            if let Some(value) = rest.strip_prefix(':') {
                let value = value.trim();
                if !value.is_empty() {
                    return Some(value);
                }
            }
        }
    }
    None
}

fn handle_doctor(worktree: &Worktree) -> Result<SlashCommandOutput, String> {
    let mut report = String::from("# Flutter Project Health Report\n\n");

    // Check pubspec.yaml
    match worktree.read_text_file("pubspec.yaml") {
        Ok(pubspec) => {
            report.push_str("## Project Info\n\n");

            if let Some(name) = extract_yaml_value(&pubspec, "name") {
                report.push_str(&format!("- **Name**: {name}\n"));
            }
            if let Some(desc) = extract_yaml_value(&pubspec, "description") {
                report.push_str(&format!("- **Description**: {desc}\n"));
            }
            if let Some(version) = extract_yaml_value(&pubspec, "version") {
                report.push_str(&format!("- **Version**: {version}\n"));
            }

            // Check for SDK constraint
            if pubspec.contains("sdk:") {
                report.push_str("- **SDK constraint**: found\n");
            } else {
                report.push_str("- **SDK constraint**: MISSING (recommended to set)\n");
            }

            // Check for Flutter SDK dependency
            if pubspec.contains("flutter:") {
                report.push_str("- **Flutter SDK**: referenced\n");
            }

            // Check common dependencies
            report.push_str("\n## Dependencies Check\n\n");
            let deps_to_check = [
                ("flutter_test", "Testing framework"),
                ("flutter_lints", "Lint rules (deprecated, use flutter_lints)"),
                ("flutter_localizations", "Localization support"),
                ("build_runner", "Code generation"),
                ("freezed", "Immutable data classes"),
                ("json_serializable", "JSON serialization"),
                ("riverpod", "State management (Riverpod)"),
                ("flutter_riverpod", "Flutter Riverpod bindings"),
                ("bloc", "State management (BLoC)"),
                ("flutter_bloc", "Flutter BLoC bindings"),
                ("provider", "State management (Provider)"),
                ("go_router", "Routing"),
                ("dio", "HTTP client"),
                ("http", "HTTP client (dart:io)"),
                ("equatable", "Value equality"),
                ("get_it", "Service locator"),
                ("injectable", "Dependency injection"),
                ("hive", "Local storage"),
                ("shared_preferences", "Key-value storage"),
                ("sqflite", "SQLite database"),
            ];

            let mut found_any = false;
            for (dep, desc) in &deps_to_check {
                if pubspec.contains(dep) {
                    report.push_str(&format!("- {dep} — {desc}\n"));
                    found_any = true;
                }
            }
            if !found_any {
                report.push_str("- No common Flutter packages detected\n");
            }
        }
        Err(_) => {
            report.push_str("**pubspec.yaml not found** — this may not be a Flutter/Dart project.\n\n");
        }
    }

    // Check project files
    report.push_str("\n## Project Files\n\n");
    let files_to_check = [
        ("pubspec.yaml", "Package manifest"),
        ("pubspec.lock", "Dependency lock file"),
        ("analysis_options.yaml", "Dart analysis configuration"),
        ("l10n.yaml", "Localization configuration"),
        ("build.yaml", "Build runner configuration"),
        (".metadata", "Flutter metadata"),
        ("lib/main.dart", "App entry point"),
        ("test/widget_test.dart", "Default widget test"),
        ("android/app/build.gradle.kts", "Android build config (Kotlin DSL)"),
        ("android/app/build.gradle", "Android build config (Groovy)"),
        ("ios/Runner.xcodeproj/project.pbxproj", "iOS project config"),
        ("web/index.html", "Web entry point"),
        ("linux/CMakeLists.txt", "Linux build config"),
        ("macos/Runner.xcodeproj/project.pbxproj", "macOS project config"),
        ("windows/CMakeLists.txt", "Windows build config"),
    ];

    for (path, desc) in &files_to_check {
        let status = if file_exists(worktree, path) { "found" } else { "not found" };
        report.push_str(&format!("- `{path}` — {desc}: {status}\n"));
    }

    // Detect platforms
    report.push_str("\n## Platforms Detected\n\n");
    let platforms = [
        ("android", "Android"),
        ("ios", "iOS"),
        ("web", "Web"),
        ("linux", "Linux"),
        ("macos", "macOS"),
        ("windows", "Windows"),
    ];

    for (dir, name) in &platforms {
        let check_file = match *dir {
            "android" => "android/settings.gradle",
            "ios" => "ios/Podfile",
            "web" => "web/index.html",
            "linux" => "linux/CMakeLists.txt",
            "macos" => "macos/Podfile",
            "windows" => "windows/CMakeLists.txt",
            _ => "",
        };
        if !check_file.is_empty() && file_exists(worktree, check_file) {
            report.push_str(&format!("- {name}\n"));
        }
    }

    Ok(SlashCommandOutput {
        sections: vec![SlashCommandOutputSection {
            range: (0..report.len()).into(),
            label: "Flutter Doctor".to_string(),
        }],
        text: report,
    })
}

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
            "flutter-doctor" => {
                let worktree = worktree.ok_or("This command requires an open project")?;
                handle_doctor(worktree)
            }
            _ => Err(format!("Unknown command: {}", command.name)),
        }
    }
}

zed::register_extension!(FlutterExtension);
