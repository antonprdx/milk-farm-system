use genpdf::Element as _;
use genpdf::{elements, fonts, style, Document, SimplePageDecorator};
use sqlx::PgPool;

use crate::errors::AppError;

pub struct TableSection {
    pub title: Option<String>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

fn load_font_family() -> Result<fonts::FontFamily<fonts::FontData>, AppError> {
    let dirs = [
        "/usr/share/fonts/liberation",
        "/usr/share/fonts/truetype/liberation",
    ];
    for dir in &dirs {
        let path = std::path::Path::new(dir).join("LiberationSans-Regular.ttf");
        if path.exists() {
            return fonts::from_files(*dir, "LiberationSans", None)
                .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to load fonts: {e}")));
        }
    }
    Err(AppError::Internal(anyhow::anyhow!(
        "LiberationSans fonts not found"
    )))
}

pub fn generate_pdf(
    doc_title: &str,
    subtitle: &str,
    sections: &[TableSection],
) -> Result<Vec<u8>, AppError> {
    let font_family = load_font_family()?;
    let mut doc = Document::new(font_family);
    doc.set_title(doc_title);
    doc.set_minimal_conformance();

    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);

    doc.push(
        elements::Paragraph::new(doc_title).styled(style::Style::new().bold().with_font_size(16)),
    );
    doc.push(elements::Paragraph::new(subtitle).styled(style::Style::new().with_font_size(9)));
    doc.push(elements::Break::new(1.0));

    for section in sections {
        if let Some(ref title) = section.title {
            doc.push(elements::Break::new(0.5));
            doc.push(
                elements::Paragraph::new(title.as_str())
                    .styled(style::Style::new().bold().with_font_size(11)),
            );
            doc.push(elements::Break::new(0.5));
        }

        let col_count = section.headers.len();
        if col_count == 0 {
            continue;
        }
        let weights: Vec<usize> = (0..col_count).map(|_| 1).collect();
        let mut table = elements::TableLayout::new(weights);
        table.set_cell_decorator(elements::FrameCellDecorator::new(true, true, false));

        {
            let mut row = table.row();
            for h in &section.headers {
                row.push_element(
                    elements::Paragraph::new(h.as_str())
                        .styled(style::Style::new().bold().with_font_size(8))
                        .padded(1),
                );
            }
            row.push()
                .map_err(|e| AppError::Internal(anyhow::anyhow!("PDF table error: {e}")))?;
        }

        for row_data in &section.rows {
            let mut row = table.row();
            for cell in row_data {
                row.push_element(
                    elements::Paragraph::new(cell.as_str())
                        .styled(style::Style::new().with_font_size(7))
                        .padded(1),
                );
            }
            row.push()
                .map_err(|e| AppError::Internal(anyhow::anyhow!("PDF table error: {e}")))?;
        }

        doc.push(table);
    }

    let mut buf = Vec::new();
    doc.render(&mut buf)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("PDF render error: {e}")))?;

    Ok(buf)
}

pub async fn generate_animal_card(pool: &PgPool, animal_id: i32) -> Result<Vec<u8>, AppError> {
    let animal = sqlx::query_as::<_, crate::models::animal::Animal>(
        "SELECT * FROM animals WHERE id = $1",
    )
    .bind(animal_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound(format!("Животное с ID {} не найдено", animal_id)))?;

    let gender = match animal.gender {
        crate::models::GenderType::Male => "М",
        crate::models::GenderType::Female => "Ж",
    };

    let title = format!(
        "Карточка животного #{}",
        animal.id
    );
    let subtitle = format!(
        "Сформировано: {}",
        chrono::Local::now().format("%d.%m.%Y")
    );

    let info_rows = vec![
        vec![
            "ID".into(),
            animal.id.to_string(),
            "Кличка".into(),
            animal.name.unwrap_or_default(),
        ],
        vec![
            "Номер жизни".into(),
            animal.life_number.unwrap_or_default(),
            "Пол".into(),
            gender.into(),
        ],
        vec![
            "Дата рождения".into(),
            animal.birth_date.to_string(),
            "Локация".into(),
            animal.location.unwrap_or_default(),
        ],
        vec![
            "UCN".into(),
            animal.ucn_number.unwrap_or_default(),
            "Статус".into(),
            if animal.active { "Активно" } else { "Неактивно" }.into(),
        ],
    ];

    let calvings: Vec<(chrono::NaiveDate, Option<i32>)> = sqlx::query_as(
        "SELECT calving_date, lac_number FROM calvings WHERE animal_id = $1 ORDER BY calving_date DESC LIMIT 10",
    )
    .bind(animal_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let mut lac_rows: Vec<Vec<String>> = vec![];
    for (date, lac) in &calvings {
        lac_rows.push(vec![date.to_string(), lac.map_or("-".into(), |l| l.to_string())]);
    }

    let vet_records: Vec<(chrono::NaiveDate, String, Option<String>)> = sqlx::query_as(
        "SELECT event_date, record_type::text, diagnosis FROM vet_records WHERE animal_id = $1 ORDER BY event_date DESC LIMIT 10",
    )
    .bind(animal_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let mut vet_rows: Vec<Vec<String>> = vec![];
    for (date, rt, diag) in &vet_records {
        vet_rows.push(vec![date.to_string(), rt.clone(), diag.clone().unwrap_or_default()]);
    }

    let weights: Vec<(chrono::NaiveDate, f64)> = sqlx::query_as(
        "SELECT measure_date, weight_kg FROM weight_records WHERE animal_id = $1 ORDER BY measure_date DESC LIMIT 10",
    )
    .bind(animal_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let mut weight_rows: Vec<Vec<String>> = vec![];
    for (date, w) in &weights {
        weight_rows.push(vec![date.to_string(), format!("{:.1}", w)]);
    }

    let milk_stats: (Option<f64>, Option<f64>) = sqlx::query_as(
        "SELECT AVG(milk_amount)::double precision, MAX(milk_amount)::double precision FROM milk_day_productions WHERE animal_id = $1 AND date >= CURRENT_DATE - INTERVAL '30 days'",
    )
    .bind(animal_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    let mut milk_rows: Vec<Vec<String>> = vec![];
    if milk_stats.0.is_some() || milk_stats.1.is_some() {
        milk_rows.push(vec![
            "Средний надой (30 дн.)".into(),
            milk_stats.0.map_or("-".into(), |v| format!("{:.1} л", v)),
            "Макс. надой (30 дн.)".into(),
            milk_stats.1.map_or("-".into(), |v| format!("{:.1} л", v)),
        ]);
    }

    let mut sections = vec![TableSection {
        title: Some("Основная информация".into()),
        headers: vec!["Поле".into(), "Значение".into(), "Поле".into(), "Значение".into()],
        rows: info_rows,
    }];

    if !lac_rows.is_empty() {
        sections.push(TableSection {
            title: Some("История лактаций".into()),
            headers: vec!["Дата отёла".into(), "Лактация".into()],
            rows: lac_rows,
        });
    }

    if !vet_rows.is_empty() {
        sections.push(TableSection {
            title: Some("Ветеринарные записи".into()),
            headers: vec!["Дата".into(), "Тип".into(), "Диагноз".into()],
            rows: vet_rows,
        });
    }

    if !weight_rows.is_empty() {
        sections.push(TableSection {
            title: Some("История веса".into()),
            headers: vec!["Дата".into(), "Вес, кг".into()],
            rows: weight_rows,
        });
    }

    if !milk_rows.is_empty() {
        sections.push(TableSection {
            title: Some("Показатели надоя".into()),
            headers: vec!["Показатель".into(), "Значение".into(), "Показатель".into(), "Значение".into()],
            rows: milk_rows,
        });
    }

    generate_pdf(&title, &subtitle, &sections)
}
