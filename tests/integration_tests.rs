use assert_cmd::prelude::*; // Adds methods on commands
use chrono::Local; // Allows to fetch local dates to match against
use predicates::prelude::*; // Allows easier asssertions
use std::process::Command; // Runs programs

#[test]
fn test_that_invoice_add_arrives_in_ledger() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tabula")?;
    let assert = cmd.arg("invoices").arg("create").assert();

    assert
        .success()
        .stdout(predicate::str::contains(format!(
            "{} ! \"Invoice #{}\"",
            today(),
            "TBD"
        )))
        .stdout(predicate::str::contains(format!(
            "invoice_number: {}",
            "TBD"
        )));

    Ok(())
}

#[test]
fn test_that_invoice_build_generates_template_json() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tabula")?;
    let assert = cmd.arg("invoices").arg("build").assert();

    let expected = format!(
        r#"
        {{
            "to": "",
            "description": "",
            "created_on": "{}",
            "invoice_number": "TBD",
            "line_items": [
                {{
                  "description": "",
                  "amount": 0,
                  "unit_price": 65.00,
                  "vat_percentage": 21
                }}
            ]
        }}"#,
        today(),
    );
    let expected_json: serde_json::Value = serde_json::from_str(&expected)?;

    let binding = assert.success();
    let actual = std::str::from_utf8(&binding.get_output().stdout)?;
    let actual_json: serde_json::Value = serde_json::from_str(actual)?;

    assert_eq!(expected_json, actual_json);

    Ok(())
}

fn today() -> String {
    Local::now().format("%F").to_string()
}
