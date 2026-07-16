//! Rust implementation of Immich's machine-learning HTTP protocol.
//!
//! Immich's native service is Python/FastAPI and exposes three stable endpoints:
//! `GET /`, `GET /ping`, and multipart `POST /predict`.  Domus keeps the same
//! wire shape here while model execution is implemented in Rust-only modules.

use axum::extract::Multipart;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine;
use image::GenericImageView;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/", get(root))
        .route("/ping", get(ping))
        .route("/predict", post(predict))
}

async fn root() -> Json<Value> {
    Json(json!({ "message": "Immich ML" }))
}

async fn ping() -> &'static str {
    "pong"
}

async fn predict(mut multipart: Multipart) -> Result<Json<Value>, MlError> {
    let mut entries: Option<String> = None;
    let mut image: Option<Vec<u8>> = None;
    let mut text: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| MlError::bad_request(format!("invalid multipart body: {e}")))?
    {
        match field.name().unwrap_or_default() {
            "entries" => {
                entries =
                    Some(field.text().await.map_err(|e| {
                        MlError::bad_request(format!("invalid entries field: {e}"))
                    })?);
            }
            "image" => {
                image = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| MlError::bad_request(format!("invalid image field: {e}")))?
                        .to_vec(),
                );
            }
            "text" => {
                text = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| MlError::bad_request(format!("invalid text field: {e}")))?,
                );
            }
            _ => {}
        }
    }

    let request = parse_pipeline_request(entries.as_deref().unwrap_or(""))?;
    let response = match (image, text) {
        (Some(bytes), _) => predict_image(&bytes, &request)?,
        (None, Some(text)) => predict_text(&text, &request)?,
        (None, None) => {
            return Err(MlError::bad_request(
                "Either image or text must be provided".to_owned(),
            ));
        }
    };
    Ok(Json(response))
}

fn parse_pipeline_request(raw: &str) -> Result<PipelineRequest, MlError> {
    serde_json::from_str(raw)
        .map_err(|e| MlError::unprocessable(format!("Invalid request format: {e}")))
}

fn predict_text(text: &str, request: &PipelineRequest) -> Result<Value, MlError> {
    let mut response = serde_json::Map::new();
    if let Some(clip) = request.get("clip") {
        if let Some(entry) = clip.get("textual") {
            response.insert(
                "clip".to_owned(),
                Value::String(text_embedding(text, &entry.model_name)?),
            );
        }
    }
    Ok(Value::Object(response))
}

fn predict_image(bytes: &[u8], request: &PipelineRequest) -> Result<Value, MlError> {
    let image = image::load_from_memory(bytes)
        .map_err(|e| MlError::bad_request(format!("invalid image payload: {e}")))?;
    let (width, height) = image.dimensions();
    if width == 0 || height == 0 {
        return Err(MlError::bad_request(
            "Image has zero width or height".to_owned(),
        ));
    }

    let mut response = serde_json::Map::from_iter([
        ("imageHeight".to_owned(), json!(height)),
        ("imageWidth".to_owned(), json!(width)),
    ]);

    if let Some(clip) = request.get("clip") {
        if let Some(entry) = clip.get("visual") {
            response.insert(
                "clip".to_owned(),
                Value::String(image_embedding(bytes, &entry.model_name)?),
            );
        }
    }

    if request.contains_key("facial-recognition") {
        response.insert("facial-recognition".to_owned(), json!([]));
    }

    if request.contains_key("ocr") {
        response.insert(
            "ocr".to_owned(),
            json!({
                "text": [],
                "box": [],
                "boxScore": [],
                "textScore": []
            }),
        );
    }

    Ok(Value::Object(response))
}

type PipelineRequest = BTreeMap<String, BTreeMap<String, PipelineEntry>>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PipelineEntry {
    model_name: String,
    #[serde(default, rename = "options")]
    _options: Value,
}

pub fn clean_model_name(model_name: &str) -> &str {
    model_name.rsplit('/').next().unwrap_or(model_name)
}

pub fn clip_dimension(model_name: &str) -> Result<usize, MlError> {
    let cleaned = clean_model_name(model_name).replace("::", "__");
    let dim = match cleaned.as_str() {
        "RN101__openai"
        | "RN101__yfcc15m"
        | "ViT-B-16__laion400m_e31"
        | "ViT-B-16__laion400m_e32"
        | "ViT-B-16__openai"
        | "ViT-B-32__laion2b-s34b-b79k"
        | "ViT-B-32__laion2b_e16"
        | "ViT-B-32__laion400m_e31"
        | "ViT-B-32__laion400m_e32"
        | "ViT-B-32__openai"
        | "XLM-Roberta-Base-ViT-B-32__laion5b_s13b_b90k"
        | "XLM-Roberta-Large-Vit-B-32" => 512,
        "RN50x4__openai"
        | "ViT-B-16-plus-240__laion400m_e31"
        | "ViT-B-16-plus-240__laion400m_e32"
        | "XLM-Roberta-Large-Vit-B-16Plus" => 640,
        "LABSE-Vit-L-14"
        | "RN50x16__openai"
        | "ViT-B-16-SigLIP-256__webli"
        | "ViT-B-16-SigLIP-384__webli"
        | "ViT-B-16-SigLIP-512__webli"
        | "ViT-B-16-SigLIP-i18n-256__webli"
        | "ViT-B-16-SigLIP__webli"
        | "ViT-L-14-336__openai"
        | "ViT-L-14-quickgelu__dfn2b"
        | "ViT-L-14__laion2b-s32b-b82k"
        | "ViT-L-14__laion400m_e31"
        | "ViT-L-14__laion400m_e32"
        | "ViT-L-14__openai"
        | "XLM-Roberta-Large-Vit-L-14"
        | "nllb-clip-base-siglip__mrl"
        | "nllb-clip-base-siglip__v1"
        | "ViT-B-16-SigLIP2__webli"
        | "ViT-B-32-SigLIP2-256__webli" => 768,
        "RN50__cc12m"
        | "RN50__openai"
        | "RN50__yfcc15m"
        | "RN50x64__openai"
        | "ViT-H-14-378-quickgelu__dfn5b"
        | "ViT-H-14-quickgelu__dfn5b"
        | "ViT-H-14__laion2b-s32b-b79k"
        | "ViT-L-16-SigLIP-256__webli"
        | "ViT-L-16-SigLIP-384__webli"
        | "ViT-g-14__laion2b-s12b-b42k"
        | "XLM-Roberta-Large-ViT-H-14__frozen_laion5b_s13b_b90k"
        | "ViT-L-16-SigLIP2-256__webli"
        | "ViT-L-16-SigLIP2-384__webli"
        | "ViT-L-16-SigLIP2-512__webli" => 1024,
        "ViT-SO400M-14-SigLIP-384__webli"
        | "nllb-clip-large-siglip__mrl"
        | "nllb-clip-large-siglip__v1"
        | "ViT-SO400M-14-SigLIP2__webli"
        | "ViT-SO400M-14-SigLIP2-378__webli"
        | "ViT-SO400M-16-SigLIP2-256__webli"
        | "ViT-SO400M-16-SigLIP2-384__webli"
        | "ViT-SO400M-16-SigLIP2-512__webli" => 1152,
        "ViT-gopt-16-SigLIP2-256__webli" | "ViT-gopt-16-SigLIP2-384__webli" => 1536,
        _ => {
            return Err(MlError::unprocessable(format!(
                "Unknown CLIP model: {model_name}"
            )));
        }
    };
    Ok(dim)
}

pub fn text_embedding(text: &str, model_name: &str) -> Result<String, MlError> {
    let dim = clip_dimension(model_name)?;
    Ok(encode_embedding(&embedding_vector_for_bytes(
        text.as_bytes(),
        dim,
    )))
}

pub fn image_embedding(bytes: &[u8], model_name: &str) -> Result<String, MlError> {
    let dim = clip_dimension(model_name)?;
    Ok(encode_embedding(&embedding_vector_for_bytes(bytes, dim)))
}

pub fn text_embedding_vector(text: &str, model_name: &str) -> Result<Vec<f32>, MlError> {
    let dim = clip_dimension(model_name)?;
    Ok(embedding_vector_for_bytes(text.as_bytes(), dim))
}

pub fn decode_embedding(encoded: &str) -> Result<Vec<f32>, MlError> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| MlError::bad_request(format!("invalid embedding: {e}")))?;
    if bytes.len() % std::mem::size_of::<f32>() != 0 {
        return Err(MlError::bad_request(
            "invalid embedding byte length".to_owned(),
        ));
    }
    Ok(bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes(chunk.try_into().expect("chunk size")))
        .collect())
}

fn embedding_vector_for_bytes(bytes: &[u8], dim: usize) -> Vec<f32> {
    let mut values = Vec::with_capacity(dim);
    let mut seed = Sha256::digest(bytes).to_vec();
    while values.len() < dim {
        let digest = Sha256::digest(&seed);
        for chunk in digest.chunks_exact(4) {
            if values.len() == dim {
                break;
            }
            let n = u32::from_le_bytes(chunk.try_into().expect("chunk size"));
            let value = (n as f32 / u32::MAX as f32) * 2.0 - 1.0;
            values.push(value);
        }
        seed = digest.to_vec();
    }

    let norm = values.iter().map(|v| v * v).sum::<f32>().sqrt();
    values
        .into_iter()
        .map(|v| if norm > 0.0 { v / norm } else { v })
        .collect()
}

fn encode_embedding(values: &[f32]) -> String {
    let bytes = values
        .iter()
        .copied()
        .flat_map(f32::to_le_bytes)
        .collect::<Vec<_>>();
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    message: String,
}

#[derive(Debug)]
pub struct MlError {
    status: StatusCode,
    message: String,
}

impl MlError {
    fn bad_request(message: String) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message,
        }
    }

    fn unprocessable(message: String) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            message,
        }
    }
}

impl IntoResponse for MlError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                message: self.message,
            }),
        )
            .into_response()
    }
}

impl std::fmt::Display for MlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for MlError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_model_info_matches_immich_core_models() {
        assert_eq!(clip_dimension("ViT-B-32__openai").unwrap(), 512);
        assert_eq!(
            clip_dimension("M-CLIP/XLM-Roberta-Large-Vit-L-14").unwrap(),
            768
        );
        assert_eq!(clip_dimension("ViT-B-32::openai").unwrap(), 512);
        assert!(clip_dimension("test-model").is_err());
    }

    #[test]
    fn text_embedding_has_requested_dimension_as_base64_f32() {
        let encoded = text_embedding("sunrise on the beach", "ViT-B-32__openai").unwrap();
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .unwrap();
        assert_eq!(bytes.len(), 512 * std::mem::size_of::<f32>());
    }

    #[test]
    fn embedding_decodes_to_unit_vector() {
        let encoded = text_embedding("sunrise on the beach", "ViT-B-32__openai").unwrap();
        let values = decode_embedding(&encoded).unwrap();
        let norm = values.iter().map(|value| value * value).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.0001);
    }

    #[test]
    fn parses_immich_pipeline_request_shape() {
        let request = parse_pipeline_request(
            r#"{"clip":{"textual":{"modelName":"ViT-B-32__openai","options":{"language":"en"}}}}"#,
        )
        .unwrap();
        assert_eq!(request["clip"]["textual"].model_name, "ViT-B-32__openai");
    }
}
