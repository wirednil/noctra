#[cfg(test)]
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

#[cfg(test)]
mod template_tests {
    use super::template::*;
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
        
        assert_eq!(session_vars.get("#dept"), Some(&"IT".to_string()));
        assert_eq!(session_vars.get("#activo"), Some(&"true".to_string()));
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

#[cfg(test)]
mod error_tests {
    use super::*;

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