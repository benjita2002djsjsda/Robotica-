// src/main.rs
// Ejecuta Value Iteration, simula con Macroquad y evalúa robustez

mod config;
mod experimentos;
mod mdp_model;
mod plot_utils;
mod robustness;
mod simulation;
mod transition_matrices;

use config::obtener_recompensas;
use mdp_model::value_iteration;
use plot_utils::{graficar_recompensas_barras, graficar_resultados_finales, leer_recompensas_csv};
use robustness::{construir_modelo_ruido, MODELOS_ROBUSTEZ};
use simulation::{ejecutar_simulacion, simulacion_1000_pasos};
use transition_matrices::guardar_matrices_transicion_csv;

#[macroquad::main("Simulacion MDP Robot")]
async fn main() {
    let factores_landa = vec![0.86, 0.90, 0.94, 0.98];
    let probabilidades_exito = vec![0.5, 0.7, 0.8, 0.9];

    let mut resumen_1000_pasos = vec![];
    let mut politicas_optimas = vec![]; // Para guardar las políticas óptimas
    let mut recompensas_map = obtener_recompensas();

    for &landa in &factores_landa {
        println!("\n=== Ejecutando Value Iteration para λ = {:.2} ===", landa);
        println!("Resultados de simulaciones de 1000 pasos:");
        println!("----------------------------------------");
        let (valores, politica) = value_iteration(landa, Some(0.001), None);

        for &prob in &probabilidades_exito {
            let (metas, peligros, recompensa) = simulacion_1000_pasos(&politica, 1000, prob);
            resumen_1000_pasos.push((landa, prob, recompensa));

            println!(
                "λ={:.2} - Prob éxito {:.0}% Recompensa: {:.2} (Meta: {}, Peligros: {})",
                landa,
                prob * 100.0,
                recompensa,
                metas,
                peligros
            );
        }
        println!("\nValor de los estados:");
        let mut keys: Vec<_> = valores.keys().collect();
        keys.sort();
        for k in keys {
            println!("{}: {:.2}", k, valores[k]);
        }

        // Mostrar política óptima
        println!("\nPolítica óptima para λ={}:", landa);
        println!("\nMapa de política óptima:");
        println!("-------------------------");
        for fila in config::MAPA_ESTADOS.iter() {
            for estado in fila {
                let simbolo = if config::OBSTACULOS.contains(estado) {
                    "⬛"
                } else if *estado == config::ESTADO_META {
                    "🎯"
                } else if config::ESTADOS_PELIGRO.contains(estado) {
                    "⚠️"
                } else {
                    match politica.get(*estado).map(String::as_str) {
                        Some("N") => "↑",
                        Some("S") => "↓",
                        Some("E") => "→",
                        Some("O") => "←",
                        _ => " ",
                    }
                };
                print!("{} ", simbolo);
            }
            println!();
        }
        println!("-------------------------");
        println!("Leyenda: ⬛ Obstáculo | 🎯 Meta | ⚠️ Peligro | ↑↓←→ Dirección óptima");

        // Ejecutar simulación estricta
        println!("\n→ Iniciando simulación visual (siguiendo política óptima)...");
        ejecutar_simulacion(&politica, 100, &mut recompensas_map, landa).await;

        // Guardar política para evaluación de robustez
        politicas_optimas.push((landa, politica.clone()));
    }

    // === EVALUACIÓN DE ROBUSTEZ CON MODELOS ESPECÍFICOS ===
    println!("\n=== EVALUANDO ROBUSTEZ CON MODELOS ESPECÍFICOS ===");
    println!("Evaluando con: (10%,80%,10%), (5%,90%,5%), (15%,70%,15%), (25%,50%,25%)");

    // Evaluar robustez para cada política óptima
    for (lambda, _politica_base) in &politicas_optimas {
        println!("\n--- Evaluación robustez λ={:.2} ---", lambda);

        for (izq, centro, der) in MODELOS_ROBUSTEZ.iter() {
            let modelo_ruido = construir_modelo_ruido(*izq, *centro, *der);
            let (_valores, politica_adaptada) =
                value_iteration(*lambda, Some(0.001), Some(&modelo_ruido));

            // Simular 1000 pasos con la política adaptada
            let (metas, peligros, _recompensa) =
                simulacion_1000_pasos(&politica_adaptada, 1000, *centro);

            println!(
                "Modelo ({:.0}%,{:.0}%,{:.0}%): {} metas, {} peligros",
                izq * 100.0,
                centro * 100.0,
                der * 100.0,
                metas,
                peligros
            );
        }
    }

    // === GUARDADO DE DATOS Y GRÁFICOS ===
    println!("\n=== GENERANDO GRÁFICOS Y GUARDANDO DATOS ===");

    // Guardar los datos de simulaciones de 1000 pasos en CSV
    experimentos::guardar_recompensas_csv(&resumen_1000_pasos, "datos_1000_pasos.csv")
        .expect("Error guardando datos de 1000 pasos");

    // Generar gráfico de barras para simulaciones de 1000 pasos
    if let Err(e) = graficar_recompensas_barras(&resumen_1000_pasos) {
        eprintln!("Error al generar gráfico de barras: {:?}", e);
    }

    // Guardar CSV de simulación más completa usando experimentos
    experimentos::simular_y_guardar_csv(
        &factores_landa,
        &probabilidades_exito,
        1000,
        100,
        "resultados_simulacion.csv",
    );

    // Leer CSV y generar gráficos finales de robustez
    let resumen_recompensas_csv = leer_recompensas_csv("resultados_simulacion.csv");
    if let Err(e) = graficar_resultados_finales(&resumen_recompensas_csv) {
        eprintln!("Error al graficar resultados finales: {:?}", e);
    }

    guardar_matrices_transicion_csv();
}
