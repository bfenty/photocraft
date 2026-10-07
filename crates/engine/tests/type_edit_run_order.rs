//! Regression tests for issue #714: `type.edit` with a reversed `runs` range
//! (`end < start`) must not panic or wrap into a huge run length. Following the
//! in-file precedent (`range_param` swaps, `replace` uses `b.max(a)`), a
//! reversed run is normalised to the same ordered range.
use photocraft_engine::Session;
use serde_json::json;

fn setup() -> (Session, u64) {
    let mut s = Session::new();
    s.execute("file.new", json!({"width": 80, "height": 80})).unwrap();
    let r = s.execute("type.create", json!({"x": 0, "y": 40, "text": "Hello world", "size": 20})).unwrap();
    let id = r["layer"].as_u64().or_else(|| r["id"].as_u64()).unwrap_or_else(|| panic!("layer id in {r}"));
    (s, id)
}

fn faux_bold_chars(s: &mut Session, id: u64) -> Vec<bool> {
    let v = s.execute("type.info", json!({"layer": id})).unwrap_or_else(|e| panic!("type.info: {e:?}"));
    // Fall back: if type.info doesn't expose runs, the tests below rely on
    // execute() returning Err/Ok rather than panicking.
    let runs = v["runs"].as_array().cloned().unwrap_or_default();
    let text = "Hello world";
    let mut out = vec![false; text.len()];
    for r in runs {
        let (a, b) = (r["start"].as_u64().unwrap_or(0), r["end"].as_u64().unwrap_or(0));
        let bold = r["style"]["fauxBold"].as_bool().unwrap_or(false);
        if bold {
            for c in out.iter_mut().take(b as usize).skip(a as usize) {
                *c = true;
            }
        }
    }
    out
}

#[test]
fn reversed_run_range_does_not_panic() {
    let (mut s, id) = setup();
    // The issue's repro: end < start. Must be a graceful Err or a normalised
    // Ok — never the catch_unwind generic internal error.
    let r = s.execute("type.edit", json!({"layer": id, "runs": [{"start": 5, "end": 2, "fauxBold": true}]}));
    match &r {
        Ok(_) => {}
        Err(e) => {
            let msg = format!("{e:?}");
            assert!(!msg.contains("internal error"), "reversed run must not surface as internal error: {msg}");
        }
    }
}

#[test]
fn reversed_run_matches_ordered_run() {
    let (mut s, id) = setup();
    s.execute("type.edit", json!({"layer": id, "runs": [{"start": 5, "end": 2, "fauxBold": true}]})).ok();
    let reversed = faux_bold_chars(&mut s, id);
    let (mut s2, id2) = setup();
    s2.execute("type.edit", json!({"layer": id2, "runs": [{"start": 2, "end": 5, "fauxBold": true}]})).unwrap();
    let ordered = faux_bold_chars(&mut s2, id2);
    assert_eq!(reversed, ordered, "reversed run must behave like the swapped range (or both empty)");
}
