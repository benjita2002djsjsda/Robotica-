// src/main.rs
// Ejecuta Value Iteration, simula con Macroquad y evalúa robustez

mod config;
mod mdp_model;
mod plot_utils;
mod robustness;
mod simulation;
mod transition_matrices;

use config::obtener_recompensas;
use mdp_model::value_iteration;
use plot_utils::graficar_resultados_finales;
use robustness::evaluar_robustez;
use simulation::{ejecutar_simulacion, simulacion_1000_pasos};
use transition_matrices::guardar_matrices_transicion_csv;
#[macroquad::main("Simulacion MDP Robot")]
async fn main() {
    let factores_landa = vec![0.86, 0.90, 0.94, 0.98];
    let mut graficos_robustez = vec![];
    let mut resumen_1000_pasos = vec![];

    let mut recompensas_map = obtener_recompensas();

    for &landa in &factores_landa {
        println!("\n=== Ejecutando Value Iteration para λ = {:.2} ===", landa);

        let (valores, mut politica) = value_iteration(landa, 0.001, None);

        println!("\nValor de los estados:");
        let mut keys: Vec<_> = valores.keys().collect();
        keys.sort();
        for k in keys {
            println!("{}: {:.2}", k, valores[k]);
        }

        println!("\nPolítica óptima:");
        let mut keys: Vec<_> = politica.keys().collect();
        keys.sort();
        for k in keys {
            println!("{}: {:?}", k, politica[k]);
        }

        println!("\n→ Iniciando simulación visual...");
        ejecutar_simulacion(&mut politica, 70, &mut recompensas_map).await;

        let resultados = evaluar_robustez(&politica, landa);
        graficos_robustez.push((landa, resultados));

        let (metas, pozos) = simulacion_1000_pasos(&politica, 1000);
        resumen_1000_pasos.push((landa, metas, pozos));
    }

    if let Err(e) = graficar_resultados_finales(&graficos_robustez, &resumen_1000_pasos) {
        eprintln!("Error al graficar resultados: {:?}", e);
    }

    guardar_matrices_transicion_csv();
}
