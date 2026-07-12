use axum::response::Html;
use minijinja::Environment;
use serde::Serialize;

use crate::error::{AppError, Result};

/// Render the named template against `ctx`, mapping any MiniJinja error
/// (unknown template, render-time error) into a typed `AppError::Render`
/// rather than letting it surface as an opaque string/panic.
pub fn render(env: &Environment<'_>, name: &str, ctx: impl Serialize) -> Result<Html<String>> {
    let template = env
        .get_template(name)
        .map_err(|e| AppError::Render(e.to_string()))?;
    let rendered = template
        .render(ctx)
        .map_err(|e| AppError::Render(e.to_string()))?;
    Ok(Html(rendered))
}
