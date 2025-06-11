use std::collections::HashMap;

// Modelos de robustez específicos para evaluación
pub const MODELOS_ROBUSTEZ: &[(f64, f64, f64)] = &[
    (0.1, 0.8, 0.1),   // (10%, 80%, 10%)
    (0.05, 0.9, 0.05), // (5%, 90%, 5%)
    (0.15, 0.7, 0.15), // (15%, 70%, 15%)
    (0.25, 0.5, 0.25), // (25%, 50%, 25%)
];

pub fn construir_modelo_ruido(
    izq: f64,
    centro: f64,
    der: f64,
) -> HashMap<String, HashMap<String, f64>> {
    let acciones = vec!["N", "S", "E", "O"];
    let mut modelo = HashMap::new();

    for accion in &acciones {
        let mut transiciones = HashMap::new();
        match *accion {
            "N" => {
                transiciones.insert("N".to_string(), centro);
                transiciones.insert("E".to_string(), der);
                transiciones.insert("O".to_string(), izq);
            }
            "S" => {
                transiciones.insert("S".to_string(), centro);
                transiciones.insert("E".to_string(), der);
                transiciones.insert("O".to_string(), izq);
            }
            "E" => {
                transiciones.insert("E".to_string(), centro);
                transiciones.insert("N".to_string(), izq);
                transiciones.insert("S".to_string(), der);
            }
            "O" => {
                transiciones.insert("O".to_string(), centro);
                transiciones.insert("N".to_string(), izq);
                transiciones.insert("S".to_string(), der);
            }
            _ => {}
        }
        modelo.insert(accion.to_string(), transiciones);
    }

    modelo
}
