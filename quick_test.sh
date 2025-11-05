#!/bin/bash
echo "=== Noctra Quick Test ==="

# Verificar que el servidor esté corriendo
if curl -f -s http://127.0.0.1:8080/health > /dev/null; then
    echo "✅ Servidor disponible"
    
    # Test básico
    RESULT=$(curl -s -X POST http://127.0.0.1:8080/api/v1/query/execute \
        -H "Content-Type: application/json" \
        -d '{"sql": "SELECT 1 as test", "parameters": []}')
    
    if echo "$RESULT" | jq -e '.success == true' > /dev/null; then
        echo "✅ API funcionando"
    else
        echo "❌ API falló"
        echo "$RESULT"
    fi
else
    echo "❌ Servidor no disponible"
fi
