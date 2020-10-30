use futures::StreamExt;
use qstring;
use serde::{Deserialize, Serialize};
use std::env;
use std::error;
use telegram_bot::*;

#[derive(Debug, Serialize, Deserialize)]
struct SearchifyResults {
    docid: String,
    icon_url: String,
    format: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
struct SearchifyResponse {
    results: Option<Vec<SearchifyResults>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let token = env::var("TELEGRAM_BOT_KEY").expect("TELEGRAM_BOT_KEY not set");
    env::var("IMGPROXY_URL").expect("IMGPROXY_URL not set");

    let api = Api::new(token);
    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        let update = update.unwrap();
        if let UpdateKind::InlineQuery(message) = update.kind {
            // let query = format!(
            //     "(__any:({query}) \
            //     OR title:({query}) \
            //     OR image_file_name:({query}) \
            //     OR tags:({query}) \
            //     OR entry_name:({query}) \
            //     OR category_name:({query}) \
            //     OR entry_type:({query}) \
            //     OR notes:({query}) \
            //     OR format:({query}) \
            //     OR original_content:({query}) \
            //     OR oc:({query}) \
            //     OR source:({query}) \
            //     OR source_url:({query}) \
            //     OR text:({query}))",
            //     query = message.query
            // );

            if message.query != "" {
                let qs_instant = qstring::QString::new(vec![
                    ("query", message.query.clone()),
                    ("fetch", "icon_url,format".to_string()),
                    ("len", "50".to_string()),
                ]);

                let instant_url = format!(
                    "{}?{}",
                    "https://rkgk.api.searchify.com/v1/indexes/kym_production/instantlinks",
                    qs_instant
                );

                let qs_photos = qstring::QString::new(vec![
                    ("q", message.query),
                    ("fetch", "icon_url,format,title".to_string()),
                    ("__type", "Photo".to_string()),
                    ("len", "50".to_string()),
                ]);

                let photos_url = format!(
                    "{}?{}",
                    "https://rkgk.api.searchify.com/v1/indexes/kym_production/search", qs_photos
                );

                let mut images: Vec<InlineQueryResult> = vec![];

                let instant_response: SearchifyResponse =
                    reqwest::get(&instant_url).await?.json().await?;

                let photos_response: SearchifyResponse =
                    reqwest::get(&photos_url).await?.json().await?;

                if let Some(results) = instant_response.results {
                    images.extend(parse_results(results));
                };

                if let Some(results) = photos_response.results {
                    images.extend(parse_results(results));
                };

                let images = images.into_iter().take(50).collect();

                api.send(AnswerInlineQuery::new(message.id, images).cache_time(0))
                    .await?;
            }
        };
    }
    Ok(())
}

fn parse_results(results: Vec<SearchifyResults>) -> Vec<InlineQueryResult> {
    let imgproxy_url = env::var("IMGPROXY_URL").unwrap();

    results
        .iter()
        .filter_map(|i| {
            if let Some(format) = &i.format {
                if format == "gif" || i.icon_url.ends_with(".gif") {
                    None
                } else {
                    let photo_url = format!("{}/{}@jpg", imgproxy_url, i.icon_url.clone());
                    println!("{}", photo_url);
                    Some(InlineQueryResult::InlineQueryResultPhoto(
                        InlineQueryResultPhoto {
                            id: i.docid.clone(),
                            photo_url: photo_url.clone(),
                            thumb_url: photo_url,
                            photo_width: Some(500),
                            photo_height: Some(500),
                            title: None,
                            description: None,
                            caption: None,
                            parse_mode: None,
                            reply_markup: None,
                            input_message_content: None,
                        },
                    ))
                }
            } else {
                if i.icon_url.ends_with(".gif") {
                    None
                } else {
                    let photo_url = format!("{}/{}@jpg", imgproxy_url, i.icon_url.clone());
                    println!("{}", photo_url);

                    Some(InlineQueryResult::InlineQueryResultPhoto(
                        InlineQueryResultPhoto {
                            id: i.docid.clone(),
                            photo_url: photo_url.clone(),
                            thumb_url: photo_url,
                            photo_width: Some(500),
                            photo_height: Some(500),
                            title: None,
                            description: None,
                            caption: None,
                            parse_mode: None,
                            reply_markup: None,
                            input_message_content: None,
                        },
                    ))
                }
            }
        })
        .collect()
}
