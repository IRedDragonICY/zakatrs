use zakat::types::{CalculationStep, CalculationTrace};

#[test]
fn test_trace_serialization() {
    let trace = CalculationTrace(vec![
        CalculationStep::initial("step-initial", "Initial Step", 100),
        CalculationStep::add("step-added", "Added Value", 50),
        CalculationStep::rate("step-rate", "Rate Applied", 0.025),
    ]);

    let json = serde_json::to_string(&trace).unwrap();
    
    // Verify that the Operation enum variants are serialized as strings.
    
    println!("Serialized JSON: {}", json);
    
    assert!(json.contains(r#""operation":"Initial""#));
    assert!(json.contains(r#""operation":"Add""#));
    assert!(json.contains(r#""operation":"Rate""#));
    assert!(json.contains(r#""amount":"100""#));
}
