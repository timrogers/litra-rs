use std::str::FromStr;

use litra::DeviceType;
use rmcp::{
    handler::server::{
        tool::ToolRouter,
        wrapper::{Json, Parameters},
    },
    model::*,
    schemars, tool, tool_handler, tool_router,
    transport::stdio,
    ErrorData as McpError, ServerHandler, ServiceExt,
};

use crate::{
    get_connected_devices, handle_brightness_command, handle_brightness_down_command,
    handle_brightness_up_command, handle_off_command, handle_on_command,
    handle_temperature_command, handle_temperature_down_command, handle_temperature_up_command,
    handle_toggle_command, CliError, CliResult, DeviceInfo,
};

/// Wrapper struct for device list to satisfy MCP's requirement for object root type
#[derive(serde::Serialize, schemars::JsonSchema)]
pub struct DeviceListResponse {
    /// List of connected Litra devices
    pub devices: Vec<DeviceInfo>,
}

// Helper function to convert string to DeviceType
fn parse_device_type(device_type_str: Option<&String>) -> Option<DeviceType> {
    device_type_str.and_then(|s| DeviceType::from_str(s).ok())
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LitraToolParams {
    /// The serial number of the device to target (optional - if not specified, all devices are targeted)
    pub serial_number: Option<String>,
    /// The device path to target (optional - useful for devices that don't show a serial number)
    pub device_path: Option<String>,
    /// The device type to target: "litra_glow", "litra_beam", or "litra_beam_lx" (optional)
    pub device_type: Option<String>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LitraBrightnessParams {
    /// The serial number of the device to target (optional - if not specified, all devices are targeted)
    pub serial_number: Option<String>,
    /// The device path to target (optional - useful for devices that don't show a serial number)
    pub device_path: Option<String>,
    /// The device type to target: "litra_glow", "litra_beam", or "litra_beam_lx" (optional)
    pub device_type: Option<String>,
    /// The brightness value to set in lumens (use either this or percentage, not both)
    pub value: Option<u16>,
    /// The brightness as a percentage of maximum brightness (use either this or value, not both)
    pub percentage: Option<u8>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LitraTemperatureParams {
    /// The serial number of the device to target (optional - if not specified, all devices are targeted)
    pub serial_number: Option<String>,
    /// The device path to target (optional - useful for devices that don't show a serial number)
    pub device_path: Option<String>,
    /// The device type to target: "litra_glow", "litra_beam", or "litra_beam_lx" (optional)
    pub device_type: Option<String>,
    /// The temperature value in Kelvin (must be a multiple of 100 between 2700K and 6500K)
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

    #[tool(
        description = "Turn Logitech Litra device(s) on. By default, all devices will be targeted, but you can optionally specify a serial number, device path, or device type.",
        annotations(
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn litra_on(
        &self,
        Parameters(params): Parameters<LitraToolParams>,
    ) -> Result<CallToolResult, McpError> {
        match handle_on_command(
            params.serial_number.as_deref(),
            params.device_path.as_deref(),
            parse_device_type(params.device_type.as_ref()).as_ref(),
        ) {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(
                "Device(s) turned on successfully",
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(
        description = "Turn Logitech Litra device(s) off. By default, all devices will be targeted, but you can optionally specify a serial number, device path, or device type.",
        annotations(
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn litra_off(
        &self,
        Parameters(params): Parameters<LitraToolParams>,
    ) -> Result<CallToolResult, McpError> {
        match handle_off_command(
            params.serial_number.as_deref(),
            params.device_path.as_deref(),
            parse_device_type(params.device_type.as_ref()).as_ref(),
        ) {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(
                "Device(s) turned off successfully",
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(
        description = "Toggle Logitech Litra device(s) on or off. By default, all devices will be targeted, but you can optionally specify a serial number, device path, or device type.",
        annotations(
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn litra_toggle(
        &self,
        Parameters(params): Parameters<LitraToolParams>,
    ) -> Result<CallToolResult, McpError> {
        match handle_toggle_command(
            params.serial_number.as_deref(),
            params.device_path.as_deref(),
            parse_device_type(params.device_type.as_ref()).as_ref(),
        ) {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(
                "Device(s) toggled successfully",
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(
        description = "Set the brightness of Logitech Litra device(s). By default, all devices will be targeted, but you can optionally specify a serial number, device path, or device type.",
        annotations(
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn litra_brightness(
        &self,
        Parameters(params): Parameters<LitraBrightnessParams>,
    ) -> Result<CallToolResult, McpError> {
        match handle_brightness_command(
            params.serial_number.as_deref(),
            params.device_path.as_deref(),
            parse_device_type(params.device_type.as_ref()).as_ref(),
            params.value,
            params.percentage,
        ) {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(
                "Brightness set successfully for device(s)",
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(
        description = "Increase the brightness of Logitech Litra device(s). By default, all devices will be targeted, but you can optionally specify a serial number, device path, or device type.",
        annotations(
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn litra_brightness_up(
        &self,
        Parameters(params): Parameters<LitraBrightnessParams>,
    ) -> Result<CallToolResult, McpError> {
        match handle_brightness_up_command(
            params.serial_number.as_deref(),
            params.device_path.as_deref(),
            parse_device_type(params.device_type.as_ref()).as_ref(),
            params.value,
            params.percentage,
        ) {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(
                "Brightness increased successfully for device(s)",
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(
        description = "Decrease the brightness of Logitech Litra device(s). By default, all devices will be targeted, but you can optionally specify a serial number, device path, or device type.",
        annotations(
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn litra_brightness_down(
        &self,
        Parameters(params): Parameters<LitraBrightnessParams>,
    ) -> Result<CallToolResult, McpError> {
        match handle_brightness_down_command(
            params.serial_number.as_deref(),
            params.device_path.as_deref(),
            parse_device_type(params.device_type.as_ref()).as_ref(),
            params.value,
            params.percentage,
        ) {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(
                "Brightness decreased successfully for device(s)",
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(
        description = "Set the temperature of Logitech Litra device(s). By default, all devices will be targeted, but you can optionally specify a serial number, device path, or device type.",
        annotations(
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn litra_temperature(
        &self,
        Parameters(params): Parameters<LitraTemperatureParams>,
    ) -> Result<CallToolResult, McpError> {
        match handle_temperature_command(
            params.serial_number.as_deref(),
            params.device_path.as_deref(),
            parse_device_type(params.device_type.as_ref()).as_ref(),
            params.value,
        ) {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(
                "Temperature set successfully for device(s)",
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(
        description = "Increase the temperature of Logitech Litra device(s). By default, all devices will be targeted, but you can optionally specify a serial number, device path, or device type.",
        annotations(
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn litra_temperature_up(
        &self,
        Parameters(params): Parameters<LitraTemperatureParams>,
    ) -> Result<CallToolResult, McpError> {
        match handle_temperature_up_command(
            params.serial_number.as_deref(),
            params.device_path.as_deref(),
            parse_device_type(params.device_type.as_ref()).as_ref(),
            params.value,
        ) {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(
                "Temperature increased successfully for device(s)",
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(
        description = "Decrease the temperature of Logitech Litra device(s). By default, all devices will be targeted, but you can optionally specify a serial number, device path, or device type.",
        annotations(
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn litra_temperature_down(
        &self,
        Parameters(params): Parameters<LitraTemperatureParams>,
    ) -> Result<CallToolResult, McpError> {
        match handle_temperature_down_command(
            params.serial_number.as_deref(),
            params.device_path.as_deref(),
            parse_device_type(params.device_type.as_ref()).as_ref(),
            params.value,
        ) {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(
                "Temperature decreased successfully for device(s)",
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(
        description = "List Logitech Litra devices connected to computer",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn litra_devices(&self) -> Result<Json<DeviceListResponse>, McpError> {
        let litra_devices =
            get_connected_devices().map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(Json(DeviceListResponse {
            devices: litra_devices,
        }))
    }
}

#[tool_handler]
impl ServerHandler for LitraMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_03_26,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: env!("CARGO_PKG_NAME").to_owned(),
                title: Some("Litra".to_owned()),
                version: env!("CARGO_PKG_VERSION").to_owned(),
                icons: None,
                website_url: Some("https://github.com/timrogers/litra-rs".to_owned()),
            },
            instructions: None,
        }
    }
}

pub fn handle_mcp_command() -> CliResult {
    // Set up tracing for the MCP server
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::DEBUG.into()),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    // Create the MCP server
    let rt = tokio::runtime::Runtime::new().map_err(|_| CliError::DeviceNotFound)?;
    rt.block_on(async {
        tracing::info!("Starting Litra MCP server");

        let service = LitraMcpServer::new()
            .serve(stdio())
            .await
            .map_err(|e| CliError::MCPError(format!("{e}")))?;
        service
            .waiting()
            .await
            .map_err(|e| CliError::MCPError(format!("{e}")))?;
        Ok(())
    })
}
