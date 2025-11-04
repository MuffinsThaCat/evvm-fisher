//! WASM bindings for browser/Enarx integration

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use crate::{FisherRelayer, FisherConfig, Intent};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmFisherRelayer {
    relayer: FisherRelayer,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmFisherRelayer {
    /// Create new Fisher relayer from JSON config
    #[wasm_bindgen(constructor)]
    pub fn new(config_json: &str) -> Result<WasmFisherRelayer, JsValue> {
        let config: FisherConfig = serde_json::from_str(config_json)
            .map_err(|e| JsValue::from_str(&format!("Config parse error: {}", e)))?;
        
        let relayer = FisherRelayer::new(config)
            .map_err(|e| JsValue::from_str(&format!("Relayer init error: {}", e)))?;
        
        Ok(WasmFisherRelayer { relayer })
    }
    
    /// Submit intent (async)
    pub async fn submit_intent(&self, intent_json: &str) -> Result<String, JsValue> {
        let intent: Intent = serde_json::from_str(intent_json)
            .map_err(|e| JsValue::from_str(&format!("Intent parse error: {}", e)))?;
        
        self.relayer.submit_intent(intent)
            .await
            .map_err(|e| JsValue::from_str(&format!("Submit error: {}", e)))
    }
    
    /// Process current batch
    pub async fn process_batch(&self) -> Result<String, JsValue> {
        let result = self.relayer.process_batch()
            .await
            .map_err(|e| JsValue::from_str(&format!("Batch error: {}", e)))?;
        
        serde_json::to_string(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
    }
    
    /// Get metrics
    pub async fn get_metrics(&self) -> Result<String, JsValue> {
        let metrics = self.relayer.get_metrics().await;
        
        serde_json::to_string(&metrics)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
    }
    
    /// Get attestation report
    pub fn get_attestation(&self) -> Result<String, JsValue> {
        let report = self.relayer.get_attestation()
            .map_err(|e| JsValue::from_str(&format!("Attestation error: {}", e)))?;
        
        serde_json::to_string(&report)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main() {
    // Set up panic hook for better error messages
    console_error_panic_hook::set_once();
    
    // Initialize tracing
    tracing_wasm::set_as_global_default();
}
