use crate::config::{prob_transicion, MAPA_ESTADOS, OBSTACULOS};
use crate::mdp_model::{mover, obtener_estado, obtener_posicion};
use ndarray::Array2;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

/// Módulo de matrices de transición - Construcción y exportación de modelos probabilísticos
///
/// Genera las matrices de transición estocásticas para cada acción, representando
/// las probabilidades de moverse de cualquier estado a cualquier otro estado
/// al ejecutar una acción específica en el MDP.

/// Construye la matriz de transición para una acción específica
///
/// Crea una matriz NxN donde N es el número de estados válidos (no obstáculos).
/// Cada elemento [i,j] representa la probabilidad de transición del estado i
/// al estado j al ejecutar la acción dada, considerando el ruido del modelo.

pub fn construir_matriz_transicion(accion: &str) -> Array2<f32> {
    let modelo_transicion = prob_transicion();

    // Filtrado de estados válidos (excluyendo obstáculos)
    let estados: Vec<String> = MAPA_ESTADOS
        .iter()
        .flatten()
        .filter(|s| !s.is_empty() && !OBSTACULOS.contains(s))
        .map(|s| s.to_string())
        .collect();

    let total = estados.len();

    // Mapeo de nombres de estados a índices matriciales
    let mut estado_a_indice: HashMap<String, usize> = HashMap::new();
    for (i, estado) in estados.iter().enumerate() {
        estado_a_indice.insert(estado.clone(), i);
    }

    // Inicialización de la matriz de transición
    let mut matriz = Array2::<f32>::zeros((total, total));

    // Construcción de probabilidades de transición para cada estado origen
    for estado_origen in &estados {
        if let Ok((fila, col)) = obtener_posicion(estado_origen) {
            if let Some(transiciones) = modelo_transicion.get(accion) {
                // Procesamiento de cada posible resultado de la acción (con ruido)
                for (resultado, prob) in transiciones {
                    let (nueva_fila, nueva_col) = mover(fila, col, resultado);
                    let mut estado_destino = obtener_estado(nueva_fila, nueva_col)
                        .unwrap_or_else(|| estado_origen.as_str());

                    // Manejo de colisiones: si se choca con obstáculo, quedarse en el mismo lugar
                    if OBSTACULOS.contains(&estado_destino) {
                        estado_destino = estado_origen.as_str();
                    }

                    // Actualización de la matriz de transición
                    let i_origen = *estado_a_indice.get(estado_origen).unwrap();
                    let i_destino = *estado_a_indice.get(estado_destino).unwrap();
                    matriz[[i_origen, i_destino]] += *prob as f32;
                }
            }
        }
    }

    matriz
}

/// Genera y guarda las matrices de transición para todas las acciones en formato CSV
///
/// Construye las matrices de transición para las cuatro acciones cardinales
/// (N, S, E, O) y las exporta como archivos CSV separados. Útil para análisis
/// externos, visualización o verificación del modelo usado en Q-Value Iteration.
pub fn guardar_matrices_transicion_csv() {
    for accion in ["N", "S", "E", "O"] {
        let matriz = construir_matriz_transicion(accion);
        let nombre_archivo = format!("matriz_transicion_{}.csv", accion);
        let mut archivo = File::create(&nombre_archivo).expect("No se pudo crear el archivo");

        // Escritura de la matriz fila por fila en formato CSV
        for fila in matriz.rows() {
            let linea: Vec<String> = fila.iter().map(|v| format!("{:.2}", v)).collect();
            writeln!(archivo, "{}", linea.join(",")).expect("Error escribiendo archivo");
        }

        println!("✅ {} guardada.", nombre_archivo);
    }
}
