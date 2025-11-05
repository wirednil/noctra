# API Reference - Noctra Server

## Overview

El servidor Noctra expone una API REST completa para ejecutar consultas SQL/RQL, interactuar con formularios FDL2 y gestionar sesiones de trabajo.

**Base URL:** `http://localhost:8080`

**API Version:** `v1`

**Content-Type:** `application/json`

---

## Authentication

### Token Authentication (Opcional)

Para autenticación básica con token:

```http
Authorization: Bearer <token>
```

Ejemplo:

```bash
curl -H "Authorization: Bearer mi-token-secreto" \
     -H "Content-Type: application/json" \
     http://localhost:8080/api/v1/query/execute \
     -d '{"sql": "SELECT * FROM users", "parameters": []}'
```

---

## Endpoints

### Health Check

**GET** `/health`

Verifica el estado del servidor.

#### Response

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime": 1234,
  "timestamp": "2024-01-15T10:30:00Z"
}
```

#### Status Codes

- `200 OK` - Servidor funcionando correctamente

---

### Query API

#### Execute Query

**POST** `/api/v1/query/execute`

Ejecuta una consulta SQL/RQL contra la base de datos.

##### Request Body

```json
{
  "sql": "SELECT * FROM employees WHERE dept = ?",
  "parameters": [3],
  "session_id": "optional_session_123",
  "options": {
    "timeout": 30,
    "max_rows": 1000
  }
}
```

##### Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `sql` | string | Yes | Consulta SQL/RQL a ejecutar |
| `parameters` | array | No | Parámetros para la consulta (posicionales) |
| `session_id` | string | No | ID de sesión para mantener estado |
| `options` | object | No | Opciones de ejecución |

##### Options

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `timeout` | integer | 30 | Timeout en segundos |
| `max_rows` | integer | 1000 | Máximo número de filas a retornar |
| `format` | string | "json" | Formato de respuesta ("json", "csv") |

##### Response

```json
{
  "success": true,
  "data": {
    "columns": [
      {"name": "id", "type": "INTEGER", "position": 0},
      {"name": "name", "type": "TEXT", "position": 1},
      {"name": "dept", "type": "TEXT", "position": 2}
    ],
    "rows": [
      [1, "Juan Pérez", "Ventas"],
      [2, "María García", "IT"]
    ]
  },
  "message": "Consulta ejecutada exitosamente",
  "execution_time_ms": 42,
  "session_id": "session_123"
}
```

##### Status Codes

- `200 OK` - Consulta ejecutada exitosamente
- `400 Bad Request` - Consulta inválida o parámetros incorrectos
- `422 Unprocessable Entity` - Error de validación de SQL
- `500 Internal Server Error` - Error interno del servidor

##### Examples

**Simple Select:**

```bash
curl -X POST http://localhost:8080/api/v1/query/execute \
     -H "Content-Type: application/json" \
     -d '{
       "sql": "SELECT name, salary FROM employees WHERE dept = ?",
       "parameters": ["Ventas"]
     }'
```

**With Named Parameters:**

```bash
curl -X POST http://localhost:8080/api/v1/query/execute \
     -H "Content-Type: application/json" \
     -d '{
       "sql": "SELECT * FROM users WHERE age > :min_age AND dept = :dept",
       "parameters": {"min_age": 25, "dept": "IT"}
     }'
```

---

#### Validate Query

**POST** `/api/v1/query/validate`

Valida una consulta SQL sin ejecutarla.

##### Request Body

```json
{
  "sql": "SELECT * FROM employees WHERE dept = ?",
  "parameters": [3]
}
```

##### Response

```json
{
  "success": true,
  "valid": true,
  "message": "Consulta válida",
  "parameters_detected": ["dept_id"]
}
```

---

#### Batch Execute

**POST** `/api/v1/query/batch`

Ejecuta múltiples consultas en una sola transacción.

##### Request Body

```json
{
  "queries": [
    {
      "sql": "INSERT INTO employees (name, dept) VALUES (?, ?)",
      "parameters": ["Nuevo Empleado", "Ventas"]
    },
    {
      "sql": "SELECT COUNT(*) FROM employees",
      "parameters": []
    }
  ],
  "transaction": true
}
```

##### Response

```json
{
  "success": true,
  "results": [
    {
      "success": true,
      "affected_rows": 1,
      "execution_time_ms": 15
    },
    {
      "success": true,
      "data": {
        "columns": [{"name": "COUNT", "type": "INTEGER"}],
        "rows": [[25]]
      },
      "execution_time_ms": 8
    }
  ],
  "transaction_id": "txn_123"
}
```

---

### Form API

#### Execute Form

**POST** `/api/v1/form/{form_name}`

Ejecuta un formulario FDL2.

##### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `form_name` | string | Nombre del formulario (sin extensión) |

##### Request Body

```json
{
  "action": "query",
  "parameters": {
    "nroleg": 12345,
    "nombre": "Juan Pérez"
  },
  "submit": true
}
```

##### Response

```json
{
  "success": true,
  "data": {
    "form_result": "Operación completada",
    "generated_sql": "SELECT * FROM employees WHERE nroleg = ?",
    "rows_affected": 1
  },
  "message": "Formulario ejecutado exitosamente",
  "form_title": "Consulta Empleados",
  "form_name": "empleados"
}
```

##### Examples

**Execute Employee Form:**

```bash
curl -X POST http://localhost:8080/api/v1/form/empleados \
     -H "Content-Type: application/json" \
     -d '{
       "action": "query",
       "parameters": {
         "dept": "Ventas",
         "limit": 10
       },
       "submit": true
     }'
```

---

#### Validate Form

**POST** `/api/v1/form/{form_name}/validate`

Valida los parámetros de un formulario sin ejecutarlo.

##### Request Body

```json
{
  "parameters": {
    "nroleg": 12345,
    "nombre": "Juan Pérez"
  }
}
```

##### Response

```json
{
  "success": true,
  "valid": true,
  "validation_errors": [],
  "message": "Formulario válido",
  "detected_fields": ["nroleg", "nombre"]
}
```

---

#### List Forms

**GET** `/api/v1/forms`

Lista todos los formularios disponibles.

##### Response

```json
{
  "forms": [
    {
      "name": "empleados",
      "title": "Consulta Empleados",
      "description": "Formulario para consultar empleados",
      "fields_count": 3,
      "actions_count": 1
    },
    {
      "name": "reportes",
      "title": "Generación de Reportes",
      "description": "Formulario para generar reportes",
      "fields_count": 5,
      "actions_count": 2
    }
  ],
  "total": 2
}
```

---

### Session API

#### Create Session

**POST** `/api/v1/session`

Crea una nueva sesión para mantener estado entre consultas.

##### Response

```json
{
  "session_id": "session_abc123",
  "message": "Sesión creada exitosamente",
  "expires_in": 3600,
  "created_at": "2024-01-15T10:30:00Z"
}
```

##### Headers

La sesión se mantiene via cookie:

```
Set-Cookie: noctra_session=session_abc123; Path=/; HttpOnly; Max-Age=3600
```

---

#### Get Session

**GET** `/api/v1/session/{session_id}`

Obtiene información de una sesión existente.

##### Response

```json
{
  "session_id": "session_abc123",
  "status": "active",
  "created_at": "2024-01-15T10:30:00Z",
  "last_activity": "2024-01-15T10:35:00Z",
  "variables": {
    "user_id": "user123",
    "current_dept": "Ventas"
  }
}
```

---

#### List Sessions

**GET** `/api/v1/sessions`

Lista todas las sesiones activas (solo para administradores).

##### Response

```json
{
  "sessions": [
    {
      "id": "session_abc123",
      "created_at": "2024-01-15T10:30:00Z",
      "status": "active",
      "last_activity": "2024-01-15T10:35:00Z"
    }
  ],
  "total": 1
}
```

---

#### Delete Session

**DELETE** `/api/v1/session/{session_id}`

Elimina una sesión.

##### Response

```json
{
  "message": "Sesión session_abc123 eliminada"
}
```

---

### Metrics API

#### Server Metrics

**GET** `/api/v1/metrics`

Obtiene métricas del servidor (requiere métrica habilitada).

##### Response

```json
{
  "server": {
    "uptime": 3600,
    "version": "0.1.0",
    "memory_usage": {
      "rss": "45.2 MB",
      "heap": "12.1 MB"
    }
  },
  "database": {
    "connections": 5,
    "active_queries": 2,
    "total_queries": 150
  },
  "api": {
    "total_requests": 1250,
    "success_rate": 0.987,
    "average_response_time": 45
  },
  "sessions": {
    "active": 8,
    "total_created": 25,
    "average_lifetime": 1800
  }
}
```

---

## Error Handling

### Error Response Format

```json
{
  "success": false,
  "error": "DESCRIPCION_DEL_ERROR",
  "code": "ERROR_CODE",
  "details": {
    "field": "sql",
    "message": "Detalle específico del error"
  },
  "timestamp": "2024-01-15T10:30:00Z"
}
```

### Common Error Codes

| Code | Description | HTTP Status |
|------|-------------|-------------|
| `INVALID_SQL` | Sintaxis SQL inválida | 422 |
| `DATABASE_ERROR` | Error de base de datos | 500 |
| `SESSION_NOT_FOUND` | Sesión no encontrada | 404 |
| `FORM_NOT_FOUND` | Formulario no encontrado | 404 |
| `VALIDATION_ERROR` | Error de validación | 400 |
| `TIMEOUT` | Consulta expiró | 408 |
| `UNAUTHORIZED` | No autorizado | 401 |

### Example Error Response

```json
{
  "success": false,
  "error": "Error parseando SQL: syntax error at line 1",
  "code": "INVALID_SQL",
  "details": {
    "sql": "SELECT * FORM users",
    "position": {"line": 1, "column": 16}
  },
  "timestamp": "2024-01-15T10:30:00Z"
}
```

---

## Rate Limiting

El servidor implementa rate limiting para prevenir abuso:

- **Default:** 100 requests por minuto por IP
- **Authenticated:** 1000 requests por minuto
- **Burst:** 10 requests por segundo

### Headers de Rate Limit

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1642249200
X-RateLimit-Type: "api"
```

---

## WebSocket API

### Connection

**GET** `/ws`

Conecta al endpoint WebSocket para updates en tiempo real.

#### Client Message Format

```json
{
  "type": "ping|query|subscribe|stats",
  "data": {},
  "timestamp": "2024-01-15T10:30:00Z"
}
```

#### Server Message Format

```json
{
  "type": "welcome|pong|query_result|error",
  "data": {},
  "timestamp": "2024-01-15T10:30:00Z"
}
```

#### Example WebSocket Client

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = function() {
    console.log('Conectado al servidor WebSocket');
    
    // Enviar ping
    ws.send(JSON.stringify({
        type: 'ping',
        timestamp: new Date().toISOString()
    }));
};

ws.onmessage = function(event) {
    const message = JSON.parse(event.data);
    console.log('Mensaje recibido:', message);
};

ws.onerror = function(error) {
    console.error('Error WebSocket:', error);
};
```

---

## Examples

### JavaScript/Node.js

```javascript
const axios = require('axios');

class NoctraClient {
    constructor(baseURL = 'http://localhost:8080') {
        this.baseURL = baseURL;
    }
    
    async executeQuery(sql, parameters = []) {
        const response = await axios.post(`${this.baseURL}/api/v1/query/execute`, {
            sql,
            parameters
        });
        return response.data;
    }
    
    async executeForm(formName, parameters) {
        const response = await axios.post(`${this.baseURL}/api/v1/form/${formName}`, {
            action: 'query',
            parameters,
            submit: true
        });
        return response.data;
    }
    
    async createSession() {
        const response = await axios.post(`${this.baseURL}/api/v1/session`);
        return response.data.session_id;
    }
}

// Uso
const client = new NoctraClient();

// Consulta simple
const result = await client.executeQuery(
    'SELECT * FROM employees WHERE dept = ?',
    ['Ventas']
);
console.log(result.data);

// Ejecutar formulario
const formResult = await client.executeForm('empleados', {
    dept: 'IT',
    limit: 10
});
console.log(formResult);
```

### Python

```python
import requests
import json

class NoctraClient:
    def __init__(self, base_url='http://localhost:8080'):
        self.base_url = base_url
        
    def execute_query(self, sql, parameters=None):
        if parameters is None:
            parameters = []
            
        response = requests.post(
            f'{self.base_url}/api/v1/query/execute',
            json={'sql': sql, 'parameters': parameters}
        )
        return response.json()
    
    def execute_form(self, form_name, parameters):
        response = requests.post(
            f'{self.base_url}/api/v1/form/{form_name}',
            json={
                'action': 'query',
                'parameters': parameters,
                'submit': True
            }
        )
        return response.json()

# Uso
client = NoctraClient()

# Consulta simple
result = client.execute_query(
    'SELECT * FROM employees WHERE dept = ?',
    ['Ventas']
)
print(result['data'])

# Ejecutar formulario
form_result = client.execute_form('empleados', {
    'dept': 'IT',
    'limit': 10
})
print(form_result)
```

### curl

```bash
# Health check
curl -X GET http://localhost:8080/health

# Consulta simple
curl -X POST http://localhost:8080/api/v1/query/execute \
     -H "Content-Type: application/json" \
     -d '{
       "sql": "SELECT COUNT(*) FROM users",
       "parameters": []
     }'

# Consulta con parámetros
curl -X POST http://localhost:8080/api/v1/query/execute \
     -H "Content-Type: application/json" \
     -d '{
       "sql": "SELECT * FROM employees WHERE dept = :dept AND salary > :min_salary",
       "parameters": {
         "dept": "IT",
         "min_salary": 50000
       }
     }'

# Ejecutar formulario
curl -X POST http://localhost:8080/api/v1/form/empleados \
     -H "Content-Type: application/json" \
     -d '{
       "action": "query",
       "parameters": {
         "dept": "Ventas",
         "limit": 5
       },
       "submit": true
     }'

# Crear sesión
curl -X POST http://localhost:8080/api/v1/session

# Batch queries
curl -X POST http://localhost:8080/api/v1/query/batch \
     -H "Content-Type: application/json" \
     -d '{
       "queries": [
         {
           "sql": "INSERT INTO logs (message) VALUES (?)",
           "parameters": ["Test log entry"]
         },
         {
           "sql": "SELECT COUNT(*) FROM logs",
           "parameters": []
         }
       ],
       "transaction": true
     }'
```

---

## Changelog

### v0.1.0 (2024-01-15)

- **Added**: Query execution API
- **Added**: Form execution API
- **Added**: Session management
- **Added**: WebSocket support
- **Added**: Health check endpoint
- **Added**: Batch query execution
- **Added**: Query validation
- **Added**: Metrics endpoint

---

## Support

Para soporte técnico:

- **Issues**: Crear un issue en el repositorio
- **Documentation**: [docs/](./)
- **Examples**: [examples/](../examples/)

---

## License

MIT License - ver [LICENSE](../LICENSE) para detalles.