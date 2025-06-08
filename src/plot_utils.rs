// plot_utils.rs

use plotters::prelude::*;

pub fn graficar_resultados_finales(
    graficos_robustez: &Vec<(f64, Vec<(String, usize)>)>,
    resumen_1000_pasos: &Vec<(f64, usize, usize)>,
) -> Result<(), Box<dyn std::error::Error>> {
    // === Primer gráfico: robustez ===
    let root = BitMapBackend::new("robustez_politicas.png", (960, 640)).into_drawing_area();
    root.fill(&WHITE)?;

    let n = graficos_robustez.len();
    let cols = 2;
    let rows = (n as f64 / cols as f64).ceil() as usize;
    let areas = root.split_evenly((rows, cols));

    for (i, (landa, resultados)) in graficos_robustez.iter().enumerate() {
        if i >= areas.len() {
            break; // Evita panic si hay más landas que áreas
        }

        let area = &areas[i];
        let etiquetas: Vec<String> = resultados.iter().map(|r| r.0.clone()).collect();
        let cambios: Vec<usize> = resultados.iter().map(|r| r.1).collect();

        let max_val = *cambios.iter().max().unwrap_or(&0) as i32;

        let mut chart = ChartBuilder::on(area)
            .caption(format!("λ = {:.2}", landa), ("sans-serif", 20))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(0..etiquetas.len() as i32, 0..(max_val + 1))?;

        chart.configure_mesh().x_labels(5).draw()?;

        chart.draw_series(cambios.iter().enumerate().map(|(i, val)| {
            Rectangle::new(
                [(i as i32, 0), ((i + 1) as i32, *val as i32)],
                BLUE.mix(0.5).filled(),
            )
        }))?;
    }

    // === Segundo gráfico: desempeño de políticas ===
    let root2 = BitMapBackend::new("simulacion_1000pasos.png", (800, 500)).into_drawing_area();
    root2.fill(&WHITE)?;

    let (mut lambdas, mut metas, mut peligros) = (Vec::new(), Vec::new(), Vec::new());
    for (l, m, p) in resumen_1000_pasos.iter() {
        lambdas.push(*l);
        metas.push(*m);
        peligros.push(*p);
    }

    let cantidad_max = metas.iter().chain(&peligros).copied().max().unwrap_or(0) as i32 + 10;

    let mut chart = ChartBuilder::on(&root2)
        .caption("Desempeño de Políticas (1000 pasos)", ("sans-serif", 20))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0..lambdas.len() as i32, 0..cantidad_max)?;

    chart
        .configure_mesh()
        .x_labels(lambdas.len())
        .x_label_formatter(&|idx| {
            let i = *idx as usize;
            if i < lambdas.len() {
                format!("λ = {:.2}", lambdas[i])
            } else {
                "".to_string()
            }
        })
        .draw()?;

    let ancho_barra = 1;

    chart
        .draw_series(metas.iter().enumerate().map(|(i, val)| {
            let x0 = i as i32;
            let x1 = i as i32 + ancho_barra;
            Rectangle::new([(x0, 0), (x1, *val as i32)], GREEN.mix(0.5).filled())
        }))?
        .label("Llegadas a Meta")
        .legend(|(x, y)| Rectangle::new([(x, y - 5), (x + 10, y + 5)], GREEN.filled()));

    chart
        .draw_series(peligros.iter().enumerate().map(|(i, val)| {
            let x0 = i as i32;
            let x1 = i as i32 + ancho_barra;
            Rectangle::new([(x0, 0), (x1, *val as i32)], RED.mix(0.5).filled())
        }))?
        .label("En peligro")
        .legend(|(x, y)| Rectangle::new([(x, y - 5), (x + 10, y + 5)], RED.filled()));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .draw()?;

    println!("✅ Imagen 'robustez_politicas.png' guardada correctamente.");
    println!("✅ Imagen 'simulacion_1000pasos.png' guardada correctamente.");

    Ok(())
}
