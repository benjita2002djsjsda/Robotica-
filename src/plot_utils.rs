/// Utility functions for generating and saving plots related to MDP results using the Plotters crate.
// plot_utils.rs

use plotters::prelude::*;

/// Generates and saves two plots:
/// 1. `robustez_politicas.png`: A set of bar charts (one for each lambda/policy) showing
///    how many policy actions change under different noise models (robustness).
/// 2. `simulacion_1000pasos.png`: A bar chart comparing the performance (goals reached vs.
///    danger states entered) of policies derived from different lambdas over 1000 simulation steps.
///
/// # Arguments
///
/// * `ref_vec_tpl_f64_vec_tpl_s_uiGraficosRobustez` - Data for robustness plots. A vector where each
///   element is a tuple: `(lambda_value, Vec<(noise_model_label, num_policy_changes)>)`.
/// * `ref_vec_tpl_f64_ui_uiResumen1000Pasos` - Data for the 1000-step simulation plot. A vector
///   where each element is a tuple: `(lambda_value, num_goals_reached, num_danger_states_entered)`.
///
/// # Returns
///
/// `Result<(), Box<dyn std::error::Error>>` indicating success or plotting error.
pub fn graficar_resultados_finales(
    ref_vec_tpl_f64_vec_tpl_s_uiGraficosRobustez: &Vec<(f64, Vec<(String, usize)>)>,
    ref_vec_tpl_f64_ui_uiResumen1000Pasos: &Vec<(f64, usize, usize)>,
) -> Result<(), Box<dyn std::error::Error>> {
    // === Robustness Policies Plot ===
    let daRootRobustez = BitMapBackend::new("robustez_politicas.png", (960, 640)).into_drawing_area();
    daRootRobustez.fill(&WHITE)?;

    let uiNumGraficosRobustez = ref_vec_tpl_f64_vec_tpl_s_uiGraficosRobustez.len();
    // Determine layout for multiple robustness charts (e.g., 2x2 grid)
    let uiColsPlotRobustez = 2;
    let uiRowsPlotRobustez = (uiNumGraficosRobustez as f64 / uiColsPlotRobustez as f64).ceil() as usize;
    let vec_daAreasRobustez = daRootRobustez.split_evenly((uiRowsPlotRobustez, uiColsPlotRobustez));

    for (uiIndex, (f64LandaRef, ref_vec_tpl_s_uiResultados)) in ref_vec_tpl_f64_vec_tpl_s_uiGraficosRobustez.iter().enumerate() {
        if uiIndex >= vec_daAreasRobustez.len() {
            break; // Avoid panic if there are more lambdas than areas
        }

        let daAreaActual = &vec_daAreasRobustez[uiIndex];
        let vec_sEtiquetasRobustez: Vec<String> = ref_vec_tpl_s_uiResultados.iter().map(|tpl_s_uiResultadoItemRef| tpl_s_uiResultadoItemRef.0.clone()).collect();
        let vec_uiCambiosRobustez: Vec<usize> = ref_vec_tpl_s_uiResultados.iter().map(|tpl_s_uiResultadoItemRef| tpl_s_uiResultadoItemRef.1).collect();

        let i32MaxValRobustez = *vec_uiCambiosRobustez.iter().max().unwrap_or(&0) as i32;

        // Configure chart appearance (caption, labels, mesh)
        let mut ccChartRobustez = ChartBuilder::on(daAreaActual)
            .caption(format!("λ = {:.2}", *f64LandaRef), ("sans-serif", 20))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(0..vec_sEtiquetasRobustez.len() as i32, 0..(i32MaxValRobustez + 1))?;

        ccChartRobustez.configure_mesh().x_labels(5).draw()?;

        // Draw bars for policy changes
        ccChartRobustez.draw_series(vec_uiCambiosRobustez.iter().enumerate().map(|(uiBarIndex, uiValRef)| {
            Rectangle::new(
                [(uiBarIndex as i32, 0), ((uiBarIndex + 1) as i32, *uiValRef as i32)],
                BLUE.mix(0.5).filled(),
            )
        }))?;
    }

    // === 1000-Step Simulation Performance Plot ===
    let daRootSim1000Pasos = BitMapBackend::new("simulacion_1000pasos.png", (800, 500)).into_drawing_area();
    daRootSim1000Pasos.fill(&WHITE)?;

    // Prepare data for plotting (lambdas, goals, dangers)
    let mut vec_f64LambdasSim = Vec::new();
    let mut vec_uiMetasSim = Vec::new();
    let mut vec_uiPeligrosSim = Vec::new();
    for (f64LandaItemRef, uiMetaItemRef, uiPeligroItemRef) in ref_vec_tpl_f64_ui_uiResumen1000Pasos.iter() {
        vec_f64LambdasSim.push(*f64LandaItemRef);
        vec_uiMetasSim.push(*uiMetaItemRef);
        vec_uiPeligrosSim.push(*uiPeligroItemRef);
    }

    let i32CantidadMaxSim = vec_uiMetasSim.iter().chain(&vec_uiPeligrosSim).copied().max().unwrap_or(0) as i32 + 10;

    // Configure chart appearance (caption, labels, mesh)
    let mut ccChartSim1000Pasos = ChartBuilder::on(&daRootSim1000Pasos)
        .caption("Desempeño de Políticas (1000 pasos)", ("sans-serif", 20))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0..vec_f64LambdasSim.len() as i32, 0..i32CantidadMaxSim)?;

    ccChartSim1000Pasos
        .configure_mesh()
        .x_labels(vec_f64LambdasSim.len())
        // Custom formatter for x-axis labels to show lambda values
        .x_label_formatter(&|i32IdxRef| {
            let uiIndiceXLabel = *i32IdxRef as usize;
            if uiIndiceXLabel < vec_f64LambdasSim.len() {
                format!("λ = {:.2}", vec_f64LambdasSim[uiIndiceXLabel])
            } else {
                "".to_string()
            }
        })
        .draw()?;

    let i32AnchoBarra = 1;

    // Draw bars for goals reached
    ccChartSim1000Pasos
        .draw_series(vec_uiMetasSim.iter().enumerate().map(|(uiBarIndex, uiValRef)| {
            let i32X0 = uiBarIndex as i32;
            let i32X1 = uiBarIndex as i32 + i32AnchoBarra;
            Rectangle::new([(i32X0, 0), (i32X1, *uiValRef as i32)], GREEN.mix(0.5).filled())
        }))?
        .label("Llegadas a Meta")
        .legend(|(i32LegX, i32LegY)| Rectangle::new([(i32LegX, i32LegY - 5), (i32LegX + 10, i32LegY + 5)], GREEN.filled()));

    // Draw bars for dangers encountered
    ccChartSim1000Pasos
        .draw_series(vec_uiPeligrosSim.iter().enumerate().map(|(uiBarIndex, uiValRef)| {
            let i32X0 = uiBarIndex as i32;
            let i32X1 = uiBarIndex as i32 + i32AnchoBarra;
            Rectangle::new([(i32X0, 0), (i32X1, *uiValRef as i32)], RED.mix(0.5).filled())
        }))?
        .label("En peligro")
        .legend(|(i32LegX, i32LegY)| Rectangle::new([(i32LegX, i32LegY - 5), (i32LegX + 10, i32LegY + 5)], RED.filled()));

    // Configure and draw legend
    ccChartSim1000Pasos
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .draw()?;

    println!("✅ Imagen 'robustez_politicas.png' guardada correctamente.");
    println!("✅ Imagen 'simulacion_1000pasos.png' guardada correctamente.");

    Ok(())
}
