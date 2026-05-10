use serde::Deserialize;
use std::env;

fn main() {
    let token_key = env::var("MINIMAX_API_KEY")
        .expect("MINIMAX_API_KEY environment variable not set");

    println!("=== Testing OpenAI Models ===");
    test_openai_models(&token_key);

    println!("\n=== Testing OpenAI Chat ===");
    test_openai(&token_key);

    println!("\n=== Testing OpenAI Streaming ===");
    test_openai_stream(&token_key);

    println!("\n=== Testing Anthropic Models ===");
    test_anthropic_models(&token_key);

    println!("\n=== Testing Anthropic Chat ===");
    test_anthropic(&token_key);

    println!("\n=== Testing Anthropic Streaming ===");
    test_anthropic_stream(&token_key);

    println!("\n=== Testing OpenAI Function Calling ===");
    test_openai_function_call(&token_key);

    println!("\n=== Testing Anthropic Function Calling ===");
    test_anthropic_function_call(&token_key);
}

fn test_openai_models(api_key: &str) {
    use openai::Client;

    let client = Client::new(api_key).expect("Failed to create OpenAI client");

    println!("OpenAI Models List:");
    let result = client.models().list();
    match result {
        Ok(models) => {
            for model in &models.data {
                println!("  - {}", model.id);
            }
        }
        Err(e) => eprintln!("  Error: {:?}", e),
    }

    println!("\nOpenAI Model Detail (MiniMax-M2.7-highspeed):");
    match client.models().get("MiniMax-M2.7-highspeed") {
        Ok(model) => {
            println!("  ID: {}", model.id);
            println!("  Created: {}", model.created);
            println!("  Owned by: {}", model.owned_by);
        }
        Err(e) => eprintln!("  Error: {:?}", e),
    }
}

fn test_anthropic_models(api_key: &str) {
    let client = anthropic::AnthropicClient::new(api_key).expect("Failed to create Anthropic client");

    println!("Anthropic Models List:");
    match client.models().list() {
        Ok(models) => {
            for model in &models.data {
                println!("  - {} ({})", model.id, model.display_name);
            }
        }
        Err(e) => eprintln!("  Error: {:?}", e),
    }

    println!("\nAnthropic Model Detail (MiniMax-M2.7-highspeed):");
    match client.models().get("MiniMax-M2.7-highspeed") {
        Ok(model) => {
            println!("  ID: {}", model.id);
            println!("  Created at: {}", model.created_at);
            println!("  Display name: {}", model.display_name);
            println!("  Type: {}", model.type_);
        }
        Err(e) => eprintln!("  Error: {:?}", e),
    }
}

fn test_openai(api_key: &str) {
    let client = openai::Client::new(api_key).expect("Failed to create OpenAI client");
    let response = client.chat()
        .model("MiniMax-M2.7-highspeed")
        .messages(vec![openai::Message::user("Hello")])
        .send()
        .expect("OpenAI request failed");

    println!("OpenAI Response:");
    println!("  ID: {}", response.id);
    println!("  Model: {}", response.model);
    if let Some(choice) = response.choices.first() {
        println!("  Content: {}", choice.message.content);
    }
    println!("  Usage: {:?}", response.usage);
}

fn test_openai_stream(api_key: &str) {
    let client = openai::Client::new(api_key).expect("Failed to create OpenAI client");
    let chunks = client.chat()
        .model("MiniMax-M2.7-highspeed")
        .messages(vec![openai::Message::user("Hello")])
        .send_stream()
        .expect("OpenAI streaming request failed");

    print!("OpenAI Streaming: ");
    for chunk in chunks {
        let chunk = chunk.expect("Failed to parse chunk");
        for choice in &chunk.choices {
            if let Some(content) = &choice.delta.content {
                print!("{}", content);
            }
        }
    }
    println!();
}

fn test_anthropic(api_key: &str) {
    let client = anthropic::AnthropicClient::new(api_key).expect("Failed to create Anthropic client");
    let response = client.anthropic()
        .model("MiniMax-M2.7-highspeed")
        .messages(vec![anthropic::Message::user("Hello")])
        .max_tokens(1024)
        .send()
        .expect("Anthropic request failed");

    println!("Anthropic Response:");
    println!("  ID: {}", response.id);
    println!("  Model: {}", response.model);
    for block in &response.content {
        match block {
            anthropic::ContentBlock::Text(text) => println!("  Text: {}", text.text),
            anthropic::ContentBlock::Thinking(_) => {}
            anthropic::ContentBlock::ToolUse(tool) => println!("  Tool Use: {} - {:?}", tool.name, tool.input),
        }
    }
    println!("  Usage: {:?}", response.usage);
}

fn test_anthropic_stream(api_key: &str) {
    let client = anthropic::AnthropicClient::new(api_key).expect("Failed to create Anthropic client");
    let chunks = client.anthropic()
        .model("MiniMax-M2.7-highspeed")
        .messages(vec![anthropic::Message::user("Hello")])
        .max_tokens(1024)
        .send_stream()
        .expect("Anthropic streaming request failed");

    print!("Anthropic Streaming:");
    for chunk in chunks {
        let chunk = chunk.expect("Failed to parse chunk");
        match chunk {
            anthropic::StreamEvent::ContentBlockDelta(delta) => {
                if let Some(text) = &delta.delta.text {
                    print!("{}", text);
                }
            }
            _ => {}
        }
    }
    println!();
}

fn call_weather_api(city: &str) -> String {
    println!("Calling weather API for city: {}", city);

    let url = format!("https://uapis.cn/api/v1/misc/weather?city={}", city);
    let response = reqwest::blocking::get(&url)
        .expect("Failed to call weather API")
        .text()
        .expect("Failed to get response body");

    #[derive(Deserialize)]
    struct WeatherResponse {
        province: String,
        city: String,
        weather: String,
        temperature: u32,
        wind_direction: String,
        wind_power: String,
        humidity: u32,
        report_time: String,
    }

    let weather: WeatherResponse = serde_json::from_str(&response)
        .expect("Failed to parse weather response");

    format!("{}省{}市天气{}，温度{}°C，风向{}，风力{}，湿度{}%，发布于{}",
        weather.province, weather.city, weather.weather, weather.temperature,
        weather.wind_direction, weather.wind_power, weather.humidity, weather.report_time)
}

fn test_openai_function_call(api_key: &str) {
    use openai::{Client, Message, Tool, FunctionDefinition, ToolCall, ToolCallFunction};

    let client = Client::new(api_key).expect("Failed to create OpenAI client");

    // 定义天气查询工具
    let get_weather_tool = Tool {
        type_: "function".to_string(),
        function: FunctionDefinition {
            name: "get_weather".to_string(),
            description: "获取城市天气信息".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "城市名称"
                    }
                },
                "required": ["location"]
            }),
        },
    };

    // 第一次请求：用户问天气，LLM 应该返回工具调用
    println!("Step 1: User asks about weather");
    let response = client.chat()
        .model("MiniMax-M2.7")
        .messages(vec![Message::user("北京今天天气怎么样？")])
        .tools(vec![get_weather_tool.clone()])
        .send()
        .expect("Failed to send message");

    // 获取工具调用的 ID 和参数
    let (tool_call_id, location) = if let Some(choice) = response.choices.first() {
        println!("  Response: {}", choice.message.content);

        if let Some(tool_calls) = &choice.message.tool_calls {
            if let Some(call) = tool_calls.first() {
                println!("  Tool call: {} - {}", call.function.name, call.function.arguments);

                // 解析 location 参数
                let args: serde_json::Value = serde_json::from_str(&call.function.arguments)
                    .expect("Failed to parse arguments");
                let location = args["location"].as_str().unwrap_or("北京").to_string();

                (call.id.clone(), location)
            } else {
                println!("  No tool calls found");
                return;
            }
        } else {
            println!("  No tool calls in response");
            return;
        }
    } else {
        println!("  No choices in response");
        return;
    };

    // 调用真实天气 API
    println!("\nStep 2: Call weather API");
    let weather_result = call_weather_api(&location);
    println!("  Result: {}", weather_result);

    // 第二次请求：带上工具结果
    // 重要：需要包含原始的 assistant 消息（带 tool_calls）
    println!("\nStep 3: Send tool result back to LLM");
    let response = client.chat()
        .model("MiniMax-M2.7")
        .messages(vec![
            Message::user("北京今天天气怎么样？"),
            Message {
                role: "assistant".to_string(),
                content: "".to_string(),
                name: None,
                tool_calls: Some(vec![ToolCall {
                    id: tool_call_id.clone(),
                    type_: "function".to_string(),
                    function: ToolCallFunction {
                        name: "get_weather".to_string(),
                        arguments: format!("{{\"location\":\"{}\"}}", location),
                    },
                }]),
                tool_call_id: None,
            },
            Message::tool(&tool_call_id, &weather_result),
        ])
        .tools(vec![get_weather_tool])
        .send()
        .expect("Failed to send message with tool result");

    println!("  Final Response:");
    if let Some(choice) = response.choices.first() {
        println!("  Content: {}", choice.message.content);
    }
}

fn test_anthropic_function_call(api_key: &str) {
    use anthropic::{AnthropicClient, Message, Tool};

    let client = AnthropicClient::new(api_key).expect("Failed to create Anthropic client");

    // 定义天气查询工具
    let get_weather_tool = Tool {
        name: "get_weather".to_string(),
        description: Some("获取城市天气信息".to_string()),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "城市名称"
                }
            },
            "required": ["location"]
        }),
    };

    // 第一次请求：用户问天气
    println!("Step 1: User asks about weather");
    let result = client.anthropic()
        .model("MiniMax-M2.7")
        .messages(vec![Message::user("北京今天天气怎么样？")])
        .tools(vec![get_weather_tool.clone()])
        .max_tokens(1024)
        .send();

    let response = match result {
        Ok(r) => r,
        Err(e) => {
            eprintln!("  Error: {:?}", e);
            return;
        }
    };

    println!("  Response:");
    println!("  ID: {}", response.id);

    // 查找工具调用
    let mut tool_use_id = None;
    let mut location = "北京".to_string();
    for block in &response.content {
        match block {
            anthropic::ContentBlock::Text(text) => println!("  Text: {}", text.text),
            anthropic::ContentBlock::Thinking(_) => {}
            anthropic::ContentBlock::ToolUse(tool) => {
                println!("  Tool Use: {} - {:?}", tool.name, tool.input);

                // 解析 location 参数
                if let Some(loc) = tool.input.get("location").and_then(|v| v.as_str()) {
                    location = loc.to_string();
                }

                tool_use_id = Some(tool.id.clone());
            }
        }
    }

    if tool_use_id.is_none() {
        println!("  No tool call found");
        return;
    }

    let tool_use_id = tool_use_id.unwrap();

    // 调用真实天气 API
    println!("\nStep 2: Call weather API");
    let weather_result = call_weather_api(&location);
    println!("  Result: {}", weather_result);

    // 第二次请求：带上工具结果
    println!("\nStep 3: Send tool result back to LLM");
    let response = client.anthropic()
        .model("MiniMax-M2.7")
        .messages(vec![
            Message::user("北京今天天气怎么样？"),
            Message::assistant(""),
            Message::tool(&tool_use_id, &weather_result),
        ])
        .tools(vec![get_weather_tool])
        .max_tokens(1024)
        .send()
        .expect("Failed to send message with tool result");

    println!("  Final Response:");
    for block in &response.content {
        match block {
            anthropic::ContentBlock::Text(text) => println!("  Text: {}", text.text),
            _ => {}
        }
    }
}
