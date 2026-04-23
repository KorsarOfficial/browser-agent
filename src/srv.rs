use std::sync::Arc;

use base64::Engine;
use chromiumoxide::{Browser, Page};
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use chromiumoxide::page::ScreenshotParams;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{ServerHandler, tool, tool_handler, tool_router};
use tokio::sync::Mutex;

use crate::tools;
use crate::tools::navigate::NavParams;

const JS_FIND_CONTACTS: &str = r#"(() => {
    const t = document.body.innerText || '';
    const r = {telegram:[],email:[],phone:[],url:[]};
    let m;

    // Telegram handles from text: @username
    const tg1 = /(?<![A-Za-z0-9])@([A-Za-z][A-Za-z0-9_]{4,31})(?![A-Za-z0-9_])/g;
    while((m=tg1.exec(t))!==null){
        const u=m[1].toLowerCase();
        if(!['media','keyframes','import','charset','font-face','supports','page'].includes(u))
            r.telegram.push(u);
    }
    // t.me/ links from text
    const tg2 = /(?:https?:\/\/)?t(?:elegram)?\.me\/([A-Za-z][A-Za-z0-9_]{4,31})/gi;
    while((m=tg2.exec(t))!==null) r.telegram.push(m[1].toLowerCase());
    // t.me/ links from <a> href
    document.querySelectorAll('a[href]').forEach(a=>{
        const h=a.href||'';
        const u=h.match(/t(?:elegram)?\.me\/([A-Za-z][A-Za-z0-9_]{4,31})/i);
        if(u) r.telegram.push(u[1].toLowerCase());
    });

    // Emails from text
    const em = /[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}/g;
    while((m=em.exec(t))!==null) r.email.push(m[0].toLowerCase());
    // mailto: links
    document.querySelectorAll('a[href^="mailto:"]').forEach(a=>{
        const e=(a.href||'').replace('mailto:','').split('?')[0];
        if(e) r.email.push(e.toLowerCase());
    });

    // Phone numbers (international, +prefix required)
    const ph = /\+\d[\d\s\-().]{7,18}\d/g;
    while((m=ph.exec(t))!==null) r.phone.push(m[0].replace(/[\s\-().]/g,''));

    // Site URLs from <a> links
    document.querySelectorAll('a[href^="http"]').forEach(a=>{
        try{
            const u=new URL(a.href);
            if(!u.hostname.includes('t.me')&&!u.hostname.includes('telegram.me'))
                r.url.push(u.origin+u.pathname);
        }catch{}
    });

    for(const k of Object.keys(r)) r[k]=[...new Set(r[k])];
    return JSON.stringify(r);
})()"#;
use crate::tools::click::ClickParams;
use crate::tools::click_at::ClickAtParams;
use crate::tools::type_text::TypeParams;
use crate::tools::press_key::PressKeyParams;

#[derive(Clone)]
pub struct Srv {
    r: ToolRouter<Self>,
    b: Arc<Browser>,
    p: Arc<Mutex<Option<Page>>>,
}

#[tool_router]
impl Srv {
    pub fn new(b: Browser) -> Self {
        Self {
            r: Self::tool_router(),
            b: Arc::new(b),
            p: Arc::new(Mutex::new(None)),
        }
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub(crate) async fn page(&self) -> Result<Page, String> {
        let g = self.p.lock().await;
        g.as_ref().cloned().ok_or_else(|| "no page - call navigate first".into())
    }

    #[tool(description = "Health check — returns pong")]
    #[inline(always)]
    async fn ping(&self) -> Result<CallToolResult, ErrorData> {
        Ok(tools::ping::pong())
    }

    #[tool(description = "Navigate browser to a URL")]
    async fn navigate(&self, Parameters(p): Parameters<NavParams>) -> Result<CallToolResult, ErrorData> {
        let mut g = self.p.lock().await;
        let pg = match g.as_ref() {
            Some(pg) => {
                if let Err(e) = pg.goto(&p.url).await {
                    *g = None;
                    return Ok(CallToolResult::error(vec![Content::text(e.to_string())]));
                }
                pg.clone()
            }
            None => match self.b.new_page(&p.url).await {
                Ok(pg) => { *g = Some(pg.clone()); pg }
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
            }
        };
        match pg.url().await {
            Ok(Some(u)) => Ok(CallToolResult::success(vec![Content::text(format!("navigated to {u}"))])),
            Ok(None) => Ok(CallToolResult::success(vec![Content::text(format!("navigated to {}", p.url))])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(description = "Get visible text content of the current page")]
    async fn get_content(&self) -> Result<CallToolResult, ErrorData> {
        let pg = match self.page().await {
            Ok(p) => p,
            Err(msg) => return Ok(CallToolResult::error(vec![Content::text(msg)])),
        };
        match pg.evaluate("document.body.innerText").await {
            Ok(v) => match v.into_value::<String>() {
                Ok(t) => {
                    let s = if t.len() > 100_000 { &t[..100_000] } else { &t };
                    Ok(CallToolResult::success(vec![Content::text(s)]))
                }
                Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
            },
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(description = "Click an element by CSS selector")]
    async fn click(&self, Parameters(p): Parameters<ClickParams>) -> Result<CallToolResult, ErrorData> {
        let pg = match self.page().await {
            Ok(p) => p,
            Err(msg) => return Ok(CallToolResult::error(vec![Content::text(msg)])),
        };
        match pg.find_element(&p.selector).await {
            Ok(el) => match el.click().await {
                Ok(_) => Ok(CallToolResult::success(vec![Content::text(format!("clicked {}", p.selector))])),
                Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
            },
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!("selector not found: {} -- {}", p.selector, e))])),
        }
    }

    #[tool(description = "Click at pixel coordinates (x, y) on the page")]
    async fn click_at(&self, Parameters(p): Parameters<ClickAtParams>) -> Result<CallToolResult, ErrorData> {
        let pg = match self.page().await {
            Ok(p) => p,
            Err(msg) => return Ok(CallToolResult::error(vec![Content::text(msg)])),
        };
        let pt = chromiumoxide::layout::Point::new(p.x, p.y);
        match pg.click(pt).await {
            Ok(_) => Ok(CallToolResult::success(vec![Content::text(format!("clicked at ({}, {})", p.x, p.y))])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(description = "Type text into an input element (focuses first)")]
    async fn type_text(&self, Parameters(p): Parameters<TypeParams>) -> Result<CallToolResult, ErrorData> {
        let pg = match self.page().await {
            Ok(p) => p,
            Err(msg) => return Ok(CallToolResult::error(vec![Content::text(msg)])),
        };
        match pg.find_element(&p.selector).await {
            Ok(el) => match el.click().await {
                Ok(el) => match el.type_str(&p.text).await {
                    Ok(_) => Ok(CallToolResult::success(vec![Content::text(format!("typed into {}", p.selector))])),
                    Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
                },
                Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
            },
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!("selector not found: {} -- {}", p.selector, e))])),
        }
    }

    #[tool(description = "Take a screenshot of the current page (returns PNG)")]
    async fn screenshot(&self) -> Result<CallToolResult, ErrorData> {
        let pg = match self.page().await {
            Ok(p) => p,
            Err(msg) => return Ok(CallToolResult::error(vec![Content::text(msg)])),
        };
        match pg.screenshot(
            ScreenshotParams::builder()
                .format(CaptureScreenshotFormat::Png)
                .build()
        ).await {
            Ok(v) => {
                let b = base64::engine::general_purpose::STANDARD.encode(&v);
                Ok(CallToolResult::success(vec![Content::image(b, "image/png")]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(description = "Press a keyboard key on the currently focused element")]
    async fn press_key(&self, Parameters(p): Parameters<PressKeyParams>) -> Result<CallToolResult, ErrorData> {
        let pg = match self.page().await {
            Ok(p) => p,
            Err(msg) => return Ok(CallToolResult::error(vec![Content::text(msg)])),
        };
        match pg.find_element(":focus").await {
            Ok(el) => match el.press_key(&p.key).await {
                Ok(_) => Ok(CallToolResult::success(vec![Content::text(format!("pressed {}", p.key))])),
                Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
            },
            Err(_) => {
                let js = format!(
                    r#"document.activeElement&&document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{{key:'{}',bubbles:true,cancelable:true}}))"#,
                    p.key
                );
                match pg.evaluate(js).await {
                    Ok(_) => Ok(CallToolResult::success(vec![Content::text(format!("pressed {} (js)", p.key))])),
                    Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
                }
            }
        }
    }

    #[tool(description = "Extract structured contact data (Telegram, email, phone, URLs) from the current page")]
    async fn find_contacts(&self) -> Result<CallToolResult, ErrorData> {
        let pg = match self.page().await {
            Ok(p) => p,
            Err(msg) => return Ok(CallToolResult::error(vec![Content::text(msg)])),
        };
        match pg.evaluate(JS_FIND_CONTACTS).await {
            Ok(v) => match v.into_value::<String>() {
                Ok(j) => Ok(CallToolResult::success(vec![Content::text(j)])),
                Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
            },
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }
}

#[tool_handler(router = "self.r")]
impl ServerHandler for Srv {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_06_18,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("Browser agent MCP server".into()),
        }
    }
}
