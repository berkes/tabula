use beancount_core::metadata::MetaValue;
use chrono::NaiveDate;
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize, Clone)]
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

#[derive(Clone)]
pub struct Date(pub NaiveDate);
impl From<&str> for Date {
    fn from(value: &str) -> Self {
        let date = NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap();
        Date(date)
    }
}

impl From<String> for Date {
    fn from(value: String) -> Self {
        Date::from(value.as_str())
    }
}

impl From<&MetaValue<'_>> for Date {
    fn from(mv: &MetaValue) -> Self {
        match mv {
            MetaValue::Date(date) => date.clone().into(),
            _ => panic!("Expected a date value"),
        }
    }
}

impl From<beancount_core::Date<'_>> for Date {
    fn from(date: beancount_core::Date) -> Self {
        date.to_string().into()
    }
}

impl Serialize for Date {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}

#[derive(Serialize, Clone)]
pub struct LineItem {
    pub description: String,
    pub unit_price: String,
    pub quantity: String,
    pub total: String,
}

#[derive(Serialize, Clone)]
pub struct Invoice {
    pub date: Date,
    pub due_date: Option<Date>,
    pub narration: String,
    pub number: InvoiceNumber,
    pub total: String,
    pub line_items: Vec<LineItem>,
}

impl Default for Invoice {
    fn default() -> Self {
        let today: NaiveDate = chrono::Local::now().date_naive();
        Self {
            date: Date(today),
            due_date: None,
            narration: String::default(),
            number: InvoiceNumber("TBD".to_string()),
            total: "1337 USD".to_string(),
            line_items: vec![],
        }
    }
}

#[derive(Serialize)]
pub struct InvoiceList {
    pub invoices: Vec<Invoice>,
}
