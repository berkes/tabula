use beancount_core::{
    directives::Transaction,
    metadata::{Meta, MetaValue},
    Account,
    AccountType::Assets,
    Date, Flag, IncompleteAmount, Ledger, Posting,
};

use beancount_parser::parse;
use beancount_render::render;
use core::fmt;
use serde::Serialize;
use std::{borrow::Cow, error::Error, io::Read};

use crate::output::Output;

pub fn handle_create() -> String {
    let today: Date<'static> = chrono::Local::now().date_naive().into();

    let accts_recievable = Account::builder()
        .ty(Assets)
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
        .date(today)
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

pub fn handle_build() -> String {
    let invoice = Invoice::default();

    serde_json::to_string_pretty(&invoice).unwrap()
}

pub fn handle_list() -> String {
    let input = read_stdin().expect("To read input");
    let ledger = parse(&input).expect("To parse input");

    let summary = summarize_invoices(ledger);
    tabulate(summary)
}

#[derive(Debug, PartialEq, Serialize)]
pub struct InvoiceNumber(pub String);
impl From<&MetaValue<'_>> for InvoiceNumber {
    fn from(mv: &MetaValue) -> Self {
        match mv {
            MetaValue::Text(s) => InvoiceNumber(s.to_string()),
            MetaValue::Currency(s) => InvoiceNumber(s.to_string()),
            MetaValue::Number(n) => InvoiceNumber(n.to_string()),
            _ => panic!("Expected a text value"),
        }
    }
}

impl From<Option<&MetaValue<'_>>> for InvoiceNumber {
    fn from(mv: Option<&MetaValue>) -> Self {
        match mv {
            Some(mv) => InvoiceNumber::from(mv),
            None => InvoiceNumber("TBD".to_string()),
        }
    }
}

impl fmt::Display for InvoiceNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone)]
pub struct MyDate<'a>(Date<'a>);
impl From<&str> for MyDate<'_> {
    fn from(value: &str) -> Self {
        let date = chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap();
        MyDate(date.into())
    }
}

impl Serialize for MyDate<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl fmt::Display for MyDate<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

#[derive(Serialize)]
pub struct Invoice<'a> {
    pub date: MyDate<'a>,
    pub due_date: Option<MyDate<'a>>,
    pub narration: String,
    pub number: InvoiceNumber,
    pub total: String,
}

impl Default for Invoice<'_> {
    fn default() -> Self {
        Self {
            date: MyDate(chrono::Local::now().date_naive().into()),
            due_date: None,
            narration: String::default(),
            number: InvoiceNumber("TBD".to_string()),
            total: "1337 USD".to_string(),
        }
    }
}

impl<'a> From<Transaction<'a>> for Invoice<'a> {
    fn from(tx: Transaction<'a>) -> Self {
        let date = tx.date;

        let due_date = if let Some(MetaValue::Date(due_date)) = tx.meta.get("due") {
            Some(due_date).cloned()
        } else {
            None
        };

        let narration = tx.narration.to_string();
        let number: InvoiceNumber = tx.meta.get("invoice_number").into();
        let total = "1337 USD".to_string();

        Self {
            date: MyDate(date),
            due_date: due_date.map(MyDate),
            narration,
            number,
            total,
        }
    }
}

pub fn handle_convert(args: crate::cli::ConvertArgs) -> String {
    let input = read_stdin().expect("To read input");
    let ledger = parse(&input).expect("To parse input");

    let tx = find_invoice(ledger, args.invoice_number).expect("To find invoice");

    let invoice: Invoice = tx.into();

    match args.format {
        crate::cli::OutputFormat::Json => invoice.as_json(),
        crate::cli::OutputFormat::Txt => invoice.as_txt(),
        crate::cli::OutputFormat::Pdf => todo!(),
    }
}

fn find_invoice(ledger: Ledger<'_>, invoice_number: String) -> Option<Transaction<'_>> {
    let expected_invoice_number = InvoiceNumber(invoice_number);

    let txs = find_invoices(ledger);
    txs.into_iter().find(|tx| {
        let possible_invoice_number: InvoiceNumber = tx.meta.get("invoice_number").unwrap().into();
        possible_invoice_number == expected_invoice_number
    })
}

pub fn read_stdin() -> Result<String, Box<dyn Error>> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    Ok(input)
}

pub fn summarize_invoices(input: Ledger) -> Vec<Vec<Option<String>>> {
    let txs = find_invoices(input);

    // format their descriptions, dates, and invoice_numbers collect in a string
    txs.into_iter()
        .map(|tx| {
            let invoice_number: InvoiceNumber = tx.meta.get("invoice_number").unwrap().into();
            // Assign a due date if it exists and is a Date
            let due_date = tx.meta.get("due").and_then(|v| match v {
                MetaValue::Date(d) => Some(d),
                _ => None,
            });

            vec![
                Some(invoice_number.to_string()),
                Some(tx.date.to_string()),
                Some(tx.narration.to_string()),
                due_date.map(|d| d.to_string()),
            ]
        })
        .collect::<Vec<Vec<Option<String>>>>()
}

fn find_invoices(input: Ledger) -> Vec<beancount_core::Transaction> {
    // keep only the directives that are transactions
    // keep only the transactions that have an invoice_number
    input
        .directives
        .into_iter()
        .filter_map(|directive| {
            if let beancount_core::Directive::Transaction(tx) = directive {
                Some(tx)
            } else {
                None
            }
        })
        .filter(|d| d.meta.get("invoice_number").is_some())
        .collect()
}

fn tabulate(input: Vec<Vec<Option<String>>>) -> String {
    let mut output = String::new();
    let max_columns = input.iter().map(|row| row.len()).max().unwrap_or(0);

    for row in input {
        for (i, col) in row.iter().enumerate() {
            output.push_str(&format!("{}", col.as_ref().unwrap_or(&"".to_string())));
            if i < max_columns - 1 {
                output.push('\t');
            }
        }
        output.push('\n');
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_list() {
        let input = "2023-06-02 ! \"Invoice #42\"\n\
                     \tinvoice_number: 42\n\
                     \tAssets:AccountsReceivable\t1337 USD\n\
                     \tIncome:Work\t1337 USD\n\
                     \n\
                     2023-06-02 ! \"Invoice #43\"\n\
                     \tinvoice_number: 43\n\
                     \tAssets:AccountsReceivable\t1338 USD\n\
                     \tIncome:Work\t1338 USD\n";

        let expected_output = vec![
            vec![
                Some("42".to_string()),
                Some("2023-06-02".to_string()),
                Some("Invoice #42".to_string()),
                None,
            ],
            vec![
                Some("43".to_string()),
                Some("2023-06-02".to_string()),
                Some("Invoice #43".to_string()),
                None,
            ],
        ];

        assert_eq!(summarize_invoices(parse(input).unwrap()), expected_output);
    }

    #[test]
    fn test_handle_list_with_due_date() {
        let input = "2023-06-02 ! \"Invoice #42\"\n\
                     \tinvoice_number: 42\n\
                     \tdue: 2023-07-02\n\
                     \tAssets:AccountsReceivable\t1337 USD\n\
                     \tIncome:Work\t1337 USD\n";
        let expected_output = vec![vec![
            Some("42".to_string()),
            Some("2023-06-02".to_string()),
            Some("Invoice #42".to_string()),
            Some("2023-07-02".to_string()),
        ]];

        assert_eq!(summarize_invoices(parse(input).unwrap()), expected_output);
    }

    #[test]
    fn test_handle_list_with_string_numbers() {
        let input = "2023-06-02 ! \"Invoice #42\"\n\
                     \tinvoice_number: \"2023-42\"\n\
                     \tAssets:AccountsReceivable\t1337 USD\n\
                     \tIncome:Work\t1337 USD\n\
                     \n\
                     2023-06-02 ! \"Invoice #TBD\"\n\
                     \tinvoice_number: \"TBD\"\n\
                     \tAssets:AccountsReceivable\t1338 USD\n\
                     \tIncome:Work\t1338 USD\n";

        let expected_output = vec![
            vec![
                Some("2023-42".to_string()),
                Some("2023-06-02".to_string()),
                Some("Invoice #42".to_string()),
                None,
            ],
            vec![
                Some("TBD".to_string()),
                Some("2023-06-02".to_string()),
                Some("Invoice #TBD".to_string()),
                None,
            ],
        ];

        assert_eq!(summarize_invoices(parse(input).unwrap()), expected_output);
    }

    // Edge case where we have a number that is not quoted but a valid formula. E.g. 2023-42 is
    // 1981.
    #[test]
    fn test_handle_list_with_evald_numbers() {
        let input = "2023-06-02 ! \"Invoice #42\"\n\
                     \tinvoice_number: 2023-42\n\
                     \tAssets:AccountsReceivable\t1337 USD\n\
                     \tIncome:Work\t1338 USD\n";

        let expected_output = vec![vec![
            Some("1981".to_string()),
            Some("2023-06-02".to_string()),
            Some("Invoice #42".to_string()),
            None,
        ]];

        assert_eq!(summarize_invoices(parse(input).unwrap()), expected_output);
    }

    #[test]
    fn test_tabulate() {
        let input = vec![
            vec![
                Some("INV-001".to_string()),
                Some("2023-07-27".to_string()),
                Some("Sample invoice".to_string()),
                Some("2023-08-10".to_string()),
            ],
            vec![
                Some("INV-002".to_string()),
                Some("2023-07-28".to_string()),
                Some("Another invoice".to_string()),
                None, // No due date
            ],
        ];

        let expected_output = "\
            INV-001\t2023-07-27\tSample invoice\t2023-08-10\n\
            INV-002\t2023-07-28\tAnother invoice\t\n";

        assert_eq!(tabulate(input), expected_output);
    }
}
