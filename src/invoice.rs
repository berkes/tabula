use beancount_core::{
    directives::Transaction,
    metadata::{Meta, MetaValue},
    Account,
    AccountType::Assets,
    Date, Flag, IncompleteAmount, Ledger, Posting,
};
use beancount_render::render;
use std::borrow::Cow;

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

type InvoiceNumber = String;

fn next_invoice_number(ledger: Ledger) -> InvoiceNumber {
    "1".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_that_invoice_number_defaults_to_1() {
        let ledger = Ledger { directives: vec![] };

        assert_eq!("1", next_invoice_number(ledger));
    }
}
