use crate::mdp_model::value_iteration;
use std::collections::HashMap;

// Modelos de ruido alternativos
const MODELOS_RUIDO: &[(f64, f64, f64)] = &[
    (0.1, 0.8, 0.1),
    (0.05, 0.9, 0.05),
    (0.15, 0.7, 0.15),
    (0.25, 0.5, 0.25),
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

pub fn evaluar_robustez(
    politica_base: &HashMap<String, String>,
    landa: f64,
) -> Vec<(String, usize)> {
    let mut resultados = Vec::new();

    for (izq, centro, der) in MODELOS_RUIDO.iter() {
        let porcentaje_exito = (*centro * 100.0) as usize;
        let etiqueta = format!(
            "{}% éxito ({}% L/R)",
            porcentaje_exito,
            (*izq * 100.0) as usize
        );
        let modelo_ruido = construir_modelo_ruido(*izq, *centro, *der);

        // Usar umbral más estricto
        let (_, politica_adaptada) = value_iteration(landa, Some(1e-6), Some(&modelo_ruido));

        let cambios = politica_base
            .iter()
            .filter(|(estado, accion)| match politica_adaptada.get(*estado) {
                Some(a) => a != *accion,
                None => true,
            })
            .count();

        println!("Ruido {}: {} cambios", etiqueta, cambios);
        resultados.push((etiqueta, cambios));
    }

    resultados
}
