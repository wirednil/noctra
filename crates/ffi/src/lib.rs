//! Noctra FFI - Foreign Function Interface
//!
//! Esta crate proporciona una interfaz C para integrar Noctra
//! con otros lenguajes y aplicaciones.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

/// Resultado de funciones FFI
pub type FfiResult = c_int;

/// Constantes de resultado
pub const FFI_SUCCESS: c_int = 0;
pub const FFI_ERROR: c_int = -1;
pub const FFI_INVALID_INPUT: c_int = -2;

/// Ejecutar consulta SQL y retornar resultado JSON
///
/// # Arguments
/// * `sql` - Query SQL como string C
/// * `out_json` - Buffer para resultado JSON (allocado por la función)
///
/// # Returns
/// FFI_SUCCESS on success, FFI_ERROR on failure
#[no_mangle]
pub extern "C" fn noctra_exec(sql: *const c_char, out_json: *mut *mut c_char) -> FfiResult {
    // Verificar input válido
    if sql.is_null() || out_json.is_null() {
        return FFI_INVALID_INPUT;
    }

    // Convertir C string a Rust string
    let _sql_str = match unsafe { CStr::from_ptr(sql).to_str() } {
        Ok(s) => s,
        Err(_) => return FFI_INVALID_INPUT,
    };

    // TODO: Implementar ejecución real de query
    // Por ahora retornamos un resultado de ejemplo

    let result_json = r#"{
        "success": true,
        "message": "Query executed (FFI mock)",
        "rows": 0,
        "execution_time_ms": 0
    }"#;

    // Convertir a C string
    let c_json = match CString::new(result_json) {
        Ok(s) => s,
        Err(_) => return FFI_ERROR,
    };

    // Retornar JSON al caller
    unsafe {
        *out_json = c_json.into_raw();
    }

    FFI_SUCCESS
}

/// Obtener versión de la librería
///
/// # Returns
/// String C con la versión
#[no_mangle]
pub extern "C" fn noctra_version() -> *const c_char {
    "0.1.0\0".as_ptr() as *const c_char
}

/// Liberar memoria de strings retornados por funciones FFI
///
/// # Arguments
/// * `ptr` - Puntero a liberar
#[no_mangle]
pub extern "C" fn noctra_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

/// Inicializar librería Noctra
///
/// # Returns
/// FFI_SUCCESS si inicialización exitosa
#[no_mangle]
pub extern "C" fn noctra_init() -> FfiResult {
    // TODO: Inicializar configuración, conexiones, etc.
    // Por ahora siempre exitoso
    FFI_SUCCESS
}

/// Cerrar librería Noctra
#[no_mangle]
pub extern "C" fn noctra_shutdown() {
    // TODO: Cleanup de recursos
    // Cerrar conexiones, liberar memoria, etc.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let version = unsafe { CStr::from_ptr(noctra_version()) };
        assert_eq!(version.to_str().unwrap(), "0.1.0");
    }

    #[test]
    fn test_exec_invalid_input() {
        let mut out_json: *mut c_char = std::ptr::null_mut();
        let result = noctra_exec(std::ptr::null(), &mut out_json);
        assert_eq!(result, FFI_INVALID_INPUT);
    }
}
