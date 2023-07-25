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
use std::{borrow::Cow, io::Read};

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
    let today: Date<'static> = chrono::Local::now().date_naive().into();

    format!(
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
        today,
    )
}

pub fn handle_list() -> String {
    // read stdin into a String
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .expect("To read input");

    summarize_invoices(input)
}

#[derive(Debug)]
struct InvoiceNumber(String);
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
impl fmt::Display for InvoiceNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn summarize_invoices(input: String) -> String {
    let ledger = parse(&input).unwrap();
    // keep only the directives that are transactions
    // keep only the transactions that have an invoice_number
    // and where the invoice_number is a Number
    let txs: Vec<beancount_core::Transaction> = ledger
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
        .collect();

    // format their descriptions, dates, and invoice_numbers collect in a string
    let summary: String = txs
        .into_iter()
        .map(|tx| {
            let invoice_number: InvoiceNumber = tx.meta.get("invoice_number").unwrap().into();

            format!("{} {} {}", invoice_number, tx.date, tx.narration)
        })
        .collect::<Vec<String>>()
        .join("\n");
    // join the strings with newlines and return this string
    summary
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

        let expected_output = "42 2023-06-02 Invoice #42\n\
                               43 2023-06-02 Invoice #43";

        assert_eq!(summarize_invoices(input.to_string()), expected_output);
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

        let expected_output = "2023-42 2023-06-02 Invoice #42\n\
                               TBD 2023-06-02 Invoice #TBD";

        assert_eq!(summarize_invoices(input.to_string()), expected_output);
    }

    // Edge case where we have a number that is not quoted but a valid formula. E.g. 2023-42 is
    // 1981.
    #[test]
    fn test_handle_list_with_evald_numbers() {
        let input = "2023-06-02 ! \"Invoice #42\"\n\
                     \tinvoice_number: 2023-42\n\
                     \tAssets:AccountsReceivable\t1337 USD\n\
                     \tIncome:Work\t1338 USD\n";

        let expected_output = "1981 2023-06-02 Invoice #42";

        assert_eq!(summarize_invoices(input.to_string()), expected_output);
    }
}
