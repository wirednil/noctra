//! Parser principal para RQL (Extended SQL)

use crate::error::{ParserError, ParserResult};
use crate::rql_ast::{
    OutputDestination, OutputFormat, ParameterType, RqlAst, RqlParameter, RqlStatement,
};
use regex::Regex;
use std::collections::HashMap;
use std::time::Instant;

/// Parser principal para RQL
#[derive(Debug, Clone)]
pub struct RqlParser {
    /// Configuración del parser
    #[allow(dead_code)]
    config: ParserConfig,
}

impl RqlParser {
    /// Crear nuevo parser
    pub fn new() -> Self {
        Self {
            config: ParserConfig::default(),
        }
    }

    /// Crear parser con configuración específica
    pub fn with_config(config: ParserConfig) -> Self {
        Self { config }
    }

    /// Parsear input RQL completo
    pub async fn parse_rql(&self, input: &str) -> ParserResult<RqlAst> {
        let start_time = Instant::now();

        let mut ast = RqlAst::new();

        // Dividir input en líneas para procesamiento
        let lines: Vec<&str> = input.lines().collect();
        ast.metadata.lines_processed = lines.len();

        // Procesar cada línea
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed_line = line.trim();

            // Saltar líneas vacías y comentarios
            if trimmed_line.is_empty() || trimmed_line.starts_with("--") {
                continue;
            }

            // Parsear línea individual
            match self.parse_line(trimmed_line, line_num + 1) {
                Ok(statement) => {
                    ast.add_statement(statement);
                    // Extraer parámetros de la línea
                    self.extract_parameters(trimmed_line, line_num + 1, &mut ast)?;
                }
                Err(e) => {
                    return Err(ParserError::syntax_error(
                        line_num + 1,
                        1,
                        format!("Failed to parse line: {}", e),
                    ));
                }
            }
        }

        // Actualizar metadatos
        ast.metadata.parsing_time_us = start_time.elapsed().as_micros() as u64;

        Ok(ast)
    }

    /// Parsear línea individual
    fn parse_line(&self, line: &str, line_num: usize) -> ParserResult<RqlStatement> {
        let upper_line = line.to_uppercase();

        // Detectar comandos RQL
        if upper_line.starts_with("USE ") {
            self.parse_use_command(line, line_num)
        } else if upper_line.starts_with("LET ") {
            self.parse_let_command(line, line_num)
        } else if upper_line.starts_with("FORM LOAD ") {
            self.parse_form_load_command(line, line_num)
        } else if upper_line.starts_with("EXECFORM ") {
            self.parse_exec_form_command(line, line_num)
        } else if upper_line.starts_with("OUTPUT TO ") {
            self.parse_output_to_command(line, line_num)
        } else {
            // Es SQL estándar
            self.parse_sql_statement(line, line_num)
        }
    }

    /// Parsear comando USE
    fn parse_use_command(&self, line: &str, line_num: usize) -> ParserResult<RqlStatement> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(ParserError::syntax_error(
                line_num,
                1,
                "USE command requires schema name",
            ));
        }

        let schema = parts[1].to_string();
        Ok(RqlStatement::Use { schema })
    }

    /// Parsear comando LET
    fn parse_let_command(&self, line: &str, line_num: usize) -> ParserResult<RqlStatement> {
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() < 3 {
            return Err(ParserError::syntax_error(
                line_num,
                1,
                "LET command requires variable and expression",
            ));
        }

        let variable = parts[1].to_string();
        let expression = parts[2].to_string();
        Ok(RqlStatement::Let {
            variable,
            expression,
        })
    }

    /// Parsear comando FORM LOAD
    fn parse_form_load_command(&self, line: &str, line_num: usize) -> ParserResult<RqlStatement> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(ParserError::syntax_error(
                line_num,
                1,
                "FORM LOAD command requires file path",
            ));
        }

        let form_path = parts[1].to_string();
        Ok(RqlStatement::FormLoad { form_path })
    }

    /// Parsear comando EXECFORM
    fn parse_exec_form_command(&self, line: &str, line_num: usize) -> ParserResult<RqlStatement> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(ParserError::syntax_error(
                line_num,
                1,
                "EXECFORM command requires file path",
            ));
        }

        let form_path = parts[1].to_string();
        Ok(RqlStatement::ExecForm {
            form_path,
            parameters: HashMap::new(),
        })
    }

    /// Parsear comando OUTPUT TO
    fn parse_output_to_command(&self, line: &str, line_num: usize) -> ParserResult<RqlStatement> {
        let upper_line = line.to_uppercase();

        // Detectar formato
        let format = if upper_line.contains("FORMAT CSV") {
            OutputFormat::Csv
        } else if upper_line.contains("FORMAT JSON") {
            OutputFormat::Json
        } else if upper_line.contains("FORMAT XML") {
            OutputFormat::Xml
        } else {
            OutputFormat::Table
        };

        // Detectar destino
        let destination = if upper_line.contains("STDOUT") {
            OutputDestination::Stdout
        } else if upper_line.contains("PRINTER") {
            OutputDestination::Printer
        } else {
            // Extraer nombre de archivo
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                OutputDestination::File(parts[1].to_string())
            } else {
                return Err(ParserError::syntax_error(
                    line_num,
                    1,
                    "OUTPUT TO requires destination",
                ));
            }
        };

        Ok(RqlStatement::OutputTo {
            destination,
            format,
        })
    }

    /// Parsear statement SQL
    fn parse_sql_statement(&self, line: &str, line_num: usize) -> ParserResult<RqlStatement> {
        // Validar que es SQL válido usando sqlparser
        let sql_ast = sqlparser::parser::Parser::new(&sqlparser::dialect::GenericDialect {})
            .try_with_sql(line)
            .map_err(|e| ParserError::SqlParserError(e.to_string()))?
            .parse_statements()
            .map_err(|e| ParserError::SqlParserError(e.to_string()))?;

        if sql_ast.is_empty() {
            return Err(ParserError::syntax_error(
                line_num,
                1,
                "Invalid SQL statement",
            ));
        }

        Ok(RqlStatement::Sql {
            sql: line.to_string(),
            parameters: HashMap::new(),
        })
    }

    /// Extraer parámetros de una línea
    fn extract_parameters(
        &self,
        line: &str,
        line_num: usize,
        ast: &mut RqlAst,
    ) -> ParserResult<()> {
        // Parámetros posicionados: $1, $2, etc.
        let positional_regex = Regex::new(r"\$(\d+)").unwrap();
        for cap in positional_regex.captures_iter(line) {
            let param_name = cap[0].to_string();
            let position = cap[1].parse::<usize>().unwrap_or(0);

            let parameter = RqlParameter {
                name: param_name.clone(),
                param_type: ParameterType::Positional,
                position: Some(position),
                line: line_num,
                column: line.find(&param_name).unwrap_or(0) + 1,
            };
            ast.add_parameter(parameter);
        }

        // Parámetros nombrados: :name
        let named_regex = Regex::new(r":([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
        for cap in named_regex.captures_iter(line) {
            let param_name = format!(":{}", &cap[1]);

            let parameter = RqlParameter {
                name: param_name,
                param_type: ParameterType::Named,
                position: None,
                line: line_num,
                column: line.find(&cap[0]).unwrap_or(0) + 1,
            };
            ast.add_parameter(parameter);
        }

        // Variables de sesión: #variable
        let session_var_regex = Regex::new(r"#([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
        for cap in session_var_regex.captures_iter(line) {
            ast.add_session_variable(cap[1].to_string());
        }

        Ok(())
    }

    /// Extraer parámetros de query SQL usando sqlparser
    pub fn extract_sql_parameters(&self, sql: &str) -> ParserResult<Vec<RqlParameter>> {
        let mut parameters = Vec::new();

        // Parsear SQL con sqlparser
        let mut parser = sqlparser::parser::Parser::new(&sqlparser::dialect::GenericDialect {})
            .try_with_sql(sql)
            .map_err(|e| ParserError::SqlParserError(e.to_string()))?;

        let sql_ast = parser
            .parse_statements()
            .map_err(|e| ParserError::SqlParserError(e.to_string()))?;

        if let Some(_statement) = sql_ast.first() {
            // Por ahora, usamos regex como fallback
            // En implementación futura, usar visitor pattern de sqlparser
            let named_regex = Regex::new(r"\$(\d+)").unwrap();
            for (i, cap) in named_regex.captures_iter(sql).enumerate() {
                parameters.push(RqlParameter {
                    name: cap[0].to_string(),
                    param_type: ParameterType::Positional,
                    position: Some(i + 1),
                    line: 1,
                    column: sql.find(&cap[0]).unwrap_or(0) + 1,
                });
            }

            let named_params_regex = Regex::new(r":([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
            for cap in named_params_regex.captures_iter(sql) {
                parameters.push(RqlParameter {
                    name: format!(":{}", &cap[1]),
                    param_type: ParameterType::Named,
                    position: None,
                    line: 1,
                    column: sql.find(&cap[0]).unwrap_or(0) + 1,
                });
            }
        }

        Ok(parameters)
    }
}

/// Configuración del parser
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Modo estricto (más validaciones)
    pub strict_mode: bool,

    /// Permitir comandos RQL extendidos
    pub allow_extended_commands: bool,

    /// Tiempo máximo de parsing
    pub max_parsing_time_ms: u64,

    /// Límite de caracteres por línea
    pub max_line_length: usize,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            strict_mode: false,
            allow_extended_commands: true,
            max_parsing_time_ms: 1000,
            max_line_length: 10000,
        }
    }
}

impl Default for RqlParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Procesador RQL que maneja parsing y post-procesamiento
#[derive(Debug)]
pub struct RqlProcessor {
    parser: RqlParser,
    #[allow(dead_code)]
    template_processor: TemplateProcessor,
}

impl Default for RqlProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl RqlProcessor {
    /// Crear nuevo procesador
    pub fn new() -> Self {
        Self {
            parser: RqlParser::new(),
            template_processor: TemplateProcessor::new(),
        }
    }

    /// Procesar input RQL completo
    pub async fn process(&self, input: &str) -> ParserResult<RqlAst> {
        let mut ast = self.parser.parse_rql(input).await?;

        // Post-procesar AST
        self.post_process_ast(&mut ast)?;

        Ok(ast)
    }

    /// Post-procesar AST (validaciones, optimizaciones)
    fn post_process_ast(&self, ast: &mut RqlAst) -> ParserResult<()> {
        // Validar parámetros duplicados
        self.validate_duplicate_parameters(ast)?;

        // Optimizar statements
        self.optimize_statements(ast)?;

        Ok(())
    }

    /// Validar parámetros duplicados
    fn validate_duplicate_parameters(&self, ast: &mut RqlAst) -> ParserResult<()> {
        let mut param_names = std::collections::HashSet::new();
        let mut duplicates = Vec::new();

        for param in &ast.parameters {
            if !param_names.insert(&param.name) {
                duplicates.push(param.name.clone());
            }
        }

        if !duplicates.is_empty() {
            ast.metadata.warnings.push(format!(
                "Duplicate parameters found: {}",
                duplicates.join(", ")
            ));
        }

        Ok(())
    }

    /// Optimizar statements
    fn optimize_statements(&self, ast: &mut RqlAst) -> ParserResult<()> {
        // Por ahora, solo logging
        log::debug!("Processing {} statements", ast.statements.len());
        Ok(())
    }
}

/// Procesador de templates simple
#[derive(Debug, Clone)]
pub struct TemplateProcessor {
    #[allow(dead_code)]
    config: TemplateConfig,
}

impl Default for TemplateProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateProcessor {
    /// Crear nuevo procesador de templates
    pub fn new() -> Self {
        Self {
            config: TemplateConfig::default(),
        }
    }

    /// Procesar template con variables
    pub fn process_template(&self, template: &str, variables: &HashMap<String, String>) -> String {
        let mut result = template.to_string();

        for (key, value) in variables {
            let placeholder = format!("#{}", key);
            result = result.replace(&placeholder, value);
        }

        result
    }
}

/// Configuración de templates
#[derive(Debug, Clone)]
pub struct TemplateConfig {
    /// Delimitador de apertura
    pub open_delimiter: String,

    /// Delimitador de cierre
    pub close_delimiter: String,

    /// Permitir variables de sesión
    pub allow_session_variables: bool,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            open_delimiter: "{{".to_string(),
            close_delimiter: "}}".to_string(),
            allow_session_variables: true,
        }
    }
}
