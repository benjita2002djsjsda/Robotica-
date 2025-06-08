/// Functions for constructing and saving transition probability matrices for the MDP.
use crate::config::{prob_transicion, MAPA_ESTADOS, OBSTACULOS};
use crate::mdp_model::{mover, obtener_estado, obtener_posicion};
use ndarray::Array2;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

/// Constructs a 2D transition matrix for a given action.
///
/// The matrix represents P(s' | s, a), where s is the origin state (row)
/// and s' is the destination state (column). Values are probabilities.
/// Obstacle states are excluded from the matrix dimensions.
/// If a move leads to an obstacle or out of bounds, the agent stays in the current state.
///
/// # Arguments
///
/// * `sAccion` - The action for which to build the matrix (e.g., "N", "S").
///
/// # Returns
///
/// An `ndarray::Array2<f32>` representing the transition matrix.
pub fn construir_matriz_transicion(sAccion: &str) -> Array2<f32> {
    let hm_s_hm_s_f64ModeloTransicion = prob_transicion();

    let vec_sEstados: Vec<String> = MAPA_ESTADOS
        .iter()
        .flatten()
        .filter(|s| !s.is_empty() && !OBSTACULOS.contains(s))
        .map(|s| s.to_string())
        .collect();

    let uiTotalEstados = vec_sEstados.len();
    let mut hm_s_uiEstadoAIndice: HashMap<String, usize> = HashMap::new();
    for (uiIndex, sEstado) in vec_sEstados.iter().enumerate() {
        hm_s_uiEstadoAIndice.insert(sEstado.clone(), uiIndex);
    }

    let mut arr2_f32Matriz = Array2::<f32>::zeros((uiTotalEstados, uiTotalEstados));

    for sEstadoOrigen in &vec_sEstados {
        // Ensure `obtener_posicion` is called with `&str` as expected by its updated signature `sEstado: &str`
        if let Some((uiFila, uiCol)) = obtener_posicion(sEstadoOrigen.as_str()) {
            if let Some(opt_ref_hm_s_f64Transiciones) = hm_s_hm_s_f64ModeloTransicion.get(sAccion) {
                for (sResultado, f64Prob) in opt_ref_hm_s_f64Transiciones.iter() {
                    // Ensure `mover` is called with `usize, usize, &str` as expected by its updated signature
                    // `sResultado` is `&&str` from iter(), so `*sResultado` is `&str`
                    let (iNuevaFila, iNuevaCol) = mover(uiFila, uiCol, *sResultado);

                    // Ensure `obtener_estado` is called with `isize, isize`
                    let mut sEstadoDestino = obtener_estado(iNuevaFila, iNuevaCol)
                        .unwrap_or_else(|| sEstadoOrigen.as_str()); // sEstadoOrigen is &String, convert to &str

                    if OBSTACULOS.contains(&sEstadoDestino) { // sEstadoDestino is &str
                        sEstadoDestino = sEstadoOrigen.as_str(); // Ensure consistent type
                    }

                    let uiIndiceOrigen = *hm_s_uiEstadoAIndice.get(sEstadoOrigen.as_str()).unwrap();
                    let uiIndiceDestino = *hm_s_uiEstadoAIndice.get(sEstadoDestino).unwrap(); // sEstadoDestino is &str
                    arr2_f32Matriz[[uiIndiceOrigen, uiIndiceDestino]] += *f64Prob as f32;
                }
            }
        }
    }

    arr2_f32Matriz
}

/// Constructs transition matrices for all actions (N, S, E, O) and saves them to CSV files.
///
/// File names are in the format "matriz_transicion_{ACCION}.csv".
/// Each row in the CSV corresponds to an origin state, and each column to a destination state.
/// Values are probabilities formatted to two decimal places.
pub fn guardar_matrices_transicion_csv() {
    for sAccion in ["N", "S", "E", "O"].iter() { // Iterate over references to avoid moving
        let arr2_f32Matriz = construir_matriz_transicion(sAccion);
        let sNombreArchivo = format!("matriz_transicion_{}.csv", sAccion);
        let mut fArchivo = File::create(&sNombreArchivo).expect("No se pudo crear el archivo");

        for view_f32Fila in arr2_f32Matriz.rows() {
            let vec_sLinea: Vec<String> = view_f32Fila.iter().map(|f32Val| format!("{:.2}", f32Val)).collect();
            writeln!(fArchivo, "{}", vec_sLinea.join(",")).expect("Error escribiendo archivo");
        }

        println!("✅ {} guardada.", sNombreArchivo);
    }
}
