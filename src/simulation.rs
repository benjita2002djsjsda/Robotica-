// src/simulation.rs
use crate::config::{
    obtener_recompensas, ESTADOS_PELIGRO, ESTADO_META, INTERVALO_MOVIMIENTO, MAPA_ESTADOS,
    OBSTACULOS,
};
use crate::mdp_model::{obtener_estado, obtener_posicion};
use ::rand::seq::SliceRandom;
use ::rand::thread_rng;

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
    politica: &HashMap<String, String>,
    pasos: usize,
    recompensas_map: &mut HashMap<&'static str, f64>,
    landa: f64,
) {
    let mut rng = ::rand::thread_rng();
    let mut historial_estados = Vec::new();

    let estados_validos: Vec<String> = MAPA_ESTADOS
        .iter()
        .flatten()
        .filter(|&&estado| estado != ESTADO_META && !OBSTACULOS.contains(&estado))
        .map(|&estado| estado.to_string())
        .collect();

    let mut estado_actual = estados_validos.choose(&mut rng).unwrap().clone();
    historial_estados.push(estado_actual.clone());
    let mut paso_actual = 0;
    let mut recompensa_total = 0.0;
    recompensa_total += obtener_recompensas()
        .get(estado_actual.as_str())
        .unwrap_or(&0.0);

    let mut ultimo_movimiento = get_time();
    let intervalo_movimiento = INTERVALO_MOVIMIENTO;

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
                // Dibuja la letra del estado (por ejemplo, "S1", "S2", etc.) en el centro
                draw_text(
                    estado,
                    x + TAMANO_CELDA / 2.0 - 12.0,
                    y + TAMANO_CELDA / 2.0 + 8.0,
                    24.0,
                    BLACK,
                );

                // Si es peligro, dibuja una "P" en la esquina superior derecha
                if ESTADOS_PELIGRO.contains(estado) {
                    draw_text("P", x + TAMANO_CELDA - 22.0, y + 22.0, 24.0, RED);
                }
            }
        }

        draw_text(
            &format!(
                "λ={:.2} | Paso: {} | Estado: {} | Acción: {} | Recompensa: {:.2}",
                landa,
                paso_actual,
                estado_actual,
                politica.get(&estado_actual).unwrap_or(&"N/A".to_string()),
                recompensa_total
            ),
            10.0,
            20.0,
            20.0,
            BLACK,
        );

        next_frame().await;

        let ahora = get_time();
        if ahora - ultimo_movimiento < intervalo_movimiento {
            continue;
        }
        ultimo_movimiento = ahora;

        if paso_actual >= pasos || estado_actual == ESTADO_META {
            println!("Camino seguido: {:?}", historial_estados);
            break;
        }

        // SIGUE ESTRICTAMENTE LA POLÍTICA ÓPTIMA (sin exploración aleatoria)
        let accion = politica.get(&estado_actual).unwrap().clone();

        if let Ok((fila_act, col_act)) = obtener_posicion(&estado_actual) {
            let (nueva_fila, nueva_col) = mover(fila_act, col_act, &accion);
            if let Some(nuevo_estado) = obtener_estado(nueva_fila as isize, nueva_col as isize) {
                if !OBSTACULOS.contains(&nuevo_estado) {
                    // Suma la recompensa SOLO al entrar al nuevo estado
                    recompensa_total += obtener_recompensas().get(nuevo_estado).unwrap_or(&0.0);
                    estado_actual = nuevo_estado.to_string();
                    historial_estados.push(estado_actual.clone());
                }
            }
        }

        paso_actual += 1;
    }

    if estado_actual == ESTADO_META {
        *recompensas_map.entry(ESTADO_META).or_insert(0.0) += 1.0;
        println!(
            "¡Meta alcanzada en {} pasos! Recompensa acumulada: {:.2}",
            paso_actual, recompensa_total
        );
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

        // Solo suma recompensa al entrar al nuevo estado
        if let Some(accion) = politica.get(&estado_actual) {
            if let Ok((fila, col)) = obtener_posicion(&estado_actual) {
                let (nueva_fila, nueva_col) = mover(fila, col, accion);
                let nuevo_estado = obtener_estado(nueva_fila as isize, nueva_col as isize)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| estado_actual.clone());
                recompensa_total += obtener_recompensas()
                    .get(nuevo_estado.as_str())
                    .unwrap_or(&0.0);
                estado_actual = nuevo_estado;
            }
        } else {
            break;
        }
    }
    println!("\nResumen final después de {} pasos:", max_pasos);
    println!("- Llegadas a meta: {}", llego_meta);
    println!("- Caídas en peligro: {}", cayo_peligro);
    println!("- Recompensa total acumulada: {:.2}", recompensa_total);

    (llego_meta, cayo_peligro)
}
