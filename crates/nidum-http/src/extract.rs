use axum::{
    async_trait,
    extract::{FromRequest, Request, rejection::JsonRejection},
};
use nidum_core::HttpError;
use validator::Validate;

/// A wrapper around `axum::Json` that automatically validates the payload using the `validator` crate.
/// If validation fails, it returns a structured `HttpError::ValidationFailed`.
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidJson<T>
where
    T: Validate + 'static,
    S: Send + Sync,
    axum::Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = HttpError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let axum::Json(value) = axum::Json::<T>::from_request(req, state)
            .await
            .map_err(|e| HttpError::BadRequest(e.to_string()))?;

        if let Err(errors) = value.validate() {
            let mut formatted_errors = serde_json::Map::new();
            
            for (field, field_errors) in errors.field_errors() {
                let mut rules = Vec::new();
                for err in field_errors {
                    let mut err_obj = serde_json::Map::new();
                    err_obj.insert("code".to_string(), serde_json::Value::String(err.code.to_string()));
                    if let Some(msg) = &err.message {
                        err_obj.insert("message".to_string(), serde_json::Value::String(msg.to_string()));
                    }
                    rules.push(serde_json::Value::Object(err_obj));
                }
                formatted_errors.insert(field.to_string(), serde_json::Value::Array(rules));
            }

            return Err(HttpError::ValidationFailed(serde_json::Value::Object(formatted_errors)));
        }

        Ok(ValidJson(value))
    }
}
