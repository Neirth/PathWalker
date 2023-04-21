use actix_web::{post, HttpResponse, web::{Json}};
use lazy_static::lazy_static;
use log::info;
use serde_json::json;

use crate::models::{Matrix, Result};
use crate::services::Walker;

lazy_static! {
    static ref MAX_SIZE: i32 = 16 * 1024 * 1024;
    static ref WALKER: Walker = Walker::new();
}

/// The walk endpoint
///
/// Exposes a endpoint that receives a matrix with the height and width of the matrix
/// and the matrix itself and returns the path of the walk
///
/// # Arguments
///
/// * `req` - The request
///
/// # Returns
///
/// * `HttpResponder` - The response
///
#[post("/walk")]
pub async fn walk_endpoint(item: Json<Matrix>) -> HttpResponse {
    // Read the request and deserialize it
    let matrix: Matrix = item.into_inner();

    // Validate the matrix with the following rules
    match 1 {
        // The matrix is empty
        1 | _ if matrix.data.len() == 0 => HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "The matrix is empty"
        })),

        // The matrix is not square
        1 | _ if matrix.height != matrix.width => HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "The matrix is not square"
        })),

        // The matrix is too big
        1 | _ if matrix.height > 128 => HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "The matrix is too big"
        })),

        // The matrix size and dimensions are not the same
        1 | _ if (matrix.width * matrix.height) as usize != matrix.data.len() => HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "The matrix size and dimensions are not the same"
        })),

        // The matrix is validated
        _ => {
            // Print the request
            info!("Received request for matrix: {:?}", matrix);

            // Get the path of the walk and return it
            return match WALKER.get_walk_path(matrix) {
                Ok(path) => HttpResponse::Ok().json(json!({ "status": "ok", "path": path })),
                Err(err) => HttpResponse::BadGateway().json(json!({
                    "status": "error",
                    "message": err.api_status().unwrap().to_string()
                }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use actix_web::{test::{call_and_read_body_json, init_service, TestRequest}, App};
    use super::*;

    #[actix_web::test]
    async fn test_walk_rest_endpoint() {
        // Prepare the matrix
        let matrix = Matrix::new(3, 3, vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);

        // Prepare the expected result
        let expected = vec![1.0, 2.0, 3.0];

        // Get the result
        let app = init_service(
            App::new().service(walk_endpoint)
        ).await;

        // Prepare the request
        let req = TestRequest::post()
            .uri("/walk")
            .set_json(&matrix)
            .to_request();

        // Get the response
        let resp: Result = call_and_read_body_json(&app, req).await;

        // Check if the result is correct
        assert_eq!(resp.status, "ok");
        assert_eq!(resp.path.unwrap(), expected);
    }
}