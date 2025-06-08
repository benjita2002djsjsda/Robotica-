// src/config.rs
use std::collections::HashMap;

pub const FILAS_MAPA: usize = 6;
pub const COLUMNAS_MAPA: usize = 8;

pub const ESTADO_META: &str = "M";

pub const ESTADOS_PELIGRO: [&str; 4] = ["P1", "P2", "P3", "P4"];
pub const OBSTACULOS: [&str; 10] = ["O1", "O2", "O3", "O4", "O5", "O6", "O7", "O8", "O9", "O10"];

pub const MAPA_ESTADOS: [[&str; 8]; 6] = [
    ["S0", "S1", "P1", "O1", "S3", "O2", "S4", "S5"],
    ["O3", "S6", "S7", "S8", "S9", "S10", "S11", "O4"],
    ["S12", "P2", "S14", "O5", "S15", "P3", "S17", "S18"],
    ["S19", "S20", "S21", "S22", "M", "S24", "S25", "O6"],
    ["S26", "O7", "O8", "S27", "S28", "S29", "P4", "S31"],
    ["S32", "O9", "S33", "S34", "O10", "S35", "S36", "S37"],
];

// Recompensas como función global
pub fn obtener_recompensas() -> HashMap<&'static str, f64> {
    let mut recompensas = HashMap::new();
    for fila in MAPA_ESTADOS.iter() {
        for &estado in fila.iter() {
            let recompensa = if estado == ESTADO_META {
                10.0
            } else if ESTADOS_PELIGRO.contains(&estado) {
                -0.5
            } else {
                -0.1
            };
            recompensas.insert(estado, recompensa);
        }
    }
    recompensas
}

// Acciones válidas como strings
pub fn acciones() -> Vec<&'static str> {
    vec!["N", "S", "E", "O"]
}

// Probabilidades de transición expresadas como cadenas (&str)
pub fn prob_transicion() -> HashMap<&'static str, HashMap<&'static str, f64>> {
    HashMap::from([
        ("N", HashMap::from([("N", 0.8), ("E", 0.1), ("O", 0.1)])),
        ("S", HashMap::from([("S", 0.8), ("E", 0.1), ("O", 0.1)])),
        ("E", HashMap::from([("E", 0.8), ("N", 0.1), ("S", 0.1)])),
        ("O", HashMap::from([("O", 0.8), ("N", 0.1), ("S", 0.1)])),
    ])
}
