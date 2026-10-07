//! Regression tests for issue #497: `type.edit` with `name` must record ONE history step,
//! so a single `edit.undo` reverts the whole command (rename and text edit together).
use photocraft_engine::Session;
use serde_json::json;

fn setup() -> Session {
    let mut s = Session::new();
    s.execute("file.new", json!({"width": 192, "height": 128, "background": "#808080"})).unwrap();
    s.execute("type.create", json!({"x": 8, "y": 40, "text": "Sweep", "size": 18, "color": "#ff3366"})).unwrap();
    s
}

/// History depth and the type layer's (name, bounds) after each step.
fn state(s: &mut Session) -> (usize, String, [i64; 4]) {
    let st = s.active().unwrap();
    let hist = st.history.past_len();
    let layer = st.doc.layers.iter().rev().find(|l| matches!(l.content, photocraft_engine::doc::LayerContent::Text(_))).expect("type layer");
    let b = layer.surface().map(|c| {
        let r = c.content_bounds();
        [r.x0 as i64, r.y0 as i64, r.x1 as i64, r.y1 as i64]
    });
    (hist, layer.name.clone(), b.unwrap_or([0; 4]))
}

#[test]
fn type_edit_with_name_undoes_as_one_step() {
    let mut s = setup();
    let before = state(&mut s);
    let layer_id = {
        let st = s.active().unwrap();
        st.doc.layers.iter().rev().find(|l| matches!(l.content, photocraft_engine::doc::LayerContent::Text(_))).unwrap().id
    };

    s.execute("type.edit", json!({"layer": layer_id, "text": "sweep", "box": [16, 16, 64, 48], "point": [48, 40], "antialias": "none", "name": "sweep"}))
        .unwrap();
    let after = state(&mut s);
    assert_eq!(after.0, before.0 + 1, "type.edit with name must add exactly ONE history step (issue #497)");
    assert_eq!(after.1, "sweep", "rename applied");

    // One undo must revert the whole command: name and bounds back to the post-create state.
    s.execute("edit.undo", json!({})).unwrap();
    let undone = state(&mut s);
    assert_eq!(undone.0, before.0, "one undo must return to the pre-edit history depth");
    assert_eq!(undone.1, before.1, "one undo must restore the layer name");
    assert_eq!(undone.2, before.2, "one undo must restore the layer bounds");
}

#[test]
fn type_edit_without_name_still_one_step() {
    // Control from the issue: no `name` key, one step, one undo restores everything.
    let mut s = setup();
    let before = state(&mut s);
    let layer_id = {
        let st = s.active().unwrap();
        st.doc.layers.iter().rev().find(|l| matches!(l.content, photocraft_engine::doc::LayerContent::Text(_))).unwrap().id
    };
    s.execute("type.edit", json!({"layer": layer_id, "text": "sweep", "box": [16, 16, 64, 48], "antialias": "none"})).unwrap();
    assert_eq!(state(&mut s).0, before.0 + 1);
    s.execute("edit.undo", json!({})).unwrap();
    let undone = state(&mut s);
    assert_eq!((undone.1, undone.2), (before.1, before.2));
}

#[test]
fn explicit_rename_layer_still_records_its_own_step() {
    // layer.renameLayer keeps its own single step (unchanged behaviour).
    let mut s = setup();
    let before = state(&mut s);
    let layer_id = {
        let st = s.active().unwrap();
        st.doc.layers.iter().rev().find(|l| matches!(l.content, photocraft_engine::doc::LayerContent::Text(_))).unwrap().id
    };
    s.execute("layer.renameLayer", json!({"layer": layer_id, "name": "renamed"})).unwrap();
    assert_eq!(state(&mut s).0, before.0 + 1);
    s.execute("edit.undo", json!({})).unwrap();
    assert_eq!(state(&mut s).1, before.1);
}
