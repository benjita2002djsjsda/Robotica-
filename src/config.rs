// src/config.rs
/// Configuration for the MDP model, including map layout, states, rewards, and actions.
use std::collections::HashMap;

/// Number of rows in the map.
pub const FILAS_MAPA: usize = 6;
/// Number of columns in the map.
pub const COLUMNAS_MAPA: usize = 8;

/// Representation of the goal state on the map.
pub const ESTADO_META: &str = "M";

/// Array of states considered as danger zones or pits.
pub const ESTADOS_PELIGRO: [&str; 4] = ["P1", "P2", "P3", "P4"];
/// Array of states considered as obstacles.
pub const OBSTACULOS: [&str; 10] = ["O1", "O2", "O3", "O4", "O5", "O6", "O7", "O8", "O9", "O10"];

/// 2D array defining the layout of the map and the name of each state.
pub const MAPA_ESTADOS: [[&str; 8]; 6] = [
    ["S0", "S1", "P1", "O1", "S3", "O2", "S4", "S5"],
    ["O3", "S6", "S7", "S8", "S9", "S10", "S11", "O4"],
    ["S12", "P2", "S14", "O5", "S15", "P3", "S17", "S18"],
    ["S19", "S20", "S21", "S22", "M", "S24", "S25", "O6"],
    ["S26", "O7", "O8", "S27", "S28", "S29", "P4", "S31"],
    ["S32", "O9", "S33", "S34", "O10", "S35", "S36", "S37"],
];

/// Generates and returns a map of rewards for each state.
///
/// Rewards are defined as:
/// - Goal state (`ESTADO_META`): +10.0
/// - Danger states (`ESTADOS_PELIGRO`): -0.5
/// - All other non-obstacle states: -0.1 (step cost)
///
/// # Returns
///
/// A `HashMap` where keys are state names (`&'static str`) and values are their rewards (`f64`).
pub fn obtener_recompensas() -> HashMap<&'static str, f64> {
    let mut hm_s_f64Recompensas = HashMap::new();
    for arr_sFilaEstados in MAPA_ESTADOS.iter() {
        for &sEstado in arr_sFilaEstados.iter() {
            let f64Recompensa = if sEstado == ESTADO_META {
                10.0
            } else if ESTADOS_PELIGRO.contains(&sEstado) {
                -0.5
            } else {
                -0.1
            };
            hm_s_f64Recompensas.insert(sEstado, f64Recompensa);
        }
    }
    hm_s_f64Recompensas
}

/// Returns a vector of valid actions the robot can take.
///
/// Actions are: "N" (North), "S" (South), "E" (East), "O" (West).
///
/// # Returns
///
/// A `Vec<&'static str>` containing the action strings.
pub fn acciones() -> Vec<&'static str> {
    vec!["N", "S", "E", "O"]
}

/// Defines the default transition probabilities for actions.
///
/// Assumes a stochastic environment where actions have a primary success probability
/// and a chance to move to adjacent unintended states.
/// For example, action "N" has an 80% chance of moving North, 10% East, 10% West.
///
/// # Returns
///
/// A `HashMap` where keys are action names (`&'static str`) and values are
/// another `HashMap`. This inner map's keys are the resulting effective directions
/// (`&'static str`) and values are their probabilities (`f64`).
pub fn prob_transicion() -> HashMap<&'static str, HashMap<&'static str, f64>> {
    HashMap::from([
        ("N", HashMap::from([("N", 0.8), ("E", 0.1), ("O", 0.1)])),
        ("S", HashMap::from([("S", 0.8), ("E", 0.1), ("O", 0.1)])),
        ("E", HashMap::from([("E", 0.8), ("N", 0.1), ("S", 0.1)])),
        ("O", HashMap::from([("O", 0.8), ("N", 0.1), ("S", 0.1)])),
    ])
}
