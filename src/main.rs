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

    let mut recompensas_map = obtener_recompensas();

    for &landa in &factores_landa {
        // --- Todo esto debe estar dentro del ciclo de landa ---
        println!("\n=== Ejecutando Value Iteration para λ = {:.2} ===", landa);

        let (valores, politica) = value_iteration(landa, Some(0.001), None);

        println!("\nValor de los estados:");
        let mut keys: Vec<_> = valores.keys().collect();
        keys.sort();
        for k in keys {
            println!("{}: {:.2}", k, valores[k]);
        }

        // Mostrar política óptima
        println!("\nPolítica óptima para λ={}:", landa);
        let mut estados: Vec<_> = politica.keys().collect();
        estados.sort();
        for estado in estados {
            println!("{}: {}", estado, politica[estado]);
        }

        // Ejecutar simulación estricta
        println!("\n→ Iniciando simulación visual (siguiendo política óptima)...");
        ejecutar_simulacion(&politica, 100, &mut recompensas_map, landa).await;

        // Evaluar robustez
        let resultados = evaluar_robustez(&politica, landa);
        graficos_robustez.push((landa, resultados));

        // Simulación de 1000 pasos
        let (metas, pozos) = simulacion_1000_pasos(&politica, 1000);
        resumen_1000_pasos.push((landa, metas, pozos));
        println!(
            "Resultados 1000 pasos - λ={:.2}: {} metas, {} peligros",
            landa, metas, pozos
        );
    }

    // Guardar CSV de simulación real
    experimentos::simular_y_guardar_csv(
        &factores_landa,
        &probabilidades_exito,
        1000,
        100,
        "resultados_simulacion.csv",
    );

    // Leer CSV y graficar usando esos datos reales
    let resumen_recompensas = leer_recompensas_csv("resultados_simulacion.csv");
    if let Err(e) = graficar_resultados_finales(
        &graficos_robustez,
        &resumen_1000_pasos,
        &resumen_recompensas,
    ) {
        eprintln!("Error al graficar resultados: {:?}", e);
    }

    guardar_matrices_transicion_csv();
}
