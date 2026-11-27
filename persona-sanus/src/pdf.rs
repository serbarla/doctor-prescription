use std::collections::BTreeMap;

use printpdf::{
    Color, Line, LinePoint, Mm, Op, ParsedFont, PdfDocument, PdfFontHandle, PdfPage,
    PdfSaveOptions, Point, Pt, RawImage, Rgb, TextItem, TextMatrix, XObjectId, XObjectTransform,
};
use qrcode_generator::qr::{Encoder, ErrorCorrection};
use qrcode_generator::Renderer;

use crate::forms::{
    paginate_document, parse_document, DocumentLayout, DocumentMeta, LayoutBlock, LayoutPanel,
    MedicationBlock, TableRow, TextColor, TABLE_BODY_SIZE, TABLE_HEADER_SIZE,
};

const FONT_REGULAR_BYTES: &[u8] = include_bytes!("../assets/fonts/NotoSans-Regular.ttf");
const FONT_BOLD_BYTES: &[u8] = include_bytes!("../assets/fonts/NotoSans-Bold.ttf");
const PT_PER_MM: f32 = 2.834_646;

#[derive(Debug, Clone)]
pub enum RenderMode {
    Final {
        code: String,
        verify_url: String,
    },
    #[allow(dead_code)]
    Provisional,
}

#[derive(Debug, Clone)]
pub struct PdfInput {
    pub doc_json: String,
    pub values: Vec<(String, String)>,
    pub medications: Vec<MedicationBlock>,
    pub assets: Vec<(String, Vec<u8>)>,
    pub margin_mm: u32,
    pub rx_number: Option<i32>,
    pub issued_at: Option<String>,
    pub mode: RenderMode,
    pub title: String,
}

struct Fonts {
    regular: PdfFontHandle,
    bold: PdfFontHandle,
}

struct PreparedImage {
    id: XObjectId,
    width: f32,
    height: f32,
}

pub fn render(input: &PdfInput) -> Result<Vec<u8>, String> {
    let document = parse_document(&input.doc_json)?;
    let values = if input.values.is_empty() {
        crate::forms::sample_values()
    } else {
        input.values.clone()
    };
    let (provisional, code) = match &input.mode {
        RenderMode::Final { code, .. } => (false, Some(code.clone())),
        RenderMode::Provisional => (true, None),
    };
    let meta = DocumentMeta {
        verification_code: code.clone(),
        rx_number: input.rx_number,
        issued_at: input.issued_at.clone(),
        provisional,
        qr_data_url: None,
    };
    let layout = paginate_document(
        &document,
        &values,
        input.margin_mm,
        &input.medications,
        &meta,
    );

    let mut font_warnings = Vec::new();
    let parsed_regular = ParsedFont::from_bytes(FONT_REGULAR_BYTES, 0, &mut font_warnings)
        .ok_or_else(|| "No se pudo cargar la tipografía del documento.".to_owned())?;
    let parsed_bold = ParsedFont::from_bytes(FONT_BOLD_BYTES, 0, &mut font_warnings)
        .ok_or_else(|| "No se pudo cargar la tipografía en negrita.".to_owned())?;
    let mut pdf = PdfDocument::new(&input.title);
    let regular_id = pdf.add_font(&parsed_regular);
    let bold_id = pdf.add_font(&parsed_bold);
    let fonts = Fonts {
        regular: PdfFontHandle::External(regular_id),
        bold: PdfFontHandle::External(bold_id),
    };

    let mut warnings = Vec::new();
    let mut images: BTreeMap<String, PreparedImage> = BTreeMap::new();
    for (asset_id, bytes) in &input.assets {
        if let Ok(image) = RawImage::decode_from_bytes(bytes, &mut warnings) {
            let id = pdf.add_image(&image);
            images.insert(
                asset_id.clone(),
                PreparedImage {
                    id,
                    width: image.width as f32,
                    height: image.height as f32,
                },
            );
        }
    }
    let qr = final_qr(&input.mode)
        .and_then(|payload| qr_png(&payload))
        .and_then(|png| {
            RawImage::decode_from_bytes(&png, &mut warnings)
                .ok()
                .map(|image| {
                    (
                        pdf.add_image(&image),
                        image.width as f32,
                        image.height as f32,
                    )
                })
        });

    let total = layout.pages.len();
    let mut pages = Vec::with_capacity(total);
    for (index, page) in layout.pages.iter().enumerate() {
        pages.push(render_page(
            page,
            &layout,
            &fonts,
            &images,
            qr.as_ref(),
            provisional,
            index + 1,
            total,
        ));
    }
    pdf.with_pages(pages);
    let options = PdfSaveOptions {
        image_optimization: None,
        ..Default::default()
    };
    Ok(pdf.save(&options, &mut Vec::new()))
}

fn final_qr(mode: &RenderMode) -> Option<String> {
    match mode {
        RenderMode::Final { verify_url, .. } => Some(verify_url.clone()),
        RenderMode::Provisional => None,
    }
}

pub fn qr_png(payload: &str) -> Option<Vec<u8>> {
    let symbol = Encoder::new(ErrorCorrection::Medium)
        .encode_text(payload)
        .ok()?;
    Renderer::new(&symbol, 512).to_png_vec().ok()
}

pub fn color(hex: &str) -> Color {
    let bytes = hex.as_bytes();
    let (r, g, b) = if bytes.len() == 7 && bytes[0] == b'#' {
        let value = |index: usize| {
            u8::from_str_radix(&hex[index..index + 2], 16).unwrap_or(0) as f32 / 255.0
        };
        (value(1), value(3), value(5))
    } else {
        (0.0, 0.0, 0.0)
    };
    Color::Rgb(Rgb {
        r,
        g,
        b,
        icc_profile: None,
    })
}

fn push_text(
    ops: &mut Vec<Op>,
    fonts: &Fonts,
    size: f32,
    color: &Color,
    x: f32,
    y: f32,
    text: &str,
) {
    if text.is_empty() {
        return;
    }
    ops.push(Op::SetFont {
        font: fonts.regular.clone(),
        size: Pt(size),
    });
    ops.push(Op::SetFillColor { col: color.clone() });
    ops.push(Op::SetTextMatrix {
        matrix: TextMatrix::Translate(Pt(x * PT_PER_MM), Pt(y * PT_PER_MM)),
    });
    ops.push(Op::ShowText {
        items: vec![TextItem::Text(text.to_owned())],
    });
}
#[allow(clippy::too_many_arguments)]
fn push_text_bold_prefix(
    ops: &mut Vec<Op>,
    fonts: &Fonts,
    size: f32,
    color: &Color,
    x: f32,
    y: f32,
    prefix: &str,
    rest: &str,
) {
    if prefix.is_empty() && rest.is_empty() {
        return;
    }
    ops.push(Op::SetFillColor { col: color.clone() });
    ops.push(Op::SetTextMatrix {
        matrix: TextMatrix::Translate(Pt(x * PT_PER_MM), Pt(y * PT_PER_MM)),
    });
    ops.push(Op::SetFont {
        font: fonts.bold.clone(),
        size: Pt(size),
    });
    ops.push(Op::ShowText {
        items: vec![TextItem::Text(prefix.to_owned())],
    });
    if !rest.is_empty() {
        ops.push(Op::SetFont {
            font: fonts.regular.clone(),
            size: Pt(size),
        });
        ops.push(Op::ShowText {
            items: vec![TextItem::Text(rest.to_owned())],
        });
    }
}

fn push_rule(ops: &mut Vec<Op>, x1: f32, y1: f32, x2: f32, y2: f32, color: &Color, width: f32) {
    ops.push(Op::SetOutlineColor { col: color.clone() });
    ops.push(Op::SetOutlineThickness { pt: Pt(width) });
    ops.push(Op::DrawLine {
        line: Line {
            points: vec![
                LinePoint {
                    p: Point::new(Mm(x1), Mm(y1)),
                    bezier: false,
                },
                LinePoint {
                    p: Point::new(Mm(x2), Mm(y2)),
                    bezier: false,
                },
            ],
            is_closed: false,
        },
    });
}

fn push_rect(
    ops: &mut Vec<Op>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: &Color,
    width_pt: f32,
) {
    push_rule(ops, x, y, x + width, y, color, width_pt);
    push_rule(ops, x + width, y, x + width, y + height, color, width_pt);
    push_rule(ops, x + width, y + height, x, y + height, color, width_pt);
    push_rule(ops, x, y + height, x, y, color, width_pt);
}

fn push_image(
    ops: &mut Vec<Op>,
    image: &PreparedImage,
    x: f32,
    y: f32,
    box_width: f32,
    box_height: f32,
) {
    let natural_width = image.width * 72.0 / 300.0;
    let natural_height = image.height * 72.0 / 300.0;
    if natural_width <= 0.0 || natural_height <= 0.0 {
        return;
    }
    let max_width = box_width * PT_PER_MM;
    let max_height = box_height * PT_PER_MM;
    let scale = (max_width / natural_width).min(max_height / natural_height);
    ops.push(Op::UseXobject {
        id: image.id.clone(),
        transform: XObjectTransform {
            translate_x: Some(Pt(x * PT_PER_MM)),
            translate_y: Some(Pt(y * PT_PER_MM)),
            scale_x: Some(scale),
            scale_y: Some(scale),
            dpi: Some(300.0),
            ..Default::default()
        },
    });
}

fn text_color(layout: &DocumentLayout, text_color: &TextColor) -> Color {
    match text_color {
        TextColor::Primary => color(&layout.primary),
        TextColor::Accent => color(&layout.accent),
        TextColor::Ink => color("#111111"),
        TextColor::Muted => color("#555555"),
    }
}
#[allow(clippy::too_many_arguments)]
fn render_page(
    page: &crate::forms::LayoutPage,
    layout: &DocumentLayout,
    fonts: &Fonts,
    images: &BTreeMap<String, PreparedImage>,
    qr: Option<&(XObjectId, f32, f32)>,
    provisional: bool,
    page_number: usize,
    total: usize,
) -> PdfPage {
    let muted = color("#555555");
    let mut ops = vec![Op::StartTextSection];

    let page_width = layout.page_width_mm;
    let page_height = layout.page_height_mm;
    let margin = layout.margin_mm as f32;

    for panel in &page.panels {
        render_panel(
            &mut ops,
            panel,
            layout,
            fonts,
            images,
            qr,
            provisional,
            page_width,
            page_height,
            margin,
        );
    }

    push_text(
        &mut ops,
        fonts,
        7.0,
        &muted,
        page_width / 2.0 - 12.0,
        margin + 2.0,
        &format!("Página {page_number} de {total}"),
    );
    ops.push(Op::EndTextSection);
    PdfPage::new(Mm(page_width), Mm(page_height), ops)
}
#[allow(clippy::too_many_arguments)]
fn render_panel(
    ops: &mut Vec<Op>,
    panel: &LayoutPanel,
    layout: &DocumentLayout,
    fonts: &Fonts,
    images: &BTreeMap<String, PreparedImage>,
    qr: Option<&(XObjectId, f32, f32)>,
    provisional: bool,
    page_width: f32,
    page_height: f32,
    margin: f32,
) {
    let panel_x = panel.x_mm;
    let panel_width = panel.width_mm;
    let mut y = page_height - margin;

    if panel.has_letterhead {
        if let Some(logo) = layout.logo_asset_id.as_ref().and_then(|id| images.get(id)) {
            push_image(
                ops,
                logo,
                panel_x + panel_width - 18.0,
                y - 18.0,
                18.0,
                18.0,
            );
        }
    }

    for block in &panel.blocks {
        match block {
            LayoutBlock::Text {
                lines,
                size_pt,
                bold,
                color,
                bold_prefix,
            } => {
                let color_value = text_color(layout, color);
                for line in lines {
                    y -= size_pt * 0.4;
                    if let Some(prefix) = bold_prefix
                        .as_ref()
                        .filter(|prefix| line.starts_with(prefix.as_str()))
                    {
                        let rest = line.strip_prefix(prefix).unwrap_or(line.as_str());
                        push_text_bold_prefix(
                            ops,
                            fonts,
                            *size_pt,
                            &color_value,
                            panel_x,
                            y,
                            prefix,
                            rest,
                        );
                    } else {
                        if *bold {
                            push_text_bold_prefix(
                                ops,
                                fonts,
                                *size_pt,
                                &color_value,
                                panel_x,
                                y,
                                line,
                                "",
                            );
                        } else {
                            push_text(ops, fonts, *size_pt, &color_value, panel_x, y, line);
                        }
                    }
                }
            }
            LayoutBlock::Rule => {
                y -= 1.0;
                push_rule(
                    ops,
                    panel_x,
                    y,
                    panel_x + panel_width,
                    y,
                    &primary_for_layout(layout),
                    1.8,
                );
                y -= 1.0;
            }
            LayoutBlock::Table {
                header,
                rows,
                col_weights,
            } => {
                y = render_table(
                    ops,
                    fonts,
                    panel_x,
                    y,
                    panel_width,
                    header,
                    rows,
                    col_weights,
                );
            }
            LayoutBlock::Image { slot_index } => {
                if let Some(image) = layout.images.get(*slot_index) {
                    if let Some(prepared) = image.asset_id.as_ref().and_then(|id| images.get(id)) {
                        y -= 29.0;
                        push_image(ops, prepared, panel_x, y, panel_width, 27.0);
                        if !image.caption.trim().is_empty() {
                            y -= 2.0;
                            push_text(
                                ops,
                                fonts,
                                8.0,
                                &color("#555555"),
                                panel_x,
                                y,
                                &image.caption,
                            );
                        }
                    }
                }
            }
            LayoutBlock::Spacer { height_mm } => {
                y -= *height_mm;
            }
            LayoutBlock::Qr => {
                let qr_size = if page_width > page_height { 18.0 } else { 25.0 };
                let qr_x = panel_x + panel_width - qr_size;
                let qr_y = margin + 2.0;
                if provisional {
                    push_rect(ops, qr_x, qr_y, qr_size, qr_size, &color("#555555"), 0.8);
                    push_text(
                        ops,
                        fonts,
                        7.0,
                        &color("#555555"),
                        qr_x + 6.0,
                        qr_y + qr_size / 2.0,
                        "QR",
                    );
                } else if let Some((id, width, height)) = qr {
                    let prepared = PreparedImage {
                        id: id.clone(),
                        width: *width,
                        height: *height,
                    };
                    push_image(ops, &prepared, qr_x, qr_y, qr_size, qr_size);
                }
                y = y.min(qr_y + qr_size + 2.0);
            }
        }
    }
}

fn primary_for_layout(layout: &DocumentLayout) -> Color {
    color(&layout.primary)
}
#[allow(clippy::too_many_arguments)]
fn render_table(
    ops: &mut Vec<Op>,
    fonts: &Fonts,
    panel_x: f32,
    mut y: f32,
    panel_width: f32,
    header: &[String],
    rows: &[TableRow],
    col_weights: &[f32],
) -> f32 {
    let border_color = color("#111111");
    let cell_widths: Vec<f32> = col_weights
        .iter()
        .map(|weight| panel_width * weight)
        .collect();

    let header_row_height = header
        .iter()
        .enumerate()
        .map(|(index, caption)| {
            let width = cell_widths.get(index).copied().unwrap_or(panel_width);
            let lines =
                crate::forms::wrap_table_text(caption, width, TABLE_HEADER_SIZE).len() as f32;
            lines * TABLE_HEADER_SIZE * 0.4 + 1.5
        })
        .fold(0.0, f32::max)
        .max(5.0);

    let row_heights: Vec<f32> = rows
        .iter()
        .map(|row| {
            if row.span_all {
                let lines = crate::forms::wrap_table_text(
                    &row.cells.join(" "),
                    panel_width,
                    TABLE_BODY_SIZE,
                )
                .len() as f32;
                lines * TABLE_BODY_SIZE * 0.4 + 1.5
            } else {
                row.cells
                    .iter()
                    .enumerate()
                    .map(|(index, cell)| {
                        let width = cell_widths.get(index).copied().unwrap_or(panel_width);
                        let lines = crate::forms::wrap_table_text(cell, width, TABLE_BODY_SIZE)
                            .len() as f32;
                        lines * TABLE_BODY_SIZE * 0.4 + 1.5
                    })
                    .fold(0.0, f32::max)
                    .max(5.0)
            }
        })
        .collect();

    let table_height = header_row_height + row_heights.iter().sum::<f32>();
    y -= table_height;

    let mut current_y = y + table_height;
    let top_y = current_y;

    current_y -= header_row_height;
    for (index, caption) in header.iter().enumerate() {
        let x = panel_x + cell_widths.iter().take(index).sum::<f32>();
        let width = cell_widths.get(index).copied().unwrap_or(panel_width);
        let lines = crate::forms::wrap_table_text(caption, width, TABLE_HEADER_SIZE);
        let line_height = TABLE_HEADER_SIZE * 0.4;
        let block_height = lines.len() as f32 * line_height;
        let start_y =
            current_y + (header_row_height - block_height) / 2.0 + block_height - line_height;
        for (line_index, line) in lines.iter().enumerate() {
            push_text(
                ops,
                fonts,
                TABLE_HEADER_SIZE,
                &color("#111111"),
                x + 1.0,
                start_y - line_index as f32 * line_height,
                line,
            );
        }
    }

    for (row_index, row) in rows.iter().enumerate() {
        let row_height = row_heights[row_index];
        current_y -= row_height;
        if row.span_all {
            let lines =
                crate::forms::wrap_table_text(&row.cells.join(" "), panel_width, TABLE_BODY_SIZE);
            let line_height = TABLE_BODY_SIZE * 0.4;
            for (line_index, line) in lines.iter().enumerate() {
                let font = if row.bold {
                    fonts.bold.clone()
                } else {
                    fonts.regular.clone()
                };
                ops.push(Op::SetFont {
                    font,
                    size: Pt(TABLE_BODY_SIZE),
                });
                push_text(
                    ops,
                    fonts,
                    TABLE_BODY_SIZE,
                    &color("#111111"),
                    panel_x + 1.0,
                    current_y + row_height - line_height - line_index as f32 * line_height,
                    line,
                );
            }
        } else {
            for (cell_index, cell) in row.cells.iter().enumerate() {
                let x = panel_x + cell_widths.iter().take(cell_index).sum::<f32>();
                let width = cell_widths.get(cell_index).copied().unwrap_or(panel_width);
                let lines = crate::forms::wrap_table_text(cell, width, TABLE_BODY_SIZE);
                let line_height = TABLE_BODY_SIZE * 0.4;
                for (line_index, line) in lines.iter().enumerate() {
                    let font = if row.bold {
                        fonts.bold.clone()
                    } else {
                        fonts.regular.clone()
                    };
                    ops.push(Op::SetFont {
                        font,
                        size: Pt(TABLE_BODY_SIZE),
                    });
                    push_text(
                        ops,
                        fonts,
                        TABLE_BODY_SIZE,
                        &color("#111111"),
                        x + 1.0,
                        current_y + row_height - line_height - line_index as f32 * line_height,
                        line,
                    );
                }
            }
        }
    }

    let mut x = panel_x;
    push_rule(
        ops,
        panel_x,
        top_y,
        panel_x + panel_width,
        top_y,
        &border_color,
        0.4,
    );
    push_rule(
        ops,
        panel_x,
        y,
        panel_x + panel_width,
        y,
        &border_color,
        0.4,
    );
    for width in &cell_widths {
        x += width;
        push_rule(ops, x, top_y, x, y, &border_color, 0.4);
    }
    let mut row_y = top_y;
    push_rule(
        ops,
        panel_x,
        row_y - header_row_height,
        panel_x + panel_width,
        row_y - header_row_height,
        &border_color,
        0.4,
    );
    row_y -= header_row_height;
    for row_height in &row_heights {
        row_y -= *row_height;
        push_rule(
            ops,
            panel_x,
            row_y,
            panel_x + panel_width,
            row_y,
            &border_color,
            0.4,
        );
    }

    y
}
