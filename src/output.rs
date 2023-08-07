use crate::invoice::Invoice;

pub trait Output {
    fn as_json(&self) -> String;
    fn as_txt(&self) -> String;
}

impl Output for Invoice<'_> {
    fn as_json(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }

    fn as_txt(&self) -> String {
        format!(
            r#"
Invoice: #{}
Date issued: {}
Due date: {}
Income:Work: {}

{}"#,
            self.number.0,
            self.date.clone(),
            self.due_date.as_ref().unwrap_or(&self.date),
            self.total,
            self.narration,
        )
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

        let expected = r#"
Invoice: #2023-002
Date issued: 2023-06-02
Due date: 2023-07-02
Income:Work: 1337 USD

Invoice #2"#;
        assert_eq!(expected, actual);
    }
}
