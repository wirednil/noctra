// Ejemplo: Crear base de datos de demostración
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open("demo.db")?;

    // Crear tabla de empleados
    conn.execute(
        "CREATE TABLE IF NOT EXISTS employees (
            nroleg INTEGER PRIMARY KEY,
            nombre TEXT NOT NULL,
            dept TEXT NOT NULL,
            cargo TEXT,
            salario REAL,
            fecha_ingreso DATE,
            activo BOOLEAN DEFAULT 1
        )",
        [],
    )?;

    // Crear tabla de departamentos
    conn.execute(
        "CREATE TABLE IF NOT EXISTS departments (
            codigo TEXT PRIMARY KEY,
            descripcion TEXT NOT NULL,
            manager_id INTEGER
        )",
        [],
    )?;

    // Insertar departamentos
    let departments = vec![
        ("IT", "Tecnología de la Información"),
        ("VENTAS", "Ventas y Comercial"),
        ("RRHH", "Recursos Humanos"),
        ("FINANZAS", "Finanzas y Contabilidad"),
        ("MARKETING", "Marketing y Comunicación"),
        ("OPERACIONES", "Operaciones"),
    ];

    for (codigo, desc) in departments {
        conn.execute(
            "INSERT OR REPLACE INTO departments (codigo, descripcion, manager_id) VALUES (?1, ?2, NULL)",
            [codigo, desc],
        )?;
    }

    // Insertar empleados
    let employees = vec![
        (1001, "Juan Pérez", "IT", "Senior Developer", 85000.0, "2020-01-15", 1),
        (1002, "María González", "VENTAS", "Sales Manager", 75000.0, "2021-03-20", 1),
        (1003, "Carlos Rodríguez", "IT", "Tech Lead", 95000.0, "2019-08-10", 1),
        (1004, "Ana Martínez", "RRHH", "HR Specialist", 55000.0, "2022-02-01", 1),
        (1005, "Luis García", "VENTAS", "Sales Representative", 60000.0, "2020-11-05", 1),
        (1006, "Elena Fernández", "IT", "Junior Developer", 50000.0, "2023-01-10", 1),
        (1007, "Roberto Silva", "FINANZAS", "Financial Analyst", 70000.0, "2021-06-15", 1),
        (1008, "Patricia Ruiz", "MARKETING", "Marketing Manager", 80000.0, "2020-09-01", 1),
        (1009, "Diego López", "IT", "DevOps Engineer", 88000.0, "2021-04-12", 1),
        (1010, "Carmen Díaz", "VENTAS", "Sales Representative", 58000.0, "2022-07-20", 1),
        (1011, "Fernando Torres", "OPERACIONES", "Operations Manager", 82000.0, "2019-05-30", 1),
        (1012, "Laura Sánchez", "RRHH", "Recruiter", 52000.0, "2023-03-15", 1),
        (1013, "Miguel Ángel Castro", "IT", "Backend Developer", 78000.0, "2021-11-08", 1),
        (1014, "Isabel Romero", "FINANZAS", "Accountant", 62000.0, "2022-01-25", 1),
        (1015, "Javier Moreno", "MARKETING", "Content Creator", 55000.0, "2022-09-10", 1),
        (1016, "Sofía Herrera", "IT", "Frontend Developer", 76000.0, "2020-12-03", 1),
        (1017, "Andrés Jiménez", "VENTAS", "Account Executive", 72000.0, "2021-08-17", 1),
        (1018, "Gabriela Vargas", "OPERACIONES", "Logistics Coordinator", 54000.0, "2023-02-28", 1),
        (1019, "Ricardo Mendoza", "IT", "Database Administrator", 82000.0, "2020-07-22", 1),
        (1020, "Valentina Ortiz", "RRHH", "HR Manager", 78000.0, "2019-10-14", 1),
        (1021, "Pablo Guerrero", "FINANZAS", "Controller", 92000.0, "2020-03-05", 1),
        (1022, "Natalia Reyes", "MARKETING", "Social Media Manager", 58000.0, "2022-11-12", 1),
        (1023, "Sebastián Cruz", "IT", "Security Analyst", 86000.0, "2021-02-18", 1),
        (1024, "Marina Ramírez", "VENTAS", "Regional Manager", 95000.0, "2019-12-01", 1),
        (1025, "Tomás Flores", "OPERACIONES", "Supply Chain Analyst", 64000.0, "2022-04-07", 1),
        (1026, "Lucía Paredes", "IT", "QA Engineer", 68000.0, "2021-09-23", 1),
        (1027, "Mateo Vega", "FINANZAS", "Tax Specialist", 74000.0, "2020-06-11", 1),
        (1028, "Camila Navarro", "MARKETING", "Brand Manager", 82000.0, "2021-01-29", 1),
        (1029, "Daniel Rojas", "VENTAS", "Business Development", 88000.0, "2020-08-16", 0),
        (1030, "Victoria Medina", "IT", "Cloud Architect", 105000.0, "2019-11-25", 1),
    ];

    for (nroleg, nombre, dept, cargo, salario, fecha, activo) in employees {
        conn.execute(
            "INSERT OR REPLACE INTO employees (nroleg, nombre, dept, cargo, salario, fecha_ingreso, activo)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![nroleg, nombre, dept, cargo, salario, fecha, activo],
        )?;
    }

    // Verificar datos
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM employees", [], |row| row.get(0))?;
    println!("✓ Base de datos creada exitosamente en: demo.db");
    println!("✓ Total empleados insertados: {}", count);

    let activos: i64 = conn.query_row("SELECT COUNT(*) FROM employees WHERE activo = 1", [], |row| row.get(0))?;
    println!("✓ Empleados activos: {}", activos);

    let depts: i64 = conn.query_row("SELECT COUNT(*) FROM departments", [], |row| row.get(0))?;
    println!("✓ Departamentos: {}", depts);

    println!("\n📊 Estadísticas por departamento:");
    let mut stmt = conn.prepare(
        "SELECT dept, COUNT(*) as count, AVG(salario) as avg_sal
         FROM employees
         WHERE activo = 1
         GROUP BY dept
         ORDER BY count DESC"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, f64>(2)?,
        ))
    })?;

    for row in rows {
        let (dept, count, avg_sal) = row?;
        println!("  {} - {} empleados (salario promedio: ${:.2})", dept, count, avg_sal);
    }

    Ok(())
}
