use plotters::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// Módulo de utilidades de visualización - Generación de gráficos y análisis visual

pub fn leer_recompensas_csv(path: &str) -> Vec<(f64, f64, f64)> {
    let file = File::open(path).expect("No se pudo abrir el archivo CSV");
    let reader = BufReader::new(file);
    let mut datos = Vec::new();

    for (i, line) in reader.lines().enumerate() {
        let line = line.expect("Error leyendo línea");
        if i == 0 {
            continue; // Saltar línea de encabezados
        }
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

/// Genera un gráfico de dispersión (scatter plot) con mapa de calor
///
/// Crea una visualización donde cada punto representa una combinación de parámetros
/// (lambda, probabilidad_éxito) y el color del punto indica la recompensa obtenida.
/// Útil para identificar regiones óptimas en el espacio de parámetros.

pub fn graficar_resultados_finales(
    resumen_recompensas: &Vec<(f64, f64, f64)>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Configuración del lienzo para el gráfico
    let root3 = BitMapBackend::new("recompensa_media.png", (800, 500)).into_drawing_area();
    root3.fill(&WHITE)?;

    // Extracción y ordenamiento de valores únicos para los ejes
    let mut lambdas: Vec<f64> = resumen_recompensas.iter().map(|r| r.0).collect();
    lambdas.sort_by(|a, b| a.partial_cmp(b).unwrap());
    lambdas.dedup();
    let mut exitos: Vec<f64> = resumen_recompensas.iter().map(|r| r.1).collect();
    exitos.sort_by(|a, b| a.partial_cmp(b).unwrap());
    exitos.dedup();

    // Determinación de rangos para los ejes
    let min_lambda = *lambdas.first().unwrap();
    let max_lambda = *lambdas.last().unwrap();
    let min_exito = *exitos.first().unwrap();
    let max_exito = *exitos.last().unwrap();

    // Cálculo del rango de recompensas para el mapeo de colores
    let min_recompensa = resumen_recompensas
        .iter()
        .map(|r| r.2)
        .fold(f64::INFINITY, f64::min);
    let max_recompensa = resumen_recompensas
        .iter()
        .map(|r| r.2)
        .fold(f64::NEG_INFINITY, f64::max);

    // Construcción del sistema de coordenadas del gráfico
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

    // Renderizado de puntos con colores basados en recompensa
    chart.draw_series(
        resumen_recompensas
            .iter()
            .map(|(landa, exito, recompensa)| {
                // Mapeo de recompensa a color: verde (bajo) -> azul (alto)
                let color = HSLColor(
                    0.33 + 0.33 * (recompensa - min_recompensa)
                        / (max_recompensa - min_recompensa + 1e-8),
                    0.7,
                    0.5,
                );
                Circle::new((*landa, *exito), 8, color.filled())
            }),
    )?;

    // Generación de leyenda de colores (barra de gradiente)
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

    // Etiquetas de valores máximo y mínimo en la leyenda
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

/// Genera gráficos de barras agrupados por factor de descuento (lambda)
///
/// Crea un panel con 4 subgráficos (2x2), cada uno mostrando cómo varía la
/// recompensa promedio según la probabilidad de éxito para un valor fijo de lambda.
/// Útil para comparar el comportamiento bajo diferentes niveles de descuento.

pub fn graficar_recompensas_barras(
    datos: &[(f64, f64, f64)],
) -> Result<(), Box<dyn std::error::Error>> {
    // Agrupación de datos por lambda para análisis separado
    let mut grupos: Vec<(f64, Vec<(f64, f64)>)> = Vec::new();

    // Extracción de valores únicos de lambda
    let mut lambdas: Vec<f64> = datos.iter().map(|d| d.0).collect();
    lambdas.sort_by(|a, b| a.partial_cmp(b).unwrap());
    lambdas.dedup();

    // Agrupación de datos por cada valor de lambda
    for &landa in &lambdas {
        let grupo: Vec<(f64, f64)> = datos
            .iter()
            .filter(|(l, _, _)| (*l - landa).abs() < f64::EPSILON)
            .map(|(_, exito, recompensa)| (*exito, *recompensa))
            .collect();
        grupos.push((landa, grupo));
    }

    // Configuración del lienzo principal dividido en subgráficos
    let root = BitMapBackend::new("recompensa_barras.png", (1200, 800)).into_drawing_area();
    root.fill(&WHITE)?;
    let areas = root.split_evenly((2, 2)); // Panel 2x2 para 4 valores de lambda

    // Generación de un subgráfico para cada valor de lambda
    for (i, (landa, datos_grupo)) in grupos.iter().enumerate() {
        if i >= areas.len() {
            break; // Solo procesar los primeros 4 valores de lambda
        }

        let area = &areas[i];

        // Ordenamiento de datos por probabilidad de éxito
        let mut datos_ordenados = datos_grupo.clone();
        datos_ordenados.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let exitos: Vec<f64> = datos_ordenados.iter().map(|d| d.0).collect();
        let recompensas: Vec<f64> = datos_ordenados.iter().map(|d| d.1).collect();

        // Cálculo de rangos para el eje Y
        let max_recompensa = recompensas.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let min_recompensa = recompensas.iter().fold(f64::INFINITY, |a, &b| a.min(b));

        // Construcción del sistema de coordenadas para este subgráfico
        let mut chart = ChartBuilder::on(area)
            .caption(
                format!("Recompensa vs Prob. Éxito (λ = {:.2})", landa),
                ("sans-serif", 20),
            )
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(
                exitos[0]..exitos[exitos.len() - 1],
                min_recompensa..max_recompensa,
            )?;

        chart
            .configure_mesh()
            .x_desc("Probabilidad de éxito")
            .y_desc("Recompensa promedio")
            .x_labels(5)
            .y_labels(5)
            .draw()?;

        // Dibujado de barras verticales
        chart.draw_series(exitos.iter().zip(recompensas.iter()).map(|(x, y)| {
            let width = 0.08; // Ancho de las barras
            Rectangle::new(
                [(*x - width / 2.0, 0.0), (*x + width / 2.0, *y)],
                BLUE.filled(),
            )
        }))?;

        // Etiquetas de valores sobre cada barra
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
