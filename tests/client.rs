use panasonic_kairos::http_async::Client;
use panasonic_kairos::{AuxPatch, Credentials, HttpConfig, LayerPatch};
use pretty_assertions::assert_eq;
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn config(server: &MockServer) -> HttpConfig {
    let url = server.uri();
    // wiremock URIs already include scheme + port
    HttpConfig::new(url).with_credentials(Credentials::password("secret"))
}

#[tokio::test]
async fn list_inputs_parses_spec_payload() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/inputs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "index": 0,
                "name": "IP1",
                "tally": 1,
                "uuid": "e53210f7-2235-5ae3-9c02-4f58d67bf8b8"
            }
        ])))
        .mount(&server)
        .await;

    let client = Client::connect(config(&server)).unwrap();
    let inputs = client.list_inputs().await.unwrap();
    assert_eq!(inputs.len(), 1);
    assert_eq!(inputs[0].name, "IP1");
}

#[tokio::test]
async fn get_scene_accepts_bare_object_or_array() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/scenes/Main"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([{
            "actions": [],
            "layers": [],
            "macros": [],
            "name": "Main",
            "path": "",
            "snapshots": [],
            "tally": 3,
            "uuid": "7939ed36-beae-5047-8ba4-0d17f4d859ce"
        }])))
        .mount(&server)
        .await;

    let client = Client::connect(config(&server)).unwrap();
    let scene = client.get_scene("Main").await.unwrap();
    assert_eq!(scene.name, "Main");
    assert_eq!(scene.tally, 3);
}

#[tokio::test]
async fn patch_aux_sends_merge_patch_json() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/aux/0"))
        .and(header("content-type", "application/merge-patch+json"))
        .and(body_json(json!({"source": "Black"})))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = Client::connect(config(&server)).unwrap();
    client
        .patch_aux(0, &AuxPatch::source("Black"))
        .await
        .unwrap();
}

#[tokio::test]
async fn patch_layer_encodes_source_a() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/scenes/Main/Background"))
        .and(body_json(json!({"sourceA": "Black"})))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = Client::connect(config(&server)).unwrap();
    client
        .patch_layer("Main", "Background", &LayerPatch::source_a("Black"))
        .await
        .unwrap();
}

#[tokio::test]
async fn trailing_slash_is_not_emitted() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/macros/GM-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "color": "rgb(255,255,255)",
            "name": "GM-1",
            "path": "",
            "state": null,
            "uuid": "433a501a-e9c2-531a-a141-149855589d17"
        })))
        .mount(&server)
        .await;

    let client = Client::connect(config(&server)).unwrap();
    let macro_obj = client.get_macro("GM-1").await.unwrap();
    assert_eq!(macro_obj.name, "GM-1");
}

#[tokio::test]
async fn http_400_is_surfaced() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/aux/0"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "code": 400,
            "text": "Bad Request. Invalid json format"
        })))
        .mount(&server)
        .await;

    let client = Client::connect(config(&server)).unwrap();
    let err = client
        .set_aux_source(0, "Black")
        .await
        .expect_err("400 should fail");
    match err {
        panasonic_kairos::Error::Http { status, .. } => assert_eq!(status, 400),
        other => panic!("unexpected error: {other}"),
    }
}
