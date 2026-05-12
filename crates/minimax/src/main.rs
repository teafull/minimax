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

    println!("\n=== Testing OpenAI Hotboard ===");
    test_openai_hotboard(&token_key);

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

fn call_hotboard_api(site: &str) -> String {
    println!("Calling hotboard API for site: {}", site);

    let url = format!("https://uapis.cn/api/v1/misc/hotboard?type={}", site);
    let response = reqwest::blocking::get(&url)
        .expect("Failed to call hotboard API")
        .text()
        .expect("Failed to get response body");

    #[derive(Deserialize)]
    struct HotboardItem {
        index: u32,
        title: String,
        url: String,
        hot_value: String,
    }

    #[derive(Deserialize)]
    struct HotboardResponse {
        #[serde(rename = "type")]
        site_type: String,
        update_time: String,
        list: Vec<HotboardItem>,
    }

    let hotboard: HotboardResponse = serde_json::from_str(&response)
        .expect("Failed to parse hotboard response");

    let mut result = format!("{} 热点排行 (更新时间: {}):\n", hotboard.site_type.to_uppercase(), hotboard.update_time);
    for item in hotboard.list.iter().take(10) {
        result.push_str(&format!("{}. {} - {}\n", item.index, item.title, item.hot_value));
    }
    result
}

fn test_openai_function_call(api_key: &str) {
    use openai::{Client, Message, Tool, FunctionDefinition, ToolExecutor};
    use std::sync::Arc;
    use serde_json::Value;

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

    // 定义 executor
    struct WeatherExecutor;
    impl ToolExecutor for WeatherExecutor {
        fn execute(&self, tool_name: &str, args: Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
            if tool_name == "get_weather" {
                let location = args["location"].as_str().unwrap_or("北京");
                let result = call_weather_api(location);
                Ok(result)
            } else {
                Err(format!("Unknown tool: {}", tool_name).into())
            }
        }
    }

    println!("Step 1: User asks about weather");
    let response = client.chat_with_executor(
        vec![get_weather_tool],
        Arc::new(WeatherExecutor),
    )
    .model("MiniMax-M2.7")
    .messages(vec![Message::user("杭州今天天气怎么样？")])
    .max_completion_tokens(4096)
    .send()
    .expect("Failed to send message with tools");

    println!("  >>>>> Final Response:");
    if let Some(choice) = response.choices.first() {
        println!("  Content: {}", choice.message.content);
    }
    println!("  >>>>> Final Response with tool result.");
}

fn test_openai_hotboard(api_key: &str) {
    use openai::{Client, Message, Tool, FunctionDefinition, ToolExecutor};
    use std::sync::Arc;
    use serde_json::Value;

    let client = Client::new(api_key).expect("Failed to create OpenAI client");

    // 定义热点查询工具
    let get_hotboard_tool = Tool {
        type_: "function".to_string(),
        function: FunctionDefinition {
            name: "get_hotboard".to_string(),
            description: "获取指定网站的热点排行".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "site": {
                        "type": "string",
                        "description": "网站标识，如 juejin, zhihu, weibo 等"
                    }
                },
                "required": ["site"]
            }),
        },
    };

    // 定义 executor
    struct HotboardExecutor;
    impl ToolExecutor for HotboardExecutor {
        fn execute(&self, tool_name: &str, args: Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
            if tool_name == "get_hotboard" {
                let site = args["site"].as_str().unwrap_or("juejin");
                let result = call_hotboard_api(site);
                Ok(result)
            } else {
                Err(format!("Unknown tool: {}", tool_name).into())
            }
        }
    }

    println!("Step 1: User asks about hotboard");
    let response = client.chat_with_executor(
        vec![get_hotboard_tool],
        Arc::new(HotboardExecutor),
    )
    .model("MiniMax-M2.7")
    .messages(vec![Message::user("给我看看掘金的热点排行")])
    .max_completion_tokens(4096)
    .send()
    .expect("Failed to send message with tools");

    println!("  Final Response:");
    if let Some(choice) = response.choices.first() {
        println!("  Content: {}", choice.message.content);
    }
}

fn test_anthropic_function_call(api_key: &str) {
    use anthropic::{AnthropicClient, Message, Tool, ToolExecutor};
    use std::sync::Arc;
    use serde_json::Value;

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

    // 定义 executor
    struct WeatherExecutor;
    impl ToolExecutor for WeatherExecutor {
        fn execute(&self, tool_name: &str, args: Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
            if tool_name == "get_weather" {
                let location = args["location"].as_str().unwrap_or("北京");
                let result = call_weather_api(location);
                Ok(result)
            } else {
                Err(format!("Unknown tool: {}", tool_name).into())
            }
        }
    }

    println!("Step 1: User asks about weather");
    let response = client.chat_with_executor(
        vec![get_weather_tool],
        Arc::new(WeatherExecutor),
    )
    .model("MiniMax-M2.7")
    .messages(vec![Message::user("上海今天天气怎么样？")])
    .max_tokens(4096)
    .send()
    .expect("Failed to send message with tools");

    println!("  Final Response:");
    for block in &response.content {
        match block {
            anthropic::ContentBlock::Text(text) => println!("  Text: {}", text.text),
            _ => {}
        }
    }
}
