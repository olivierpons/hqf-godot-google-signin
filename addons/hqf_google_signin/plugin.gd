## Registers [HqfGoogleSignInExportPlugin] while the addon is enabled.
@tool
extends EditorPlugin

# ── Private variables ──────────────────────────────────────────
var _export_plugin: HqfGoogleSignInExportPlugin = null

# ── Built-in virtual methods ───────────────────────────────────
func _enter_tree() -> void:
	_export_plugin = HqfGoogleSignInExportPlugin.new()
	add_export_plugin(_export_plugin)

func _exit_tree() -> void:
	remove_export_plugin(_export_plugin)
	_export_plugin = null
