use assert_cmd::Command; // Run programs
use chrono::Local; // Allows to fetch local dates to match against
use predicates::prelude::*; // Allows easier asssertions
use std::{fs::File, io::Read}; // Runs programs

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

#[test]
fn test_that_invoice_list_shows_all_invoices() -> Result<(), Box<dyn std::error::Error>> {
    // Open the fxiture and read its contents into a String
    let mut file_content = String::new();
    File::open("./tests/fixtures/invoices.beancount")?.read_to_string(&mut file_content)?;

    let mut cmd = Command::cargo_bin("tabula")?;
    let assert = cmd
        .arg("invoices")
        .arg("list")
        .write_stdin(file_content)
        .assert();

    assert
        .success()
        .stdout(predicate::str::contains("2023-001 2023-06-01 Invoice #1"))
        .stdout(predicate::str::contains(
            "2023-002 2023-06-02 Invoice #2 due:2023-07-02",
        ))
        .stdout(predicate::str::contains("TBD 2023-06-05 Invoice #TBD"));

    Ok(())
}

fn today() -> String {
    Local::now().format("%F").to_string()
}
