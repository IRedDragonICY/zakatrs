use wasm_bindgen::prelude::*;
use zakat_i18n::ResourceLoader;
use gloo_net::http::Request;
use std::pin::Pin;
use std::future::Future;

#[wasm_bindgen]
pub struct WasmResourceLoader {
    base_url: String,
}

#[wasm_bindgen]
impl WasmResourceLoader {
    #[wasm_bindgen(constructor)]
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}

impl ResourceLoader for WasmResourceLoader {
    fn load_resource(&self, locale: &str) -> Pin<Box<dyn Future<Output = Result<String, String>>>> {
        let url = format!("{}/{}/main.ftl", self.base_url, locale);
        
        Box::pin(async move {
            let resp = Request::get(&url)
                .send()
                .await
                .map_err(|e| e.to_string())?;
                
            if !resp.ok() {
                return Err(format!("HTTP {} fetching {}", resp.status(), url));
            }
            
            let text = resp.text().await.map_err(|e| e.to_string())?;
            Ok(text)
        })
    }
}
