//! Editor classes that add the Google libraries to the Android Gradle build.

use godot::classes::{
    EditorExportPlatform, EditorExportPlugin, EditorPlugin, IEditorExportPlugin, IEditorPlugin,
};
use godot::prelude::*;

/// Maven artifacts the Android build needs, `group:artifact:version`.
pub const ANDROID_DEPENDENCIES: [&str; 3] = [
    "androidx.credentials:credentials:1.6.0",
    "androidx.credentials:credentials-play-services-auth:1.6.0",
    "com.google.android.libraries.identity.googleid:googleid:1.2.1",
];

/// Adds [`ANDROID_DEPENDENCIES`] to Android exports that use the Gradle build.
#[derive(GodotClass)]
#[class(tool, init, base = EditorExportPlugin)]
pub struct HqfGoogleSignInExportPlugin {
    owner: Base<EditorExportPlugin>,
}

#[godot_api]
impl IEditorExportPlugin for HqfGoogleSignInExportPlugin {
    fn customize_resource(
        &mut self,
        _resource: Gd<Resource>,
        _path: GString,
    ) -> Option<Gd<Resource>> {
        None
    }

    fn customize_scene(&mut self, _scene: Gd<Node>, _path: GString) -> Option<Gd<Node>> {
        None
    }

    fn get_customization_configuration_hash(&self) -> u64 {
        0
    }

    fn get_name(&self) -> GString {
        "HqfGoogleSignIn".into()
    }

    fn supports_platform(&self, platform: Option<Gd<EditorExportPlatform>>) -> bool {
        platform.is_some_and(|platform| platform.get_os_name() == "Android")
    }

    fn get_android_dependencies(
        &self,
        _platform: Option<Gd<EditorExportPlatform>>,
        _debug: bool,
    ) -> PackedStringArray {
        ANDROID_DEPENDENCIES
            .iter()
            .map(|dep| GString::from(*dep))
            .collect()
    }
}

/// Registers [`HqfGoogleSignInExportPlugin`] in the editor.
#[derive(GodotClass)]
#[class(tool, init, base = EditorPlugin)]
pub struct HqfGoogleSignInEditorPlugin {
    export_plugin: Option<Gd<HqfGoogleSignInExportPlugin>>,
    owner: Base<EditorPlugin>,
}

#[godot_api]
impl IEditorPlugin for HqfGoogleSignInEditorPlugin {
    fn enter_tree(&mut self) {
        let plugin = HqfGoogleSignInExportPlugin::new_gd();
        self.base_mut().add_export_plugin(&plugin);
        self.export_plugin = Some(plugin);
    }

    fn exit_tree(&mut self) {
        if let Some(plugin) = self.export_plugin.take() {
            self.base_mut().remove_export_plugin(&plugin);
        }
    }
}
