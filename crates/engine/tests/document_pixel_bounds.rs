//! Regression tests for issue #702: `document.pixel` must not panic on
//! out-of-canvas coordinates — the composite outside the canvas is transparent.
use photocraft_engine::Session;
use serde_json::json;

fn session() -> Session {
    let mut s = Session::new();
    s.execute("file.new", json!({"width": 24, "height": 16, "background": "#ff0000"})).unwrap();
    s
}

#[test]
fn document_pixel_out_of_canvas_is_transparent_not_panic() {
    let mut s = session();
    // The issue's repro: i32::MAX made Rect::from_xywh saturate to an empty
    // rect and the old code indexed [0] on the empty buffer (caught panic ->
    // generic internal error). Must now be a graceful transparent pixel.
    for (x, y) in [(2147483647i64, 2147483647i64), (-1, 0), (0, -1), (24, 0), (0, 16), (24, 16)] {
        let r = s.execute("document.pixel", json!({"x": x, "y": y}));
        let v = r.unwrap_or_else(|e| panic!("document.pixel({x},{y}) errored: {e:?}"));
        let px = v.as_array().unwrap_or_else(|| panic!("no px array in {v}"));
        let alpha = px[3].as_f64().unwrap();
        assert_eq!(alpha, 0.0, "composite outside the canvas must be transparent at ({x},{y}), got {px:?}");
    }
}

#[test]
fn document_pixel_in_canvas_still_reads_composite() {
    let mut s = session();
    let v = s.execute("document.pixel", json!({"x": 5, "y": 5})).unwrap();
    let px = v.as_array().unwrap();
    // Solid red background: r=1, a=1.
    assert_eq!(px[0].as_f64().unwrap(), 1.0, "red channel: {px:?}");
    assert_eq!(px[3].as_f64().unwrap(), 1.0, "alpha: {px:?}");
}
