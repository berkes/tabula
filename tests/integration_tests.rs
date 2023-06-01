use assert_cmd::prelude::*; // Adds methods on commands
use assert_fs::prelude::*; // Adds filesystem fixtures and assertions
use chrono::Local; // Allows to fetch local dates to match against
use predicates::prelude::*; // Allows easier asssertions
use std::process::Command; // Runs programs

#[test]
fn test_that_invoice_add_arrives_in_ledger() -> Result<(), Box<dyn std::error::Error>> {
    let tmpdir = assert_fs::TempDir::new().unwrap();
    let ledger_file = tmpdir.child("ledger.beancount");
    ledger_file.touch().unwrap();

    let mut cmd = Command::cargo_bin("tabula")?;
    let assert = cmd
        .arg("invoices")
        .arg("create")
        .arg(ledger_file.to_str().unwrap())
        .assert();

    assert
        .success()
        .stdout(predicate::str::contains("Invoice #1 created"));

    let today = Local::now().format("%F");
    ledger_file.assert(predicate::str::contains(format!(
        "{} ! \"Invoice #{}",
        today, 1
    )));

    Ok(())
}
