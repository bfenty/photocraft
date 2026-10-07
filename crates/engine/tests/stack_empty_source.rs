//! Regression tests for issue #717: Stack Mode over a smart object whose
//! contents have no visible layers must return a clear error, not panic.
//! On main the empty frame list reaches Surface::write_region with a 0-length
//! buffer over the canvas rect; the dispatch guard turns the panic into a
//! generic internal error, so the test asserts the message is the specific
//! one rather than the catch-all.
use photocraft_doc::{LayerContent, LayerId};
use photocraft_engine::Session;
use serde_json::json;

fn smart_id(s: &Session) -> u64 {
    s.active()
        .unwrap()
        .doc
        .walk()
        .into_iter()
        .find_map(|(_, _, l)| match &l.content {
            LayerContent::Smart(_) => Some(l.id.0),
            _ => None,
        })
        .unwrap()
}

fn setup() -> Session {
    let mut s = Session::new();
    s.execute("file.new", json!({"width": 24, "height": 16})).unwrap();
    s.execute("layer.new.layer", json!({})).unwrap();
    s.execute("select.all", json!({})).unwrap();
    s.execute("edit.fill", json!({"color": "#808080"})).unwrap();
    s.execute("select.deselect", json!({})).unwrap();
    s.execute("layer.smartObjects.convertToSmartObject", json!({})).unwrap();
    s
}

#[test]
fn stack_mode_errors_when_contents_have_no_visible_layers() {
    let mut s = setup();
    s.execute("layer.smartObjects.editContents", json!({})).unwrap();
    let inner = s.active().unwrap().doc.walk()[0].2.id.0;
    s.execute("layer.hideLayers", json!({"layer": inner})).unwrap();
    s.execute("layer.smartObjects.saveContents", json!({})).unwrap();
    s.execute("file.close", json!({})).unwrap();
    s.select_layer(LayerId(smart_id(&s))).unwrap();
    let res = s.execute("layer.smartObjects.stackMode.mean", json!({}));
    let err = match &res {
        Err(e) => format!("{e:?}"),
        Ok(v) => panic!("stack mode over an all-hidden source must error, got {v:?}"),
    };
    assert!(err.contains("visible"), "must be the specific no-visible-layers error, not a catch-all: {err}");
}

#[test]
fn stack_mode_still_works_with_visible_layers() {
    let mut s = setup();
    s.select_layer(LayerId(smart_id(&s))).unwrap();
    let res = s.execute("layer.smartObjects.stackMode.mean", json!({}));
    assert!(res.is_ok(), "stack mode over a visible source must succeed: {res:?}");
}
