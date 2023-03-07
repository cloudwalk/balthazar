use balthazar::{throw, Result};

#[test]
// Verify if our exported Result can be used correctly
fn test_export_eyre_result() -> Result<()> {
    Ok(())
}

#[test]
fn test_export_eyre_macro() -> Result<()> {
    let _ = throw!("test");

    Ok(())
}
