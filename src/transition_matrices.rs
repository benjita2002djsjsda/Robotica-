use crate::config::{prob_transicion, MAPA_ESTADOS, OBSTACULOS};
use crate::mdp_model::{mover, obtener_estado, obtener_posicion};
use ndarray::Array2;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

pub fn construir_matriz_transicion(accion: &str) -> Array2<f32> {
    let modelo_transicion = prob_transicion();

    let estados: Vec<String> = MAPA_ESTADOS
        .iter()
        .flatten()
        .filter(|s| !s.is_empty() && !OBSTACULOS.contains(s))
        .map(|s| s.to_string())
        .collect();

    let total = estados.len();
    let mut estado_a_indice: HashMap<String, usize> = HashMap::new();
    for (i, estado) in estados.iter().enumerate() {
        estado_a_indice.insert(estado.clone(), i);
    }

    let mut matriz = Array2::<f32>::zeros((total, total));

    for estado_origen in &estados {
        if let Some((fila, col)) = obtener_posicion(estado_origen) {
            if let Some(transiciones) = modelo_transicion.get(accion) {
                for (resultado, prob) in transiciones {
                    let (nueva_fila, nueva_col) = mover(fila, col, resultado);
                    let mut estado_destino = obtener_estado(nueva_fila, nueva_col)
                        .unwrap_or_else(|| estado_origen.as_str());

                    if OBSTACULOS.contains(&estado_destino) {
                        estado_destino = estado_origen.as_str();
                    }

                    let i_origen = *estado_a_indice.get(estado_origen).unwrap();
                    let i_destino = *estado_a_indice.get(estado_destino).unwrap();
                    matriz[[i_origen, i_destino]] += *prob as f32;
                }
            }
        }
    }

    matriz
}

pub fn guardar_matrices_transicion_csv() {
    for accion in ["N", "S", "E", "O"] {
        let matriz = construir_matriz_transicion(accion);
        let nombre_archivo = format!("matriz_transicion_{}.csv", accion);
        let mut archivo = File::create(&nombre_archivo).expect("No se pudo crear el archivo");

        for fila in matriz.rows() {
            let linea: Vec<String> = fila.iter().map(|v| format!("{:.2}", v)).collect();
            writeln!(archivo, "{}", linea.join(",")).expect("Error escribiendo archivo");
        }

        println!("✅ {} guardada.", nombre_archivo);
    }
}
