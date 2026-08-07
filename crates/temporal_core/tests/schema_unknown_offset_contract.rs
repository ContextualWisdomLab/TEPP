//! JSON Schema parity for RFC 3339's unknown-local-offset convention.

use serde_json::json;
use temporal_core::{EventTime, TemporalInterval};

#[test]
fn clock_and_interval_schemas_reject_negative_zero_offsets() {
    let clock_schema = EventTime::wire_json_schema();
    let interval_schema = TemporalInterval::<EventTime>::wire_json_schema();

    assert_eq!(
        clock_schema["properties"]["timestamp"]["not"]["pattern"],
        json!("-00:00$")
    );
    for boundary_name in ["lower", "upper"] {
        for variant_index in [1, 2] {
            assert_eq!(
                interval_schema["properties"][boundary_name]["oneOf"][variant_index]
                    ["properties"]["timestamp"]["not"]["pattern"],
                json!("-00:00$")
            );
        }
    }
}
