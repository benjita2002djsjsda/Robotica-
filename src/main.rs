/// Módulo principal del proyecto de análisis MDP
///
/// Implementa un análisis completo de un Proceso de Decisión de Markov incluyendo:
/// - Cálculo de políticas óptimas con Q-Value Iteration (Q(s,a))
/// - Simulación visual interactiva del agente
/// - Análisis de robustez bajo diferentes niveles de ruido
/// - Generación de visualizaciones y reportes en CSV
/// - Evaluación de rendimiento bajo múltiples parámetros
///
/// ALGORITMO IMPLEMENTADO: Q-Value Iteration
/// - Calcula Q(s,a) para todos los pares estado-acción
/// - Política óptima: π(s) = argmax_a Q(s,a)
/// - Valores de estado: V(s) = max_a Q(s,a)
mod config;
mod experimentos;
mod mdp_model;
mod plot_utils;
mod robustness;
mod simulation;
mod transition_matrices;

use config::obtener_recompensas;
use mdp_model::q_value_iteration;
use plot_utils::{graficar_recompensas_barras, graficar_resultados_finales, leer_recompensas_csv};
use robustness::{construir_modelo_ruido, MODELOS_ROBUSTEZ};
use simulation::{ejecutar_simulacion, simulacion_1000_pasos};
use std::collections::HashMap;
use transition_matrices::guardar_matrices_transicion_csv;

/// Función principal - Orquesta el análisis completo del MDP usando Q-Value Iteration

#[macroquad::main("Simulacion MDP Robot - Q-Value Iteration")]
async fn main() {
    // Configuración del espacio de parámetros para el análisis
    let factores_landa = vec![0.86, 0.90, 0.94, 0.98]; // Factores de descuento a evaluar
    let probabilidades_exito = vec![0.5, 0.7, 0.8, 0.9]; // Niveles de ruido en movimientos

    let mut resumen_1000_pasos = vec![];
    let mut politicas_optimas = vec![]; // Almacén de políticas para análisis de robustez
    let mut recompensas_map = obtener_recompensas();

    // === FASE 1: CÁLCULO DE POLÍTICAS ÓPTIMAS Y EVALUACIÓN INICIAL ===
    for &landa in &factores_landa {
        println!(
            "\n=== Ejecutando Q-Value Iteration para λ = {:.2} ===",
            landa
        );
        println!("Resultados de simulaciones de 1000 pasos:");
        println!("----------------------------------------");

        // Cálculo de la política óptima usando Q-Value Iteration Q(s,a)
        let (_q_valores, politica, v_valores) = q_value_iteration(landa, Some(0.001), None);

        // Convertir v_valores de HashMap<String, f64> a HashMap<&str, f64> para compatibilidad
        let valores: HashMap<&str, f64> = v_valores.iter().map(|(k, &v)| (k.as_str(), v)).collect();

        // Evaluación de rendimiento bajo diferentes niveles de ruido
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

        // Reporte de valores de estado calculados
        println!("\nValor de los estados:");
        let mut keys: Vec<_> = valores.keys().collect();
        keys.sort();
        for k in keys {
            println!("{}: {:.2}", k, valores[k]);
        }

        // Visualización de la política óptima en formato de mapa
        println!("\nPolítica óptima para λ={}:", landa);
        println!("\nMapa de política óptima:");
        println!("-------------------------");
        for fila in config::MAPA_ESTADOS.iter() {
            for estado in fila {
                let simbolo = if config::OBSTACULOS.contains(estado) {
                    "⬛" // Obstáculos
                } else if *estado == config::ESTADO_META {
                    "🎯" // Estado meta
                } else if config::ESTADOS_PELIGRO.contains(estado) {
                    "⚠️" // Estados peligrosos
                } else {
                    // Dirección óptima según la política
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

        // === FASE 2: SIMULACIÓN VISUAL INTERACTIVA ===
        println!("\n→ Iniciando simulación visual (siguiendo política óptima)...");
        ejecutar_simulacion(&politica, 100, &mut recompensas_map, landa).await;

        // Almacenar política para análisis de robustez posterior
        politicas_optimas.push((landa, politica.clone()));
    }

    // === FASE 3: EVALUACIÓN DE ROBUSTEZ BAJO DIFERENTES MODELOS DE RUIDO ===
    println!("\n=== EVALUANDO ROBUSTEZ CON MODELOS ESPECÍFICOS ===");
    println!("Evaluando con: (10%,80%,10%), (5%,90%,5%), (15%,70%,15%), (25%,50%,25%)");

    // Análisis de robustez: cómo se adapta cada política a diferentes niveles de ruido
    for (lambda, _politica_base) in &politicas_optimas {
        println!("\n--- Evaluación robustez λ={:.2} ---", lambda);

        for (izq, centro, der) in MODELOS_ROBUSTEZ.iter() {
            // Recálculo de política óptima bajo el modelo de ruido específico
            let modelo_ruido = construir_modelo_ruido(*izq, *centro, *der);
            let (_q_valores, politica_adaptada, _v_valores) =
                q_value_iteration(*lambda, Some(0.001), Some(&modelo_ruido));

            // Evaluación de rendimiento con la política adaptada al ruido
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

    // === FASE 4: GENERACIÓN DE REPORTES Y VISUALIZACIONES ===
    println!("\n=== GENERANDO GRÁFICOS Y GUARDANDO DATOS ===");

    // Exportación de datos de simulaciones de 1000 pasos
    experimentos::guardar_recompensas_csv(&resumen_1000_pasos, "datos_1000_pasos.csv")
        .expect("Error guardando datos de 1000 pasos");

    // Generación de gráficos de barras comparativos
    if let Err(e) = graficar_recompensas_barras(&resumen_1000_pasos) {
        eprintln!("Error al generar gráfico de barras: {:?}", e);
    }

    // Simulación exhaustiva y guardado de resultados completos
    experimentos::simular_y_guardar_csv(
        &factores_landa,
        &probabilidades_exito,
        1000, // Episodios por combinación de parámetros
        100,  // Pasos máximos por episodio
        "resultados_simulacion.csv",
    );

    // Generación de visualizaciones finales basadas en datos CSV
    let resumen_recompensas_csv = leer_recompensas_csv("resultados_simulacion.csv");
    if let Err(e) = graficar_resultados_finales(&resumen_recompensas_csv) {
        eprintln!("Error al graficar resultados finales: {:?}", e);
    }

    // Exportación de matrices de transición para análisis externo
    guardar_matrices_transicion_csv();
}
