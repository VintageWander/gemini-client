use std::collections::HashMap;

use gemini_client_rs::{
    types::{ContentData, GenerateContentRequest},
    FunctionHandler, GeminiClient,
};

use dotenvy::dotenv;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");

    let client = GeminiClient::new(api_key);
    let model_name = "gemini-2.5-flash";

    let req_json = json!({
        "contents": [
            {
                "parts": [
                    {
                        "text": "What's the current weather in London, UK?"
                    }
                ],
                "role": "user"
            }
        ],
        "tools": [
             {
                "functionDeclarations": [
                    {
                        "name": "get_current_weather",
                        "description": "Get the current weather in a given location",
                        "parameters": {
                            "type": "OBJECT",
                            "properties": {
                                "location": {
                                    "type": "string",
                                    "description": "The city and state, e.g. 'San Francisco, CA'"
                                }
                            },
                            "required": ["location"]
                        }
                    }
                ]
            }
        ]
    });

    let request = serde_json::from_value::<GenerateContentRequest>(req_json)?;

    let mut function_handlers: HashMap<String, FunctionHandler> = HashMap::new();

    function_handlers.insert(
        "get_current_weather".to_string(),
        FunctionHandler::Sync(Box::new(|args: &mut serde_json::Value| {
            let location = args
                .get("location")
                .and_then(|v| v.as_str())
                .unwrap_or("London, UK");
            let weather_info =
                format!("The current weather in {location} is sunny with a temperature of 20°C.");
            Ok(serde_json::json!({
                "weather": weather_info
            }))
        })),
    );

    let response = client
        .generate_content_with_function_calling(model_name, request, &function_handlers)
        .await?;

    let first_candidate = response.candidates.first().unwrap();

    let first_part = first_candidate.content.parts.first().unwrap();

    let weather = match &first_part.data {
        ContentData::Text(text) => text,
        ContentData::FunctionCall(_) => "Function call found",
        ContentData::FunctionResponse(_) => "Function response found",
        ContentData::ExecutableCode(_) => "Executable code found",
        ContentData::CodeExecutionResult(_) => "Code execution result found",
        ContentData::InlineData(_) => "Inline data found",
        ContentData::FileData(_) => "File data found",
    };

    println!("{weather}");

    Ok(())
}
