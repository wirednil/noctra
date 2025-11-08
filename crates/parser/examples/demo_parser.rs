// Demostración de capacidades del Parser RQL
use noctra_parser::parser::RqlParser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Demostración del Parser RQL de Noctra\n");
    println!("{}", "=".repeat(70));

    let parser = RqlParser::new();

    // Test 1: SELECT simple
    println!("\n1️⃣  SELECT Simple");
    println!("{}", "-".repeat(70));
    let sql1 = "SELECT * FROM employees";
    let ast1 = parser.parse_rql(sql1).await?;
    println!("Input:  {}", sql1);
    println!("Output: {} statement(s) parsed", ast1.statements.len());
    println!("Debug:  {:#?}", ast1.debug_info());

    // Test 2: SELECT con parámetros nombrados
    println!("\n2️⃣  SELECT con Parámetros Nombrados");
    println!("{}", "-".repeat(70));
    let sql2 = "SELECT * FROM employees WHERE dept = :dept AND salario > :min_salary";
    let ast2 = parser.parse_rql(sql2).await?;
    println!("Input:  {}", sql2);
    println!("Parámetros detectados: {}", ast2.parameters.len());
    for param in &ast2.parameters {
        println!("  - {} (tipo: {:?})", param.name, param.param_type);
    }

    // Test 3: SELECT con parámetros posicionados
    println!("\n3️⃣  SELECT con Parámetros Posicionados");
    println!("{}", "-".repeat(70));
    let sql3 = "SELECT nombre, salario FROM employees WHERE dept = $1 AND activo = $2";
    let ast3 = parser.parse_rql(sql3).await?;
    println!("Input:  {}", sql3);
    println!("Parámetros detectados: {}", ast3.parameters.len());
    for param in &ast3.parameters {
        println!("  - {} (tipo: {:?})", param.name, param.param_type);
    }

    // Test 4: Comando USE
    println!("\n4️⃣  Comando USE");
    println!("{}", "-".repeat(70));
    let cmd1 = "USE payroll";
    let ast4 = parser.parse_rql(cmd1).await?;
    println!("Input:  {}", cmd1);
    println!("Tipo:   {:?}", ast4.statements[0].statement_type());
    println!("Es comando: {}", ast4.statements[0].is_command());

    // Test 5: Comando LET
    println!("\n5️⃣  Comando LET (variables de sesión)");
    println!("{}", "-".repeat(70));
    let cmd2 = "LET dept_filter = 'IT'";
    let ast5 = parser.parse_rql(cmd2).await?;
    println!("Input:  {}", cmd2);
    println!("Tipo:   {:?}", ast5.statements[0].statement_type());

    // Test 6: Comando FORM LOAD
    println!("\n6️⃣  Comando FORM LOAD");
    println!("{}", "-".repeat(70));
    let cmd3 = "FORM LOAD 'empleados.toml'";
    let ast6 = parser.parse_rql(cmd3).await?;
    println!("Input:  {}", cmd3);
    println!("Tipo:   {:?}", ast6.statements[0].statement_type());

    // Test 7: Comando OUTPUT TO
    println!("\n7️⃣  Comando OUTPUT TO");
    println!("{}", "-".repeat(70));
    let cmd4 = "OUTPUT TO 'reporte.csv' FORMAT csv";
    let ast7 = parser.parse_rql(cmd4).await?;
    println!("Input:  {}", cmd4);
    println!("Tipo:   {:?}", ast7.statements[0].statement_type());

    // Test 8: Script completo con múltiples statements
    println!("\n8️⃣  Script RQL Completo (múltiples statements)");
    println!("{}", "-".repeat(70));
    let script = r#"
        USE payroll;
        LET dept = 'IT';
        LET salario_min = 70000;
        SELECT * FROM employees WHERE dept = :dept AND salario > :salario_min;
    "#;
    let ast8 = parser.parse_rql(script).await?;
    println!("Input:  Script con {} líneas", script.lines().count());
    println!("Statements: {}", ast8.statements.len());
    println!("Parámetros: {}", ast8.parameters.len());
    println!("Variables de sesión: {}", ast8.session_variables.len());

    for (i, stmt) in ast8.statements.iter().enumerate() {
        println!("  Statement {}: {} (es_sql: {}, es_comando: {})",
            i + 1,
            stmt.statement_type(),
            stmt.is_sql(),
            stmt.is_command()
        );
    }

    // Test 9: Extracción de parámetros mezclados
    println!("\n9️⃣  Parámetros Mezclados (posicionales + nombrados)");
    println!("{}", "-".repeat(70));
    let sql9 = "SELECT * FROM employees WHERE dept = $1 AND nombre LIKE :nombre AND salario > $2";
    let params9 = parser.extract_sql_parameters(sql9)?;
    println!("Input:  {}", sql9);
    println!("Parámetros extraídos: {}", params9.len());
    for param in &params9 {
        println!("  - {} (tipo: {:?})", param.name, param.param_type);
    }

    // Test 10: Variables de sesión con #
    println!("\n🔟  Variables de Sesión con #");
    println!("{}", "-".repeat(70));
    let sql10 = "SELECT * FROM #tabla WHERE dept = #dept_var";
    let ast10 = parser.parse_rql(sql10).await?;
    println!("Input:  {}", sql10);
    println!("Variables de sesión: {}", ast10.session_variables.len());
    for var in &ast10.session_variables {
        println!("  - {}", var);
    }

    // Test 11: Comentarios y líneas vacías
    println!("\n1️⃣1️⃣  Manejo de Comentarios y Líneas Vacías");
    println!("{}", "-".repeat(70));
    let sql11 = r#"
        -- Este es un comentario
        SELECT * FROM employees;

        -- Otro comentario
        USE payroll;
    "#;
    let ast11 = parser.parse_rql(sql11).await?;
    println!("Input:  Script con comentarios");
    println!("Statements (sin comentarios): {}", ast11.statements.len());

    // Test 12: Conversión a SQL
    println!("\n1️⃣2️⃣  Conversión AST a SQL");
    println!("{}", "-".repeat(70));
    let sql12 = parser.parse_rql("USE demo; SELECT * FROM employees LIMIT 5").await?;
    let generated_sql = sql12.to_sql();
    println!("SQL generado desde AST:");
    println!("{}", generated_sql);

    println!("\n{}", "=".repeat(70));
    println!("✅ Todas las pruebas del parser completadas exitosamente!\n");

    Ok(())
}
