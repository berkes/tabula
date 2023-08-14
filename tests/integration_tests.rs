use assert_cmd::Command;
use assert_json_diff::assert_json_eq;
// Run programs
use chrono::Local; // Allows to fetch local dates to match against
use predicates::prelude::*;
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize}; // Allows to deserialize JSON
use serde_json::json; // Allows easier asssertions
use std::{fs::File, io::Read}; // Runs programs

#[derive(Deserialize, Serialize, Debug)]
pub struct Invoice {
    date: String,
    due_date: String,
    narration: String,
    number: String,
    total: String,
}

#[test]
fn test_that_invoice_build_renders_text() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tabula")?;
    let out = cmd
        .args(&["--format", "txt"]) // --format txt is default, but we provide it explicitly here
        .arg("invoices")
        .arg("build")
        .unwrap()
        .stdout;

    let expected_output = format!(
        r#"Invoice: TBD
Date issued: {}
Due date: 
Income:Work: 1337 USD


"#,
        today()
    );

    assert_eq!(expected_output, String::from_utf8(out).unwrap());

    Ok(())
}

#[test]
fn test_that_invoice_build_renders_ledger() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tabula")?;
    let assert = cmd
        .args(&["--format", "beancount"])
        .arg("invoices")
        .arg("build")
        .assert();

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

    let out = cmd
        .args(&["--format", "json"])
        .arg("invoices")
        .arg("build")
        .unwrap()
        .stdout;

    let actual: serde_json::Value = serde_json::from_slice(&out).unwrap();

    let expected = json!(
        {
            "narration": "",
            "date": today(),
            "due_date": None::<String>,
            "number": "TBD",
            "total": "1337 USD",
            "line_items": []
        }
    );

    assert_json_eq!(expected, actual);

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
        .unwrap()
        .stdout;

    let expected_output = r#"+----------+------------+--------------+------------+
| Number   | Date       | Narration    | Due date   |
+----------+------------+--------------+------------+
| 2023-001 | 2023-06-01 | Invoice #1   |            |
+----------+------------+--------------+------------+
| 2023-002 | 2023-06-02 | Invoice #2   | 2023-07-02 |
+----------+------------+--------------+------------+
| TBD      | 2023-06-05 | Invoice #TBD |            |
+----------+------------+--------------+------------+

"#;
    assert_eq!(expected_output, String::from_utf8(assert).unwrap());
    Ok(())
}

#[test]
fn test_that_invoice_convert_converts_to_json() -> Result<(), Box<dyn std::error::Error>> {
    let mut file_content = String::new();
    File::open("./tests/fixtures/invoices.beancount")?.read_to_string(&mut file_content)?;

    let mut cmd = Command::cargo_bin("tabula")?;
    let out = cmd
        .args(&["--format", "json"])
        .arg("invoices")
        .arg("convert")
        .args(&["--invoice-number", "2023-002"])
        .write_stdin(file_content)
        .unwrap()
        .stdout;

    let expected = json!(
        {
            "date": "2023-06-02",
            "due_date": "2023-07-02",
            "narration": "Invoice #2",
            "number": "2023-002",
            "total": "1337 USD",
        }
    );

    let actual: Invoice = serde_json::from_slice(&out).unwrap();

    assert_json_eq!(expected, actual);

    Ok(())
}

#[test]
fn test_that_invoice_convert_converts_to_txt() -> Result<(), Box<dyn std::error::Error>> {
    // Open the fixture and read its contents into a String
    let mut file_content = String::new();
    File::open("./tests/fixtures/invoices.beancount")?.read_to_string(&mut file_content)?;

    let mut cmd = Command::cargo_bin("tabula")?;
    let assert = cmd
        .args(&["--format", "txt"])
        .arg("invoices")
        .arg("convert")
        .arg("--invoice-number")
        .arg("2023-002")
        .write_stdin(file_content);

    let expected_output = r#"Invoice: 2023-002
Date issued: 2023-06-02
Due date: 2023-07-02
Income:Work: 1337 USD

+------------------------+-----+------------+----------+
| Name                   | Qty | Unit price | Amount   |
+------------------------+-----+------------+----------+
| Work done on project X | 100 | 13.37 USD  | 1337 USD |
+------------------------+-----+------------+----------+

Invoice #2
"#;

    let actual = String::from_utf8(assert.unwrap().stdout).unwrap();
    assert_eq!(expected_output, actual);

    Ok(())
}

fn today() -> String {
    Local::now().format("%F").to_string()
}
