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
use plot_utils::{graficar_resultados_finales, leer_recompensas_csv};
use robustness::evaluar_robustez;
use simulation::{ejecutar_simulacion, simulacion_1000_pasos};
use transition_matrices::guardar_matrices_transicion_csv;

#[macroquad::main("Simulacion MDP Robot")]
async fn main() {
    let factores_landa = vec![0.86, 0.90, 0.94, 0.98];
    let probabilidades_exito = vec![0.5, 0.7, 0.8, 0.9];

    let mut graficos_robustez = vec![];
    let mut resumen_1000_pasos = vec![];

    let mut resumen_recompensas = vec![]; // Añade esta línea
    let mut recompensas_map = obtener_recompensas();

    for &landa in &factores_landa {
        // --- Todo esto debe estar dentro del ciclo de landa ---
        println!("\n=== Ejecutando Value Iteration para λ = {:.2} ===", landa);
        println!("Resultados de simulaciones de 1000 pasos:");
        println!("----------------------------------------");
        let (valores, politica) = value_iteration(landa, Some(0.001), None);
        for &prob in &probabilidades_exito {
            let (metas, peligros, recompensa) = simulacion_1000_pasos(&politica, 1000, prob);
            // Guarda en ambos vectores

            // Guardamos en ambos vectores manteniendo consistencia
            let dato = (landa, prob, recompensa);
            resumen_1000_pasos.push(dato);
            resumen_recompensas.push(dato);

            println!(
                "λ={:.2} - Prob éxito {:.0}% Recompensa: {:.2} (Meta: {}, Peligros: {})",
                landa,
                prob * 100.0,
                recompensa,
                metas,
                peligros
            );
            resumen_1000_pasos.push((landa, prob, recompensa));
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

        // Evaluar robustez
        let resultados = evaluar_robustez(&politica, landa);
        graficos_robustez.push((landa, resultados));
    }

    // Guardar CSV de simulación real
    experimentos::simular_y_guardar_csv(
        &factores_landa,
        &probabilidades_exito,
        1000,
        100,
        "resultados_simulacion.csv",
    );

    // Guardar CSV - versión optimizada
    experimentos::guardar_recompensas_csv(&resumen_recompensas, "resultados_simulacion_1000.csv")
        .expect("Error al guardar CSV");

    // Leer CSV y graficar usando esos datos reales
    let resumen_recompensas = leer_recompensas_csv("resultados_simulacion.csv");

    // Leer CSV y graficar usando esos datos reales
    let resumen_recompensas_1000 = leer_recompensas_csv("resultados_simulacion_1000.csv");

    // Graficar los resultados en barras
    if let Err(e) = plot_utils::graficar_recompensas_barras(&resumen_recompensas_1000) {
        eprintln!("Error al graficar barras: {:?}", e);
    }

    if let Err(e) = graficar_resultados_finales(&graficos_robustez, &resumen_recompensas) {
        eprintln!("Error al graficar resultados: {:?}", e);
    }

    guardar_matrices_transicion_csv();
}
