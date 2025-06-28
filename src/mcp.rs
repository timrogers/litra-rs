use std::future::Future;

use rmcp::{
    Error as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    schemars,
    ServiceExt, transport::stdio,
    tool, tool_handler, tool_router,
};

use crate::{
    handle_on_command, handle_off_command, handle_toggle_command, 
    handle_brightness_command, handle_brightness_up_command, handle_brightness_down_command,
    handle_temperature_command, handle_temperature_up_command, handle_temperature_down_command,
    DeviceInfo, CliResult, CliError,
};

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LitraToolParams {
    pub serial_number: Option<String>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LitraBrightnessParams {
    pub serial_number: Option<String>,
    pub value: Option<u16>,
    pub percentage: Option<u8>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LitraTemperatureParams {
    pub serial_number: Option<String>,
    pub value: u16,
}

#[derive(Clone)]
pub struct LitraMcpServer {
    tool_router: ToolRouter<LitraMcpServer>,
}

#[tool_router]
impl LitraMcpServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Turn your Logitech Litra device on")]
    async fn litra_on(&self, Parameters(params): Parameters<LitraToolParams>) -> Result<CallToolResult, McpError> {
        match handle_on_command(params.serial_number.as_deref()) {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text("Device turned on successfully")])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(description = "Turn your Logitech Litra device off")]
    async fn litra_off(&self, Parameters(params): Parameters<LitraToolParams>) -> Result<CallToolResult, McpError> {
        match handle_off_command(params.serial_number.as_deref()) {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text("Device turned off successfully")])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(description = "Toggle your Logitech Litra device on or off")]
    async fn litra_toggle(&self, Parameters(params): Parameters<LitraToolParams>) -> Result<CallToolResult, McpError> {
        match handle_toggle_command(params.serial_number.as_deref()) {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text("Device toggled successfully")])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(description = "Set the brightness of your Logitech Litra device")]
    async fn litra_brightness(&self, Parameters(params): Parameters<LitraBrightnessParams>) -> Result<CallToolResult, McpError> {
        match handle_brightness_command(params.serial_number.as_deref(), params.value, params.percentage) {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text("Brightness set successfully")])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(description = "Increase the brightness of your Logitech Litra device")]
    async fn litra_brightness_up(&self, Parameters(params): Parameters<LitraBrightnessParams>) -> Result<CallToolResult, McpError> {
        match handle_brightness_up_command(params.serial_number.as_deref(), params.value, params.percentage) {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text("Brightness increased successfully")])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(description = "Decrease the brightness of your Logitech Litra device")]
    async fn litra_brightness_down(&self, Parameters(params): Parameters<LitraBrightnessParams>) -> Result<CallToolResult, McpError> {
        match handle_brightness_down_command(params.serial_number.as_deref(), params.value, params.percentage) {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text("Brightness decreased successfully")])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(description = "Set the temperature of your Logitech Litra device")]
    async fn litra_temperature(&self, Parameters(params): Parameters<LitraTemperatureParams>) -> Result<CallToolResult, McpError> {
        match handle_temperature_command(params.serial_number.as_deref(), params.value) {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text("Temperature set successfully")])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(description = "Increase the temperature of your Logitech Litra device")]
    async fn litra_temperature_up(&self, Parameters(params): Parameters<LitraTemperatureParams>) -> Result<CallToolResult, McpError> {
        match handle_temperature_up_command(params.serial_number.as_deref(), params.value) {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text("Temperature increased successfully")])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(description = "Decrease the temperature of your Logitech Litra device")]
    async fn litra_temperature_down(&self, Parameters(params): Parameters<LitraTemperatureParams>) -> Result<CallToolResult, McpError> {
        match handle_temperature_down_command(params.serial_number.as_deref(), params.value) {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text("Temperature decreased successfully")])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(description = "List Logitech Litra devices connected to your computer")]
    async fn litra_devices(&self) -> Result<CallToolResult, McpError> {
        use litra::Litra;
        
        let context = Litra::new().map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let litra_devices: Vec<DeviceInfo> = context
            .get_connected_devices()
            .filter_map(|device| {
                let device_handle = device.open(&context).ok()?;
                Some(DeviceInfo {
                    serial_number: device
                        .device_info()
                        .serial_number()
                        .unwrap_or("")
                        .to_string(),
                    device_type: device.device_type().to_string(),
                    is_on: device_handle.is_on().ok()?,
                    brightness_in_lumen: device_handle.brightness_in_lumen().ok()?,
                    temperature_in_kelvin: device_handle.temperature_in_kelvin().ok()?,
                    minimum_brightness_in_lumen: device_handle.minimum_brightness_in_lumen(),
                    maximum_brightness_in_lumen: device_handle.maximum_brightness_in_lumen(),
                    minimum_temperature_in_kelvin: device_handle.minimum_temperature_in_kelvin(),
                    maximum_temperature_in_kelvin: device_handle.maximum_temperature_in_kelvin(),
                })
            })
            .collect();
        
        let json_str = serde_json::to_string_pretty(&litra_devices).map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json_str)]))
    }
}

#[tool_handler]
impl ServerHandler for LitraMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("This server provides tools to control Logitech Litra light devices. You can turn devices on/off, adjust brightness and temperature, and list connected devices. Most tools accept an optional serial_number parameter to target a specific device.".to_string()),
        }
    }
}

pub fn handle_mcp_command() -> CliResult {
    // Set up tracing for the MCP server
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    // Create the MCP server
    let rt = tokio::runtime::Runtime::new().map_err(|_| CliError::DeviceNotFound)?;
    rt.block_on(async {
        tracing::info!("Starting Litra MCP server");

        let service = LitraMcpServer::new().serve(stdio()).await.map_err(|_| CliError::DeviceNotFound)?;
        service.waiting().await.map_err(|_| CliError::DeviceNotFound)?;
        Ok(())
    })
}