use super::*;

#[test]
fn test_ledstate_new() {
    assert_eq!(LedState::new(),
        LedState {
            color1: Some(ColorHsv::new(0.0, 0.0, 0.0)),
            color2: Some(ColorHsv::new(0.0, 0.0, 0.0)),
            color3: Some(ColorHsv::new(0.0, 0.0, 0.0)),
            pattern: Some(0),
        }
    );
}
