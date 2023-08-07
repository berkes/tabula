use core::fmt;
use std::fmt::Display;

use crate::invoice::{Invoice, InvoiceNumber};

pub trait Output {
    fn as_json(&self) -> String;
    fn as_txt(&self) -> String;
}

impl Output for Invoice<'_> {
    fn as_json(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }

    fn as_txt(&self) -> String {
        format!("{}", self)
    }
}

impl Display for Invoice<'_> {
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

        write!(f, "{}\n{}", meta, self.narration)
    }
}

impl Display for InvoiceNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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

impl<'a> fmt::Display for KeyValueRenderer<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render())
    }
}

#[cfg(test)]
mod tests {
    use assert_json_diff::assert_json_eq;
    use serde_json::json;

    use crate::invoice::InvoiceNumber;

    use super::*;

    #[test]
    fn test_as_json_has_all_fields() {
        let invoice = Invoice {
            date: "2023-06-02".into(),
            due_date: Some("2023-07-02".into()),
            narration: "Invoice #2".to_string(),
            number: InvoiceNumber("2023-002".to_string()),
            total: "1337 USD".to_string(),
        };

        let actual = serde_json::from_str::<serde_json::Value>(&invoice.as_json()).unwrap();

        let expected = json!(
            {
                "date": "2023-06-02",
                "due_date": "2023-07-02",
                "narration": "Invoice #2",
                "number": "2023-002",
                "total": "1337 USD"
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
        };

        let actual = invoice.as_txt();

        let expected = r#"Invoice: 2023-002
Date issued: 2023-06-02
Due date: 2023-07-02
Income:Work: 1337 USD

Invoice #2"#;
        assert_eq!(expected, actual);
    }
}
