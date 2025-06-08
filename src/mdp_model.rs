use crate::config::{
    acciones, obtener_recompensas, prob_transicion, COLUMNAS_MAPA, ESTADO_META, FILAS_MAPA,
    MAPA_ESTADOS, OBSTACULOS,
};
use std::collections::HashMap;

pub fn obtener_posicion(estado: &str) -> Option<(usize, usize)> {
    for (fila, fila_estados) in MAPA_ESTADOS.iter().enumerate() {
        for (col, nombre_estado) in fila_estados.iter().enumerate() {
            if *nombre_estado == estado {
                return Some((fila, col));
            }
        }
    }
    None
}

pub fn obtener_estado(fila: isize, col: isize) -> Option<&'static str> {
    if fila >= 0 && fila < FILAS_MAPA as isize && col >= 0 && col < COLUMNAS_MAPA as isize {
        let estado = MAPA_ESTADOS[fila as usize][col as usize];
        if OBSTACULOS.contains(&estado) {
            return None;
        }
        return Some(estado);
    }
    None
}

pub fn mover(fila: usize, col: usize, accion: &str) -> (isize, isize) {
    match accion {
        "N" => (fila as isize - 1, col as isize),
        "S" => (fila as isize + 1, col as isize),
        "E" => (fila as isize, col as isize + 1),
        "O" => (fila as isize, col as isize - 1),
        _ => (fila as isize, col as isize),
    }
}

pub fn value_iteration(
    lambda: f64,
    epsilon: f64,
    prob_transicion_externa: Option<&HashMap<String, HashMap<String, f64>>>,
) -> (HashMap<&'static str, f64>, HashMap<String, String>) {
    let mut v: HashMap<&'static str, f64> = HashMap::new();
    let mut politica: HashMap<String, String> = HashMap::new();

    let recompensas_map = obtener_recompensas();
    // Inicializar valores en cero
    for fila in MAPA_ESTADOS.iter() {
        for estado in fila.iter() {
            v.insert(*estado, 0.0);
        }
    }

    // Convertir modelo base solo si se usa
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

    let mut cambios = true;
    while cambios {
        cambios = false;
        let mut v_nuevo = v.clone();

        for fila in MAPA_ESTADOS.iter() {
            for estado in fila.iter() {
                if *estado == ESTADO_META {
                    v_nuevo.insert(*estado, *recompensas_map.get(estado).unwrap());
                    continue;
                }

                let mut mejor_valor = f64::NEG_INFINITY;
                let mut mejor_accion = String::new();

                for accion in acciones().iter() {
                    let prob_accion = match prob_transicion_externa {
                        Some(dct) => dct.get(&accion.to_string()).unwrap(),
                        None => modelo_base
                            .as_ref()
                            .unwrap()
                            .get(&accion.to_string())
                            .unwrap(),
                    };

                    let (fila_actual, col_actual) = obtener_posicion(estado).unwrap();
                    let mut valor_esperado = 0.0;

                    for (resultado, probabilidad) in prob_accion.iter() {
                        let (nueva_fila, nueva_col) = mover(fila_actual, col_actual, resultado);
                        let estado_destino = match obtener_estado(nueva_fila, nueva_col) {
                            Some(e) => e,
                            None => *estado,
                        };
                        valor_esperado += probabilidad * v.get(estado_destino).unwrap();
                    }

                    let valor_total =
                        recompensas_map.get(estado).unwrap() + lambda * valor_esperado;

                    if valor_total > mejor_valor {
                        mejor_valor = valor_total;
                        mejor_accion = accion.to_string();
                    }
                }

                v_nuevo.insert(*estado, mejor_valor);
                if (v.get(estado).unwrap() - mejor_valor).abs() > epsilon {
                    cambios = true;
                }

                politica.insert(estado.to_string(), mejor_accion);
            }
        }

        v = v_nuevo;
    }

    (v, politica)
}
