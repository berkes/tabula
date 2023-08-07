use core::fmt;
use std::fmt::Display;
use prettytable::{Table, Row, Cell};

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

        // If there are line-items, render them to a table with prettytable
        if !self.line_items.is_empty() {
            let mut table = Table::new();
            table.add_row(Row::new(vec![Cell::new("Name"), Cell::new("Qty"), Cell::new("Unit price"), Cell::new("Amount")]));
            for line_item in &self.line_items {
                table.add_row(Row::new(vec![
                    Cell::new(&line_item.description),
                    Cell::new(&line_item.quantity),
                    Cell::new(&line_item.unit_price),
                    Cell::new(&line_item.total),
                ]));
            }
            write!(f, "{}\n{}\n{}", meta, table, self.narration)?;
            return Ok(());
        } else {
            write!(f, "{}\n{}", meta, self.narration)?;
            return Ok(());
        }
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
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::invoice::{InvoiceNumber, LineItem};

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
