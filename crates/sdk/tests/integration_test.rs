use pushover_sdk::{Message, PushOverClient};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer,
};

#[tokio::test]
async fn test_send_message_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/1/messages.json"))
        .respond_with(
            wiremock::ResponseTemplate::new(200)
                .set_body_raw(r#"{"status":1,"request":"test-id"}"#, "application/json"),
        )
        .mount(&mock_server)
        .await;

    // MockServer URL을 사용하는 클라이언트 생성
    let client = PushOverClient::with_base_url(
        "test_user".into(),
        "test_token".into(),
        mock_server.uri(),
    );

    let msg = Message {
        message: "Hello".to_string(),
        title: Some("Test".to_string()),
        priority: Some(0),
        sound: Some("pushover".to_string()),
        device: None,
        url: None,
        url_title: None,
        priority_arg: None,
        html: None,
        timestamp: None,
    };

    let result = client.send(msg).await.unwrap();
    assert_eq!(result.status, 1);
    assert_eq!(result.request, "test-id");
}
