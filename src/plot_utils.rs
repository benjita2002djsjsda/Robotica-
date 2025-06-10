use plotters::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn leer_recompensas_csv(path: &str) -> Vec<(f64, f64, f64)> {
    let file = File::open(path).expect("No se pudo abrir el archivo CSV");
    let reader = BufReader::new(file);
    let mut datos = Vec::new();
    for (i, line) in reader.lines().enumerate() {
        let line = line.expect("Error leyendo línea");
        if i == 0 {
            continue;
        } // Saltar encabezado
        let campos: Vec<&str> = line.split(',').collect();
        if campos.len() >= 3 {
            let landa = campos[0].parse().unwrap_or(0.0);
            let exito = campos[1].parse().unwrap_or(0.0);
            let recompensa = campos[2].parse().unwrap_or(0.0);
            datos.push((landa, exito, recompensa));
        }
    }
    datos
}

pub fn graficar_resultados_finales(
    graficos_robustez: &Vec<(f64, Vec<(String, usize)>)>,
    resumen_1000_pasos: &Vec<(f64, usize, usize)>,
    resumen_recompensas: &Vec<(f64, f64, f64)>,
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
            break;
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
        chart
            .configure_mesh()
            .x_desc("Nivel de ruido")
            .y_desc("Cambios en política")
            .draw()?;

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

    // === Tercer gráfico: recompensa media según lambda y éxito ===
    let root3 = BitMapBackend::new("recompensa_media.png", (800, 500)).into_drawing_area();
    root3.fill(&WHITE)?;

    // Extraer valores únicos de lambda y prob_exito
    let mut lambdas: Vec<f64> = resumen_recompensas.iter().map(|r| r.0).collect();
    lambdas.sort_by(|a, b| a.partial_cmp(b).unwrap());
    lambdas.dedup();
    let mut exitos: Vec<f64> = resumen_recompensas.iter().map(|r| r.1).collect();
    exitos.sort_by(|a, b| a.partial_cmp(b).unwrap());
    exitos.dedup();

    let min_lambda = *lambdas.first().unwrap();
    let max_lambda = *lambdas.last().unwrap();
    let min_exito = *exitos.first().unwrap();
    let max_exito = *exitos.last().unwrap();

    let min_recompensa = resumen_recompensas
        .iter()
        .map(|r| r.2)
        .fold(f64::INFINITY, f64::min);
    let max_recompensa = resumen_recompensas
        .iter()
        .map(|r| r.2)
        .fold(f64::NEG_INFINITY, f64::max);

    let mut chart = ChartBuilder::on(&root3)
        .caption("Recompensa media según λ y éxito", ("sans-serif", 20))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(min_lambda..max_lambda, min_exito..max_exito)?;

    chart
        .configure_mesh()
        .x_desc("λ (factor de descuento)")
        .y_desc("Probabilidad de éxito")
        .draw()?;

    // Dibuja los puntos como un scatter plot coloreado por recompensa media
    chart.draw_series(
        resumen_recompensas
            .iter()
            .map(|(landa, exito, recompensa)| {
                let color = HSLColor(
                    0.33 + 0.33 * (recompensa - min_recompensa)
                        / (max_recompensa - min_recompensa + 1e-8), // verde a azul
                    0.7,
                    0.5,
                );
                Circle::new((*landa, *exito), 8, color.filled())
            }),
    )?;

    // Opcional: agregar leyenda de color
    let legend_area = root3.titled("Leyenda: Recompensa media", ("sans-serif", 15))?;
    let grad_steps = 100;
    for i in 0..grad_steps {
        let frac = i as f64 / (grad_steps - 1) as f64;

        let color = HSLColor(0.33 + 0.33 * frac, 0.7, 0.5);
        legend_area.draw(&Rectangle::new(
            [(700, 50 + i * 3), (750, 50 + (i + 1) * 3)],
            color.filled(),
        ))?;
    }
    legend_area.draw(&Text::new(
        format!("{:.2}", max_recompensa),
        (755, 50),
        ("sans-serif", 15),
    ))?;
    legend_area.draw(&Text::new(
        format!("{:.2}", min_recompensa),
        (755, 50 + grad_steps * 3 - 10),
        ("sans-serif", 15),
    ))?;

    println!("✅ Imagen 'robustez_politicas.png' guardada correctamente.");
    println!("✅ Imagen 'simulacion_1000pasos.png' guardada correctamente.");
    println!("✅ Imagen 'recompensa_media.png' guardada correctamente.");

    Ok(())
}
