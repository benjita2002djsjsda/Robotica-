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

    println!("✅ Imagen 'recompensa_barras.png' guardada correctamente.");

    println!("✅ Imagen 'robustez_politicas.png' guardada correctamente.");
    println!("✅ Imagen 'simulacion_1000pasos.png' guardada correctamente.");
    println!("✅ Imagen 'recompensa_media.png' guardada correctamente.");

    Ok(())
}
// Agrega esto en plot_utils.rs

pub fn graficar_recompensas_barras(
    datos: &[(f64, f64, f64)],
) -> Result<(), Box<dyn std::error::Error>> {
    // Agrupar los datos por lambda usando una precisión fija para evitar problemas de punto flotante
    let mut grupos: Vec<(f64, Vec<(f64, f64)>)> = Vec::new();

    // Primero recolectamos todos los lambdas únicos
    let mut lambdas: Vec<f64> = datos.iter().map(|d| d.0).collect();
    lambdas.sort_by(|a, b| a.partial_cmp(b).unwrap());
    lambdas.dedup();

    // Para cada lambda, recolectamos sus datos
    for &landa in &lambdas {
        let grupo: Vec<(f64, f64)> = datos
            .iter()
            .filter(|(l, _, _)| (*l - landa).abs() < f64::EPSILON)
            .map(|(_, exito, recompensa)| (*exito, *recompensa))
            .collect();
        grupos.push((landa, grupo));
    }

    // Crear el gráfico
    let root = BitMapBackend::new("recompensa_barras.png", (1200, 800)).into_drawing_area();
    root.fill(&WHITE)?;

    // Dividir el área en subgráficos para cada lambda
    let areas = root.split_evenly((2, 2)); // 4 gráficos (2x2)

    for (i, (landa, datos_grupo)) in grupos.iter().enumerate() {
        if i >= areas.len() {
            break;
        }

        let area = &areas[i];

        // Ordenar por probabilidad de éxito
        let mut datos_ordenados = datos_grupo.clone();
        datos_ordenados.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let exitos: Vec<f64> = datos_ordenados.iter().map(|d| d.0).collect();
        let recompensas: Vec<f64> = datos_ordenados.iter().map(|d| d.1).collect();

        let max_recompensa = recompensas.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let min_recompensa = recompensas.iter().fold(f64::INFINITY, |a, &b| a.min(b));

        // Cambia el build_cartesian_2d para usar coordenadas continuas en lugar de segmentadas
        let mut chart = ChartBuilder::on(area)
            .caption(
                format!("Recompensa vs Prob. Éxito (λ = {:.2})", landa),
                ("sans-serif", 20),
            )
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(
                exitos[0]..exitos[exitos.len() - 1], // Coordenadas continuas
                min_recompensa..max_recompensa,
            )?;

        chart
            .configure_mesh()
            .x_desc("Probabilidad de éxito")
            .y_desc("Recompensa promedio")
            .x_labels(5)
            .y_labels(5)
            .draw()?;

        // Dibujar barras usando coordenadas continuas
        chart.draw_series(exitos.iter().zip(recompensas.iter()).map(|(x, y)| {
            let width = 0.08; // Ancho de las barras
            Rectangle::new(
                [(*x - width / 2.0, 0.0), (*x + width / 2.0, *y)],
                BLUE.filled(),
            )
        }))?;

        // Etiquetas de valores - ahora funciona con coordenadas continuas
        chart.draw_series(exitos.iter().zip(recompensas.iter()).map(|(x, y)| {
            Text::new(
                format!("{:.1}", y),
                (*x, *y + (max_recompensa - min_recompensa) * 0.02),
                ("sans-serif", 12).into_font().color(&RED),
            )
        }))?;
    }

    println!("✅ Gráficos de barras guardados en 'recompensa_barras.png'");
    Ok(())
}
