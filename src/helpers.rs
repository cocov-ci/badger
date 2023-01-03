use ab_glyph::FontArc;
use actix_web::HttpResponse;

pub(crate) fn response_for_svg(svg: String) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("image/svg+xml;charset=utf-8")
        .insert_header(("cache-control", "max-age=300, s-maxage=300"))
        .body(svg)
}

pub(crate) fn badge_for(font: &FontArc, label: String, message: String, color: String) -> String {
    let meta = shield_maker::Metadata {
        style: shield_maker::Style::Plastic,
        label: &label,
        message: &message,
        font: font.clone(),
        font_family: shield_maker::FontFamily::Default,
        label_color: None,
        color: Some(&color),
    };

    shield_maker::Renderer::render(&meta)
}
