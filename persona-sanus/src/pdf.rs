use std::collections::BTreeMap;

use printpdf::{
    Color, Line, LinePoint, Mm, Op, ParsedFont, PdfDocument, PdfFontHandle, PdfPage,
    PdfSaveOptions, Point, Pt, RawImage, Rgb, TextItem, XObjectId, XObjectTransform,
};
use qrcode_generator::qr::{Encoder, ErrorCorrection};
use qrcode_generator::Renderer;

use crate::forms::{
    paginate_document, parse_document, DocumentLayout, DocumentMeta, DocumentPage, MedicationBlock,
};

const PAGE_WIDTH: f32 = 210.0;
const PAGE_HEIGHT: f32 = 297.0;
const FONT_BYTES: &[u8] = include_bytes!("../assets/fonts/NotoSans-Regular.ttf");
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
    let parsed = ParsedFont::from_bytes(FONT_BYTES, 0, &mut font_warnings)
        .ok_or_else(|| "No se pudo cargar la tipografía del documento.".to_owned())?;
    let mut pdf = PdfDocument::new(&input.title);
    let font_id = pdf.add_font(&parsed);
    let fonts = Fonts {
        regular: PdfFontHandle::External(font_id),
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
            &PageContext {
                layout: &layout,
                fonts: &fonts,
                images: &images,
                qr: qr.as_ref(),
                provisional,
            },
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
    ops.push(Op::SetTextCursor {
        pos: Point::new(Mm(x), Mm(y)),
    });
    ops.push(Op::ShowText {
        items: vec![TextItem::Text(text.to_owned())],
    });
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

struct PageContext<'a> {
    layout: &'a DocumentLayout,
    fonts: &'a Fonts,
    images: &'a BTreeMap<String, PreparedImage>,
    qr: Option<&'a (XObjectId, f32, f32)>,
    provisional: bool,
}

fn render_page(
    page: &DocumentPage,
    context: &PageContext<'_>,
    page_number: usize,
    total: usize,
) -> PdfPage {
    let layout = context.layout;
    let fonts = context.fonts;
    let images = context.images;
    let qr = context.qr;
    let provisional = context.provisional;
    let margin = layout.margin_mm as f32;
    let primary = color(&layout.primary);
    let accent = color(&layout.accent);
    let ink = color("#111111");
    let muted = color("#555555");
    let mut ops = vec![Op::StartTextSection];
    let mut y = PAGE_HEIGHT - margin;

    if let Some(logo) = layout.logo_asset_id.as_ref().and_then(|id| images.get(id)) {
        push_image(
            &mut ops,
            logo,
            PAGE_WIDTH - margin - 20.0,
            y - 20.0,
            20.0,
            20.0,
        );
    }
    for line in &layout.header_doctor {
        y -= 6.0;
        push_text(&mut ops, fonts, 14.0, &primary, margin, y, line);
    }
    for line in &layout.header_clinic {
        y -= 4.6;
        push_text(&mut ops, fonts, 8.5, &muted, margin, y, line);
    }
    y -= 3.0;
    push_rule(&mut ops, margin, y, PAGE_WIDTH - margin, y, &primary, 1.8);

    for index in &page.images {
        if let Some(image) = layout.images.get(*index) {
            if let Some(prepared) = image.asset_id.as_ref().and_then(|id| images.get(id)) {
                y -= 31.0;
                push_image(
                    &mut ops,
                    prepared,
                    margin,
                    y,
                    PAGE_WIDTH - 2.0 * margin,
                    29.0,
                );
                if !image.caption.trim().is_empty() {
                    push_text(
                        &mut ops,
                        fonts,
                        8.0,
                        &muted,
                        margin,
                        y - 2.0,
                        &image.caption,
                    );
                }
            }
        }
    }

    y -= 6.0;
    for line in &page.lines {
        y -= 5.2;
        push_text(&mut ops, fonts, 10.0, &ink, margin, y, line);
    }

    let footer_lines = layout.footer.len() as f32;
    let footer_top = margin + footer_lines * 4.6 + 12.0;
    push_rule(
        &mut ops,
        margin,
        footer_top,
        PAGE_WIDTH - margin,
        footer_top,
        &accent,
        0.8,
    );
    let mut footer_y = footer_top - 5.0;
    for line in &layout.footer {
        push_text(&mut ops, fonts, 7.5, &muted, margin, footer_y, line);
        footer_y -= 4.6;
    }

    let qr_size = 25.0;
    let qr_x = PAGE_WIDTH - margin - qr_size;
    let qr_y = margin + 2.0;
    if provisional {
        push_rule(&mut ops, qr_x, qr_y, qr_x + qr_size, qr_y, &muted, 0.8);
        push_rule(
            &mut ops,
            qr_x + qr_size,
            qr_y,
            qr_x + qr_size,
            qr_y + qr_size,
            &muted,
            0.8,
        );
        push_rule(
            &mut ops,
            qr_x + qr_size,
            qr_y + qr_size,
            qr_x,
            qr_y + qr_size,
            &muted,
            0.8,
        );
        push_rule(&mut ops, qr_x, qr_y + qr_size, qr_x, qr_y, &muted, 0.8);
        push_text(&mut ops, fonts, 7.0, &muted, qr_x + 6.0, qr_y + 12.0, "QR");
    } else if let Some((id, width, height)) = qr {
        let prepared = PreparedImage {
            id: id.clone(),
            width: *width,
            height: *height,
        };
        push_image(&mut ops, &prepared, qr_x, qr_y, qr_size, qr_size);
    }

    push_text(
        &mut ops,
        fonts,
        7.0,
        &muted,
        PAGE_WIDTH / 2.0 - 12.0,
        margin + 2.0,
        &format!("Página {page_number} de {total}"),
    );
    ops.push(Op::EndTextSection);
    PdfPage::new(Mm(PAGE_WIDTH), Mm(PAGE_HEIGHT), ops)
}
