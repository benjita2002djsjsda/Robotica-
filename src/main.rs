/// Main entry point for the MDP Robot Simulation application.
///
/// This program orchestrates the execution of several key components:
/// 1. Value Iteration for different discount factors (lambda) to find optimal policies.
/// 2. Visual simulation of the robot's navigation using Macroquad.
/// 3. Robustness evaluation of the generated policies under various noise models.
/// 4. A 1000-step simulation to gather performance statistics.
/// 5. Generation and saving of transition matrices and result plots.
// src/main.rs
mod config;
mod mdp_model;
mod plot_utils;
mod robustness;
mod simulation;
mod transition_matrices;

use config::obtener_recompensas; // Assuming this is already updated
use mdp_model::value_iteration; // Assuming this is already updated
use plot_utils::graficar_resultados_finales; // Assuming this is already updated
use robustness::evaluar_robustez; // Assuming this is already updated
use simulation::{ejecutar_simulacion, simulacion_1000_pasos}; // Assuming these are already updated
use transition_matrices::guardar_matrices_transicion_csv; // Assuming this is already updated

/// Orchestrates the MDP processing, simulation, and result generation.
/// Iterates through predefined lambda values, performs value iteration,
/// simulates robot behavior, evaluates policy robustness, and plots results.
#[macroquad::main("Simulacion MDP Robot")]
async fn main() {
    let vec_f64FactoresLanda = vec![0.86, 0.90, 0.94, 0.98];
    // Initialize vectors to store results for final plotting
    let mut vec_tpl_f64_vec_tpl_s_uiGraficosRobustez = vec![];
    let mut vec_tpl_f64_ui_uiResumen1000Pasos = vec![];

    // Load the base reward map
    // Note: This map is modified by `ejecutar_simulacion` if the goal is reached.
    // This could affect subsequent iterations if not intended.
    let mut hm_s_f64RecompensasMap = obtener_recompensas();

    // Main loop: Iterate through each lambda factor
    for f64LandaRef in &vec_f64FactoresLanda {
        println!(
            "\n=== Ejecutando Value Iteration para λ = {:.2} ===",
            *f64LandaRef
        );

        // Perform Value Iteration to get optimal values and policy
        // value_iteration(f64Lambda, f64Epsilon, opt_hm_s_hm_s_f64ProbTransExt)
        let (hm_s_f64ValoresEstados, mut hm_s_sPoliticaOptima) =
            value_iteration(*f64LandaRef, 0.001, None);

        // Print state values and optimal policy
        println!("\nValor de los estados:");
        let mut vec_sKeysValores: Vec<_> = hm_s_f64ValoresEstados.keys().collect();
        vec_sKeysValores.sort();
        for sKeyEstadoRef in vec_sKeysValores {
            println!(
                "{}: {:.2}",
                sKeyEstadoRef, hm_s_f64ValoresEstados[sKeyEstadoRef]
            );
        }

        println!("\nPolítica óptima:");
        let mut vec_sKeysPolitica: Vec<_> = hm_s_sPoliticaOptima.keys().collect();
        vec_sKeysPolitica.sort();
        for sKeyEstadoStrRef in vec_sKeysPolitica {
            println!(
                "{}: {:?}",
                sKeyEstadoStrRef, hm_s_sPoliticaOptima[sKeyEstadoStrRef]
            );
        }

        // Run visual simulation (Macroquad)
        // ejecutar_simulacion(ref_mut_hm_s_sPolitica, uiPasos, ref_mut_hm_s_f64RecompensasMap)
        println!("\n→ Iniciando simulación visual...");
        ejecutar_simulacion(&mut hm_s_sPoliticaOptima, 70, &mut hm_s_f64RecompensasMap).await;

        // Evaluate robustness of the current policy
        // evaluar_robustez(ref_hm_s_sPoliticaBase, f64Lambda)
        let vec_tpl_s_uiResultadosRobustez = evaluar_robustez(&hm_s_sPoliticaOptima, *f64LandaRef);
        vec_tpl_f64_vec_tpl_s_uiGraficosRobustez
            .push((*f64LandaRef, vec_tpl_s_uiResultadosRobustez));

        // Run 1000-step simulation for performance statistics
        // simulacion_1000_pasos(ref_hm_s_sPolitica, uiMaxPasos)
        let (uiMetasAlcanzadas, uiPozosCaidos) = simulacion_1000_pasos(&hm_s_sPoliticaOptima, 1000);
        vec_tpl_f64_ui_uiResumen1000Pasos.push((*f64LandaRef, uiMetasAlcanzadas, uiPozosCaidos));
    }

    // Generate and save final plots
    // graficar_resultados_finales(ref_vec_tpl_f64_vec_tpl_s_uiGraficosRobustez, ref_vec_tpl_f64_ui_uiResumen1000Pasos)
    if let Err(errBoxedError) = graficar_resultados_finales(
        &vec_tpl_f64_vec_tpl_s_uiGraficosRobustez,
        &vec_tpl_f64_ui_uiResumen1000Pasos,
    ) {
        eprintln!("Error al graficar resultados: {:?}", errBoxedError);
    }

    // Save transition matrices to CSV files
    guardar_matrices_transicion_csv();
}
