//! WhatsApp platform tests

use aof_triggers::platforms::whatsapp::{WhatsAppPlatform, WhatsAppConfig};
use aof_triggers::platforms::TriggerPlatform;
use std::collections::HashMap;

fn test_config() -> WhatsAppConfig {
    WhatsAppConfig {
        phone_number_id: "123456789012345".to_string(),
        access_token: "EAABcDtest".to_string(),
        verify_token: "test_verify_token".to_string(),
        app_secret: "test_app_secret".to_string(),
        business_account_id: None,
        allowed_numbers: None,
        api_version: "v18.0".to_string(),
    }
}

#[tokio::test]
async fn test_whatsapp_platform_creation() {
    let platform = WhatsAppPlatform::new(test_config());
    assert!(platform.is_ok());
}

#[tokio::test]
async fn test_whatsapp_bot_name() {
    let platform = WhatsAppPlatform::new(test_config()).unwrap();
    assert_eq!(platform.bot_name(), "aofbot");
}

#[tokio::test]
async fn test_whatsapp_platform_name() {
    let platform = WhatsAppPlatform::new(test_config()).unwrap();
    assert_eq!(platform.platform_name(), "whatsapp");
}

#[tokio::test]
async fn test_whatsapp_supports_interactive() {
    let platform = WhatsAppPlatform::new(test_config()).unwrap();
    assert!(platform.supports_interactive());
}

#[tokio::test]
async fn test_whatsapp_parse_text_message() {
    let payload = r#"{
        "object": "whatsapp_business_account",
        "entry": [{
            "id": "123456789",
            "changes": [{
                "value": {
                    "messaging_product": "whatsapp",
                    "metadata": {
                        "display_phone_number": "15551234567",
                        "phone_number_id": "123456789012345"
                    },
                    "messages": [{
                        "id": "wamid.test123",
                        "from": "15559876543",
                        "timestamp": "1638360000",
                        "type": "text",
                        "text": {"body": "/run agent test-bot hello"}
                    }]
                },
                "field": "messages"
            }]
        }]
    }"#;

    let platform = WhatsAppPlatform::new(test_config()).unwrap();
    let headers = HashMap::new();
    let message = platform.parse_message(payload.as_bytes(), &headers).await.unwrap();

    assert_eq!(message.platform, "whatsapp");
    assert_eq!(message.user.id, "15559876543");
    assert_eq!(message.channel_id, "15559876543");
    assert_eq!(message.text, "/run agent test-bot hello");
}

#[tokio::test]
async fn test_whatsapp_parse_missing_entry() {
    let payload = r#"{"object": "whatsapp_business_account", "entry": []}"#;
    let platform = WhatsAppPlatform::new(test_config()).unwrap();
    let headers = HashMap::new();
    let result = platform.parse_message(payload.as_bytes(), &headers).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_whatsapp_parse_invalid_json() {
    let payload = "invalid json";
    let platform = WhatsAppPlatform::new(test_config()).unwrap();
    let headers = HashMap::new();
    let result = platform.parse_message(payload.as_bytes(), &headers).await;

    assert!(result.is_err());
}
