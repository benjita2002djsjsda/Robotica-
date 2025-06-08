/// Handles MDP simulation, including visual simulation with Macroquad and a 1000-step statistical simulation.
// src/simulation.rs
use crate::config::{
    acciones, obtener_recompensas, ESTADOS_PELIGRO, ESTADO_META, MAPA_ESTADOS, OBSTACULOS,
};
use crate::mdp_model::{obtener_estado, obtener_posicion}; // Assuming these are already updated
use ::rand::seq::SliceRandom;
use ::rand::thread_rng;
use ::rand::Rng;
use macroquad::prelude::*;
use std::collections::HashMap;

/// Size of each cell in pixels for visual simulation.
const F32_TAMANO_CELDA: f32 = 80.0;
/// Margin around cells in pixels.
const F32_MARGEN: f32 = 2.0;
/// Default color for map cells.
const MQ_COLOR_NORMAL: Color = GRAY;
/// Color for danger/pit cells.
const MQ_COLOR_PELIGRO: Color = RED;
/// Color for the goal cell.
const MQ_COLOR_META: Color = GREEN;
/// Color for the robot's current cell.
const MQ_COLOR_ROBOT: Color = BLUE;
/// Color for obstacle cells.
const MQ_COLOR_OBSTACULO: Color = DARKGRAY;

/// Runs a visual simulation of the robot navigating the map using Macroquad.
///
/// The robot starts at a random non-goal, non-obstacle state.
/// It attempts to follow the provided policy but includes an epsilon chance for random exploration.
/// The simulation runs for a specified number of steps or until the robot reaches the goal.
///
/// # Arguments
///
/// * `ref_mut_hm_s_sPolitica` - A mutable reference to the policy (State -> Action) to follow.
/// * `uiPasos` - Maximum number of steps for this simulation run.
/// * `ref_mut_hm_s_f64RecompensasMap` - Mutable reference to rewards map (used to increment if goal is reached, though this seems unusual here).
pub async fn ejecutar_simulacion(
    ref_mut_hm_s_sPolitica: &mut HashMap<String, String>,
    uiPasos: usize,
    ref_mut_hm_s_f64RecompensasMap: &mut HashMap<&'static str, f64>,
) {
    let mut rngThreadRng = ::rand::thread_rng();
    let f64EpsilonSim = 0.8; // Epsilon for exploration in simulation

    // Initialize the current state from a random valid state
    let vec_sEstadosValidos: Vec<String> = MAPA_ESTADOS
        .iter()
        .flatten()
        .filter(|&&sEstadoRef| sEstadoRef != ESTADO_META && !OBSTACULOS.contains(&sEstadoRef))
        .map(|&sEstadoRef| sEstadoRef.to_string())
        .collect();

    let mut sEstadoActual = vec_sEstadosValidos.choose(&mut rngThreadRng).unwrap().clone();
    let mut uiPasoActual = 0;

    // Speed control: time between movements
    let mut f64UltimoMovimiento = get_time();
    let f64IntervaloMovimiento = 0.5; // Adjust this value to change speed (in seconds)

    loop {
        clear_background(WHITE);

        // Draw the map
        for (uiIFila, ref_arr_sFilaEstados) in MAPA_ESTADOS.iter().enumerate() {
            for (uiICol, ref_sEstadoNombre) in ref_arr_sFilaEstados.iter().enumerate() {
                let sEstadoDeCelda: &'static str = *ref_sEstadoNombre;
                let mqColorCell = if OBSTACULOS.contains(&sEstadoDeCelda) {
                    MQ_COLOR_OBSTACULO
                } else if ESTADOS_PELIGRO.contains(&sEstadoDeCelda) {
                    MQ_COLOR_PELIGRO
                } else if sEstadoDeCelda == ESTADO_META {
                    MQ_COLOR_META
                } else if sEstadoDeCelda == sEstadoActual.as_str() { // Compare &str with &str
                    MQ_COLOR_ROBOT
                } else {
                    MQ_COLOR_NORMAL
                };

                let f32X = uiICol as f32 * F32_TAMANO_CELDA;
                let f32Y = uiIFila as f32 * F32_TAMANO_CELDA;

                draw_rectangle(
                    f32X + F32_MARGEN,
                    f32Y + F32_MARGEN,
                    F32_TAMANO_CELDA - 2.0 * F32_MARGEN,
                    F32_TAMANO_CELDA - 2.0 * F32_MARGEN,
                    mqColorCell,
                );
            }
        }

        draw_text(
            &format!("Paso: {} - Estado: {}", uiPasoActual, sEstadoActual),
            10.0,
            20.0,
            20.0,
            BLACK,
        );

        next_frame().await;

        // Speed control: move only if enough time has passed
        let f64Ahora = get_time();
        if f64Ahora - f64UltimoMovimiento < f64IntervaloMovimiento {
            continue; // Wait until the interval passes
        }
        f64UltimoMovimiento = f64Ahora;

        if uiPasoActual >= uiPasos || sEstadoActual.as_str() == ESTADO_META {
            break;
        }

        let sAccionElegida = if rngThreadRng.gen::<f64>() < f64EpsilonSim {
            // Explore: choose a random action
            let vec_sAccionesPosibles = acciones(); // From config.rs
            vec_sAccionesPosibles.choose(&mut rngThreadRng).unwrap().to_string()
        } else {
            // Follow the optimal policy
            ref_mut_hm_s_sPolitica.get(&sEstadoActual).unwrap().clone()
        };

        // Call mdp_model::obtener_posicion, which expects sEstado: &str
        if let Some((uiFilaActual, uiColActual)) = obtener_posicion(sEstadoActual.as_str()) {
            // Call local mover
            let (uiNuevaFila, uiNuevaCol) = mover(uiFilaActual, uiColActual, &sAccionElegida);
            // Call mdp_model::obtener_estado, which expects iFila: isize, iCol: isize
            if let Some(sNuevoEstado) = obtener_estado(uiNuevaFila as isize, uiNuevaCol as isize) {
                if !OBSTACULOS.contains(&sNuevoEstado) {
                    sEstadoActual = sNuevoEstado.to_string();
                }
            }
        }

        uiPasoActual += 1;
    }

    if sEstadoActual.as_str() == ESTADO_META {
        *ref_mut_hm_s_f64RecompensasMap.get_mut(ESTADO_META).unwrap() += 1.0;
    }
}

/// Calculates new (row, col) after an action, specific to simulation grid movement.
/// Uses `saturating_sub` for boundary safety at 0. Upper bounds are not checked here.
///
/// # Arguments
/// * `uiFila` - Current row.
/// * `uiCol` - Current column.
/// * `sAccion` - Action string ("N", "S", "E", "O").
/// # Returns
/// Tuple `(usize, usize)` for new row and column.
fn mover(uiFila: usize, uiCol: usize, sAccion: &str) -> (usize, usize) {
    match sAccion {
        "N" => (uiFila.saturating_sub(1), uiCol),
        "S" => (uiFila + 1, uiCol), // Potential panic if uiFila+1 is out of bounds, not handled by this local mover
        "E" => (uiFila, uiCol + 1),   // Potential panic
        "O" => (uiFila, uiCol.saturating_sub(1)),
        _ => (uiFila, uiCol),
    }
}

/// Runs a non-visual simulation for a fixed number of steps (typically 1000).
///
/// Collects statistics on how many times the robot reaches the goal state and
/// how many times it falls into a danger state.
/// If the robot reaches the goal or a danger state, its position is reset to a
/// new random valid starting state, and the simulation continues for the remaining steps.
///
/// # Arguments
///
/// * `ref_hm_s_sPolitica` - The policy (State -> Action) to follow.
/// * `uiMaxPasos` - The total number of steps for the simulation.
///
/// # Returns
///
/// A tuple `(usize, usize)`:
///   - Number of times the goal state was reached.
///   - Number of times a danger state was entered.
pub fn simulacion_1000_pasos(
    ref_hm_s_sPolitica: &HashMap<String, String>,
    uiMaxPasos: usize,
) -> (usize, usize) {
    let vec_sEstadosValidos: Vec<String> = MAPA_ESTADOS
        .iter()
        .flatten()
        .filter(|&&sEstadoRef| sEstadoRef != ESTADO_META && !OBSTACULOS.contains(&sEstadoRef))
        .map(|&sEstadoRef| sEstadoRef.to_string()) // sEstadoRef is &&str, so *sEstadoRef is &str
        .collect();

    let mut rngThreadRng = thread_rng();
    let mut sEstadoActual = vec_sEstadosValidos.choose(&mut rngThreadRng).unwrap().clone();

    let mut uiLlegoMetaCount = 0;
    let mut uiCayoPeligroCount = 0;
    let mut f64RecompensaTotalSim = 0.0; // Note: This accumulates rewards but isn't part of the return tuple.

    for _ in 0..uiMaxPasos {
        // This call to obtener_recompensas() is inefficient as it rebuilds the map each time.
        // However, sticking to requested changes.
        f64RecompensaTotalSim += obtener_recompensas()
            .get(sEstadoActual.as_str())
            .unwrap_or(&0.0);

        if sEstadoActual.as_str() == ESTADO_META {
            uiLlegoMetaCount += 1;
            sEstadoActual = vec_sEstadosValidos.choose(&mut rngThreadRng).unwrap().clone();
            continue;
        }

        if ESTADOS_PELIGRO.contains(&sEstadoActual.as_str()) {
            uiCayoPeligroCount += 1;
            sEstadoActual = vec_sEstadosValidos.choose(&mut rngThreadRng).unwrap().clone();
            continue;
        }

        // Policy lookup. sEstadoActual is String.
        if let Some(sAccionRef) = ref_hm_s_sPolitica.get(&sEstadoActual) {
            // Call mdp_model::obtener_posicion, expects sEstado: &str
            if let Some((uiFila, uiCol)) = obtener_posicion(sEstadoActual.as_str()) {
                // Call local mover, expects uiFila: usize, uiCol: usize, sAccion: &str
                // sAccionRef is &String, which coerces to &str.
                let (uiNuevaFila, uiNuevaCol) = mover(uiFila, uiCol, sAccionRef);
                // Call mdp_model::obtener_estado, expects iFila: isize, iCol: isize
                sEstadoActual = obtener_estado(uiNuevaFila as isize, uiNuevaCol as isize)
                    .map(|sNuevoEstadoStr| sNuevoEstadoStr.to_string()) // sNuevoEstadoStr is &'static str
                    .unwrap_or_else(|| sEstadoActual.clone()); // Fallback to current state if move is invalid
            }
        } else {
            // No action found in policy for current state, end simulation or handle error.
            // Current behavior is to break the loop.
            break;
        }
    }
    println!("Llegadas a meta: {}", uiLlegoMetaCount);
    println!("Caídas en peligro: {}", uiCayoPeligroCount);
    println!("Recompensa total: {:.2}", f64RecompensaTotalSim); // This is printed but not returned.
    (uiLlegoMetaCount, uiCayoPeligroCount)
}
