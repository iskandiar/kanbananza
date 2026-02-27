use serde::Deserialize;

const BASE: &str = "https://api.notion.com/v1";
const NOTION_VERSION: &str = "2022-06-28";

// ---------------------------------------------------------------------------
// Response shapes
// ---------------------------------------------------------------------------

/// A Notion page returned by `GET /v1/pages/{page_id}`.
pub struct NotionPage {
    pub id: String,
    pub title: String,
    pub last_edited_time: String,
}

// Internal deserialization types — not part of the public API surface.

#[derive(Deserialize)]
struct PageResponse {
    id: String,
    last_edited_time: String,
    properties: serde_json::Value,
}

#[derive(Deserialize)]
struct BlocksResponse {
    results: Vec<BlockObject>,
}

#[derive(Deserialize)]
struct BlockObject {
    #[serde(rename = "type")]
    block_type: String,
    paragraph: Option<RichTextContainer>,
    heading_1: Option<RichTextContainer>,
    heading_2: Option<RichTextContainer>,
    heading_3: Option<RichTextContainer>,
    bulleted_list_item: Option<RichTextContainer>,
    numbered_list_item: Option<RichTextContainer>,
}

#[derive(Deserialize)]
struct RichTextContainer {
    rich_text: Vec<RichTextItem>,
}

#[derive(Deserialize)]
struct RichTextItem {
    plain_text: String,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Fetches basic page metadata from `GET /v1/pages/{page_id}`.
///
/// Extracts the page title from `properties.title.title[0].plain_text`.
/// Falls back to `"(untitled)"` when the title property is absent or empty.
pub async fn fetch_page(api_key: &str, page_id: &str) -> Result<NotionPage, String> {
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{BASE}/pages/{page_id}"))
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Notion-Version", NOTION_VERSION)
        .send()
        .await
        .map_err(|e| format!("Notion /pages/{page_id} fetch failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Notion API error {status}: {body}"));
    }

    let page: PageResponse = resp
        .json()
        .await
        .map_err(|e| format!("failed to parse Notion page response: {e}"))?;

    // Extract title: properties.title.title[0].plain_text
    let title = page
        .properties
        .get("title")
        .and_then(|p| p.get("title"))
        .and_then(|arr| arr.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("plain_text"))
        .and_then(|t| t.as_str())
        .unwrap_or("(untitled)")
        .to_string();

    Ok(NotionPage {
        id: page.id,
        title,
        last_edited_time: page.last_edited_time,
    })
}

/// Fetches block children from `GET /v1/blocks/{page_id}/children` and
/// concatenates the plain text of all paragraph, heading, bulleted list, and
/// numbered list blocks into a single string separated by newlines.
pub async fn fetch_blocks(api_key: &str, page_id: &str) -> Result<String, String> {
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{BASE}/blocks/{page_id}/children"))
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Notion-Version", NOTION_VERSION)
        .send()
        .await
        .map_err(|e| format!("Notion /blocks/{page_id}/children fetch failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Notion API error {status}: {body}"));
    }

    let blocks: BlocksResponse = resp
        .json()
        .await
        .map_err(|e| format!("failed to parse Notion blocks response: {e}"))?;

    let text: Vec<String> = blocks
        .results
        .iter()
        .filter_map(|block| {
            let container = match block.block_type.as_str() {
                "paragraph" => block.paragraph.as_ref(),
                "heading_1" => block.heading_1.as_ref(),
                "heading_2" => block.heading_2.as_ref(),
                "heading_3" => block.heading_3.as_ref(),
                "bulleted_list_item" => block.bulleted_list_item.as_ref(),
                "numbered_list_item" => block.numbered_list_item.as_ref(),
                _ => None,
            }?;
            let plain: String = container
                .rich_text
                .iter()
                .map(|rt| rt.plain_text.as_str())
                .collect::<Vec<_>>()
                .join("");
            if plain.is_empty() { None } else { Some(plain) }
        })
        .collect();

    Ok(text.join("\n"))
}
