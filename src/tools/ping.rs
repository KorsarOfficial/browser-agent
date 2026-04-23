use rmcp::model::{CallToolResult, Content};

#[inline(always)]
pub fn pong() -> CallToolResult {
    CallToolResult::success(vec![Content::text("pong")])
}
