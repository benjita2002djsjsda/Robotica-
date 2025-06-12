use std::collections::HashMap;

/// Módulo de análisis de robustez - Evaluación bajo diferentes niveles de ruido
/// Los modelos van desde alta precisión (90% éxito) hasta alta incertidumbre (50% éxito)
/// para políticas generadas con Q-Value Iteration
///
pub const MODELOS_ROBUSTEZ: &[(f64, f64, f64)] = &[
    (0.1, 0.8, 0.1),   // Modelo estándar: 80% éxito, 20% ruido
    (0.05, 0.9, 0.05), // Alta precisión: 90% éxito, 10% ruido
    (0.15, 0.7, 0.15), // Más ruido: 70% éxito, 30% ruido
    (0.25, 0.5, 0.25), // Alto ruido: 50% éxito, 50% ruido
];

/// Construye un modelo de transición estocástico con ruido direccional
///
/// Crea un modelo donde cada acción tiene una probabilidad de éxito y probabilidades
/// de error en las direcciones perpendiculares. Simula la incertidumbre real en
/// sistemas robóticos donde los movimientos no siempre son perfectos. Se usa
/// para evaluar la robustez de las políticas obtenidas con Q-Value Iteration.
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
                // Norte: éxito=Norte, errores=Este/Oeste
                transiciones.insert("N".to_string(), centro);
                transiciones.insert("E".to_string(), der);
                transiciones.insert("O".to_string(), izq);
            }
            "S" => {
                // Sur: éxito=Sur, errores=Este/Oeste
                transiciones.insert("S".to_string(), centro);
                transiciones.insert("E".to_string(), der);
                transiciones.insert("O".to_string(), izq);
            }
            "E" => {
                // Este: éxito=Este, errores=Norte/Sur
                transiciones.insert("E".to_string(), centro);
                transiciones.insert("N".to_string(), izq);
                transiciones.insert("S".to_string(), der);
            }
            "O" => {
                // Oeste: éxito=Oeste, errores=Norte/Sur
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
