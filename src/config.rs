// src/config.rs
use std::collections::HashMap;

/// Módulo de configuración del MDP - Definición del mundo y parámetros
///
/// Contiene todas las constantes, estructuras de datos y configuraciones
/// necesarias para definir el entorno del Proceso de Decisión de Markov,
/// incluyendo el mapa de estados, recompensas y probabilidades de transición.

pub const FILAS_MAPA: usize = 6;
pub const COLUMNAS_MAPA: usize = 8;

pub const INTERVALO_MOVIMIENTO: f64 = 0.5; // Intervalo de tiempo entre movimientos en segundos
pub const UMBRAL_CONVERGENCIA: f64 = 0.001; // Umbral de convergencia para Value Iteration

pub const ESTADO_META: &str = "M";
pub const ESTADOS_PELIGRO: [&str; 4] = ["P1", "P2", "P3", "P4"];
pub const OBSTACULOS: [&str; 10] = ["O1", "O2", "O3", "O4", "O5", "O6", "O7", "O8", "O9", "O10"];

/// Matriz que define la topología completa del mundo
/// S = Estados seguros, P = Estados peligrosos, O = Obstáculos, M = Meta
pub const MAPA_ESTADOS: [[&str; 8]; 6] = [
    ["S0", "S1", "P1", "O1", "S3", "O2", "S4", "S5"],
    ["O3", "S6", "S7", "S8", "S9", "S10", "S11", "O4"],
    ["S12", "P2", "S14", "O5", "S15", "P3", "S17", "S18"],
    ["S19", "S20", "S21", "S22", "M", "S24", "S25", "O6"],
    ["S26", "O7", "O8", "S27", "S28", "S29", "P4", "S31"],
    ["S32", "O9", "S33", "S34", "O10", "S35", "S36", "S37"],
];

/// Función que construye el mapa de recompensas para todos los estados del mundo

pub fn obtener_recompensas() -> HashMap<&'static str, f64> {
    let mut recompensas = HashMap::new();
    for fila in MAPA_ESTADOS.iter() {
        for &estado in fila.iter() {
            let recompensa = if estado == ESTADO_META {
                10.0 // Recompensa alta por alcanzar la meta
            } else if ESTADOS_PELIGRO.contains(&estado) {
                -0.5 // Penalización por estados peligrosos
            } else {
                -0.1 // Costo por vivir (living penalty)
            };
            recompensas.insert(estado, recompensa);
        }
    }
    recompensas
}

/// Retorna las acciones válidas en el MDP
///
/// Define las cuatro direcciones cardinales posibles:
/// N = Norte, S = Sur, E = Este, O = Oeste
pub fn acciones() -> Vec<&'static str> {
    vec!["N", "S", "E", "O"]
}

/// Construye el modelo estocástico de transiciones para cada acción
///
/// Define las probabilidades de transición considerando ruido en el movimiento:
/// - 80% de probabilidad de moverse en la dirección deseada
/// - 20% dividido equitativamente entre las direcciones perpendiculares

pub fn prob_transicion() -> HashMap<&'static str, HashMap<&'static str, f64>> {
    HashMap::from([
        ("N", HashMap::from([("N", 0.8), ("E", 0.1), ("O", 0.1)])),
        ("S", HashMap::from([("S", 0.8), ("E", 0.1), ("O", 0.1)])),
        ("E", HashMap::from([("E", 0.8), ("N", 0.1), ("S", 0.1)])),
        ("O", HashMap::from([("O", 0.8), ("N", 0.1), ("S", 0.1)])),
    ])
}
