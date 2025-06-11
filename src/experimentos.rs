use crate::config::{obtener_recompensas, ESTADO_META, MAPA_ESTADOS, OBSTACULOS};
use crate::mdp_model::{mover, obtener_estado, obtener_posicion, value_iteration};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fs::File;
use std::io::Write;

pub fn simular_y_guardar_csv(
    factores_landa: &[f64],
    probabilidades_exito: &[f64],
    episodios: usize,
    max_pasos: usize,
    nombre_archivo: &str,
) {
    let mut archivo = File::create(nombre_archivo).expect("No se pudo crear el archivo");
    writeln!(archivo, "discount_factor,success_probability,total_reward").unwrap();

    for &landa in factores_landa {
        for &f_centro in probabilidades_exito {
            let f_izq = (1.0 - f_centro) / 2.0;
            let f_der = f_izq;
            let modelo = crate::robustness::construir_modelo_ruido(f_izq, f_centro, f_der);

            let (_valores, politica) = value_iteration(landa, Some(0.001), Some(&modelo));

            let mut total_recompensa = 0.0;

            for _ in 0..episodios {
                let estados_validos: Vec<String> = MAPA_ESTADOS
                    .iter()
                    .flatten()
                    .filter(|&&estado| estado != ESTADO_META && !OBSTACULOS.contains(&estado))
                    .map(|&estado| estado.to_string())
                    .collect();

                let mut rng = thread_rng();
                let mut estado_actual = estados_validos.choose(&mut rng).unwrap().clone();
                let mut recompensa = 0.0;

                // Suma la recompensa del estado inicial
                recompensa += obtener_recompensas()
                    .get(estado_actual.as_str())
                    .unwrap_or(&0.0);

                for _ in 0..max_pasos {
                    if estado_actual == ESTADO_META {
                        break;
                    }
                    if let Some(accion) = politica.get(&estado_actual) {
                        if let Ok((fila, col)) = obtener_posicion(&estado_actual) {
                            let (nueva_fila, nueva_col) = mover(fila, col, accion);
                            let nuevo_estado =
                                obtener_estado(nueva_fila as isize, nueva_col as isize)
                                    .map(|s| s.to_string())
                                    .unwrap_or_else(|| estado_actual.clone());
                            recompensa += obtener_recompensas()
                                .get(nuevo_estado.as_str())
                                .unwrap_or(&0.0);
                            estado_actual = nuevo_estado;
                            if estado_actual == ESTADO_META {
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }
                total_recompensa += recompensa;
            }

            let promedio_recompensa = total_recompensa / episodios as f64;

            writeln!(
                archivo,
                "{:.2},{:.2},{:.6}",
                landa, f_centro, promedio_recompensa
            )
            .unwrap();
        }
    }
    println!("✅ Resultados guardados en '{}'", nombre_archivo);
}
pub fn guardar_recompensas_csv(datos: &[(f64, f64, f64)], path: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "lambda,prob_exito,recompensa")?;
    for (landa, exito, recompensa) in datos {
        writeln!(file, "{},{},{}", landa, exito, recompensa)?;
    }
    Ok(())
}
