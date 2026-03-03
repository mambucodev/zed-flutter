use zed_extension_api::{self as zed};

struct FlutterExtension;

impl zed::Extension for FlutterExtension {
    fn new() -> Self {
        Self
    }
}

zed::register_extension!(FlutterExtension);
