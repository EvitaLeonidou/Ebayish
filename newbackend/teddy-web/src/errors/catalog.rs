use crate::define_route_error;
use reqwest::StatusCode;

define_route_error! {
    ItemError {
        ValidationError => (StatusCode::BAD_REQUEST, "Invalid item data provided"),
        NotFound => (StatusCode::NOT_FOUND, "Item not found"),
        CategoriesNotFound => (StatusCode::BAD_REQUEST, "One or more categories not found"),
    }
}

define_route_error! {
    ImageError {
        InvalidFileType => (StatusCode::BAD_REQUEST, "Invalid image file type"),
        FileTooLarge => (StatusCode::PAYLOAD_TOO_LARGE, "Image file too large"),
        TooManyImages => (StatusCode::BAD_REQUEST, "Item cannot have more than 5 images"),
        ItemNotFound => (StatusCode::NOT_FOUND, "Item not found"),
        ImageNotFound => (StatusCode::NOT_FOUND, "Image not found"),
        UnauthorizedAccess => (StatusCode::FORBIDDEN, "Not authorized to modify images"),
        StorageError => (StatusCode::INSUFFICIENT_STORAGE, "Storage operation failed"),
        InvalidMultipartData => (StatusCode::BAD_REQUEST, "Invalid multipart form data"),
    }
}