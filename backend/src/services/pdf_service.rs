use genpdf::Element as _;
use genpdf::{Document, SimplePageDecorator, elements, fonts, style};

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
