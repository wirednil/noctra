//! AST (Abstract Syntax Tree) para RQL (Extended SQL)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AST principal de una consulta RQL
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RqlAst {
    /// Statements parseados
    pub statements: Vec<RqlStatement>,

    /// Parámetros extraídos
    pub parameters: Vec<RqlParameter>,

    /// Variables de sesión encontradas
    pub session_variables: Vec<String>,

    /// Metadatos del parsing
    pub metadata: ParsingMetadata,
}

/// Un statement RQL
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RqlStatement {
    /// SQL Statement estándar (SELECT, INSERT, UPDATE, DELETE)
    Sql {
        sql: String,
        parameters: HashMap<String, ParameterType>,
    },

    /// Comando USE para cambiar esquema
    Use { schema: String },

    /// Comando LET para variables de sesión
    Let {
        variable: String,
        expression: String,
    },

    /// Comando FORM LOAD
    FormLoad { form_path: String },

    /// Comando EXECFORM
    ExecForm {
        form_path: String,
        parameters: HashMap<String, ParameterType>,
    },

    /// Comando OUTPUT TO
    OutputTo {
        destination: OutputDestination,
        format: OutputFormat,
    },
}

/// Parámetro extraído del código RQL
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RqlParameter {
    /// Nombre del parámetro (ej: "dept", "$1", ":name")
    pub name: String,

    /// Tipo de parámetro
    pub param_type: ParameterType,

    /// Posición en el query original
    pub position: Option<usize>,

    /// Línea donde fue encontrado
    pub line: usize,

    /// Columna donde fue encontrado
    pub column: usize,
}

/// Tipos de parámetros soportados
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterType {
    /// Parámetro posicionado ($1, $2, etc.)
    Positional,

    /// Parámetro nombrado (:name)
    Named,

    /// Variable de sesión (#variable)
    SessionVariable,

    /// Parámetro en template
    Template,
}

/// Destinos de output
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OutputDestination {
    /// Stdout (salida estándar)
    Stdout,

    /// Archivo específico
    File(String),

    /// Impresora del sistema
    Printer,
}

/// Formatos de output
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OutputFormat {
    /// Formato tabla ASCII
    Table,

    /// Formato CSV
    Csv,

    /// Formato JSON
    Json,

    /// Formato XML
    Xml,
}

/// Metadatos del proceso de parsing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsingMetadata {
    /// Timestamp del parsing
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Versión del parser
    pub parser_version: String,

    /// Tiempo de parsing en microsegundos
    pub parsing_time_us: u64,

    /// Número de líneas procesadas
    pub lines_processed: usize,

    /// Warnings generados durante el parsing
    pub warnings: Vec<String>,
}

impl Default for ParsingMetadata {
    fn default() -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            parser_version: "0.1.0".to_string(),
            parsing_time_us: 0,
            lines_processed: 0,
            warnings: Vec::new(),
        }
    }
}

impl RqlAst {
    /// Crear nuevo AST vacío
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
            parameters: Vec::new(),
            session_variables: Vec::new(),
            metadata: ParsingMetadata::default(),
        }
    }

    /// Agregar statement
    pub fn add_statement(&mut self, statement: RqlStatement) {
        self.statements.push(statement);
    }

    /// Agregar parámetro
    pub fn add_parameter(&mut self, parameter: RqlParameter) {
        self.parameters.push(parameter);
    }

    /// Agregar variable de sesión
    pub fn add_session_variable(&mut self, variable: String) {
        if !self.session_variables.contains(&variable) {
            self.session_variables.push(variable);
        }
    }

    /// Obtener todos los parámetros únicos
    pub fn get_parameters(&self) -> Vec<&RqlParameter> {
        self.parameters.iter().collect()
    }

    /// Obtener parámetros por tipo
    pub fn get_parameters_by_type(&self, param_type: &ParameterType) -> Vec<&RqlParameter> {
        self.parameters
            .iter()
            .filter(|p| &p.param_type == param_type)
            .collect()
    }

    /// Obtener parámetros nombrados
    pub fn get_named_parameters(&self) -> Vec<&RqlParameter> {
        self.get_parameters_by_type(&ParameterType::Named)
    }

    /// Obtener parámetros posicionados
    pub fn get_positional_parameters(&self) -> Vec<&RqlParameter> {
        self.get_parameters_by_type(&ParameterType::Positional)
    }

    /// Verificar si contiene parámetros
    pub fn has_parameters(&self) -> bool {
        !self.parameters.is_empty()
    }

    /// Obtener SQL statements únicamente
    pub fn get_sql_statements(&self) -> Vec<&str> {
        self.statements
            .iter()
            .filter_map(|stmt| {
                if let RqlStatement::Sql { sql, .. } = stmt {
                    Some(sql.as_str())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Convertir AST a SQL string
    pub fn to_sql(&self) -> String {
        self.statements
            .iter()
            .map(|stmt| match stmt {
                RqlStatement::Sql { sql, .. } => sql.clone(),
                RqlStatement::Use { schema } => format!("USE {};", schema),
                RqlStatement::Let {
                    variable,
                    expression,
                } => {
                    format!("LET {} = {};", variable, expression)
                }
                RqlStatement::FormLoad { form_path } => {
                    format!("FORM LOAD '{}';", form_path)
                }
                RqlStatement::ExecForm { form_path, .. } => {
                    format!("EXECFORM '{}';", form_path)
                }
                RqlStatement::OutputTo {
                    destination,
                    format,
                } => {
                    let dest_str = match destination {
                        OutputDestination::Stdout => "STDOUT",
                        OutputDestination::File(path) => path,
                        OutputDestination::Printer => "PRINTER",
                    };
                    let format_str = match format {
                        OutputFormat::Table => "table",
                        OutputFormat::Csv => "csv",
                        OutputFormat::Json => "json",
                        OutputFormat::Xml => "xml",
                    };
                    format!("OUTPUT TO {} FORMAT {};", dest_str, format_str)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Obtener información de debug
    pub fn debug_info(&self) -> AstDebugInfo {
        AstDebugInfo {
            statement_count: self.statements.len(),
            parameter_count: self.parameters.len(),
            session_var_count: self.session_variables.len(),
            has_sql_statements: self
                .statements
                .iter()
                .any(|s| matches!(s, RqlStatement::Sql { .. })),
            has_commands: self
                .statements
                .iter()
                .any(|s| !matches!(s, RqlStatement::Sql { .. })),
        }
    }
}

/// Información de debug del AST
#[derive(Debug, Clone)]
pub struct AstDebugInfo {
    pub statement_count: usize,
    pub parameter_count: usize,
    pub session_var_count: usize,
    pub has_sql_statements: bool,
    pub has_commands: bool,
}

impl Default for RqlAst {
    fn default() -> Self {
        Self::new()
    }
}

/// Utilidades para el AST
impl RqlStatement {
    /// Obtener tipo de statement
    pub fn statement_type(&self) -> &'static str {
        match self {
            RqlStatement::Sql { .. } => "SQL",
            RqlStatement::Use { .. } => "USE",
            RqlStatement::Let { .. } => "LET",
            RqlStatement::FormLoad { .. } => "FORM_LOAD",
            RqlStatement::ExecForm { .. } => "EXECFORM",
            RqlStatement::OutputTo { .. } => "OUTPUT_TO",
        }
    }

    /// Verificar si es un statement SQL
    pub fn is_sql(&self) -> bool {
        matches!(self, RqlStatement::Sql { .. })
    }

    /// Verificar si es un comando RQL
    pub fn is_command(&self) -> bool {
        !self.is_sql()
    }

    /// Extraer SQL si es statement SQL
    pub fn as_sql(&self) -> Option<&str> {
        if let RqlStatement::Sql { sql, .. } = self {
            Some(sql)
        } else {
            None
        }
    }
}
