use anyhow::{bail, Context};
use async_trait::async_trait;

pub struct GoogleTranslate;

pub struct Args<'a> {
    pub to_language: &'a str,
    pub text: &'a str,
    pub translator: &'a crate::Translator,
    pub settings: &'a crate::Settings,
}

#[async_trait]
impl<'a> super::Core<Args<'a>, anyhow::Result<String>> for GoogleTranslate {
    async fn execute(args: Args<'a>) -> anyhow::Result<String> {
        use google_translate3::api::TranslateTextRequest;
        use google_translate3::oauth2;

        let google_account = &args.settings.google_service_account_json;
        let service_account_key =
            oauth2::read_service_account_key(format!("./res/{google_account}",))
                .await
                .unwrap();
        let project_id = service_account_key.project_id.clone().unwrap();

        let (_, result) = args
            .translator
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
            .await?;

        let mut translations = result.translations.context("output is empty")?;
        if translations.is_empty() {
            bail!("output is non empty but contains no translations");
        }
        translations
            .swap_remove(0)
            .translated_text
            .context("no text was translated")
    }
}
