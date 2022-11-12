use mime::Mime;
use monk_types::{Blob, ExtractedInfo, Extractor, Item};
use pdf_extract::extract_text;
use tracing::info;

#[derive(Default)]
pub struct MonkExtractor;

impl MonkExtractor {
    async fn extract_from_html(&self, blob: &Blob) -> anyhow::Result<ExtractedInfo> {
        use scraper::{Html, Selector};

        info!("extracting from html");

        let data = tokio::fs::read_to_string(&blob.path).await?;

        let document = Html::parse_document(&data);
        let body_selector = Selector::parse("body").unwrap();

        let mut scraped_body = String::with_capacity(10 * 1024);

        for body in document.select(&body_selector) {
            for text in body.text() {
                scraped_body.push_str(text);
            }
        }

        let title_selector = Selector::parse("title").unwrap();
        let scraped_title: Option<String> = document
            .select(&title_selector)
            .next()
            .map(|node| node.text().collect());

        Ok(ExtractedInfo {
            title: scraped_title,
            body: Some(scraped_body),
            extra: None,
        })
    }

    async fn extract_from_pdf(&self, blob: &Blob) -> anyhow::Result<ExtractedInfo> {
        info!("extracting from pdf");

        let path = blob.path.clone();
        let text = tokio::task::spawn_blocking(move || extract_text(path)).await??;

        // Title is sometimes the first line of academic PDFs:
        let title = text.lines().next().map(ToString::to_string);

        Ok(ExtractedInfo {
            title,
            body: Some(text),
            extra: None,
        })
    }

    async fn extract_from_text(&self, blob: &Blob) -> anyhow::Result<ExtractedInfo> {
        info!("extracting from text");

        let data = tokio::fs::read(&blob.path).await?;
        let text = String::from_utf8_lossy(&data).to_string();

        Ok(ExtractedInfo {
            title: None,
            body: Some(text),
            extra: None,
        })
    }
}

#[async_trait::async_trait]
impl Extractor for MonkExtractor {
    async fn extract_info(
        &self,
        _item: &Item,
        blob: Option<&Blob>,
    ) -> anyhow::Result<Option<ExtractedInfo>> {
        let blob = if let Some(blob) = blob {
            blob
        } else {
            return Ok(None);
        };

        let mime: Mime = blob.content_type.parse()?;

        let info = match (mime.type_(), mime.subtype()) {
            (mime::TEXT, mime::HTML) => self.extract_from_html(blob).await?,
            (mime::APPLICATION, mime::PDF) => self.extract_from_pdf(blob).await?,
            (mime::TEXT, _) | (mime::APPLICATION, _) => self.extract_from_text(blob).await?,
            _ => {
                info!("unable to extract info");
                return Ok(None);
            }
        };

        Ok(Some(info))
    }
}
