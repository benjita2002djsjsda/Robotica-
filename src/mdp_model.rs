/// Core logic for the Markov Decision Process (MDP), including state/position lookups, movement, and the value iteration algorithm.
use crate::config::{
    acciones, obtener_recompensas, prob_transicion, COLUMNAS_MAPA, ESTADO_META, FILAS_MAPA,
    MAPA_ESTADOS, OBSTACULOS,
};
use std::collections::HashMap;

/// Finds the (row, column) coordinates of a given state name in `MAPA_ESTADOS`.
///
/// # Arguments
///
/// * `sEstado` - The name of the state to find (e.g., "S0", "M").
///
/// # Returns
///
/// An `Option<(usize, usize)>` containing the (row, col) tuple if found, otherwise `None`.
pub fn obtener_posicion(sEstado: &str) -> Option<(usize, usize)> {
    for (uiFila, arr_sFilaEstados) in MAPA_ESTADOS.iter().enumerate() {
        for (uiCol, sNombreEstado) in arr_sFilaEstados.iter().enumerate() {
            if *sNombreEstado == sEstado {
                return Some((uiFila, uiCol));
            }
        }
    }
    None
}

/// Gets the state name at a given (row, column) coordinate.
///
/// Returns `None` if the coordinates are out of bounds or point to an obstacle.
///
/// # Arguments
///
/// * `iFila` - The row index.
/// * `iCol` - The column index.
///
/// # Returns
///
/// An `Option<&'static str>` containing the state name if valid, otherwise `None`.
pub fn obtener_estado(iFila: isize, iCol: isize) -> Option<&'static str> {
    if iFila >= 0 && iFila < FILAS_MAPA as isize && iCol >= 0 && iCol < COLUMNAS_MAPA as isize {
        let sEstado = MAPA_ESTADOS[iFila as usize][iCol as usize];
        if OBSTACULOS.contains(&sEstado) {
            return None;
        }
        return Some(sEstado);
    }
    None
}

/// Calculates the new (row, column) coordinates resulting from taking an action from a given position.
///
/// Does not perform bounds checking or obstacle checking; this is purely a coordinate calculation.
///
/// # Arguments
///
/// * `uiFila` - The current row index.
/// * `uiCol` - The current column index.
/// * `sAccion` - The action taken ("N", "S", "E", "O").
///
/// # Returns
///
/// A tuple `(isize, isize)` representing the new (row, column). `isize` is used to allow temporary out-of-bounds values.
pub fn mover(uiFila: usize, uiCol: usize, sAccion: &str) -> (isize, isize) {
    match sAccion {
        "N" => (uiFila as isize - 1, uiCol as isize),
        "S" => (uiFila as isize + 1, uiCol as isize),
        "E" => (uiFila as isize, uiCol as isize + 1),
        "O" => (uiFila as isize, uiCol as isize - 1),
        _ => (uiFila as isize, uiCol as isize),
    }
}

/// Performs the value iteration algorithm to find the optimal state values and policy.
///
/// # Arguments
///
/// * `f64Lambda` - The discount factor (gamma).
/// * `f64Epsilon` - The convergence threshold. Iteration stops when the maximum change in value is less than this.
/// * `opt_hm_s_hm_s_f64ProbTransExt` - An optional external transition probability model. If `None`, uses default probabilities from `config::prob_transicion()`.
///
/// # Returns
///
/// A tuple containing:
///   - `HashMap<&'static str, f64>`: State values (V).
///   - `HashMap<String, String>`: The optimal policy (mapping state names to action names).
pub fn value_iteration(
    f64Lambda: f64,
    f64Epsilon: f64,
    opt_hm_s_hm_s_f64ProbTransExt: Option<&HashMap<String, HashMap<String, f64>>>,
) -> (HashMap<&'static str, f64>, HashMap<String, String>) {
    let mut hm_s_f64V: HashMap<&'static str, f64> = HashMap::new();
    let mut hm_s_sPolitica: HashMap<String, String> = HashMap::new();

    let hm_s_f64RecompensasMap = obtener_recompensas();
    // Initialize state values V(s) to 0 for all states.
    for arr_sFilaEstados in MAPA_ESTADOS.iter() {
        for sEstado in arr_sFilaEstados.iter() {
            hm_s_f64V.insert(*sEstado, 0.0);
        }
    }

    // Prepare the base transition model if no external one is provided.
    let opt_hm_s_hm_s_f64ModeloBase: Option<HashMap<String, HashMap<String, f64>>> =
        if opt_hm_s_hm_s_f64ProbTransExt.is_none() {
            Some(
                prob_transicion()
                    .iter()
                    .map(|(sKey, hmInnerValue)| {
                        (
                            sKey.to_string(),
                            hmInnerValue.iter().map(|(sKey2, f64Value2)| (sKey2.to_string(), *f64Value2)).collect(),
                        )
                    })
                    .collect(),
            )
        } else {
            None
        };

    let mut bCambios = true;
    // Main loop of the value iteration algorithm. Continues until convergence.
    while bCambios {
        bCambios = false;
        // Create a new map for the updated values in this iteration.
        let mut hm_s_f64VNuevo = hm_s_f64V.clone();

        for arr_sFilaEstados in MAPA_ESTADOS.iter() {
            for sEstado in arr_sFilaEstados.iter() {
                // Goal state value is fixed to its reward and does not change.
                if *sEstado == ESTADO_META {
                    hm_s_f64VNuevo.insert(*sEstado, *hm_s_f64RecompensasMap.get(sEstado).unwrap());
                    continue;
                }

                // Find the best action from the current state.
                let mut f64MejorValor = f64::NEG_INFINITY;
                let mut sMejorAccion = String::new();

                for sAccion in acciones().iter() {
                    // Get transition probabilities for the current action, either from external or base model.
                    let hm_s_f64ProbAccion = match opt_hm_s_hm_s_f64ProbTransExt {
                        Some(ref_hmOuterProbTransExt) => ref_hmOuterProbTransExt.get(&sAccion.to_string()).unwrap(),
                        None => opt_hm_s_hm_s_f64ModeloBase
                            .as_ref()
                            .unwrap()
                            .get(&sAccion.to_string())
                            .unwrap(),
                    };

                    // Get current position of the state.
                    let (uiFilaActual, uiColActual) = obtener_posicion(sEstado).unwrap();
                    // Calculate the expected value sum(P(s'|s,a) * V(s')).
                    let mut f64ValorEsperado = 0.0;

                    for (sResultado, f64Probabilidad) in hm_s_f64ProbAccion.iter() {
                        // Determine the resulting state if this outcome occurs.
                        let (iNuevaFila, iNuevaCol) = mover(uiFilaActual, uiColActual, sResultado);
                        let sEstadoDestino = match obtener_estado(iNuevaFila, iNuevaCol) {
                            Some(sEstadoObtenido) => sEstadoObtenido,
                            None => *sEstado, // If move is invalid (hits wall/obstacle), it stays in the current state.
                        };
                        f64ValorEsperado += f64Probabilidad * hm_s_f64V.get(sEstadoDestino).unwrap();
                    }

                    // Bellman equation: R(s) + lambda * sum(P(s'|s,a) * V(s')).
                    let f64ValorTotal =
                        hm_s_f64RecompensasMap.get(sEstado).unwrap() + f64Lambda * f64ValorEsperado;

                    if f64ValorTotal > f64MejorValor {
                        f64MejorValor = f64ValorTotal;
                        sMejorAccion = sAccion.to_string();
                    }
                }

                hm_s_f64VNuevo.insert(*sEstado, f64MejorValor);
                // Check for convergence: if change in value is greater than epsilon, continue iterating.
                if (hm_s_f64V.get(sEstado).unwrap() - f64MejorValor).abs() > f64Epsilon {
                    bCambios = true;
                }

                // Update the policy for the current state.
                hm_s_sPolitica.insert(sEstado.to_string(), sMejorAccion);
            }
        }

        // Update the value map for the next iteration.
        hm_s_f64V = hm_s_f64VNuevo;
    }

    (hm_s_f64V, hm_s_sPolitica)
}
