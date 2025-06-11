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

/// Implementación del algoritmo Value Iteration para resolver el MDP
///
/// Calcula iterativamente los valores óptimos de cada estado y la política óptima
/// hasta alcanzar convergencia. Utiliza la ecuación de Bellman para actualizar
/// los valores de estado basándose en las recompensas inmediatas y los valores
/// futuros descontados.

pub fn value_iteration(
    lambda: f64,
    epsilon: Option<f64>,
    prob_transicion_externa: Option<&HashMap<String, HashMap<String, f64>>>,
) -> (HashMap<&'static str, f64>, HashMap<String, String>) {
    let epsilon = epsilon.unwrap_or(UMBRAL_CONVERGENCIA);
    let mut v: HashMap<&'static str, f64> = HashMap::new();
    let mut politica: HashMap<String, String> = HashMap::new();

    let recompensas_map = obtener_recompensas();

    // Inicialización: todos los valores en cero (excepto obstáculos)
    for fila in MAPA_ESTADOS.iter() {
        for estado in fila.iter() {
            v.insert(*estado, 0.0);
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

    // Bucle principal de Value Iteration
    let mut cambios;
    loop {
        let mut delta: f64 = 0.0; // Máximo cambio en valores durante esta iteración
        let mut v_nuevo = v.clone();

        // Actualización de valor para cada estado no-obstáculo
        for fila in MAPA_ESTADOS.iter() {
            for estado in fila.iter() {
                if OBSTACULOS.contains(estado) {
                    continue; // Los obstáculos no se procesan
                }
                if *estado == ESTADO_META {
                    // Estado terminal: valor = recompensa inmediata, sin política
                    v_nuevo.insert(*estado, recompensas_map.get(estado).copied().unwrap_or(0.0));
                    politica.insert(estado.to_string(), String::new());
                    continue;
                }

                let (fila_actual, col_actual) = match obtener_posicion(estado) {
                    Ok(pos) => pos,
                    Err(_) => continue,
                };

                let mut mejor_valor = f64::NEG_INFINITY;
                let mut mejor_accion = String::new();

                // Evaluación de cada acción posible (ecuación de Bellman)
                for accion in acciones().iter() {
                    // Obtener probabilidades de transición para esta acción
                    let prob_accion = match prob_transicion_externa {
                        Some(dct) => dct.get(&accion.to_string()).unwrap(),
                        None => modelo_base
                            .as_ref()
                            .unwrap()
                            .get(&accion.to_string())
                            .unwrap(),
                    };

                    let mut valor_esperado = 0.0;

                    // Cálculo del valor esperado considerando todas las transiciones posibles
                    for (resultado, probabilidad) in prob_accion.iter() {
                        let (nueva_fila, nueva_col) = mover(fila_actual, col_actual, resultado);
                        let estado_destino = match obtener_estado(nueva_fila, nueva_col) {
                            Some(e) => e,
                            None => *estado, // Si el movimiento es inválido, quedarse en el mismo estado
                        };
                        valor_esperado += probabilidad * v.get(estado_destino).unwrap_or(&0.0);
                    }

                    // Ecuación de Bellman: R(s) + γ * Σ P(s'|s,a) * V(s')
                    let valor_total =
                        recompensas_map.get(estado).unwrap_or(&0.0) + lambda * valor_esperado;

                    // Selección de la mejor acción (política greedy)
                    if valor_total > mejor_valor {
                        mejor_valor = valor_total;
                        mejor_accion = accion.to_string();
                    }
                }

                // Actualización del valor y seguimiento del cambio máximo
                delta = delta.max((v.get(estado).unwrap_or(&0.0) - mejor_valor).abs());
                v_nuevo.insert(*estado, mejor_valor);
                politica.insert(estado.to_string(), mejor_accion);
            }
        }

        // Verificación de convergencia
        cambios = delta > epsilon;
        if !cambios {
            break; // Convergencia alcanzada
        }
        v = v_nuevo;
    }

    (v, politica)
}
