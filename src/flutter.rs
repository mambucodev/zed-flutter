use zed_extension_api::{
    self as zed,
    http_client::{HttpMethod, HttpRequest, RedirectPolicy},
    SlashCommand, SlashCommandArgumentCompletion, SlashCommandOutput, SlashCommandOutputSection,
    Worktree,
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

fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let in_script = false;
    let mut last_was_newline = false;

    for c in html.chars() {
        if c == '<' {
            in_tag = true;
            continue;
        }
        if in_tag {
            if c == '>' {
                in_tag = false;
            }
            continue;
        }
        if in_script {
            continue;
        }
        if c == '\n' || c == '\r' {
            if !last_was_newline {
                result.push('\n');
                last_was_newline = true;
            }
        } else if c.is_whitespace() {
            if !result.ends_with(' ') && !last_was_newline {
                result.push(' ');
            }
        } else {
            result.push(c);
            last_was_newline = false;
        }
    }
    let _ = in_script; // suppress unused warning
    result.trim().to_string()
}

fn try_fetch_docs(class_name: &str) -> Option<String> {
    let libraries = ["widgets", "material", "cupertino", "painting", "rendering", "services", "animation"];

    for lib in &libraries {
        let url = format!(
            "https://api.flutter.dev/flutter/{lib}/{class_name}-class.html"
        );
        let request = HttpRequest {
            method: HttpMethod::Get,
            url: url.clone(),
            headers: vec![("User-Agent".to_string(), "zed-flutter-extension/0.1.0".to_string())],
            body: None,
            redirect_policy: RedirectPolicy::FollowAll,
        };

        if let Ok(response) = request.fetch() {
            let body = String::from_utf8_lossy(&response.body);
            // Check if we got a real page (not a 404 page)
            if body.contains(&format!("{class_name} class")) || body.contains(&format!("<title>{class_name}")) {
                // Extract the description section
                let stripped = strip_html_tags(&body);

                // Try to extract a useful excerpt (first ~2000 chars after class name mention)
                if let Some(pos) = stripped.find(&format!("{class_name} class")) {
                    let excerpt_start = pos;
                    let excerpt_end = (excerpt_start + 2000).min(stripped.len());
                    let excerpt = &stripped[excerpt_start..excerpt_end];

                    return Some(format!(
                        "# {class_name}\n\n**Source**: [api.flutter.dev]({url})\n**Library**: flutter/{lib}\n\n---\n\n{excerpt}\n\n---\n*Truncated. See full docs at the link above.*"
                    ));
                }
            }
        }
    }
    None
}

const COMMON_WIDGETS: &[(&str, &str, &str)] = &[
    ("Container", "widgets", "A convenience widget that combines common painting, positioning, and sizing widgets."),
    ("Text", "widgets", "A run of text with a single style."),
    ("Column", "widgets", "A widget that displays its children in a vertical array."),
    ("Row", "widgets", "A widget that displays its children in a horizontal array."),
    ("Stack", "widgets", "A widget that positions its children relative to the edges of its box."),
    ("ListView", "widgets", "A scrollable list of widgets arranged linearly."),
    ("GridView", "widgets", "A scrollable 2D array of widgets."),
    ("Scaffold", "material", "Implements the basic Material Design visual layout structure."),
    ("AppBar", "material", "A Material Design app bar with toolbar actions."),
    ("MaterialApp", "material", "A convenience widget that wraps widgets commonly required for Material Design apps."),
    ("ElevatedButton", "material", "A Material Design elevated button."),
    ("TextButton", "material", "A Material Design text button."),
    ("IconButton", "material", "A Material Design icon button."),
    ("TextField", "material", "A Material Design text field for user input."),
    ("Card", "material", "A Material Design card panel with rounded corners and elevation."),
    ("Drawer", "material", "A Material Design panel that slides in from the edge of a Scaffold."),
    ("BottomNavigationBar", "material", "A Material Design bottom navigation bar."),
    ("FloatingActionButton", "material", "A Material Design floating action button."),
    ("AlertDialog", "material", "A Material Design alert dialog."),
    ("SnackBar", "material", "A Material Design snack bar that slides up from the bottom."),
    ("TabBar", "material", "A Material Design tab bar widget."),
    ("Padding", "widgets", "A widget that insets its child by the given padding."),
    ("Center", "widgets", "A widget that centers its child."),
    ("Expanded", "widgets", "A widget that expands a child of a Row, Column, or Flex."),
    ("SizedBox", "widgets", "A box with a specified size."),
    ("GestureDetector", "widgets", "A widget that detects gestures."),
    ("SingleChildScrollView", "widgets", "A box in which a single widget can be scrolled."),
    ("FutureBuilder", "widgets", "A widget that builds itself based on the latest snapshot of a Future."),
    ("StreamBuilder", "widgets", "A widget that builds itself based on the latest snapshot of a Stream."),
    ("Navigator", "widgets", "A widget that manages a set of child widgets with a stack discipline."),
    ("Form", "widgets", "An optional container for grouping form fields."),
    ("Image", "widgets", "A widget that displays an image."),
    ("AnimatedContainer", "widgets", "An animated version of Container that gradually changes its values."),
    ("Hero", "widgets", "A widget that marks its child as a candidate for hero animations."),
    ("Opacity", "widgets", "A widget that makes its child partially transparent."),
    ("Wrap", "widgets", "A widget that displays its children in multiple runs."),
    ("CupertinoApp", "cupertino", "A convenience widget that wraps widgets commonly required for a Cupertino Design app."),
    ("CupertinoNavigationBar", "cupertino", "An iOS-style navigation bar."),
    ("CupertinoButton", "cupertino", "An iOS-style button."),
];

fn handle_docs(args: &[String]) -> Result<SlashCommandOutput, String> {
    if args.is_empty() {
        return Err("Usage: /flutter-docs <ClassName>. Example: /flutter-docs Container".to_string());
    }

    let class_name = &args[0];

    // First check our built-in dictionary
    if let Some((_, lib, desc)) = COMMON_WIDGETS.iter().find(|(name, _, _)| name == class_name) {
        let url = format!("https://api.flutter.dev/flutter/{lib}/{class_name}-class.html");

        // Try to fetch detailed docs from the web
        let details = try_fetch_docs(class_name);

        let text = if let Some(detailed) = details {
            detailed
        } else {
            format!(
                "# {class_name}\n\n**Library**: flutter/{lib}\n**Docs**: [{class_name} class]({url})\n\n{desc}"
            )
        };

        return Ok(SlashCommandOutput {
            sections: vec![SlashCommandOutputSection {
                range: (0..text.len()).into(),
                label: format!("Flutter Docs: {class_name}"),
            }],
            text,
        });
    }

    // Not in dictionary — try fetching from the web
    if let Some(text) = try_fetch_docs(class_name) {
        return Ok(SlashCommandOutput {
            sections: vec![SlashCommandOutputSection {
                range: (0..text.len()).into(),
                label: format!("Flutter Docs: {class_name}"),
            }],
            text,
        });
    }

    // Fallback: provide search links
    let text = format!(
        "# {class_name}\n\nNo documentation found in common Flutter libraries.\n\n**Try these links:**\n- [Search api.flutter.dev](https://api.flutter.dev/flutter/search.html?q={class_name})\n- [Search pub.dev](https://pub.dev/packages?q={class_name})\n- [Search dart.dev](https://api.dart.dev/stable/dart-core/{class_name}-class.html)"
    );

    Ok(SlashCommandOutput {
        sections: vec![SlashCommandOutputSection {
            range: (0..text.len()).into(),
            label: format!("Flutter Docs: {class_name}"),
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
            "flutter-docs" => Ok(COMMON_WIDGETS
                .iter()
                .map(|(name, lib, desc)| SlashCommandArgumentCompletion {
                    label: format!("{name} ({lib}) — {desc}"),
                    new_text: name.to_string(),
                    run_command: true,
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
            "flutter-docs" => handle_docs(&args),
            _ => Err(format!("Unknown command: {}", command.name)),
        }
    }
}

zed::register_extension!(FlutterExtension);
