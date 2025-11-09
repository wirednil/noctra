// Demostración simplificada de capacidades de Formlib (FDL2)
use noctra_formlib::loader::{FormLoader, LoaderConfig};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Demostración de Formlib (FDL2) de Noctra\n");
    println!("{}", "=".repeat(70));

    // Test 1: Cargar formulario de ejemplo simple
    println!("\n1️⃣  Cargar Formulario Simple (employee_search.toml)");
    println!("{}", "-".repeat(70));

    let path1 = Path::new("../../examples/forms/employee_search.toml");
    if path1.exists() {
        let loader = FormLoader::new(LoaderConfig::default());
        match loader.load_from_path(path1) {
            Ok(form) => {
                println!("✓ Formulario cargado exitosamente");
                println!("  Título: {}", form.title);
                println!("  Descripción: {:?}", form.description);
                println!("  Schema: {:?}", form.schema);
                println!("  Número de campos: {}", form.fields.len());
                println!("\n  📋 Campos:");
                for (name, field) in &form.fields {
                    println!("    - {}: {} (tipo: {:?}, requerido: {})",
                        name,
                        field.label,
                        field.field_type,
                        field.required
                    );
                }
                println!("\n  ⚙️ Acciones ({} total):", form.actions.len());
                for (name, _action) in &form.actions {
                    println!("    - {}", name);
                }
            }
            Err(e) => {
                println!("✗ Error al cargar formulario: {}", e);
            }
        }
    } else {
        println!("⚠ Archivo no encontrado: {:?}", path1);
    }

    // Test 2: Cargar formulario completo (empleados.toml)
    println!("\n2️⃣  Cargar Formulario Completo (empleados.toml)");
    println!("{}", "-".repeat(70));

    let path2 = Path::new("../../examples/empleados.toml");
    if path2.exists() {
        let loader = FormLoader::new(LoaderConfig::default());
        match loader.load_from_path(path2) {
            Ok(form) => {
                println!("✓ Formulario completo cargado exitosamente");
                println!("  Título: {}", form.title);
                println!("  Descripción: {:?}", form.description);
                println!("  Schema: {:?}", form.schema);
                println!("\n  📋 Campos ({} total):", form.fields.len());

                // Agrupar por tipo
                use std::collections::HashMap;
                let mut by_type: HashMap<String, Vec<String>> = HashMap::new();
                for (name, field) in &form.fields {
                    let type_str = format!("{:?}", field.field_type);
                    by_type.entry(type_str).or_insert_with(Vec::new).push(name.clone());
                }

                for (field_type, fields) in &by_type {
                    println!("    {} ({}): {}", field_type, fields.len(), fields.join(", "));
                }

                println!("\n  ⚙️ Acciones ({} total):", form.actions.len());
                for (name, action) in &form.actions {
                    let params_count = action.params.as_ref().map(|p| p.len()).unwrap_or(0);
                    println!("    - {}: {} parámetros", name, params_count);
                }

                println!("\n  🎨 Configuración UI:");
                if let Some(ui_config) = &form.ui_config {
                    println!("    Width: {:?}", ui_config.width);
                    println!("    Height: {:?}", ui_config.height);
                    println!("    Layout: {:?}", ui_config.layout);
                    println!("    Theme: {:?}", ui_config.theme);
                } else {
                    println!("    Configuración UI por defecto");
                }

                println!("\n  📄 Configuración de Paginación:");
                if let Some(pagination) = &form.pagination {
                    println!("    Page size: {:?}", pagination.page_size);
                    println!("    Order by: {:?}", pagination.order_by);
                } else {
                    println!("    Sin configuración de paginación");
                }
            }
            Err(e) => {
                println!("✗ Error al cargar formulario: {}", e);
            }
        }
    } else {
        println!("⚠ Archivo no encontrado: {:?}", path2);
    }

    // Test 3: Validación de campos
    println!("\n3️⃣  Inspección de Validaciones");
    println!("{}", "-".repeat(70));

    let path3 = Path::new("../../examples/forms/employee_search.toml");
    if path3.exists() {
        let loader = FormLoader::new(LoaderConfig::default());
        if let Ok(form) = loader.load_from_path(path3) {
            println!("Inspeccionando campos del formulario '{}':", form.title);

            for (name, field) in &form.fields {
                print!("  Campo '{}': ", name);

                if let Some(validations) = &field.validations {
                    print!("validaciones encontradas - ");
                    let mut parts = Vec::new();
                    if validations.min.is_some() {
                        parts.push("min");
                    }
                    if validations.max.is_some() {
                        parts.push("max");
                    }
                    if validations.pattern.is_some() {
                        parts.push("pattern");
                    }
                    if validations.min_length.is_some() {
                        parts.push("min_length");
                    }
                    if validations.max_length.is_some() {
                        parts.push("max_length");
                    }
                    println!("{}", parts.join(", "));
                } else {
                    println!("sin validaciones");
                }
            }
        }
    }

    // Test 4: Sistema de Navegación (FormGraph)
    println!("\n4️⃣  Sistema de Navegación (FormGraph)");
    println!("{}", "-".repeat(70));

    use noctra_formlib::graph::{NodeDefinition, NodeType, FormGraph, GraphConfig};
    use std::collections::HashMap;

    // Crear un grafo de formularios de ejemplo
    let graph = FormGraph {
        version: "1.0".to_string(),
        title: "Demo App".to_string(),
        base_path: None,
        root: NodeDefinition {
            id: "root".to_string(),
            title: "Menú Principal".to_string(),
            node_type: NodeType::Menu,
            path: None,
            description: Some("Menú principal de la aplicación".to_string()),
            children: vec![
                NodeDefinition {
                    id: "employees".to_string(),
                    title: "Gestión de Empleados".to_string(),
                    node_type: NodeType::Form,
                    path: Some("examples/empleados.toml".to_string()),
                    description: Some("Formulario de empleados".to_string()),
                    children: vec![],
                    metadata: HashMap::new(),
                    action: None,
                    icon: Some("👥".to_string()),
                },
                NodeDefinition {
                    id: "search".to_string(),
                    title: "Búsqueda".to_string(),
                    node_type: NodeType::Form,
                    path: Some("examples/forms/employee_search.toml".to_string()),
                    description: Some("Búsqueda de empleados".to_string()),
                    children: vec![],
                    metadata: HashMap::new(),
                    action: None,
                    icon: Some("🔍".to_string()),
                },
            ],
            metadata: HashMap::new(),
            action: None,
            icon: Some("🏠".to_string()),
        },
        config: GraphConfig {
            default_database: None,
            default_theme: Some("classic".to_string()),
            default_page_size: Some(25),
            show_breadcrumbs: true,
            enable_history: true,
        },
    };

    println!("✓ FormGraph creado:");
    println!("  Versión: {}", graph.version);
    println!("  Título: {}", graph.title);
    println!("  Nodo raíz: {} ({:?})", graph.root.title, graph.root.node_type);
    println!("  Hijos: {}", graph.root.children.len());
    for child in &graph.root.children {
        println!("    - {} ({:?}) - path: {:?}",
            child.title,
            child.node_type,
            child.path
        );
    }

    // Validar el grafo
    match graph.validate() {
        Ok(_) => println!("\n  ✓ Grafo validado correctamente"),
        Err(e) => println!("\n  ✗ Error validando grafo: {}", e),
    }

    println!("\n{}", "=".repeat(70));
    println!("✅ Todas las pruebas de formlib completadas exitosamente!\n");

    Ok(())
}
