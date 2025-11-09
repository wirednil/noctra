mod parser_tests {
    use super::*;
    use crate::parser::RqlParser;
    use crate::rql_ast::{RqlAst, RqlStatement, RqlParameter, ParameterType};

    #[tokio::test]
    async fn test_parse_simple_select() {
        let parser = RqlParser::new();
        let input = "SELECT * FROM employees";
        
        let ast = parser.parse_rql(input).await.unwrap();
        
        assert_eq!(ast.statements.len(), 1);
        assert!(matches!(ast.statements[0], RqlStatement::Sql { .. }));
        
        if let RqlStatement::Sql { sql, .. } = &ast.statements[0] {
            assert_eq!(sql, "SELECT * FROM employees");
        }
    }

    #[tokio::test]
    async fn test_parse_select_with_named_parameter() {
        let parser = RqlParser::new();
        let input = "SELECT * FROM employees WHERE dept = :dept";
        
        let ast = parser.parse_rql(input).await.unwrap();
        
        assert_eq!(ast.statements.len(), 1);
        assert_eq!(ast.parameters.len(), 1);
        
        let param = &ast.parameters[0];
        assert_eq!(param.name, ":dept");
        assert_eq!(param.param_type, ParameterType::Named);
    }

    #[tokio::test]
    async fn test_parse_select_with_positional_parameter() {
        let parser = RqlParser::new();
        let input = "SELECT * FROM employees WHERE dept = $1 AND salario > $2";
        
        let ast = parser.parse_rql(input).await.unwrap();
        
        assert_eq!(ast.statements.len(), 1);
        assert_eq!(ast.parameters.len(), 2);
        
        assert_eq!(ast.parameters[0].name, "$1");
        assert_eq!(ast.parameters[1].name, "$2");
        assert_eq!(ast.parameters[0].param_type, ParameterType::Positional);
        assert_eq!(ast.parameters[1].param_type, ParameterType::Positional);
    }

    #[tokio::test]
    async fn test_parse_use_command() {
        let parser = RqlParser::new();
        let input = "USE payroll";
        
        let ast = parser.parse_rql(input).await.unwrap();
        
        assert_eq!(ast.statements.len(), 1);
        assert!(matches!(ast.statements[0], RqlStatement::Use { .. }));
        
        if let RqlStatement::Use { schema } = &ast.statements[0] {
            assert_eq!(schema, "payroll");
        }
    }

    #[tokio::test]
    async fn test_parse_let_command() {
        let parser = RqlParser::new();
        let input = "LET dept = 'SALES'";
        
        let ast = parser.parse_rql(input).await.unwrap();
        
        assert_eq!(ast.statements.len(), 1);
        assert!(matches!(ast.statements[0], RqlStatement::Let { .. }));
        
        if let RqlStatement::Let { variable, expression } = &ast.statements[0] {
            assert_eq!(variable, "dept");
            assert_eq!(expression, "'SALES'");
        }
    }

    #[tokio::test]
    async fn test_parse_form_load_command() {
        let parser = RqlParser::new();
        let input = "FORM LOAD 'empleados.toml'";
        
        let ast = parser.parse_rql(input).await.unwrap();
        
        assert_eq!(ast.statements.len(), 1);
        assert!(matches!(ast.statements[0], RqlStatement::FormLoad { .. }));
        
        if let RqlStatement::FormLoad { form_path } = &ast.statements[0] {
            assert_eq!(form_path, "'empleados.toml'");
        }
    }

    #[tokio::test]
    async fn test_parse_output_to_command() {
        let parser = RqlParser::new();
        let input = "OUTPUT TO 'reporte.csv' FORMAT csv";
        
        let ast = parser.parse_rql(input).await.unwrap();
        
        assert_eq!(ast.statements.len(), 1);
        assert!(matches!(ast.statements[0], RqlStatement::OutputTo { .. }));
    }

    #[tokio::test]
    async fn test_parse_multiple_statements() {
        let parser = RqlParser::new();
        let input = r#"
        USE payroll;
        LET dept = 'IT';
        SELECT * FROM employees WHERE dept = :dept;
        "#;
        
        let ast = parser.parse_rql(input).await.unwrap();
        
        assert_eq!(ast.statements.len(), 3);
        
        // Verificar parámetros extraídos
        assert_eq!(ast.parameters.len(), 1);
        assert_eq!(ast.parameters[0].name, ":dept");
    }

    #[tokio::test]
    async fn test_extract_parameters() {
        let parser = RqlParser::new();
        let sql = "SELECT * FROM employees WHERE dept = $1 AND nombre = :nombre";
        
        let params = parser.extract_sql_parameters(sql).unwrap();
        
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name, "$1");
        assert_eq!(params[1].name, ":nombre");
    }

    #[tokio::test]
    async fn test_session_variables() {
        let parser = RqlParser::new();
        let input = "SELECT * FROM employees WHERE dept = #dept_var";
        
        let ast = parser.parse_rql(input).await.unwrap();
        
        assert_eq!(ast.session_variables.len(), 1);
        assert_eq!(ast.session_variables[0], "dept_var");
    }

    #[test]
    fn test_ast_debug_info() {
        let ast = RqlAst::new();
        let debug_info = ast.debug_info();
        
        assert_eq!(debug_info.statement_count, 0);
        assert_eq!(debug_info.parameter_count, 0);
        assert_eq!(debug_info.session_var_count, 0);
        assert!(!debug_info.has_sql_statements);
        assert!(!debug_info.has_commands);
    }

    #[test]
    fn test_rql_statement_type() {
        let stmt = RqlStatement::Sql { 
            sql: "SELECT 1".to_string(), 
            parameters: std::collections::HashMap::new() 
        };
        
        assert_eq!(stmt.statement_type(), "SQL");
        assert!(stmt.is_sql());
        assert!(!stmt.is_command());
        
        let stmt2 = RqlStatement::Use { schema: "test".to_string() };
        assert_eq!(stmt2.statement_type(), "USE");
        assert!(!stmt2.is_sql());
        assert!(stmt2.is_command());
    }

    #[test]
    fn test_to_sql() {
        let mut ast = RqlAst::new();
        
        ast.add_statement(RqlStatement::Use { 
            schema: "payroll".to_string() 
        });
        
        ast.add_statement(RqlStatement::Sql {
            sql: "SELECT * FROM employees".to_string(),
            parameters: std::collections::HashMap::new(),
        });
        
        let sql = ast.to_sql();
        assert!(sql.contains("USE payroll;"));
        assert!(sql.contains("SELECT * FROM employees"));
    }

    #[tokio::test]
    async fn test_empty_input() {
        let parser = RqlParser::new();
        let input = "";
        
        let ast = parser.parse_rql(input).await.unwrap();
        
        assert_eq!(ast.statements.len(), 0);
        assert_eq!(ast.parameters.len(), 0);
    }

    #[tokio::test]
    async fn test_comments_and_empty_lines() {
        let parser = RqlParser::new();
        let input = r#"
        -- Comentario en SQL
        SELECT * FROM employees;
        
        -- Otro comentario
        USE payroll;
        
        "#;
        
        let ast = parser.parse_rql(input).await.unwrap();
        
        assert_eq!(ast.statements.len(), 2);
    }

    #[test]
    fn test_ast_default() {
        let ast = RqlAst::default();
        
        assert!(ast.statements.is_empty());
        assert!(ast.parameters.is_empty());
        assert!(ast.session_variables.is_empty());
        assert!(ast.metadata.parsing_time_us >= 0);
    }
}

mod template_tests {
    use crate::template::*;
    use std::collections::HashMap;

    #[test]
    fn test_simple_variable_replacement() {
        let engine = TemplateEngine::new();
        let template = "SELECT * FROM #table WHERE dept = '#dept'";
        let mut variables = HashMap::new();
        variables.insert("table".to_string(), "employees".to_string());
        variables.insert("dept".to_string(), "IT".to_string());
        
        let result = engine.process(template, &variables).unwrap();
        
        assert!(result.contains("FROM employees"));
        assert!(result.contains("WHERE dept = 'IT'"));
    }

    #[test]
    fn test_conditional_if() {
        let engine = TemplateEngine::new();
        let template = "SELECT * FROM employees{{#if dept}} WHERE dept = '#dept'{{/if}}";
        let mut variables = HashMap::new();
        variables.insert("dept".to_string(), "IT".to_string());
        
        let result = engine.process(template, &variables).unwrap();
        
        assert!(result.contains("WHERE dept = 'IT'"));
    }

    #[test]
    fn test_conditional_unless() {
        let engine = TemplateEngine::new();
        let template = "SELECT * FROM employees{{#unless activo}} WHERE activo = 0{{/unless}}";
        let variables = HashMap::new(); // Sin variable 'activo'
        
        let result = engine.process(template, &variables).unwrap();
        
        assert!(result.contains("WHERE activo = 0"));
    }

    #[test]
    fn test_template_validation() {
        let engine = TemplateEngine::new();
        
        // Template válido
        assert!(engine.validate_template("{{#if var}}content{{/if}}").is_ok());
        
        // Template inválido (llaves desbalanceadas)
        assert!(engine.validate_template("{{#if var}}content").is_err());
        
        // Template con condicionales desbalanceadas
        assert!(engine.validate_template("{{#if var}}content{{/if}}{{#if}}").is_err());
    }

    #[test]
    fn test_template_utils() {
        use crate::template::utils::*;

        let mut map = HashMap::new();
        map.insert("dept".to_string(), "IT".to_string());
        map.insert("activo".to_string(), "true".to_string());

        let session_vars = session_vars_from_map(&map);

        // session_vars_from_map should preserve keys (no # prefix)
        assert_eq!(session_vars.get("dept"), Some(&"IT".to_string()));
        assert_eq!(session_vars.get("activo"), Some(&"true".to_string()));
    }

    #[test]
    fn test_sql_template_processing() {
        use crate::template::utils::*;
        
        let template = "SELECT * FROM #table WHERE dept = #dept AND activo = #activo";
        let mut variables = HashMap::new();
        variables.insert("table".to_string(), "employees".to_string());
        variables.insert("dept".to_string(), "'IT'".to_string());
        variables.insert("activo".to_string(), "1".to_string());
        
        let result = process_sql_template(template, &variables);
        
        assert!(result.contains("FROM employees"));
        assert!(result.contains("WHERE dept = 'IT'"));
        assert!(result.contains("AND activo = 1"));
    }
}

mod error_tests {
    use crate::error::*;

    #[test]
    fn test_parser_error_creation() {
        let error = ParserError::syntax_error(1, 10, "Invalid syntax");
        assert!(matches!(error, ParserError::SyntaxError { .. }));
        
        let error2 = ParserError::unknown_command("UNKNOWN_CMD");
        assert!(matches!(error2, ParserError::UnknownCommand { .. }));
        
        let error3 = ParserError::template_error("Template error");
        assert!(matches!(error3, ParserError::TemplateError { .. }));
    }

    #[test]
    fn test_error_display() {
        let error = ParserError::syntax_error(5, 15, "Missing FROM clause");
        let error_str = error.to_string();

        assert!(error_str.contains("línea 5"));
        assert!(error_str.contains("columna 15"));
        assert!(error_str.contains("Missing FROM clause"));
    }
}

mod nql_parser_tests {
    use super::*;
    use crate::parser::RqlParser;
    use crate::rql_ast::{ExportFormat, MapExpression, RqlStatement};

    #[tokio::test]
    async fn test_parse_use_source_basic() {
        let parser = RqlParser::new();
        let input = "USE 'clientes.csv' AS csv";

        let ast = parser.parse_rql(input).await.unwrap();

        assert_eq!(ast.statements.len(), 1);
        assert!(matches!(ast.statements[0], RqlStatement::UseSource { .. }));

        if let RqlStatement::UseSource { path, alias, options } = &ast.statements[0] {
            assert_eq!(path, "clientes.csv");
            assert_eq!(alias, &Some("csv".to_string()));
            assert!(options.is_empty());
        }
    }

    #[tokio::test]
    async fn test_parse_use_source_with_options() {
        let parser = RqlParser::new();
        let input = "USE 'data.csv' AS mydata OPTIONS (delimiter=;, has_header=true)";

        let ast = parser.parse_rql(input).await.unwrap();

        assert_eq!(ast.statements.len(), 1);

        if let RqlStatement::UseSource { path, alias, options } = &ast.statements[0] {
            assert_eq!(path, "data.csv");
            assert_eq!(alias, &Some("mydata".to_string()));
            assert_eq!(options.get("delimiter"), Some(&";".to_string()));
            assert_eq!(options.get("has_header"), Some(&"true".to_string()));
        }
    }

    #[tokio::test]
    async fn test_parse_use_source_without_alias() {
        let parser = RqlParser::new();
        let input = "USE 'database.db'";

        let ast = parser.parse_rql(input).await.unwrap();

        assert_eq!(ast.statements.len(), 1);

        if let RqlStatement::UseSource { path, alias, options } = &ast.statements[0] {
            assert_eq!(path, "database.db");
            assert_eq!(alias, &None);
            assert!(options.is_empty());
        }
    }

    #[tokio::test]
    async fn test_parse_show_sources() {
        let parser = RqlParser::new();
        let input = "SHOW SOURCES";

        let ast = parser.parse_rql(input).await.unwrap();

        assert_eq!(ast.statements.len(), 1);
        assert!(matches!(ast.statements[0], RqlStatement::ShowSources));
    }

    #[tokio::test]
    async fn test_parse_show_tables_without_source() {
        let parser = RqlParser::new();
        let input = "SHOW TABLES";

        let ast = parser.parse_rql(input).await.unwrap();

        assert_eq!(ast.statements.len(), 1);

        if let RqlStatement::ShowTables { source } = &ast.statements[0] {
            assert_eq!(source, &None);
        }
    }

    #[tokio::test]
    async fn test_parse_show_tables_with_source() {
        let parser = RqlParser::new();
        let input = "SHOW TABLES FROM csv";

        let ast = parser.parse_rql(input).await.unwrap();

        assert_eq!(ast.statements.len(), 1);

        if let RqlStatement::ShowTables { source } = &ast.statements[0] {
            assert_eq!(source, &Some("csv".to_string()));
        }
    }

    #[tokio::test]
    async fn test_parse_show_vars() {
        let parser = RqlParser::new();
        let input = "SHOW VARS";

        let ast = parser.parse_rql(input).await.unwrap();

        assert_eq!(ast.statements.len(), 1);
        assert!(matches!(ast.statements[0], RqlStatement::ShowVars));
    }

    #[tokio::test]
    async fn test_parse_describe_table() {
        let parser = RqlParser::new();
        let input = "DESCRIBE employees";

        let ast = parser.parse_rql(input).await.unwrap();

        assert_eq!(ast.statements.len(), 1);

        if let RqlStatement::Describe { source, table } = &ast.statements[0] {
            assert_eq!(source, &None);
            assert_eq!(table, "employees");
        }
    }

    #[tokio::test]
    async fn test_parse_describe_source_table() {
        let parser = RqlParser::new();
        let input = "DESCRIBE csv.clientes";

        let ast = parser.parse_rql(input).await.unwrap();

        assert_eq!(ast.statements.len(), 1);

        if let RqlStatement::Describe { source, table } = &ast.statements[0] {
            assert_eq!(source, &Some("csv".to_string()));
            assert_eq!(table, "clientes");
        }
    }

    #[tokio::test]
    async fn test_parse_import_basic() {
        let parser = RqlParser::new();
        let input = "IMPORT 'datos.csv' AS staging";

        let ast = parser.parse_rql(input).await.unwrap();

        assert_eq!(ast.statements.len(), 1);

        if let RqlStatement::Import { file, table, options } = &ast.statements[0] {
            assert_eq!(file, "datos.csv");
            assert_eq!(table, "staging");
            assert!(options.is_empty());
        }
    }

    #[tokio::test]
    async fn test_parse_import_with_options() {
        let parser = RqlParser::new();
        let input = "IMPORT 'data.csv' AS temp OPTIONS (delimiter=;, encoding=utf-8)";

        let ast = parser.parse_rql(input).await.unwrap();

        assert_eq!(ast.statements.len(), 1);

        if let RqlStatement::Import { file, table, options } = &ast.statements[0] {
            assert_eq!(file, "data.csv");
            assert_eq!(table, "temp");
            assert_eq!(options.get("delimiter"), Some(&";".to_string()));
            assert_eq!(options.get("encoding"), Some(&"utf-8".to_string()));
        }
    }

    #[tokio::test]
    async fn test_parse_export_csv() {
        let parser = RqlParser::new();
        let input = "EXPORT employees TO 'export.csv' FORMAT CSV";

        let ast = parser.parse_rql(input).await.unwrap();

        assert_eq!(ast.statements.len(), 1);

        if let RqlStatement::Export { query, file, format, options } = &ast.statements[0] {
            assert_eq!(query, "employees");
            assert_eq!(file, "export.csv");
            assert!(matches!(format, ExportFormat::Csv));
            assert!(options.is_empty());
        }
    }

    #[tokio::test]
    async fn test_parse_export_json() {
        let parser = RqlParser::new();
        let input = "EXPORT (SELECT * FROM users) TO 'output.json' FORMAT JSON";

        let ast = parser.parse_rql(input).await.unwrap();

        assert_eq!(ast.statements.len(), 1);

        if let RqlStatement::Export { query, file, format, options } = &ast.statements[0] {
            assert_eq!(query, "(SELECT * FROM users)");
            assert_eq!(file, "output.json");
            assert!(matches!(format, ExportFormat::Json));
            assert!(options.is_empty());
        }
    }

    #[tokio::test]
    async fn test_parse_export_with_options() {
        let parser = RqlParser::new();
        let input = "EXPORT data TO 'file.csv' FORMAT CSV OPTIONS (delimiter=;, header=true)";

        let ast = parser.parse_rql(input).await.unwrap();

        assert_eq!(ast.statements.len(), 1);

        if let RqlStatement::Export { query, file, format, options } = &ast.statements[0] {
            assert_eq!(query, "data");
            assert_eq!(file, "file.csv");
            assert!(matches!(format, ExportFormat::Csv));
            assert_eq!(options.get("delimiter"), Some(&";".to_string()));
            assert_eq!(options.get("header"), Some(&"true".to_string()));
        }
    }

    #[tokio::test]
    async fn test_parse_map_single_expression() {
        let parser = RqlParser::new();
        let input = "MAP UPPER(nombre)";

        let ast = parser.parse_rql(input).await.unwrap();

        assert_eq!(ast.statements.len(), 1);

        if let RqlStatement::Map { expressions } = &ast.statements[0] {
            assert_eq!(expressions.len(), 1);
            assert_eq!(expressions[0].expression, "UPPER(nombre)");
            assert_eq!(expressions[0].alias, None);
        }
    }

    #[tokio::test]
    async fn test_parse_map_with_alias() {
        let parser = RqlParser::new();
        let input = "MAP UPPER(nombre) AS nombre_upper";

        let ast = parser.parse_rql(input).await.unwrap();

        assert_eq!(ast.statements.len(), 1);

        if let RqlStatement::Map { expressions } = &ast.statements[0] {
            assert_eq!(expressions.len(), 1);
            assert_eq!(expressions[0].expression, "UPPER(nombre)");
            assert_eq!(expressions[0].alias, Some("nombre_upper".to_string()));
        }
    }

    #[tokio::test]
    async fn test_parse_map_multiple_expressions() {
        let parser = RqlParser::new();
        let input = "MAP UPPER(nombre) AS nombre_upper, LOWER(apellido) AS apellido_lower, edad * 2 AS edad_double";

        let ast = parser.parse_rql(input).await.unwrap();

        assert_eq!(ast.statements.len(), 1);

        if let RqlStatement::Map { expressions } = &ast.statements[0] {
            assert_eq!(expressions.len(), 3);

            assert_eq!(expressions[0].expression, "UPPER(nombre)");
            assert_eq!(expressions[0].alias, Some("nombre_upper".to_string()));

            assert_eq!(expressions[1].expression, "LOWER(apellido)");
            assert_eq!(expressions[1].alias, Some("apellido_lower".to_string()));

            assert_eq!(expressions[2].expression, "edad * 2");
            assert_eq!(expressions[2].alias, Some("edad_double".to_string()));
        }
    }

    #[tokio::test]
    async fn test_parse_filter() {
        let parser = RqlParser::new();
        let input = "FILTER edad > 18";

        let ast = parser.parse_rql(input).await.unwrap();

        assert_eq!(ast.statements.len(), 1);

        if let RqlStatement::Filter { condition } = &ast.statements[0] {
            assert_eq!(condition, "edad > 18");
        }
    }

    #[tokio::test]
    async fn test_parse_filter_complex() {
        let parser = RqlParser::new();
        let input = "FILTER pais IN ('AR', 'UY', 'CL') AND edad >= 21";

        let ast = parser.parse_rql(input).await.unwrap();

        assert_eq!(ast.statements.len(), 1);

        if let RqlStatement::Filter { condition } = &ast.statements[0] {
            assert_eq!(condition, "pais IN ('AR', 'UY', 'CL') AND edad >= 21");
        }
    }

    #[tokio::test]
    async fn test_parse_unset_single() {
        let parser = RqlParser::new();
        let input = "UNSET min_age";

        let ast = parser.parse_rql(input).await.unwrap();

        assert_eq!(ast.statements.len(), 1);

        if let RqlStatement::Unset { variables } = &ast.statements[0] {
            assert_eq!(variables.len(), 1);
            assert_eq!(variables[0], "min_age");
        }
    }

    #[tokio::test]
    async fn test_parse_unset_multiple() {
        let parser = RqlParser::new();
        let input = "UNSET min_age, max_age, country";

        let ast = parser.parse_rql(input).await.unwrap();

        assert_eq!(ast.statements.len(), 1);

        if let RqlStatement::Unset { variables } = &ast.statements[0] {
            assert_eq!(variables.len(), 3);
            assert_eq!(variables[0], "min_age");
            assert_eq!(variables[1], "max_age");
            assert_eq!(variables[2], "country");
        }
    }

    #[tokio::test]
    async fn test_parse_nql_workflow() {
        let parser = RqlParser::new();
        let input = r#"
        USE 'clientes.csv' AS csv;
        SHOW TABLES;
        DESCRIBE csv.clientes;
        FILTER pais = 'AR';
        MAP UPPER(nombre) AS nombre_upper;
        SELECT * FROM csv;
        EXPORT csv TO 'resultado.json' FORMAT JSON;
        "#;

        let ast = parser.parse_rql(input).await.unwrap();

        assert_eq!(ast.statements.len(), 7);
        assert!(matches!(ast.statements[0], RqlStatement::UseSource { .. }));
        assert!(matches!(ast.statements[1], RqlStatement::ShowTables { .. }));
        assert!(matches!(ast.statements[2], RqlStatement::Describe { .. }));
        assert!(matches!(ast.statements[3], RqlStatement::Filter { .. }));
        assert!(matches!(ast.statements[4], RqlStatement::Map { .. }));
        assert!(matches!(ast.statements[5], RqlStatement::Sql { .. }));
        assert!(matches!(ast.statements[6], RqlStatement::Export { .. }));
    }

    #[tokio::test]
    async fn test_parse_use_distinguishes_legacy_vs_source() {
        let parser = RqlParser::new();

        // Legacy USE (sin comillas) - debe crear RqlStatement::Use
        let input1 = "USE payroll";
        let ast1 = parser.parse_rql(input1).await.unwrap();
        assert!(matches!(ast1.statements[0], RqlStatement::Use { .. }));

        // NQL USE SOURCE (con comillas) - debe crear RqlStatement::UseSource
        let input2 = "USE 'data.csv' AS csv";
        let ast2 = parser.parse_rql(input2).await.unwrap();
        assert!(matches!(ast2.statements[0], RqlStatement::UseSource { .. }));
    }

    #[tokio::test]
    async fn test_parse_export_xlsx() {
        let parser = RqlParser::new();
        let input = "EXPORT employees TO 'report.xlsx' FORMAT XLSX";

        let ast = parser.parse_rql(input).await.unwrap();

        assert_eq!(ast.statements.len(), 1);

        if let RqlStatement::Export { query, file, format, options } = &ast.statements[0] {
            assert_eq!(query, "employees");
            assert_eq!(file, "report.xlsx");
            assert!(matches!(format, ExportFormat::Xlsx));
            assert!(options.is_empty());
        }
    }

    #[test]
    fn test_nql_statement_types() {
        // Verificar que todos los statement types NQL son correctos
        let stmt1 = RqlStatement::ShowSources;
        assert_eq!(stmt1.statement_type(), "SHOW_SOURCES");
        assert!(!stmt1.is_sql());
        assert!(stmt1.is_command());

        let stmt2 = RqlStatement::ShowTables { source: None };
        assert_eq!(stmt2.statement_type(), "SHOW_TABLES");

        let stmt3 = RqlStatement::ShowVars;
        assert_eq!(stmt3.statement_type(), "SHOW_VARS");

        let stmt4 = RqlStatement::UseSource {
            path: "test.csv".to_string(),
            alias: Some("test".to_string()),
            options: std::collections::HashMap::new(),
        };
        assert_eq!(stmt4.statement_type(), "USE_SOURCE");

        let stmt5 = RqlStatement::Describe {
            source: None,
            table: "test".to_string(),
        };
        assert_eq!(stmt5.statement_type(), "DESCRIBE");

        let stmt6 = RqlStatement::Map {
            expressions: vec![MapExpression {
                expression: "test".to_string(),
                alias: None,
            }],
        };
        assert_eq!(stmt6.statement_type(), "MAP");

        let stmt7 = RqlStatement::Filter {
            condition: "test".to_string(),
        };
        assert_eq!(stmt7.statement_type(), "FILTER");

        let stmt8 = RqlStatement::Unset {
            variables: vec!["test".to_string()],
        };
        assert_eq!(stmt8.statement_type(), "UNSET");
    }

    // Tests for NQL validation in RqlProcessor
    #[tokio::test]
    async fn test_nql_validation_duplicate_aliases() {
        use crate::parser::RqlProcessor;

        let processor = RqlProcessor::new();
        let input = r#"
USE 'data1.csv' AS mydata;
USE 'data2.csv' AS mydata;
        "#;

        let ast = processor.process(input).await.unwrap();

        // Should have a warning about duplicate alias
        assert!(ast.metadata.warnings.iter().any(|w| w.contains("Duplicate alias")));
    }

    #[tokio::test]
    async fn test_nql_validation_invalid_alias() {
        use crate::parser::RqlProcessor;

        let processor = RqlProcessor::new();
        let input = "USE 'data.csv' AS 123invalid;";

        let ast = processor.process(input).await.unwrap();

        // Should have a warning about invalid alias
        assert!(ast.metadata.warnings.iter().any(|w| w.contains("Invalid alias")));
    }

    #[tokio::test]
    async fn test_nql_validation_valid_commands() {
        use crate::parser::RqlProcessor;

        let processor = RqlProcessor::new();
        let input = r#"
USE 'data.csv' AS mydata OPTIONS (delimiter=;, has_header=true);
SHOW SOURCES;
IMPORT 'clients.csv' AS clients;
MAP id, nombre AS name;
FILTER active = 1;
        "#;

        let ast = processor.process(input).await.unwrap();

        // Should have no critical warnings for valid commands
        let has_critical_warnings = ast.metadata.warnings.iter().any(|w|
            w.contains("Invalid") || w.contains("Empty") || w.contains("Duplicate"));

        if has_critical_warnings {
            eprintln!("Unexpected warnings: {:?}", ast.metadata.warnings);
        }
        assert!(!has_critical_warnings);
    }

    #[tokio::test]
    async fn test_nql_validation_identifier_rules() {
        use crate::parser::RqlProcessor;

        let processor = RqlProcessor::new();

        // Valid identifiers
        let valid_inputs = vec![
            "USE 'data.csv' AS mydata;",
            "USE 'data.csv' AS _private;",
            "USE 'data.csv' AS data123;",
            "IMPORT 'file.csv' AS my_table_1;",
        ];

        for input in valid_inputs {
            let ast = processor.process(input).await.unwrap();
            assert!(
                !ast.metadata.warnings.iter().any(|w| w.contains("Invalid")),
                "Input '{}' should be valid but got warnings: {:?}",
                input,
                ast.metadata.warnings
            );
        }
    }
}