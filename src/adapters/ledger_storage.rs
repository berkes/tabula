use core::fmt::{self, Display};
use std::error::Error;

use beancount_core::{metadata::MetaValue, Transaction};

use crate::domain::invoice::{Date, Invoice, InvoiceList, InvoiceNumber, LineItem};

pub trait LedgerStorage {
    fn find_invoice(&self, number: &InvoiceNumber) -> Result<Invoice, Box<dyn Error>>;
    fn find_invoices(&self) -> Result<InvoiceList, Box<dyn Error>>;
    fn build(&self) -> Result<Invoice, Box<dyn Error>>;
}

pub struct StdinLedgerStorage {
    ledger: String,
}

impl StdinLedgerStorage {
    pub fn new(stdin: String) -> Self {
        Self { ledger: stdin }
    }
}

impl<'a> From<Transaction<'a>> for Invoice {
    fn from(borrowed_tx: Transaction<'a>) -> Self {
        let tx = borrowed_tx.clone();

        let date: Date = tx.date.into();
        let due_date: Option<Date> = tx.meta.get("due").map(|d| d.into());

        let narration = tx.narration.to_string();
        let number: InvoiceNumber = tx.meta.get("invoice_number").into();
        let total = "1337 USD".to_string();

        let line_items: Vec<LineItem> = tx
            .postings
            .iter()
            .filter(|p| p.meta.get("line_item_name").is_some())
            .map(|p| {
                let description =
                    if let Some(MetaValue::Text(description)) = p.meta.get("line_item_name") {
                        description.to_string()
                    } else {
                        panic!("Expected a text value");
                    };

                LineItem {
                    description,
                    unit_price: "13.37 USD".to_string(),
                    quantity: "100".to_string(),
                    total: "1337 USD".to_string(),
                }
            })
            .collect();

        Self {
            date,
            due_date,
            narration,
            number,
            total,
            line_items,
        }
    }
}

impl LedgerStorage for StdinLedgerStorage {
    fn find_invoice(&self, number: &InvoiceNumber) -> Result<Invoice, Box<dyn Error>> {
        let invoices = self.find_invoices()?.invoices;

        invoices
            .into_iter()
            .find(|invoice| &invoice.number == number)
            .ok_or_else(|| Box::new(NotFoundError) as Box<dyn Error>)
    }

    fn find_invoices(&self) -> Result<InvoiceList, Box<dyn Error>> {
        let ledger = beancount_parser::parse(&self.ledger)?;
        let invoices = ledger
            .directives
            .clone()
            .into_iter()
            .filter_map(|directive| {
                // Only keep Transactions
                if let beancount_core::Directive::Transaction(tx) = directive {
                    Some(tx)
                } else {
                    None
                }
            })
            .filter(|d| d.meta.get("invoice_number").is_some()) // Only keep Transactions with an invoice_number
            .map(|tx| tx.into()) // Convert Transaction into Invoice
            .collect();

        Ok(InvoiceList { invoices })
    }

    fn build(&self) -> Result<Invoice, Box<dyn Error>> {
        let invoice = Invoice::default();

        Ok(invoice)
    }
}

#[derive(Debug)]
struct NotFoundError;

impl Error for NotFoundError {}

impl Display for NotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invoice not found")
    }
}
