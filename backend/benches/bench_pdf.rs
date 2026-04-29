use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

fn bench_pdf_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("pdf_generation");

    for size in [10, 50, 100, 500, 1000] {
        group.bench_with_input(
            BenchmarkId::new("generate_pdf_rows", size),
            &size,
            |b, &row_count| {
                b.iter(|| {
                    let headers: Vec<String> = [
                        "ID", "Кличка", "Надой (л)", "Жир %", "Белок %", "SCC",
                    ]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();

                    let rows: Vec<Vec<String>> = (1..=row_count)
                        .map(|i| {
                            vec![
                                i.to_string(),
                                format!("Корова-{i}"),
                                format!("{:.1}", 15.0 + (i as f64 * 0.1) % 20.0),
                                format!("{:.2}", 3.5 + (i as f64 * 0.01) % 1.5),
                                format!("{:.2}", 3.0 + (i as f64 * 0.01) % 1.0),
                                format!("{}", 100_000 + i * 1000),
                            ]
                        })
                        .collect();

                    let sections = vec![milk_farm_backend::services::pdf_service::TableSection {
                        title: Some(format!("Отчёт: {} записей", row_count)),
                        headers,
                        rows,
                    }];

                    milk_farm_backend::services::pdf_service::generate_pdf(
                        "Тестовый отчёт",
                        &format!("Записей: {}", row_count),
                        &sections,
                    )
                });
            },
        );
    }
    group.finish();
}

fn bench_pdf_multi_section(c: &mut Criterion) {
    let mut group = c.benchmark_group("pdf_multi_section");
    group.sample_size(20);

    group.bench_function("3_sections_100_rows_each", |b| {
        b.iter(|| {
            let make_section = |title: &str, col_count: usize, row_count: usize| {
                let headers: Vec<String> = (0..col_count)
                    .map(|i| format!("Колонка-{i}"))
                    .collect();
                let rows: Vec<Vec<String>> = (0..row_count)
                    .map(|r| {
                        (0..col_count)
                            .map(|c| format!("R{r}C{c}"))
                            .collect()
                    })
                    .collect();
                milk_farm_backend::services::pdf_service::TableSection {
                    title: Some(title.to_string()),
                    headers,
                    rows,
                }
            };

            let sections = vec![
                make_section("Надои", 6, 100),
                make_section("Кормление", 4, 100),
                make_section("Воспроизводство", 5, 100),
            ];

            milk_farm_backend::services::pdf_service::generate_pdf(
                "Сводный отчёт",
                "Три секции по 100 строк",
                &sections,
            )
        });
    });

    group.bench_function("single_section_2000_rows", |b| {
        b.iter(|| {
            let headers: Vec<String> = ["ID", "Имя", "Надой", "Дата"]
                .iter()
                .map(|s| s.to_string())
                .collect();
            let rows: Vec<Vec<String>> = (0..2000)
                .map(|i| {
                    vec![
                        i.to_string(),
                        format!("Животное-{i}"),
                        format!("{:.1}", 20.0 + (i as f64 * 0.01) % 15.0),
                        "2025-01-15".to_string(),
                    ]
                })
                .collect();
            let sections = vec![milk_farm_backend::services::pdf_service::TableSection {
                title: Some("Большой отчёт".to_string()),
                headers,
                rows,
            }];
            milk_farm_backend::services::pdf_service::generate_pdf(
                "Стресс-тест",
                "2000 строк",
                &sections,
            )
        });
    });
    group.finish();
}

criterion_group!(benches, bench_pdf_generation, bench_pdf_multi_section);
criterion_main!(benches);
