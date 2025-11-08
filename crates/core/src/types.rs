//! Tipos de datos fundamentales para Noctra

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Representa un valor en Noctra
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum Value {
    /// Valor nulo
    #[default]
    Null,

    /// Entero de 64 bits
    Integer(i64),

    /// Número de punto flotante
    Float(f64),

    /// Texto
    Text(String),

    /// Booleano
    Boolean(bool),

    /// Fecha
    Date(String),

    /// Fecha y hora
    DateTime(String),

    /// Array de valores
    Array(Vec<Value>),

    /// Objeto JSON (para extensibilidad)
    Json(serde_json::Value),
}

impl Value {
    /// Crear valor entero
    pub fn integer<T: Into<i64>>(val: T) -> Self {
        Self::Integer(val.into())
    }

    /// Crear valor flotante
    pub fn float<T: Into<f64>>(val: T) -> Self {
        Self::Float(val.into())
    }

    /// Crear valor texto
    pub fn text<T: Into<String>>(val: T) -> Self {
        Self::Text(val.into())
    }

    /// Crear valor booleano
    pub fn boolean(val: bool) -> Self {
        Self::Boolean(val)
    }

    /// Verificar si es nulo
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => write!(f, "NULL"),
            Self::Integer(v) => write!(f, "{}", v),
            Self::Float(v) => write!(f, "{}", v),
            Self::Text(v) => write!(f, "{}", v),
            Self::Boolean(v) => write!(f, "{}", v),
            Self::Date(v) | Self::DateTime(v) => write!(f, "{}", v),
            Self::Array(v) => {
                write!(f, "[")?;
                for (i, item) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Self::Json(v) => write!(f, "{}", v),
        }
    }
}

impl From<i64> for Value {
    fn from(val: i64) -> Self {
        Self::Integer(val)
    }
}

impl From<f64> for Value {
    fn from(val: f64) -> Self {
        Self::Float(val)
    }
}

impl From<String> for Value {
    fn from(val: String) -> Self {
        Self::Text(val)
    }
}

impl From<&str> for Value {
    fn from(val: &str) -> Self {
        Self::Text(val.to_string())
    }
}

impl From<bool> for Value {
    fn from(val: bool) -> Self {
        Self::Boolean(val)
    }
}

impl From<Vec<Value>> for Value {
    fn from(val: Vec<Value>) -> Self {
        Self::Array(val)
    }
}

impl From<serde_json::Value> for Value {
    fn from(val: serde_json::Value) -> Self {
        Self::Json(val)
    }
}

/// Una columna de resultado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    /// Nombre de la columna
    pub name: String,

    /// Tipo de datos
    pub data_type: String,

    /// Índice de la columna
    pub ordinal: usize,
}

impl Column {
    /// Crear nueva columna
    pub fn new<T: Into<String>>(name: T, data_type: T, ordinal: usize) -> Self {
        Self {
            name: name.into(),
            data_type: data_type.into(),
            ordinal,
        }
    }
}

/// Una fila de resultado
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Row {
    /// Valores de la fila
    pub values: Vec<Value>,
}

impl Row {
    /// Crear nueva fila
    pub fn new(values: Vec<Value>) -> Self {
        Self { values }
    }

    /// Obtener valor por índice
    pub fn get(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
    }

    /// Obtener valor por nombre de columna
    pub fn get_by_name(&self, columns: &[Column], name: &str) -> Option<&Value> {
        columns
            .iter()
            .find(|col| col.name == name)
            .and_then(|col| self.get(col.ordinal))
    }

    /// Cantidad de columnas
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Verificar si está vacía
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

/// Un conjunto de resultados
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultSet {
    /// Columnas del resultado
    pub columns: Vec<Column>,

    /// Filas del resultado
    pub rows: Vec<Row>,

    /// Número de filas afectadas (para INSERT/UPDATE/DELETE)
    pub rows_affected: Option<u64>,

    /// Último ID insertado (para INSERT)
    pub last_insert_rowid: Option<i64>,
}

impl ResultSet {
    /// Crear nuevo ResultSet
    pub fn new(columns: Vec<Column>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
            rows_affected: None,
            last_insert_rowid: None,
        }
    }

    /// Crear ResultSet vacío
    pub fn empty() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            rows_affected: None,
            last_insert_rowid: None,
        }
    }

    /// Agregar fila
    pub fn add_row(&mut self, row: Row) {
        self.rows.push(row);
    }

    /// Agregar múltiples filas
    pub fn add_rows(&mut self, rows: Vec<Row>) {
        self.rows.extend(rows);
    }

    /// Número de filas
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Número de columnas
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Verificar si está vacío
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Convertir a formato tabla
    pub fn to_table(&self) -> String {
        if self.columns.is_empty() {
            return "No results".to_string();
        }

        let mut result = String::new();

        // Header
        let headers: Vec<String> = self.columns.iter().map(|col| col.name.clone()).collect();
        result.push_str(&headers.join(" | "));
        result.push('\n');

        // Separador
        let separators: Vec<String> = self
            .columns
            .iter()
            .map(|col| "-".repeat(col.name.len().max(8)))
            .collect();
        result.push_str(&separators.join("-+-"));
        result.push('\n');

        // Filas
        for row in &self.rows {
            let values: Vec<String> = self
                .columns
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    row.get(i)
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "NULL".to_string())
                })
                .collect();
            result.push_str(&values.join(" | "));
            result.push('\n');
        }

        result.push('\n');
        result.push_str(&format!("({} rows)", self.row_count()));

        result
    }
}

/// Mapeo de parámetros
pub type Parameters = HashMap<String, Value>;

/// Variables de sesión
pub type SessionVariables = HashMap<String, Value>;
