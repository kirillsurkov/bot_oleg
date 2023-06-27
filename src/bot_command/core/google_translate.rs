use async_trait::async_trait;

pub struct GoogleTranslate;

pub struct Args<'a> {
    pub to_language: &'a str,
    pub text: &'a str,
}

#[async_trait]
impl<'a> super::Core<Args<'a>, Result<String, String>> for GoogleTranslate {
    async fn execute(args: Args<'a>) -> Result<String, String> {
        use google_translate3::api::TranslateTextRequest;
        use google_translate3::{hyper, hyper_rustls, oauth2, Translate};

        let service_account_key = oauth2::read_service_account_key(
            std::env::var("GOOGLE_SERVICE_ACCOUNT_JSON")
                .expect("Google service account JSON is missing"),
        )
        .await
        .unwrap();
        let project_id = service_account_key.project_id.clone().unwrap();
        let auth = oauth2::ServiceAccountAuthenticator::builder(service_account_key)
            .build()
            .await
            .unwrap();

        let hub = Translate::new(
            hyper::Client::builder().build(
                hyper_rustls::HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .https_or_http()
                    .enable_http1()
                    .enable_http2()
                    .build(),
            ),
            auth,
        );

        let result = hub
            .projects()
            .locations_translate_text(
                TranslateTextRequest {
                    contents: Some(vec![args.text.to_owned()]),
                    target_language_code: Some(args.to_language.to_owned()),
                    ..Default::default()
                },
                &format!("projects/{project_id}"),
            )
            .doit()
            .await;

        match result {
            Ok((_, result)) => match result.translations {
                Some(result) => match result.first() {
                    Some(result) => match result.translated_text.clone() {
                        Some(result) => Ok(result),
                        None => Err("No text was translated".to_owned()),
                    },
                    None => Err("Output is non empty but contains no translations".to_owned()),
                },
                None => Err("Output is empty".to_owned()),
            },
            Err(err) => Err(err.to_string()),
        }
    }
}
