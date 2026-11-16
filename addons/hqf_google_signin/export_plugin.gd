## Adds the Google libraries the sign-in needs to Android Gradle builds.
##
## The list comes from the extension itself
## ([code]HqfGoogleSignIn.android_dependencies()[/code]), so it always matches the
## library that ships with the game.
@tool
class_name HqfGoogleSignInExportPlugin
extends EditorExportPlugin

# ── Constants ──────────────────────────────────────────────────
const NODE_CLASS: StringName = &"HqfGoogleSignIn"
const DEPENDENCIES_METHOD: StringName = &"android_dependencies"

# ── Built-in virtual methods ───────────────────────────────────
func _get_name() -> String:
	return "HqfGoogleSignIn"

func _supports_platform(platform: EditorExportPlatform) -> bool:
	return platform is EditorExportPlatformAndroid

func _get_android_dependencies(
	_platform: EditorExportPlatform, _debug: bool
) -> PackedStringArray:
	if not ClassDB.class_exists(NODE_CLASS):
		push_error(
			"HqfGoogleSignIn: the extension library is not loaded in this editor, "
			+ "the Android export lacks the Google libraries"
		)
		return PackedStringArray()
	return ClassDB.class_call_static(NODE_CLASS, DEPENDENCIES_METHOD)
