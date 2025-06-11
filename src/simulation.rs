// src/simulation.rs
use crate::config::{
    obtener_recompensas, ESTADOS_PELIGRO, ESTADO_META, INTERVALO_MOVIMIENTO, MAPA_ESTADOS,
    OBSTACULOS,
};
use crate::mdp_model::{obtener_estado, obtener_posicion};
use ::rand::seq::SliceRandom;
use ::rand::thread_rng;
use ::rand::Rng;
use macroquad::prelude::*;
use std::collections::HashMap;

/// Módulo de simulación visual - Visualización en tiempo real del agente MDP
///
/// Proporciona simulación gráfica usando Macroquad donde se puede observar
/// al robot navegando por el mundo siguiendo la política óptima calculada.

const TAMANO_CELDA: f32 = 80.0; // Tamaño de cada celda en píxeles
const MARGEN: f32 = 2.0; // Espaciado entre celdas
const COLOR_NORMAL: Color = GRAY; // Color para estados normales
const COLOR_PELIGRO: Color = RED; // Color para estados peligrosos
const COLOR_META: Color = GREEN; // Color para el estado meta
const COLOR_ROBOT: Color = BLUE; // Color para la posición actual del agente
const COLOR_OBSTACULO: Color = DARKGRAY; // Color para obstáculos

/// Ejecuta una simulación visual interactiva del agente MDP
///
/// Muestra una ventana gráfica donde el robot se mueve por el mundo siguiendo
/// la política óptima. La simulación es determinística y sigue exactamente
/// las acciones dictadas por la política sin ruido adicional.

pub async fn ejecutar_simulacion(
    politica: &HashMap<String, String>,
    pasos: usize,
    recompensas_map: &mut HashMap<&'static str, f64>,
    landa: f64,
) {
    let mut rng = ::rand::thread_rng();
    let mut historial_estados = Vec::new();

    // Selección aleatoria del estado inicial (excluyendo meta y obstáculos)
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

    // Añadir recompensa del estado inicial
    recompensa_total += obtener_recompensas()
        .get(estado_actual.as_str())
        .unwrap_or(&0.0);

    // Control de temporización para movimientos suaves
    let mut ultimo_movimiento = get_time();
    let intervalo_movimiento = INTERVALO_MOVIMIENTO;

    // Bucle principal de visualización
    loop {
        clear_background(WHITE);

        // Renderizado del mapa completo
        for (i_fila, fila) in MAPA_ESTADOS.iter().enumerate() {
            for (i_col, estado) in fila.iter().enumerate() {
                // Determinación del color según el tipo de estado
                let color = if OBSTACULOS.contains(estado) {
                    COLOR_OBSTACULO
                } else if ESTADOS_PELIGRO.contains(estado) {
                    COLOR_PELIGRO
                } else if *estado == ESTADO_META {
                    COLOR_META
                } else if *estado == estado_actual {
                    COLOR_ROBOT // Resalta la posición actual del agente
                } else {
                    COLOR_NORMAL
                };

                let x = i_col as f32 * TAMANO_CELDA;
                let y = i_fila as f32 * TAMANO_CELDA;

                // Dibujado de la celda con márgenes
                draw_rectangle(
                    x + MARGEN,
                    y + MARGEN,
                    TAMANO_CELDA - 2.0 * MARGEN,
                    TAMANO_CELDA - 2.0 * MARGEN,
                    color,
                );

                // Etiqueta del estado en el centro de la celda
                draw_text(
                    estado,
                    x + TAMANO_CELDA / 2.0 - 12.0,
                    y + TAMANO_CELDA / 2.0 + 8.0,
                    24.0,
                    BLACK,
                );

                // Indicador visual adicional para estados peligrosos
                if ESTADOS_PELIGRO.contains(estado) {
                    draw_text("P", x + TAMANO_CELDA - 22.0, y + 22.0, 24.0, RED);
                }
            }
        }

        // Panel de información en tiempo real
        draw_text(
            &format!(
                "lambda={:.2} | Paso: {} | Estado: {} | Recompensa: {:.2}",
                landa, paso_actual, estado_actual, recompensa_total
            ),
            10.0,
            20.0,
            20.0,
            BLACK,
        );

        next_frame().await;

        // Control de temporización: solo mover cada INTERVALO_MOVIMIENTO segundos
        let ahora = get_time();
        if ahora - ultimo_movimiento < intervalo_movimiento {
            continue;
        }
        ultimo_movimiento = ahora;

        // Condiciones de terminación
        if paso_actual >= pasos || estado_actual == ESTADO_META {
            println!("Camino seguido: {:?}", historial_estados);
            break;
        }

        // Ejecución de movimiento siguiendo la política óptima (determinística)
        let accion = politica.get(&estado_actual).unwrap().clone();

        if let Ok((fila_act, col_act)) = obtener_posicion(&estado_actual) {
            let (nueva_fila, nueva_col) = mover(fila_act, col_act, &accion);
            if let Some(nuevo_estado) = obtener_estado(nueva_fila as isize, nueva_col as isize) {
                if !OBSTACULOS.contains(&nuevo_estado) {
                    // Acumulación de recompensa al entrar al nuevo estado
                    recompensa_total += obtener_recompensas().get(nuevo_estado).unwrap_or(&0.0);
                    estado_actual = nuevo_estado.to_string();
                    historial_estados.push(estado_actual.clone());
                }
            }
        }

        paso_actual += 1;
    }

    // Actualización de estadísticas si se alcanzó la meta
    if estado_actual == ESTADO_META {
        *recompensas_map.entry(ESTADO_META).or_insert(0.0) += 1.0;
        println!(
            "¡Meta alcanzada en {} pasos! Recompensa acumulada: {:.2}",
            paso_actual, recompensa_total
        );
    }
}

/// Función auxiliar para calcular movimientos en la simulación visual
///
/// Similar a la función mover del módulo mdp_model pero usando saturating_sub
/// para evitar underflow en coordenadas.
fn mover(fila: usize, col: usize, accion: &str) -> (usize, usize) {
    match accion {
        "N" => (fila.saturating_sub(1), col), // Norte con protección underflow
        "S" => (fila + 1, col),               // Sur
        "E" => (fila, col + 1),               // Este
        "O" => (fila, col.saturating_sub(1)), // Oeste con protección underflow
        _ => (fila, col),                     // Acción inválida: no moverse
    }
}

/// Ejecuta una simulación estocástica de múltiples episodios para análisis estadístico
///
/// A diferencia de la simulación visual, esta función realiza múltiples pasos
/// considerando probabilidades de éxito en los movimientos y reinicios automáticos
/// cuando se alcanza la meta o se cae en peligro.

pub fn simulacion_1000_pasos(
    politica: &HashMap<String, String>,
    max_pasos: usize,
    prob_exito: f64,
) -> (usize, usize, f64) {
    // Estados válidos para reiniciar episodios
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

    // Simulación de múltiples episodios en max_pasos
    for _ in 0..max_pasos {
        // Reinicio si se alcanzó la meta
        if estado_actual == ESTADO_META {
            llego_meta += 1;
            estado_actual = estados_validos.choose(&mut rng).unwrap().clone();
            continue;
        }

        // Reinicio si cayó en peligro
        if ESTADOS_PELIGRO.contains(&estado_actual.as_str()) {
            cayo_peligro += 1;
            estado_actual = estados_validos.choose(&mut rng).unwrap().clone();
            continue;
        }

        // Ejecución de acción con probabilidad de fallo
        if let Some(accion) = politica.get(&estado_actual) {
            if let Ok((fila, col)) = obtener_posicion(&estado_actual) {
                // Determinación estocástica del éxito del movimiento
                let movimiento_exitoso = rng.gen_bool(prob_exito);

                let (nueva_fila, nueva_col) = if movimiento_exitoso {
                    // Movimiento según la política
                    mover(fila, col, accion)
                } else {
                    // Movimiento fallido: dirección aleatoria (simula ruido/error)
                    let direcciones = ["N", "S", "E", "O"];
                    let direccion_fallida = direcciones.choose(&mut rng).unwrap();
                    mover(fila, col, direccion_fallida)
                };

                // Transición a nuevo estado y acumulación de recompensa
                let nuevo_estado = obtener_estado(nueva_fila as isize, nueva_col as isize)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| estado_actual.clone());

                recompensa_total += obtener_recompensas()
                    .get(nuevo_estado.as_str())
                    .unwrap_or(&0.0);
                estado_actual = nuevo_estado;
            }
        } else {
            break; // No hay acción definida, terminar simulación
        }
    }

    (llego_meta, cayo_peligro, recompensa_total)
}
