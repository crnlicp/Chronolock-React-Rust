#[ic_cdk::query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

// Export Candid interface
ic_cdk::export_candid!();
