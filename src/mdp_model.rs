use crate::config::{
    acciones, obtener_recompensas, prob_transicion, COLUMNAS_MAPA, ESTADO_META, FILAS_MAPA,
    MAPA_ESTADOS, OBSTACULOS, UMBRAL_CONVERGENCIA,
};
use std::collections::HashMap;

/// Módulo principal del modelo MDP - Algoritmos y utilidades de navegación
pub fn obtener_posicion(estado: &str) -> Result<(usize, usize), String> {
    for (fila, fila_estados) in MAPA_ESTADOS.iter().enumerate() {
        for (col, nombre_estado) in fila_estados.iter().enumerate() {
            if *nombre_estado == estado {
                return Ok((fila, col));
            }
        }
    }
    Err(format!("Estado '{}' no encontrado en el mapa", estado))
}

/// Obtiene el identificador del estado en coordenadas específicas
///
/// Verifica que las coordenadas estén dentro del mapa y que el estado
/// resultante no sea un obstáculo (estados inaccesibles).

pub fn obtener_estado(fila: isize, col: isize) -> Option<&'static str> {
    if fila >= 0 && fila < FILAS_MAPA as isize && col >= 0 && col < COLUMNAS_MAPA as isize {
        let estado = MAPA_ESTADOS[fila as usize][col as usize];
        if OBSTACULOS.contains(&estado) {
            None // Los obstáculos no son estados accesibles
        } else {
            Some(estado)
        }
    } else {
        None // Fuera de los límites del mapa
    }
}

/// Calcula las nuevas coordenadas después de ejecutar una acción
///
/// Convierte una acción direccional en un desplazamiento de coordenadas.

pub fn mover(fila: usize, col: usize, accion: &str) -> (isize, isize) {
    match accion {
        "N" => (fila as isize - 1, col as isize), // Norte: una fila arriba
        "S" => (fila as isize + 1, col as isize), // Sur: una fila abajo
        "E" => (fila as isize, col as isize + 1), // Este: una columna a la derecha
        "O" => (fila as isize, col as isize - 1), // Oeste: una columna a la izquierda
        _ => (fila as isize, col as isize),       // Acción inválida: no moverse
    }
}

/// Algoritmo Q-Value Iteration para resolver el MDP
///
/// Calcula la matriz Q(s,a) completa y deriva V(s) y π(s) óptimos.
/// Utiliza la ecuación de Bellman: Q(s,a) = R(s) + γ * Σ P(s'|s,a) * max_a' Q(s',a')

pub fn q_value_iteration(
    lambda: f64,
    epsilon: Option<f64>,
    prob_transicion_externa: Option<&HashMap<String, HashMap<String, f64>>>,
) -> (
    HashMap<String, HashMap<String, f64>>,
    HashMap<String, String>,
    HashMap<String, f64>,
) {
    let epsilon = epsilon.unwrap_or(UMBRAL_CONVERGENCIA);

    // Estructura para almacenar Q-valores: Q(estado, acción)
    let mut q_valores: HashMap<String, HashMap<String, f64>> = HashMap::new();
    let mut politica: HashMap<String, String> = HashMap::new();

    let recompensas_map = obtener_recompensas();
    let acciones_disponibles = acciones();

    // Inicialización: todos los Q-valores en cero
    for fila in MAPA_ESTADOS.iter() {
        for estado in fila.iter() {
            if !OBSTACULOS.contains(estado) {
                let mut q_estado = HashMap::new();
                for accion in &acciones_disponibles {
                    q_estado.insert(accion.to_string(), 0.0);
                }
                q_valores.insert(estado.to_string(), q_estado);
            }
        }
    }

    // Conversión del modelo de transición base si no se proporciona uno externo
    let modelo_base: Option<HashMap<String, HashMap<String, f64>>> =
        if prob_transicion_externa.is_none() {
            Some(
                prob_transicion()
                    .iter()
                    .map(|(k, v)| {
                        (
                            k.to_string(),
                            v.iter().map(|(k2, v2)| (k2.to_string(), *v2)).collect(),
                        )
                    })
                    .collect(),
            )
        } else {
            None
        };

    // Bucle principal de Q-Value Iteration
    let mut cambios;
    loop {
        let mut delta: f64 = 0.0;
        let mut q_nuevo = q_valores.clone();

        // Actualización de Q-valor para cada par (estado, acción)
        for fila in MAPA_ESTADOS.iter() {
            for estado in fila.iter() {
                if OBSTACULOS.contains(estado) {
                    continue;
                }

                // Estados terminales tienen Q-valor igual a su recompensa
                if *estado == ESTADO_META {
                    for accion in &acciones_disponibles {
                        let recompensa_terminal =
                            recompensas_map.get(estado).copied().unwrap_or(0.0);
                        q_nuevo
                            .get_mut(&estado.to_string())
                            .unwrap()
                            .insert(accion.to_string(), recompensa_terminal);
                    }
                    continue;
                }

                let (fila_actual, col_actual) = match obtener_posicion(estado) {
                    Ok(pos) => pos,
                    Err(_) => continue,
                };

                // Calcular Q(s,a) para cada acción en este estado
                for accion in &acciones_disponibles {
                    let prob_accion = match prob_transicion_externa {
                        Some(dct) => dct.get(&accion.to_string()).unwrap(),
                        None => modelo_base
                            .as_ref()
                            .unwrap()
                            .get(&accion.to_string())
                            .unwrap(),
                    };

                    let mut q_valor = 0.0;

                    // Ecuación de Bellman para Q-valores: Q(s,a) = R(s) + γ * Σ P(s'|s,a) * max_a' Q(s',a')
                    for (resultado, probabilidad) in prob_accion.iter() {
                        let (nueva_fila, nueva_col) = mover(fila_actual, col_actual, resultado);
                        let estado_destino = match obtener_estado(nueva_fila, nueva_col) {
                            Some(e) => e.to_string(),
                            None => estado.to_string(),
                        };

                        // Encontrar max_a' Q(s',a') para el estado destino
                        let max_q_destino = if let Some(q_destino) = q_valores.get(&estado_destino)
                        {
                            q_destino
                                .values()
                                .fold(f64::NEG_INFINITY, |max, &val| max.max(val))
                        } else {
                            0.0
                        };

                        q_valor += probabilidad * max_q_destino;
                    }

                    // Q(s,a) = R(s) + γ * valor_esperado
                    let q_final = recompensas_map.get(estado).unwrap_or(&0.0) + lambda * q_valor;

                    // Actualizar Q-valor y calcular cambio máximo
                    let q_anterior = q_valores
                        .get(&estado.to_string())
                        .unwrap()
                        .get(&accion.to_string())
                        .unwrap_or(&0.0);
                    delta = delta.max((q_anterior - q_final).abs());

                    q_nuevo
                        .get_mut(&estado.to_string())
                        .unwrap()
                        .insert(accion.to_string(), q_final);
                }
            }
        }

        // Verificación de convergencia
        cambios = delta > epsilon;
        if !cambios {
            break;
        }
        q_valores = q_nuevo;
    }

    // Derivar política óptima y valores V(s) desde los Q-valores
    let mut v_valores: HashMap<String, f64> = HashMap::new();

    for fila in MAPA_ESTADOS.iter() {
        for estado in fila.iter() {
            if OBSTACULOS.contains(estado) {
                continue;
            }

            if let Some(q_estado) = q_valores.get(&estado.to_string()) {
                // Encontrar la mejor acción: π(s) = argmax_a Q(s,a)
                let mut mejor_accion = String::new();
                let mut mejor_q_valor = f64::NEG_INFINITY;

                for (accion, &q_val) in q_estado.iter() {
                    if q_val > mejor_q_valor {
                        mejor_q_valor = q_val;
                        mejor_accion = accion.clone();
                    }
                }

                // V(s) = max_a Q(s,a)
                v_valores.insert(estado.to_string(), mejor_q_valor);
                politica.insert(estado.to_string(), mejor_accion);
            }
        }
    }

    (q_valores, politica, v_valores)
}
