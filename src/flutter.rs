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

const TEMPLATE_TYPES: &[(&str, &str)] = &[
    ("stateless", "StatelessWidget"),
    ("stateful", "StatefulWidget"),
    ("provider", "ChangeNotifierProvider"),
    ("riverpod", "Riverpod ConsumerWidget"),
    ("bloc", "Cubit (BLoC pattern)"),
    ("freezed", "Freezed data class"),
    ("test", "Widget test"),
];

fn template_stateless(name: &str) -> String {
    format!(
        r#"import 'package:flutter/material.dart';

class {name} extends StatelessWidget {{
  const {name}({{super.key}});

  @override
  Widget build(BuildContext context) {{
    return const Placeholder();
  }}
}}"#
    )
}

fn template_stateful(name: &str) -> String {
    format!(
        r#"import 'package:flutter/material.dart';

class {name} extends StatefulWidget {{
  const {name}({{super.key}});

  @override
  State<{name}> createState() => _{name}State();
}}

class _{name}State extends State<{name}> {{
  @override
  Widget build(BuildContext context) {{
    return const Placeholder();
  }}
}}"#
    )
}

fn template_provider(name: &str) -> String {
    format!(
        r#"import 'package:flutter/foundation.dart';

class {name} extends ChangeNotifier {{
  // Add your state fields here

  // Example:
  // int _count = 0;
  // int get count => _count;
  //
  // void increment() {{
  //   _count++;
  //   notifyListeners();
  // }}
}}"#
    )
}

fn template_riverpod(name: &str) -> String {
    format!(
        r#"import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

class {name} extends ConsumerWidget {{
  const {name}({{super.key}});

  @override
  Widget build(BuildContext context, WidgetRef ref) {{
    return const Placeholder();
  }}
}}"#
    )
}

fn template_bloc(name: &str) -> String {
    format!(
        r#"import 'package:flutter_bloc/flutter_bloc.dart';

class {name}State {{
  const {name}State();
}}

class {name}Cubit extends Cubit<{name}State> {{
  {name}Cubit() : super(const {name}State());
}}"#
    )
}

fn template_freezed(name: &str) -> String {
    let name_snake = to_snake_case(name);
    format!(
        r#"import 'package:freezed_annotation/freezed_annotation.dart';

part '{name_snake}.freezed.dart';
part '{name_snake}.g.dart';

@freezed
class {name} with _${name} {{
  const factory {name}({{
    // Add your fields here
  }}) = _{name};

  factory {name}.fromJson(Map<String, dynamic> json) => _${name}FromJson(json);
}}"#
    )
}

fn template_test(name: &str) -> String {
    format!(
        r#"import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {{
  group('{name}', () {{
    testWidgets('renders correctly', (tester) async {{
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: Placeholder(), // Replace with your widget
          ),
        ),
      );

      // Add your assertions here
    }});
  }});
}}"#
    )
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap_or(c));
        } else {
            result.push(c);
        }
    }
    result
}

fn handle_new(args: &[String]) -> Result<SlashCommandOutput, String> {
    if args.is_empty() {
        let mut help = String::from("# /flutter-new\n\nUsage: `/flutter-new <type> [ClassName]`\n\nAvailable types:\n\n");
        for (key, desc) in TEMPLATE_TYPES {
            help.push_str(&format!("- **{key}** — {desc}\n"));
        }
        help.push_str("\nExample: `/flutter-new stateless MyWidget`");
        return Ok(SlashCommandOutput {
            sections: vec![SlashCommandOutputSection {
                range: (0..help.len()).into(),
                label: "Flutter New — Help".to_string(),
            }],
            text: help,
        });
    }

    let template_type = args[0].as_str();
    let class_name = if args.len() > 1 {
        args[1].clone()
    } else {
        "MyWidget".to_string()
    };

    let code = match template_type {
        "stateless" => template_stateless(&class_name),
        "stateful" => template_stateful(&class_name),
        "provider" => template_provider(&class_name),
        "riverpod" => template_riverpod(&class_name),
        "bloc" => template_bloc(&class_name),
        "freezed" => template_freezed(&class_name),
        "test" => template_test(&class_name),
        _ => {
            return Err(format!(
                "Unknown template type: '{template_type}'. Available: stateless, stateful, provider, riverpod, bloc, freezed, test"
            ))
        }
    };

    let text = format!("```dart\n{code}\n```");

    Ok(SlashCommandOutput {
        sections: vec![SlashCommandOutputSection {
            range: (0..text.len()).into(),
            label: format!("Flutter New: {template_type} — {class_name}"),
        }],
        text,
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
        command: SlashCommand,
        _args: Vec<String>,
    ) -> Result<Vec<SlashCommandArgumentCompletion>, String> {
        match command.name.as_str() {
            "flutter-new" => Ok(TEMPLATE_TYPES
                .iter()
                .map(|(key, desc)| SlashCommandArgumentCompletion {
                    label: format!("{key} — {desc}"),
                    new_text: key.to_string(),
                    run_command: false,
                })
                .collect()),
            _ => Ok(vec![]),
        }
    }

    fn run_slash_command(
        &self,
        command: SlashCommand,
        args: Vec<String>,
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
            "flutter-new" => handle_new(&args),
            _ => Err(format!("Unknown command: {}", command.name)),
        }
    }
}

zed::register_extension!(FlutterExtension);
