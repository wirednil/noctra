//! Formateadores de output para Noctra

use noctra_core::ResultSet;
use std::io::{Write, stdout};
use serde_json;

/// Trait para formateadores de output
pub trait OutputFormatter {
    /// Formatear result set
    fn format_result(&self, result: &ResultSet) -> String;
    
    /// Escribir result set a writer
    fn write_result(&self, result: &ResultSet, writer: &mut dyn Write) -> std::io::Result<()>;
}

/// Formateador de tabla
pub struct TableFormatter;

impl OutputFormatter for TableFormatter {
    fn format_result(&self, result: &ResultSet) -> String {
        result.to_table()
    }
    
    fn write_result(&self, result: &ResultSet, writer: &mut dyn Write) -> std::io::Result<()> {
        let table = self.format_result(result);
        writer.write_all(table.as_bytes())
    }
}

/// Formateador CSV
pub struct CsvFormatter {
    delimiter: char,
}

impl CsvFormatter {
    pub fn new(delimiter: char) -> Self {
        Self { delimiter }
    }
}

impl OutputFormatter for CsvFormatter {
    fn format_result(&self, result: &ResultSet) -> String {
        let mut csv = String::new();
        
        // Headers
        if !result.columns.is_empty() {
            let headers: Vec<String> = result.columns.iter()
                .map(|col| col.name.clone())
                .collect();
            csv.push_str(&headers.join(&self.delimiter.to_string()));
            csv.push('\n');
        }
        
        // Data rows
        for row in &result.rows {
            let values: Vec<String> = row.values.iter()
                .map(|v| v.to_string())
                .collect();
            csv.push_str(&values.join(&self.delimiter.to_string()));
            csv.push('\n');
        }
        
        csv
    }
    
    fn write_result(&self, result: &ResultSet, writer: &mut dyn Write) -> std::io::Result<()> {
        let csv = self.format_result(result);
        writer.write_all(csv.as_bytes())
    }
}

/// Formateador JSON
pub struct JsonFormatter {
    pretty: bool,
}

impl JsonFormatter {
    pub fn new(pretty: bool) -> Self {
        Self { pretty }
    }
}

impl OutputFormatter for JsonFormatter {
    fn format_result(&self, result: &ResultSet) -> String {
        if self.pretty {
            serde_json::to_string_pretty(result).unwrap_or_else(|_| "Error formatting JSON".to_string())
        } else {
            serde_json::to_string(result).unwrap_or_else(|_| "Error formatting JSON".to_string())
        }
    }
    
    fn write_result(&self, result: &ResultSet, writer: &mut dyn Write) -> std::io::Result<()> {
        let json = self.format_result(result);
        writer.write_all(json.as_bytes())
    }
}

/// Utility para output estándar
pub fn format_output(
    result: &ResultSet, 
    format_type: &crate::config::OutputFormat
) -> String {
    match format_type {
        crate::config::OutputFormat::Table => TableFormatter.format_result(result),
        crate::config::OutputFormat::Csv => CsvFormatter::new(',').format_result(result),
        crate::config::OutputFormat::Json => JsonFormatter::new(false).format_result(result),
        crate::config::OutputFormat::Custom(_) => "Custom format not implemented".to_string(),
        _ => TableFormatter.format_result(result),
    }
}

/// Escribir result set a stdout
pub fn write_to_stdout(result: &ResultSet, format_type: &crate::config::OutputFormat) -> std::io::Result<()> {
    let mut stdout = stdout();
    match format_type {
        crate::config::OutputFormat::Table => TableFormatter.write_result(result, &mut stdout),
        crate::config::OutputFormat::Csv => CsvFormatter::new(',').write_result(result, &mut stdout),
        crate::config::OutputFormat::Json => JsonFormatter::new(false).write_result(result, &mut stdout),
        _ => TableFormatter.write_result(result, &mut stdout),
    }
}