//! Regression tests for issue #718: migratePresets appends groups to the
//! preset library, which persists with the preferences document, so it must
//! mark the preferences dirty (bump prefs rev) like every other preset-
//! mutating path. On main it bumps only the preset revision, so the shell's
//! persist loop (gated on prefs.rev()) never writes the migration.
use photocraft_engine::Session;
use serde_json::json;

#[test]
fn migrate_presets_marks_preferences_dirty() {
    let dir = std::env::temp_dir().join(format!("photocraft-migrate-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("merged-presets.json");
    std::fs::write(&file, r#"{"presets": {"gradients": [{"name": "My Imported Set", "items": []}]}}"#).unwrap();

    let mut s = Session::new();
    let before = s.prefs.rev();
    let out = s.execute("edit.presets.migratePresets", json!({"path": file.to_str().unwrap()})).unwrap();
    assert_eq!(out["migrated"].as_u64(), Some(1), "the group must migrate: {out}");
    assert!(s.prefs.rev() > before, "a migration that changes the library must bump the preferences revision");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn migrate_noop_leaves_preferences_untouched() {
    let dir = std::env::temp_dir().join(format!("photocraft-migrate-noop-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("empty-presets.json");
    std::fs::write(&file, r#"{"presets": {}}"#).unwrap();

    let mut s = Session::new();
    let before = s.prefs.rev();
    let out = s.execute("edit.presets.migratePresets", json!({"path": file.to_str().unwrap()})).unwrap();
    assert_eq!(out["migrated"].as_u64(), Some(0));
    assert_eq!(s.prefs.rev(), before, "a migration that adds nothing must not dirty the preferences");
    std::fs::remove_dir_all(&dir).ok();
}
