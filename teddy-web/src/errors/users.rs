// User specific errors
use crate::define_route_error;
use reqwest::StatusCode;

define_route_error! {
    CreateUserError {
        ValidationError => (StatusCode::BAD_REQUEST, "Invalid user data provided"),
    }
}

define_route_error! {
    VerifyUserError {
        UserNotFound => (StatusCode::NOT_FOUND, "User not found"),
    }
}

define_route_error! {
    SuspendUserError {
        UserNotFound => (StatusCode::NOT_FOUND, "User not found or already suspended"),
    }
}

define_route_error! {
    ActivateUserError {
        UserNotFound => (StatusCode::NOT_FOUND, "User not found or not suspended"),
    }
}

define_route_error! {
    GetAllUsersError {
        DatabaseError => (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred"),
    }
}

define_route_error! {
    GetUserError {
        NotFound => (StatusCode::NOT_FOUND, "User not found"),
    }
}