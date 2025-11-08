//! Template processor para RQL

use crate::error::{ParserError, ParserResult};
use regex::Regex;
use std::collections::HashMap;

/// Motor de templates para RQL
#[derive(Debug, Clone)]
pub struct TemplateEngine {
    #[allow(dead_code)]
    config: TemplateEngineConfig,
}

impl TemplateEngine {
    /// Crear nuevo motor de templates
    pub fn new() -> Self {
        Self {
            config: TemplateEngineConfig::default(),
        }
    }

    /// Crear motor con configuración específica
    pub fn with_config(config: TemplateEngineConfig) -> Self {
        Self { config }
    }

    /// Procesar template con variables de sesión
    pub fn process(
        &self,
        template: &str,
        variables: &HashMap<String, String>,
    ) -> ParserResult<String> {
        let mut result = template.to_string();

        // Reemplazar variables de sesión (#variable)
        let session_var_regex = Regex::new(r"#([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
        for cap in session_var_regex.captures_iter(template) {
            let var_name = &cap[1];
            if let Some(value) = variables.get(var_name) {
                result = result.replace(&format!("#{}", var_name), value);
            }
        }

        // Procesar condicionales simples
        result = self.process_conditionals(&result, variables)?;

        // Procesar loops simples
        result = self.process_loops(&result, variables)?;

        Ok(result)
    }

    /// Procesar condicionales simples
    fn process_conditionals(
        &self,
        template: &str,
        variables: &HashMap<String, String>,
    ) -> ParserResult<String> {
        let mut result = template.to_string();

        // {{#if variable}} ... {{/if}}
        let if_regex =
            Regex::new(r"\{\{#if\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\}\}(.*?)\{\{/if\}\}").unwrap();
        for cap in if_regex.captures_iter(template) {
            let var_name = &cap[1];
            let content = &cap[2];

            if variables.contains_key(var_name) && !variables.get(var_name).unwrap().is_empty() {
                // Variable existe y no está vacía
                result = result.replace(&cap[0], content);
            } else {
                // Variable no existe o está vacía
                result = result.replace(&cap[0], "");
            }
        }

        // {{#unless variable}} ... {{/unless}}
        let unless_regex =
            Regex::new(r"\{\{#unless\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\}\}(.*?)\{\{/unless\}\}")
                .unwrap();
        for cap in unless_regex.captures_iter(template) {
            let var_name = &cap[1];
            let content = &cap[2];

            if !variables.contains_key(var_name) || variables.get(var_name).unwrap().is_empty() {
                // Variable no existe o está vacía
                result = result.replace(&cap[0], content);
            } else {
                // Variable existe y no está vacía
                result = result.replace(&cap[0], "");
            }
        }

        Ok(result)
    }

    /// Procesar loops simples
    fn process_loops(
        &self,
        template: &str,
        variables: &HashMap<String, String>,
    ) -> ParserResult<String> {
        let mut result = template.to_string();

        // {{#each array}} ... {{/each}}
        let each_regex =
            Regex::new(r"\{\{#each\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\}\}(.*?)\{\{/each\}\}").unwrap();
        for cap in each_regex.captures_iter(template) {
            let var_name = &cap[1];
            let content = &cap[2];

            if let Some(items_str) = variables.get(var_name) {
                // Simular processing de array (en implementación real usar parsing JSON)
                let items: Vec<&str> = items_str.split(',').map(|s| s.trim()).collect();
                let processed_items: Vec<String> = items
                    .iter()
                    .map(|item| content.replace("@this", item))
                    .collect();

                result = result.replace(&cap[0], &processed_items.join(" "));
            }
        }

        Ok(result)
    }

    /// Validar sintaxis de template
    pub fn validate_template(&self, template: &str) -> ParserResult<()> {
        // Verificar balance de llaves
        let open_count = template.matches("{{").count();
        let close_count = template.matches("}}").count();

        if open_count != close_count {
            return Err(ParserError::template_error(
                "Unbalanced template delimiters",
            ));
        }

        // Verificar sintaxis de condicionales
        let conditional_pairs = vec![
            ("if", "{{#if", "{{/if}}"),
            ("unless", "{{#unless", "{{/unless}}"),
            ("each", "{{#each", "{{/each}}"),
        ];

        for (name, open_tag, close_tag) in conditional_pairs {
            let open_count = template.matches(open_tag).count();
            let close_count = template.matches(close_tag).count();

            if open_count != close_count {
                return Err(ParserError::template_error(format!(
                    "Unbalanced {} conditionals",
                    name
                )));
            }
        }

        Ok(())
    }
}

/// Configuración del motor de templates
#[derive(Debug, Clone)]
pub struct TemplateEngineConfig {
    /// Delimitador de apertura
    pub open_delimiter: String,

    /// Delimitador de cierre
    pub close_delimiter: String,

    /// Permitir variables de sesión
    pub allow_session_variables: bool,

    /// Permitir condicionales
    pub allow_conditionals: bool,

    /// Permitir loops
    pub allow_loops: bool,

    /// Escape automático de variables
    pub auto_escape: bool,

    /// Tamaño máximo del template
    pub max_template_size: usize,
}

impl Default for TemplateEngineConfig {
    fn default() -> Self {
        Self {
            open_delimiter: "{{".to_string(),
            close_delimiter: "}}".to_string(),
            allow_session_variables: true,
            allow_conditionals: true,
            allow_loops: true,
            auto_escape: true,
            max_template_size: 100000,
        }
    }
}

/// Procesador de templates simple (deprecated, usar TemplateEngine)
#[derive(Debug, Clone)]
pub struct TemplateProcessor {
    #[allow(dead_code)]
    config: TemplateConfig,
}

impl TemplateProcessor {
    /// Crear nuevo procesador
    pub fn new() -> Self {
        Self {
            config: TemplateConfig::default(),
        }
    }

    /// Procesar template básico
    pub fn process_template(&self, template: &str, variables: &HashMap<String, String>) -> String {
        let mut result = template.to_string();

        for (key, value) in variables {
            let placeholder = format!("#{}", key);
            result = result.replace(&placeholder, value);
        }

        result
    }
}

/// Configuración del procesador de templates
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

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TemplateProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Utilidades para templates
pub mod utils {
    use super::{TemplateEngine, TemplateEngineConfig};
    use std::collections::HashMap;

    /// Crear variables de sesión desde hashmap
    pub fn session_vars_from_map(map: &HashMap<String, String>) -> HashMap<String, String> {
        map.iter()
            .map(|(k, v)| (format!("#{}", k), v.clone()))
            .collect()
    }

    /// Crear engine con configuraciones comunes
    pub fn engine_for_sql() -> TemplateEngine {
        TemplateEngine::with_config(TemplateEngineConfig {
            open_delimiter: "{{".to_string(),
            close_delimiter: "}}".to_string(),
            allow_session_variables: true,
            allow_conditionals: true,
            allow_loops: false,
            auto_escape: true,
            max_template_size: 50000,
        })
    }

    /// Procesar SQL template común
    pub fn process_sql_template(template: &str, variables: &HashMap<String, String>) -> String {
        let engine = engine_for_sql();
        let session_vars = session_vars_from_map(variables);
        engine
            .process(template, &session_vars)
            .unwrap_or_else(|_| template.to_string())
    }
}
