//! Parser principal para RQL (Extended SQL)

use crate::error::{ParserError, ParserResult};
use crate::rql_ast::{
    ExportFormat, MapExpression, OutputDestination, OutputFormat, ParameterType, RqlAst,
    RqlParameter, RqlStatement,
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

        // Buffer para acumular líneas de un statement multi-línea
        let mut statement_buffer = String::new();
        let mut statement_start_line = 0;

        // Procesar cada línea
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed_line = line.trim();

            // Saltar líneas vacías y comentarios
            if trimmed_line.is_empty() || trimmed_line.starts_with("--") {
                continue;
            }

            // Si el buffer está vacío, esta es la primera línea del statement
            if statement_buffer.is_empty() {
                statement_start_line = line_num + 1;
            }

            // Agregar línea al buffer (con espacio si no es la primera)
            if !statement_buffer.is_empty() {
                statement_buffer.push(' ');
            }
            statement_buffer.push_str(trimmed_line);

            // Si la línea termina con punto y coma, procesar el statement completo
            if trimmed_line.ends_with(';') {
                // Remover el punto y coma final para procesamiento
                let statement_text = statement_buffer.trim_end_matches(';').trim();

                // Parsear statement completo
                match self.parse_line(statement_text, statement_start_line) {
                    Ok(statement) => {
                        ast.add_statement(statement);
                        // Extraer parámetros del statement completo
                        self.extract_parameters(statement_text, statement_start_line, &mut ast)?;
                    }
                    Err(e) => {
                        return Err(ParserError::syntax_error(
                            statement_start_line,
                            1,
                            format!("Failed to parse line: {}", e),
                        ));
                    }
                }

                // Limpiar buffer para el próximo statement
                statement_buffer.clear();
            }
        }

        // Si queda algo en el buffer (statement sin ';'), procesarlo también
        if !statement_buffer.is_empty() {
            let statement_text = statement_buffer.trim();
            match self.parse_line(statement_text, statement_start_line) {
                Ok(statement) => {
                    ast.add_statement(statement);
                    self.extract_parameters(statement_text, statement_start_line, &mut ast)?;
                }
                Err(e) => {
                    return Err(ParserError::syntax_error(
                        statement_start_line,
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

        // Detectar comandos NQL (comandos nuevos multi-fuente)
        if upper_line.starts_with("SHOW SOURCES") {
            self.parse_show_sources_command(line, line_num)
        } else if upper_line.starts_with("SHOW TABLES") {
            self.parse_show_tables_command(line, line_num)
        } else if upper_line.starts_with("SHOW VARS") {
            self.parse_show_vars_command(line, line_num)
        } else if upper_line.starts_with("DESCRIBE ") {
            self.parse_describe_command(line, line_num)
        } else if upper_line.starts_with("IMPORT ") {
            self.parse_import_command(line, line_num)
        } else if upper_line.starts_with("EXPORT ") {
            self.parse_export_command(line, line_num)
        } else if upper_line.starts_with("MAP ") {
            self.parse_map_command(line, line_num)
        } else if upper_line.starts_with("FILTER ") {
            self.parse_filter_command(line, line_num)
        } else if upper_line.starts_with("UNSET ") {
            self.parse_unset_command(line, line_num)
        } else if upper_line.starts_with("USE ") {
            // Diferenciar entre USE schema y USE 'file' AS alias
            if line.contains('\'') || line.contains('\"') {
                self.parse_use_source_command(line, line_num)
            } else {
                self.parse_use_command(line, line_num)
            }
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
        // LET variable = expression
        let upper_line = line.to_uppercase();
        if !upper_line.contains(" = ") && !upper_line.contains("=") {
            return Err(ParserError::syntax_error(
                line_num,
                1,
                "LET command requires format: LET variable = expression",
            ));
        }

        // Find the equals sign
        let eq_pos = line.find('=').unwrap();
        let before_eq = &line[..eq_pos].trim();
        let after_eq = &line[eq_pos + 1..].trim();

        // Extract variable name (skip "LET")
        let variable = before_eq
            .strip_prefix("LET ")
            .or_else(|| before_eq.strip_prefix("let "))
            .ok_or_else(|| ParserError::syntax_error(line_num, 1, "LET command malformed"))?
            .trim()
            .to_string();

        let expression = after_eq.to_string();

        Ok(RqlStatement::Let {
            variable,
            expression,
        })
    }

    /// Parsear comando FORM LOAD
    fn parse_form_load_command(&self, line: &str, line_num: usize) -> ParserResult<RqlStatement> {
        // FORM LOAD 'file.toml'
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            return Err(ParserError::syntax_error(
                line_num,
                1,
                "FORM LOAD command requires file path",
            ));
        }

        // parts[0] = "FORM", parts[1] = "LOAD", parts[2] = file path (keep quotes)
        let form_path = parts[2].to_string();
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

    /// Parsear comando USE SOURCE (NQL)
    /// Sintaxis: USE 'path' [AS alias] [OPTIONS (key=value, ...)]
    fn parse_use_source_command(&self, line: &str, line_num: usize) -> ParserResult<RqlStatement> {
        let upper_line = line.to_uppercase();

        // Extraer path (entre comillas)
        let path = if let Some(start) = line.find('\'') {
            if let Some(end) = line[start + 1..].find('\'') {
                line[start + 1..start + 1 + end].to_string()
            } else {
                return Err(ParserError::syntax_error(
                    line_num,
                    start + 1,
                    "Unclosed quote in USE command",
                ));
            }
        } else if let Some(start) = line.find('\"') {
            if let Some(end) = line[start + 1..].find('\"') {
                line[start + 1..start + 1 + end].to_string()
            } else {
                return Err(ParserError::syntax_error(
                    line_num,
                    start + 1,
                    "Unclosed quote in USE command",
                ));
            }
        } else {
            return Err(ParserError::syntax_error(
                line_num,
                1,
                "USE SOURCE command requires quoted path",
            ));
        };

        // Extraer alias (opcional)
        let alias = if upper_line.contains(" AS ") {
            // Find the position of " AS " in the uppercase version
            if let Some(as_pos) = upper_line.find(" AS ") {
                // Use the same position in the original line
                let after_as = &line[as_pos + 4..]; // Skip " AS "
                let alias_part = after_as.trim();
                let alias_end = alias_part
                    .find(" OPTIONS")
                    .or_else(|| alias_part.find(';'))
                    .unwrap_or(alias_part.len());
                Some(alias_part[..alias_end].trim().to_string())
            } else {
                None
            }
        } else {
            None
        };

        // Extraer options (opcional)
        let options = if upper_line.contains(" OPTIONS ") {
            self.parse_options(line, line_num)?
        } else {
            HashMap::new()
        };

        Ok(RqlStatement::UseSource {
            path,
            alias,
            options,
        })
    }

    /// Parsear comando SHOW SOURCES
    fn parse_show_sources_command(
        &self,
        _line: &str,
        _line_num: usize,
    ) -> ParserResult<RqlStatement> {
        Ok(RqlStatement::ShowSources)
    }

    /// Parsear comando SHOW TABLES
    /// Sintaxis: SHOW TABLES [FROM source]
    fn parse_show_tables_command(&self, line: &str, _line_num: usize) -> ParserResult<RqlStatement> {
        let upper_line = line.to_uppercase();
        let source = if upper_line.contains(" FROM ") {
            let parts: Vec<&str> = line.splitn(2, " FROM ").collect();
            if parts.len() == 2 {
                Some(parts[1].trim().trim_end_matches(';').to_string())
            } else {
                None
            }
        } else {
            None
        };

        Ok(RqlStatement::ShowTables { source })
    }

    /// Parsear comando SHOW VARS
    fn parse_show_vars_command(
        &self,
        _line: &str,
        _line_num: usize,
    ) -> ParserResult<RqlStatement> {
        Ok(RqlStatement::ShowVars)
    }

    /// Parsear comando DESCRIBE
    /// Sintaxis: DESCRIBE [source.]table
    fn parse_describe_command(&self, line: &str, line_num: usize) -> ParserResult<RqlStatement> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(ParserError::syntax_error(
                line_num,
                1,
                "DESCRIBE command requires table name",
            ));
        }

        let table_spec = parts[1].trim_end_matches(';');
        let (source, table) = if table_spec.contains('.') {
            let spec_parts: Vec<&str> = table_spec.splitn(2, '.').collect();
            (Some(spec_parts[0].to_string()), spec_parts[1].to_string())
        } else {
            (None, table_spec.to_string())
        };

        Ok(RqlStatement::Describe { source, table })
    }

    /// Parsear comando IMPORT
    /// Sintaxis: IMPORT 'file' AS table [OPTIONS (key=value, ...)]
    fn parse_import_command(&self, line: &str, line_num: usize) -> ParserResult<RqlStatement> {
        let upper_line = line.to_uppercase();

        // Extraer file (entre comillas)
        let file = if let Some(start) = line.find('\'') {
            if let Some(end) = line[start + 1..].find('\'') {
                line[start + 1..start + 1 + end].to_string()
            } else {
                return Err(ParserError::syntax_error(
                    line_num,
                    start + 1,
                    "Unclosed quote in IMPORT command",
                ));
            }
        } else {
            return Err(ParserError::syntax_error(
                line_num,
                1,
                "IMPORT command requires quoted file path",
            ));
        };

        // Extraer table name
        let table = if upper_line.contains(" AS ") {
            let parts: Vec<&str> = line.splitn(2, " AS ").collect();
            if parts.len() == 2 {
                let table_part = parts[1].trim();
                let table_end = table_part
                    .find(" OPTIONS")
                    .or_else(|| table_part.find(';'))
                    .unwrap_or(table_part.len());
                table_part[..table_end].trim().to_string()
            } else {
                return Err(ParserError::syntax_error(
                    line_num,
                    1,
                    "IMPORT command requires AS clause",
                ));
            }
        } else {
            return Err(ParserError::syntax_error(
                line_num,
                1,
                "IMPORT command requires AS clause",
            ));
        };

        // Extraer options (opcional)
        let options = if upper_line.contains(" OPTIONS ") {
            self.parse_options(line, line_num)?
        } else {
            HashMap::new()
        };

        Ok(RqlStatement::Import {
            file,
            table,
            options,
        })
    }

    /// Parsear comando EXPORT
    /// Sintaxis: EXPORT query/table TO 'file' FORMAT format [OPTIONS (key=value, ...)]
    fn parse_export_command(&self, line: &str, line_num: usize) -> ParserResult<RqlStatement> {
        let upper_line = line.to_uppercase();

        // Extraer query (entre EXPORT y TO)
        let query = if let Some(to_pos) = upper_line.find(" TO ") {
            line[7..to_pos].trim().to_string() // 7 = len("EXPORT ")
        } else {
            return Err(ParserError::syntax_error(
                line_num,
                1,
                "EXPORT command requires TO clause",
            ));
        };

        // Extraer file (entre comillas después de TO)
        let file = if let Some(to_pos) = line.to_uppercase().find(" TO ") {
            let after_to = &line[to_pos + 4..]; // 4 = len(" TO ")
            if let Some(start) = after_to.find('\'') {
                if let Some(end) = after_to[start + 1..].find('\'') {
                    after_to[start + 1..start + 1 + end].to_string()
                } else {
                    return Err(ParserError::syntax_error(
                        line_num,
                        1,
                        "Unclosed quote in EXPORT command",
                    ));
                }
            } else {
                return Err(ParserError::syntax_error(
                    line_num,
                    1,
                    "EXPORT TO requires quoted file path",
                ));
            }
        } else {
            return Err(ParserError::syntax_error(
                line_num,
                1,
                "EXPORT command requires TO clause",
            ));
        };

        // Extraer format
        let format = if upper_line.contains(" FORMAT CSV") {
            ExportFormat::Csv
        } else if upper_line.contains(" FORMAT JSON") {
            ExportFormat::Json
        } else if upper_line.contains(" FORMAT XLSX") {
            ExportFormat::Xlsx
        } else {
            return Err(ParserError::syntax_error(
                line_num,
                1,
                "EXPORT command requires FORMAT clause (CSV, JSON, or XLSX)",
            ));
        };

        // Extraer options (opcional)
        let options = if upper_line.contains(" OPTIONS ") {
            self.parse_options(line, line_num)?
        } else {
            HashMap::new()
        };

        Ok(RqlStatement::Export {
            query,
            file,
            format,
            options,
        })
    }

    /// Parsear comando MAP
    /// Sintaxis: MAP expression1 [AS alias1], expression2 [AS alias2], ...
    fn parse_map_command(&self, line: &str, line_num: usize) -> ParserResult<RqlStatement> {
        let map_part = line[4..].trim().trim_end_matches(';'); // 4 = len("MAP ")

        let mut expressions = Vec::new();
        for expr_str in map_part.split(',') {
            let trimmed = expr_str.trim();
            let upper_trimmed = trimmed.to_uppercase();

            let (expression, alias) = if upper_trimmed.contains(" AS ") {
                let parts: Vec<&str> = trimmed.splitn(2, " AS ").collect();
                if parts.len() == 2 {
                    (parts[0].trim().to_string(), Some(parts[1].trim().to_string()))
                } else {
                    (trimmed.to_string(), None)
                }
            } else {
                (trimmed.to_string(), None)
            };

            if expression.is_empty() {
                return Err(ParserError::syntax_error(
                    line_num,
                    1,
                    "MAP command requires at least one expression",
                ));
            }

            expressions.push(MapExpression { expression, alias });
        }

        if expressions.is_empty() {
            return Err(ParserError::syntax_error(
                line_num,
                1,
                "MAP command requires at least one expression",
            ));
        }

        Ok(RqlStatement::Map { expressions })
    }

    /// Parsear comando FILTER
    /// Sintaxis: FILTER condition
    fn parse_filter_command(&self, line: &str, line_num: usize) -> ParserResult<RqlStatement> {
        let condition = line[7..].trim().trim_end_matches(';').to_string(); // 7 = len("FILTER ")

        if condition.is_empty() {
            return Err(ParserError::syntax_error(
                line_num,
                1,
                "FILTER command requires condition",
            ));
        }

        Ok(RqlStatement::Filter { condition })
    }

    /// Parsear comando UNSET
    /// Sintaxis: UNSET variable1, variable2, ...
    fn parse_unset_command(&self, line: &str, line_num: usize) -> ParserResult<RqlStatement> {
        let vars_part = line[6..].trim().trim_end_matches(';'); // 6 = len("UNSET ")

        let variables: Vec<String> = vars_part
            .split(',')
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .collect();

        if variables.is_empty() {
            return Err(ParserError::syntax_error(
                line_num,
                1,
                "UNSET command requires at least one variable",
            ));
        }

        Ok(RqlStatement::Unset { variables })
    }

    /// Parsear sección OPTIONS
    /// Sintaxis: OPTIONS (key1=value1, key2=value2, ...)
    /// Soporta valores entre comillas: OPTIONS (delimiter=',', header=true)
    fn parse_options(&self, line: &str, line_num: usize) -> ParserResult<HashMap<String, String>> {
        let mut options = HashMap::new();

        if let Some(options_start) = line.to_uppercase().find(" OPTIONS (") {
            let after_options = &line[options_start + 10..]; // 10 = len(" OPTIONS (")
            if let Some(options_end) = after_options.find(')') {
                let options_str = &after_options[..options_end];

                // Parsear key=value pairs considerando comillas
                let pairs = self.split_options(options_str);

                for opt_pair in pairs {
                    let trimmed = opt_pair.trim();
                    // Skip empty parts
                    if trimmed.is_empty() {
                        continue;
                    }
                    if let Some(eq_pos) = trimmed.find('=') {
                        let key = trimmed[..eq_pos].trim().to_string();
                        let mut value = trimmed[eq_pos + 1..].trim().to_string();

                        // Remover comillas si existen
                        if (value.starts_with('\'') && value.ends_with('\'')) ||
                           (value.starts_with('"') && value.ends_with('"')) {
                            value = value[1..value.len()-1].to_string();
                        }

                        options.insert(key, value);
                    } else {
                        return Err(ParserError::syntax_error(
                            line_num,
                            1,
                            "Invalid OPTIONS format, expected key=value",
                        ));
                    }
                }
            } else {
                return Err(ParserError::syntax_error(
                    line_num,
                    1,
                    "Unclosed OPTIONS parenthesis",
                ));
            }
        }

        Ok(options)
    }

    /// Dividir string de opciones respetando comillas
    fn split_options(&self, options_str: &str) -> Vec<String> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut quote_char = ' ';

        for ch in options_str.chars() {
            match ch {
                '\'' | '"' if !in_quotes => {
                    in_quotes = true;
                    quote_char = ch;
                    current.push(ch);
                }
                c if in_quotes && c == quote_char => {
                    in_quotes = false;
                    current.push(ch);
                }
                ',' if !in_quotes => {
                    if !current.trim().is_empty() {
                        parts.push(current.trim().to_string());
                    }
                    current.clear();
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        // No olvidar la última parte
        if !current.trim().is_empty() {
            parts.push(current.trim().to_string());
        }

        parts
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

        // Validar comandos NQL
        self.validate_nql_commands(ast)?;

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

    /// Validar comandos NQL específicos
    fn validate_nql_commands(&self, ast: &mut RqlAst) -> ParserResult<()> {
        use std::collections::HashSet;

        let mut source_aliases = HashSet::new();

        for statement in &ast.statements {
            match statement {
                RqlStatement::UseSource { path, alias, .. } => {
                    // Validar que el path no esté vacío
                    if path.is_empty() {
                        ast.metadata.warnings.push(
                            "USE SOURCE: Empty path provided".to_string()
                        );
                    }

                    // Validar alias duplicado
                    if let Some(alias_name) = alias {
                        if !source_aliases.insert(alias_name.clone()) {
                            ast.metadata.warnings.push(format!(
                                "USE SOURCE: Duplicate alias '{}'",
                                alias_name
                            ));
                        }

                        // Validar que el alias sea un identificador válido
                        if !Self::is_valid_identifier(alias_name) {
                            ast.metadata.warnings.push(format!(
                                "USE SOURCE: Invalid alias '{}' (must be alphanumeric)",
                                alias_name
                            ));
                        }
                    }
                }

                RqlStatement::Import { file, table, .. } => {
                    // Validar que el archivo no esté vacío
                    if file.is_empty() {
                        ast.metadata.warnings.push(
                            "IMPORT: Empty file path provided".to_string()
                        );
                    }

                    // Validar que el nombre de tabla sea válido
                    if table.is_empty() || !Self::is_valid_identifier(table) {
                        ast.metadata.warnings.push(format!(
                            "IMPORT: Invalid table name '{}'",
                            table
                        ));
                    }
                }

                RqlStatement::Export { query, file, .. } => {
                    // Validar que la query no esté vacía
                    if query.is_empty() {
                        ast.metadata.warnings.push(
                            "EXPORT: Empty query/table provided".to_string()
                        );
                    }

                    // Validar que el archivo no esté vacío
                    if file.is_empty() {
                        ast.metadata.warnings.push(
                            "EXPORT: Empty file path provided".to_string()
                        );
                    }
                }

                RqlStatement::Describe { table, .. } => {
                    // Validar que el nombre de tabla sea válido
                    if table.is_empty() || !Self::is_valid_identifier(table) {
                        ast.metadata.warnings.push(format!(
                            "DESCRIBE: Invalid table name '{}'",
                            table
                        ));
                    }
                }

                RqlStatement::Unset { variables } => {
                    // Validar que haya al menos una variable
                    if variables.is_empty() {
                        ast.metadata.warnings.push(
                            "UNSET: No variables specified".to_string()
                        );
                    }

                    // Validar que las variables sean identificadores válidos
                    for var in variables {
                        if !Self::is_valid_identifier(var) {
                            ast.metadata.warnings.push(format!(
                                "UNSET: Invalid variable name '{}'",
                                var
                            ));
                        }
                    }
                }

                RqlStatement::Map { expressions } => {
                    // Validar que haya al menos una expresión
                    if expressions.is_empty() {
                        ast.metadata.warnings.push(
                            "MAP: No expressions provided".to_string()
                        );
                    }
                }

                RqlStatement::Filter { condition } => {
                    // Validar que la condición no esté vacía
                    if condition.trim().is_empty() {
                        ast.metadata.warnings.push(
                            "FILTER: Empty condition provided".to_string()
                        );
                    }
                }

                _ => {
                    // Otros statements no requieren validación NQL específica
                }
            }
        }

        Ok(())
    }

    /// Validar si un string es un identificador válido
    fn is_valid_identifier(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        // Primer carácter debe ser letra o underscore
        let first_char = name.chars().next().unwrap();
        if !first_char.is_alphabetic() && first_char != '_' {
            return false;
        }

        // Resto de caracteres deben ser alfanuméricos o underscore
        name.chars().all(|c| c.is_alphanumeric() || c == '_')
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
