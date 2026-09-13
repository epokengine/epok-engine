//! Stdio compatibility bridge to the already enabled editor server.
use rmcp::{
    ErrorData, Peer, RoleClient, RoleServer, ServerHandler, ServiceExt, model::*,
    service::RequestContext,
};
struct Proxy(Peer<RoleClient>);
fn remote_error(error: impl std::fmt::Display) -> ErrorData {
    ErrorData::internal_error(error.to_string(), None)
}
impl ServerHandler for Proxy {
    fn get_info(&self) -> ServerInfo {
        crate::mcp::info()
    }
    async fn list_tools(
        &self,
        p: Option<PaginatedRequestParams>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        self.0.list_tools(p).await.map_err(remote_error)
    }
    async fn call_tool(
        &self,
        p: CallToolRequestParams,
        c: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        tokio::select! {r=self.0.call_tool(p)=>r.map(Into::into).map_err(remote_error),_=c.ct.cancelled()=>Err(remote_error("Cancelled"))}
    }
    async fn list_resources(
        &self,
        p: Option<PaginatedRequestParams>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        self.0.list_resources(p).await.map_err(remote_error)
    }
    async fn read_resource(
        &self,
        p: ReadResourceRequestParams,
        c: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, ErrorData> {
        tokio::select! {r=self.0.read_resource(p)=>r.map(Into::into).map_err(remote_error),_=c.ct.cancelled()=>Err(remote_error("Cancelled"))}
    }
}
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let preferences = crate::settings::Preferences::load()?;
    if !preferences.mcp.enabled {
        return Err("Enable MCP in the running editor's Preferences > AI / MCP first.".into());
    }
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async move {
            use rmcp::transport::{
                StreamableHttpClientTransport,
                streamable_http_client::StreamableHttpClientTransportConfig,
            };
            let transport = StreamableHttpClientTransport::from_config(
                StreamableHttpClientTransportConfig::with_uri(preferences.mcp.endpoint())
                    .auth_header(preferences.mcp.token),
            );
            let client = ().serve(transport).await?;
            let server = Proxy(client.peer().clone())
                .serve(rmcp::transport::stdio())
                .await?;
            server.waiting().await?;
            client.cancel().await?;
            Ok(())
        })
}
