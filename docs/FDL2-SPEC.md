# FDL2 (Form Definition Language) - Especificación Completa

## Introducción

FDL2 es el **Form Definition Language** de Noctra, un lenguaje declarativo moderno para definir formularios que se compilan automáticamente a operaciones SQL. Basado en TOML y template processing para máxima flexibilidad.

## Sintaxis Básica

### Estructura de un Formulario

```toml
title = "Título del Formulario"
schema = "nombre_schema"

[fields.nombre_campo]
label = "Etiqueta Visible"
field_type = "tipo"
required = true
width = 40
validations = [
    { type = "min_length", value = 3 },
    { type = "max_length", value = 50 }
]

[actions.accion_principal]
sql = "SELECT * FROM tabla WHERE campo = :parametro"
params = ["parametro"]
on_success = "mostrar_resultados"
on_error = "mostrar_error"
```

## Tipos de Campos

### Tipos Básicos

| Tipo | Descripción | Validaciones Soportadas |
|------|-------------|-------------------------|
| `text` | Cadena de texto | min_length, max_length, regex, pattern |
| `integer` | Número entero | min, max, step |
| `float` | Número decimal | min, max, precision, step |
| `boolean` | Verdadero/Falso | - |
| `date` | Fecha | format, min, max |
| `datetime` | Fecha y hora | format, min, max |
| `password` | Texto oculto | min_length, max_length |
| `textarea` | Texto multilínea | min_length, max_length, rows |

### Tipos Especiales

| Tipo | Descripción | Configuración |
|------|-------------|---------------|
| `enum` | Lista desplegable | `options = ["A", "B", "C"]` |
| `select` | Selección con consulta | `query = "SELECT id, name FROM table"` |
| `file` | Selección de archivo | `accept = ".csv,.xlsx"` |
| `color` | Selector de color | `default = "#000000"` |

## Validaciones

### Validaciones por Campo

```toml
[fields.email]
label = "Correo Electrónico"
field_type = "text"
validations = [
    { type = "required" },
    { type = "email" },
    { type = "regex", pattern = "^[^@]+@[^@]+\\.[^@]+$" }
]
```

### Validaciones Globales

```toml
[validations]
# Validación condicional
[[validations.rules]]
field = "fecha_fin"
when = "fecha_inicio"
condition = "gt"
action = "require"

# Validación personalizada
[[validations.custom]]
name = "validar_rango_edad"
script = """
if (edad < 18 && tipo_usuario == "menor") {
    return { valid: false, message: "Menores de edad requieren tutor" };
}
return { valid: true };
"""
```

## Template Processing

FDL2 incluye un sistema de templates similar a Handlebars para generar SQL dinámico:

### Condicionales

```sql
{{#if campo}} AND campo = :campo {{/if}}
{{#unless campo}} AND campo IS NULL {{/unless}}

{{#if_eq campo "valor"}} 
    condition_true
{{else}}
    condition_false
{{/if_eq}}
```

### Loops

```sql
{{#each items}}
    {{#if @first}}SELECT{{else}}UNION ALL{{/if}} 
    SELECT * FROM {{this}} 
{{/each}}
```

### Funciones

```sql
{{#if_like nombre "%test%"}} AND nombre LIKE :nombre {{/if_like}}
{{#if_in dept ["VENTAS", "MARKETING"]}} AND dept IN (:dept) {{/if_in}}
```

## Acciones

### Tipos de Acciones

```toml
[actions.consultar]
type = "query"
sql = "SELECT * FROM tabla WHERE condicion = :valor"
params = ["valor"]

[actions.insertar]
type = "insert"
table = "tabla"
mapping = { campo1 = "field1", campo2 = "field2" }

[actions.actualizar]
type = "update"
table = "tabla"
where = "id = :id"
mapping = { campo1 = "field1" }

[actions.eliminar]
type = "delete"
table = "tabla"
where = "id = :id"

[actions.personalizada]
type = "custom"
sql = """
BEGIN;
INSERT INTO log (accion, usuario) VALUES ('accion', :usuario);
COMMIT;
"""
params = ["usuario"]
```

### Handlers de Resultado

```toml
[views.mostrar_resultados]
type = "table"
title = "Resultados"
pager = true
max_rows = 100
columns = ["id", "nombre", "fecha"]

[views.mostrar_mensaje]
type = "message"
title = "Éxito"
message = "Operación completada exitosamente"
style = "success"

[error_handlers.mostrar_error]
type = "modal"
title = "Error"
message_format = "Error: {error}"
style = "error"
```

## Configuración Avanzada

### Layout del Formulario

```toml
[layout]
columns = 2
spacing = 2
width = 80

[layout.sections]
[[layout.sections]]
title = "Información Personal"
fields = ["nombre", "apellido", "email"]

[[layout.sections]]
title = "Información Adicional"
fields = ["telefono", "direccion"]
```

### Estilos y Temas

```toml
[styling]
theme = "classic"  # classic, modern, minimal
colors = { 
    primary = "#0066cc",
    secondary = "#666666",
    error = "#cc0000",
    success = "#00cc00"
}

[styling.fields]
[styling.fields.text]
border = "single"
padding = 1

[styling.fields.select]
arrow = "▼"
```

### Internacionalización

```toml
[i18n]
default_locale = "es"
fallback = "en"

[i18n.translations]
[i18n.translations.en]
"Consultar" = "Query"
"Cancelar" = "Cancel"

[i18n.translations.fr]
"Consultar" = "Requête"
"Cancelar" = "Annuler"
```

## Ejemplos Completos

### Formulario de Consulta de Empleados

```toml
title = "Consulta de Empleados"
schema = "rrhh"

[fields.nroleg]
label = "Nro. Legajo"
field_type = "integer"
required = false
width = 10
validations = [
    { type = "min", value = 1 },
    { type = "max", value = 99999 }
]

[fields.nombre]
label = "Nombre"
field_type = "text"
required = false
width = 40

[fields.dept]
label = "Departamento"
field_type = "enum"
options = ["VENTAS", "MARKETING", "RRHH", "IT", "FINANZAS"]

[fields.fecha_desde]
label = "Fecha Desde"
field_type = "date"
required = false

[fields.fecha_hasta]
label = "Fecha Hasta"
field_type = "date"
required = false

[actions.consultar]
sql = """
SELECT nroleg, nombre, dept, fecha_ingreso, salario
FROM employees 
WHERE 1=1
{{#if nroleg}} AND nroleg = :nroleg {{/if}}
{{#if nombre}} AND nombre LIKE '%' || :nombre || '%' {{/if}}
{{#if dept}} AND dept = :dept {{/if}}
{{#if fecha_desde}} AND fecha_ingreso >= :fecha_desde {{/if}}
{{#if fecha_hasta}} AND fecha_ingreso <= :fecha_hasta {{/if}}
ORDER BY nombre;
"""
params = ["nroleg", "nombre", "dept", "fecha_desde", "fecha_hasta"]
on_success = "mostrar_resultados"

[validations]
[[validations.rules]]
field = "fecha_hasta"
when = "fecha_desde"
condition = "gte"
action = "require"

[views.mostrar_resultados]
type = "table"
title = "Resultados de Consulta"
pager = true
max_rows = 50
columns = ["nroleg", "nombre", "dept", "fecha_ingreso", "salario"]

[error_handlers.error_consulta]
type = "modal"
title = "Error en Consulta"
message_format = "No se pudo ejecutar la consulta: {error}"
```

### Formulario de Alta de Cliente

```toml
title = "Alta de Cliente"
schema = "ventas"

[fields.razon_social]
label = "Razón Social"
field_type = "text"
required = true
width = 50
validations = [
    { type = "min_length", value = 3 },
    { type = "max_length", value = 100 }
]

[fields.cuit]
label = "CUIT"
field_type = "text"
required = true
width = 13
validations = [
    { type = "regex", pattern = "^\\d{2}-\\d{8}-\\d{1}$" }
]

[fields.tipo_cliente]
label = "Tipo de Cliente"
field_type = "enum"
required = true
options = ["PARTICULAR", "EMPRESA", "GOBIERNO"]

[fields.limite_credito]
label = "Límite de Crédito"
field_type = "float"
required = false
width = 15
validations = [
    { type = "min", value = 0 },
    { type = "max", value = 999999.99 }
]

[actions.guardar]
type = "insert"
table = "clientes"
mapping = { 
    razon_social = "razon_social",
    cuit = "cuit",
    tipo = "tipo_cliente",
    limite_credito = "limite_credito"
}
on_success = "mostrar_confirmacion"
on_error = "mostrar_error"

[actions.cancelar]
type = "cancel"
redirect = "/menu_principal"

[views.mostrar_confirmacion]
type = "message"
title = "Cliente Registrado"
message = "El cliente ha sido registrado exitosamente"
style = "success"
redirect = "/menu_principal"

[error_handlers.mostrar_error]
type = "modal"
title = "Error"
message_format = "Error al registrar cliente: {error}"
```

## API de Programación

### Cargar Formulario

```rust
use noctra_formlib::{Form, load_form};

let form = load_form(Path::new("empleados.toml"))?;
let compiled = form.compile()?;
```

### Ejecutar Formulario

```rust
use noctra_formlib::{Form, bind_and_execute};

let mut session = Session::new();
let values = HashMap::from([
    ("nombre".to_string(), Value::Text("Juan".to_string())),
    ("dept".to_string(), Value::Text("IT".to_string())),
]);

let result = bind_and_execute(&form, &mut session, values)?;
```

### Validación Personalizada

```rust
use noctra_formlib::{ValidationRule, ValidationContext};

impl ValidationRule for CustomRule {
    fn validate(&self, value: &Value, context: &ValidationContext) -> Result<bool, String> {
        // Lógica de validación personalizada
        Ok(true)
    }
}
```

## Configuración Avanzada

### Validaciones Complejas

Para formularios con validaciones avanzadas, se recomienda:

1. **Revisar las reglas de validación** manualmente
2. **Probar la funcionalidad** con datos reales
3. **Ajustar estilos** según preferencias de UI
4. **Documentar campos especiales** en comentarios

### Mejores Prácticas

- **Mantenimiento**: Los formularios TOML son texto plano versionable
- **Testing**: Probar cada formulario con casos límite de validación
- **Documentación**: Documentar reglas de negocio en comentarios
- **Performance**: Evitar consultas complejas en acciones de formulario

---

**FDL2 v1.0** - Form Definition Language para Noctra  
**Última actualización:** 2025-11-04