use aws_lambda_events::encodings::Body;
use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use aws_sdk_dynamodb::Client;
use http::header::HeaderMap;
use lambda_runtime::{service_fn, Error as LambdaError, LambdaEvent};
use post_todo::TodoItem;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct RequestBody {
    todo: String,
}

#[derive(Serialize)]
struct ResponseBody {
    message: String,
    error: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(
    event: LambdaEvent<ApiGatewayProxyRequest>,
) -> Result<ApiGatewayProxyResponse, LambdaError> {
    let (event, _context) = event.into_parts();
    let event_body = event.body.unwrap();
    let request: RequestBody = serde_json::from_str(event_body.as_str())?;
    let config = aws_config::load_from_env().await;

    let ddb = Client::new(&config);

    let todo_item = TodoItem::new(request.todo);
    match todo_item.add(&ddb).await {
        Ok(_) => {
            let res = ResponseBody {
                message: "Success".to_owned(),
                error: None,
            };
            let res = serde_json::to_string(&res).unwrap();

            Ok(ApiGatewayProxyResponse {
                status_code: 200,
                headers: HeaderMap::new(),
                multi_value_headers: HeaderMap::new(),
                body: Some(Body::Text(res)),
                is_base64_encoded: Some(false),
            })
        }
        Err(e) => {
            let res = ResponseBody {
                message: "Error".to_owned(),
                error: Some(format!("{}", e)),
            };
            let res = serde_json::to_string(&res).unwrap();
            Ok(ApiGatewayProxyResponse {
                status_code: 400,
                headers: HeaderMap::new(),
                multi_value_headers: HeaderMap::new(),
                body: Some(Body::Text(res)),
                is_base64_encoded: Some(false),
            })
        }
    }
}
