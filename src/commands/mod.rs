use std::error::Error;

use crate::{adapters::{ledger_storage::LedgerStorage, cli::Output}, domain::invoice::InvoiceNumber};

pub trait Command {
    type LedgerStorageType: LedgerStorage;
    fn new(ledger_storage: Self::LedgerStorageType) -> Self;
    fn execute(&self) -> Result<Box<dyn Output>, Box<dyn Error>>;
    fn ledger_storage(&self) -> &Self::LedgerStorageType;
}

pub struct BuildInvoiceCommand<S: LedgerStorage> {
    ledger_storage: S,
}

impl<S: LedgerStorage> Command for BuildInvoiceCommand<S> {
    type LedgerStorageType = S;

    fn new(ledger_storage: S) -> Self {
        Self { ledger_storage }
    }

    fn execute(&self) -> Result<Box<dyn Output>, Box<dyn Error>> {
        Ok(Box::new(self.ledger_storage().build()?))
    }

    fn ledger_storage(&self) -> &S {
        &self.ledger_storage
    }
}

pub struct ListInvoicesCommand<S: LedgerStorage> {
    ledger_storage: S,
}

impl<S: LedgerStorage> Command for ListInvoicesCommand<S> {
    type LedgerStorageType = S;

    fn new(ledger_storage: S) -> Self {
        Self { ledger_storage }
    }

    fn execute(&self) -> Result<Box<dyn Output>, Box<dyn Error>> {
        Ok(Box::new(self.ledger_storage().find_invoices()?))
    }

    fn ledger_storage(&self) -> &S {
        &self.ledger_storage
    }
}

pub struct FindInvoiceCommand<S: LedgerStorage> {
    ledger_storage: S,
    invoice_number: String,
}

impl<S: LedgerStorage> Command for FindInvoiceCommand<S> {
    type LedgerStorageType = S;

    fn new(ledger_storage: S) -> Self {
        Self { ledger_storage, invoice_number: "".to_string() }
    }

    fn execute(&self) -> Result<Box<dyn Output>, Box<dyn Error>> {
        let invoice_number = InvoiceNumber(self.invoice_number.clone());
        Ok(Box::new(self.ledger_storage().find_invoice(&invoice_number)?))
    }

    fn ledger_storage(&self) -> &S {
        &self.ledger_storage
    }
}

impl<S: LedgerStorage> FindInvoiceCommand<S> {
    pub fn with_invoice_number(self, invoice_number: String) -> Self {
        Self { invoice_number, ..self }
    }
}
