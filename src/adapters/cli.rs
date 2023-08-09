use crate::{
    adapters::InputAdapter,
    commands::{BuildInvoiceCommand, Command, FindInvoiceCommand, ListInvoicesCommand},
    domain::invoice::{Date, Invoice, InvoiceList, InvoiceNumber},
};
use beancount_core::{
    metadata::{Meta, MetaValue},
    Account, AccountType, Flag, IncompleteAmount, Ledger, Posting, Transaction,
};
use beancount_render::render;
use core::fmt;
use prettytable::{Cell, Row, Table};

use std::{borrow::Cow, fmt::Display};
use std::{error::Error, io::Read};

use self::arguments::OutputFormat;

use super::ledger_storage::StdinLedgerStorage;

pub mod arguments;

#[derive(Default)]
pub struct CliAdapter {
    response: String,
}

impl CliAdapter {
    pub fn set_response(&mut self, response: String) {
        self.response = response;
    }
}

fn ledger_storage_with_stdin() -> StdinLedgerStorage {
    let mut ledger = String::new();
    std::io::stdin().read_to_string(&mut ledger).unwrap();
    StdinLedgerStorage::new(ledger)
}

fn ledger_storage_without_stdin() -> StdinLedgerStorage {
    StdinLedgerStorage::new("".to_string())
}

impl InputAdapter for CliAdapter {
    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let global_args = arguments::parse();

        let command_res = match arguments::parse().command {
            arguments::Namespace::Invoices(invoices_args) => match invoices_args.command {
                arguments::InvoiceActions::Build => {
                    BuildInvoiceCommand::new(ledger_storage_without_stdin()).execute()?
                }
                arguments::InvoiceActions::List => {
                    ListInvoicesCommand::new(ledger_storage_with_stdin()).execute()?
                }
                arguments::InvoiceActions::Convert(args) => {
                    FindInvoiceCommand::new(ledger_storage_with_stdin())
                        .with_invoice_number(args.invoice_number)
                        .execute()?
                }
            },
        };

        let output = match &global_args.format {
            OutputFormat::Json => command_res.as_json(),
            OutputFormat::Txt => command_res.as_txt(),
            OutputFormat::Beancount => command_res.as_beancount(),
        };

        self.set_response(output.to_string());
        Ok(())
    }

    fn get_response(&self) -> String {
        self.response.clone()
    }
}

pub trait Output {
    fn as_json(&self) -> String;
    fn as_txt(&self) -> String;
    fn as_beancount(&self) -> String;
}

impl Output for Invoice {
    fn as_json(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }

    fn as_txt(&self) -> String {
        // TODO: Move Display body here instead of calling it implicitly
        format!("{}", self)
    }

    // TODO: unhardcode, aby using actual invoice attributes here, as well as some
    // additional attributes to be introucded on the Invoice struct. And maybe a few
    // config and/or ENV vars
    fn as_beancount(&self) -> String {
        let today = chrono::Local::now().date_naive();

        let accts_recievable = Account::builder()
            .ty(AccountType::Assets)
            .parts(vec![Cow::Borrowed("AccountsReceivable")])
            .build();

        let work = Account::builder()
            .ty(beancount_core::AccountType::Income)
            .parts(vec![Cow::Borrowed("Work")])
            .build();

        let amt = IncompleteAmount::builder()
            .num(Some(1337.into()))
            .currency(Some(Cow::Borrowed("USD")))
            .build();

        let postings = vec![
            Posting::builder()
                .account(accts_recievable)
                .units(amt.clone())
                .build(),
            Posting::builder().account(work).units(amt).build(),
        ];

        let invoice_number = "TBD";

        let meta = Meta::from([(
            Cow::Borrowed("invoice_number"),
            MetaValue::Text(invoice_number.into()),
        )]);

        let tx = Transaction::builder()
            .flag(Flag::Warning)
            .meta(meta)
            .date(today.into())
            .narration(format!("Invoice #{}", invoice_number).into())
            .postings(postings)
            .build();

        let ledger = Ledger {
            directives: vec![beancount_core::Directive::Transaction(tx)],
        };

        let mut w = Vec::new();
        render(&mut w, &ledger).unwrap();

        String::from_utf8(w).unwrap()
    }
}

impl Output for InvoiceList {
    fn as_json(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }

    fn as_txt(&self) -> String {
        let mut table = Table::new();
        table.add_row(Row::new(vec![
            Cell::new("Number"),
            Cell::new("Date"),
            Cell::new("Narration"),
            Cell::new("Due date"),
        ]));
        for invoice in &self.invoices {
            let due_date = invoice
                .due_date
                .clone()
                .map(|d| d.to_string())
                .unwrap_or("".to_string());

            table.add_row(Row::new(vec![
                Cell::new(&invoice.number.to_string()),
                Cell::new(&invoice.date.to_string()),
                Cell::new(&invoice.narration),
                Cell::new(&due_date),
            ]));
        }
        table.to_string()
    }

    fn as_beancount(&self) -> String {
        unimplemented!()
    }
}

impl Display for Invoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let owned_date = self.date.clone();
        let owned_due_date = self.due_date.as_ref().unwrap_or(&self.date);
        let owned_number = self.number.0.clone();

        let mut renderer = KeyValueRenderer::new();
        renderer.add_field("Invoice", &owned_number);
        renderer.add_field("Date issued", &owned_date);
        renderer.add_field("Due date", &owned_due_date);
        renderer.add_field("Income:Work", &self.total);
        let meta = renderer.to_string();

        // If there are line-items, render them to a table with prettytable
        if !self.line_items.is_empty() {
            let mut table = Table::new();
            table.add_row(Row::new(vec![
                Cell::new("Name"),
                Cell::new("Qty"),
                Cell::new("Unit price"),
                Cell::new("Amount"),
            ]));
            for line_item in &self.line_items {
                table.add_row(Row::new(vec![
                    Cell::new(&line_item.description),
                    Cell::new(&line_item.quantity),
                    Cell::new(&line_item.unit_price),
                    Cell::new(&line_item.total),
                ]));
            }
            write!(f, "{}\n{}\n{}", meta, table, self.narration)?;
            Ok(())
        } else {
            write!(f, "{}\n{}", meta, self.narration)?;
            Ok(())
        }
    }
}

impl Display for InvoiceNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

struct KeyValueRenderer<'a> {
    fields: Vec<(&'static str, &'a dyn fmt::Display)>,
}

impl<'a> KeyValueRenderer<'a> {
    fn new() -> Self {
        KeyValueRenderer { fields: Vec::new() }
    }

    fn add_field(&mut self, key: &'static str, value: &'a dyn fmt::Display) {
        self.fields.push((key, value));
    }

    fn render(&self) -> String {
        let mut output = String::new();
        for (key, value) in &self.fields {
            output.push_str(&format!("{}: {}\n", key, value));
        }
        output
    }
}

impl<'a> Display for KeyValueRenderer<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render())
    }
}

#[cfg(test)]
mod tests {
    use assert_json_diff::assert_json_eq;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::domain::invoice::{InvoiceNumber, LineItem};

    use super::*;

    #[test]
    fn test_as_json_has_all_fields() {
        let invoice = Invoice {
            date: "2023-06-02".into(),
            due_date: Some("2023-07-02".into()),
            narration: "Invoice #2".to_string(),
            number: InvoiceNumber("2023-002".to_string()),
            total: "1337 USD".to_string(),
            line_items: vec![],
        };

        let actual = serde_json::from_str::<serde_json::Value>(&invoice.as_json()).unwrap();

        let expected = json!(
            {
                "date": "2023-06-02",
                "due_date": "2023-07-02",
                "narration": "Invoice #2",
                "number": "2023-002",
                "total": "1337 USD",
                "line_items": []
            }
        );

        assert_json_eq!(expected, actual);
    }

    #[test]
    fn test_as_txt_has_all_fields() {
        let invoice = Invoice {
            date: "2023-06-02".into(),
            due_date: Some("2023-07-02".into()),
            narration: "Invoice #2".to_string(),
            number: InvoiceNumber("2023-002".to_string()),
            total: "1337 USD".to_string(),
            line_items: vec![],
        };

        let actual = invoice.as_txt();

        let expected = r#"Invoice: 2023-002
Date issued: 2023-06-02
Due date: 2023-07-02
Income:Work: 1337 USD

Invoice #2"#;
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_as_txt_has_line_item_table() {
        let mut invoice = Invoice {
            date: "2023-06-02".into(),
            due_date: Some("2023-07-02".into()),
            narration: "Invoice #2".to_string(),
            number: InvoiceNumber("2023-002".to_string()),
            total: "1337 USD".to_string(),
            line_items: vec![],
        };

        invoice.line_items.push(LineItem {
            description: "Uren".to_string(),
            quantity: "65".to_string(),
            unit_price: "65".to_string(),
            total: "1337 USD".to_string(),
        });

        let expected = r#"Invoice: 2023-002
Date issued: 2023-06-02
Due date: 2023-07-02
Income:Work: 1337 USD

+------+-----+------------+----------+
| Name | Qty | Unit price | Amount   |
+------+-----+------------+----------+
| Uren | 65  | 65         | 1337 USD |
+------+-----+------------+----------+

Invoice #2"#;

        assert_eq!(expected, invoice.as_txt());
    }
}
