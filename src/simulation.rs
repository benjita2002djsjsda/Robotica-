// src/simulation.rs
use crate::config::{
    acciones, obtener_recompensas, ESTADOS_PELIGRO, ESTADO_META, MAPA_ESTADOS, OBSTACULOS,
};
use crate::mdp_model::{obtener_estado, obtener_posicion};
use ::rand::seq::SliceRandom;
use ::rand::thread_rng;
use ::rand::Rng;
use macroquad::prelude::*;
use std::collections::HashMap;

const TAMANO_CELDA: f32 = 80.0;
const MARGEN: f32 = 2.0;
const COLOR_NORMAL: Color = GRAY;
const COLOR_PELIGRO: Color = RED;
const COLOR_META: Color = GREEN;
const COLOR_ROBOT: Color = BLUE;
const COLOR_OBSTACULO: Color = DARKGRAY;

pub async fn ejecutar_simulacion(
    politica: &mut HashMap<String, String>,
    pasos: usize,
    recompensas_map: &mut HashMap<&'static str, f64>,
) {
    let mut rng = ::rand::thread_rng();
    let epsilon = 0.8;

    // Inicializar el estado actual desde un estado aleatorio válido
    let estados_validos: Vec<String> = MAPA_ESTADOS
        .iter()
        .flatten()
        .filter(|&&estado| estado != ESTADO_META && !OBSTACULOS.contains(&estado))
        .map(|&estado| estado.to_string())
        .collect();

    let mut estado_actual = estados_validos.choose(&mut rng).unwrap().clone();
    let mut paso_actual = 0;

    // Control de velocidad: tiempo entre movimientos
    let mut ultimo_movimiento = get_time();
    let intervalo_movimiento = 0.5; // Ajusta este valor para cambiar la velocidad (en segundos)

    loop {
        clear_background(WHITE);

        // Dibujar el mapa
        for (i_fila, fila) in MAPA_ESTADOS.iter().enumerate() {
            for (i_col, estado) in fila.iter().enumerate() {
                let color = if OBSTACULOS.contains(estado) {
                    COLOR_OBSTACULO
                } else if ESTADOS_PELIGRO.contains(estado) {
                    COLOR_PELIGRO
                } else if *estado == ESTADO_META {
                    COLOR_META
                } else if *estado == estado_actual {
                    COLOR_ROBOT
                } else {
                    COLOR_NORMAL
                };

                let x = i_col as f32 * TAMANO_CELDA;
                let y = i_fila as f32 * TAMANO_CELDA;

                draw_rectangle(
                    x + MARGEN,
                    y + MARGEN,
                    TAMANO_CELDA - 2.0 * MARGEN,
                    TAMANO_CELDA - 2.0 * MARGEN,
                    color,
                );
            }
        }

        draw_text(
            &format!("Paso: {} - Estado: {}", paso_actual, estado_actual),
            10.0,
            20.0,
            20.0,
            BLACK,
        );

        next_frame().await;

        // Control de velocidad: mover solo si ha pasado el tiempo suficiente
        let ahora = get_time();
        if ahora - ultimo_movimiento < intervalo_movimiento {
            continue; // Esperar hasta que pase el intervalo
        }
        ultimo_movimiento = ahora;

        if paso_actual >= pasos || estado_actual == ESTADO_META {
            break;
        }

        let accion = if rng.gen::<f64>() < epsilon {
            // Explorar: elegir una acción aleatoria
            let acciones_validas = acciones();
            acciones_validas.choose(&mut rng).unwrap().to_string()
        } else {
            // Seguir la política óptima
            politica.get(&estado_actual).unwrap().clone()
        };

        if let Some((fila_act, col_act)) = obtener_posicion(&estado_actual) {
            let (nueva_fila, nueva_col) = mover(fila_act, col_act, &accion);
            if let Some(nuevo_estado) = obtener_estado(nueva_fila as isize, nueva_col as isize) {
                if !OBSTACULOS.contains(&nuevo_estado) {
                    estado_actual = nuevo_estado.to_string();
                }
            }
        }

        paso_actual += 1;
    }

    if estado_actual == ESTADO_META {
        *recompensas_map.get_mut(ESTADO_META).unwrap() += 1.0;
    }
}

fn mover(fila: usize, col: usize, accion: &str) -> (usize, usize) {
    match accion {
        "N" => (fila.saturating_sub(1), col),
        "S" => (fila + 1, col),
        "E" => (fila, col + 1),
        "O" => (fila, col.saturating_sub(1)),
        _ => (fila, col),
    }
}

pub fn simulacion_1000_pasos(
    politica: &HashMap<String, String>,
    max_pasos: usize,
) -> (usize, usize) {
    let estados_validos: Vec<String> = MAPA_ESTADOS
        .iter()
        .flatten()
        .filter(|&s| *s != ESTADO_META && !OBSTACULOS.contains(s))
        .map(|s| s.to_string())
        .collect();

    let mut rng = thread_rng();
    let mut estado_actual = estados_validos.choose(&mut rng).unwrap().clone();

    let mut llego_meta = 0;
    let mut cayo_peligro = 0;
    let mut recompensa_total = 0.0;

    for _ in 0..max_pasos {
        recompensa_total += obtener_recompensas()
            .get(estado_actual.as_str())
            .unwrap_or(&0.0);

        if estado_actual == ESTADO_META {
            llego_meta += 1;
            estado_actual = estados_validos.choose(&mut rng).unwrap().clone();
            continue;
        }

        if ESTADOS_PELIGRO.contains(&estado_actual.as_str()) {
            cayo_peligro += 1;
            estado_actual = estados_validos.choose(&mut rng).unwrap().clone();
            continue;
        }

        if let Some(accion) = politica.get(&estado_actual) {
            if let Some((fila, col)) = obtener_posicion(&estado_actual) {
                let (nueva_fila, nueva_col) = mover(fila, col, accion);
                estado_actual = obtener_estado(nueva_fila as isize, nueva_col as isize)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| estado_actual.clone());
            }
        } else {
            break;
        }
    }
    println!("Llegadas a meta: {}", llego_meta);
    println!("Caídas en peligro: {}", cayo_peligro);
    println!("Recompensa total: {:.2}", recompensa_total);
    (llego_meta, cayo_peligro)
}
