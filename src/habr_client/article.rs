use std::{collections::HashMap, rc::Rc};

use image::DynamicImage;
use serde::{Deserialize, Serialize};
use serde_json;
use slint::{Image, SharedPixelBuffer, SharedString, VecModel};

use crate::{ArticlePreviewData, TextType as SlintTextType};

#[derive(Serialize, Deserialize, Debug)]
pub struct LeadData {
    #[serde(alias = "textHtml")]
    pub description: String,
    #[serde(alias = "imageUrl")]
    pub image_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tag {
    #[serde(alias = "titleHtml")]
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Author {
    pub(crate) id: String,
    pub(crate) alias: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArticlePreviewResponse {
    pub id: String,
    #[serde(alias = "timePublished")]
    pub published_at: String,
    #[serde(alias = "titleHtml")]
    pub title: String,
    #[serde(rename(deserialize = "leadData"))]
    pub lead_data: LeadData,
    pub(crate) tags: Vec<Tag>,
    pub(crate) complexity: Option<String>,
    #[serde(alias = "readingTime")]
    pub(crate) reading_time: usize,
    pub(crate) author: Option<Author>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArticlesResponse {
    #[serde(rename(deserialize = "pagesCount"))]
    pub(crate) pages_count: usize,
    #[serde(rename(deserialize = "publicationIds"))]
    pub(crate) article_ids: Vec<serde_json::Value>,
    #[serde(rename(deserialize = "publicationRefs"))]
    pub(crate) articles: HashMap<String, ArticlePreviewResponse>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ArticleResponse {
    #[serde(alias = "titleHtml")]
    pub title: String,
    #[serde(alias = "textHtml")]
    pub text: String,
}

pub struct ArticleData {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) tags: Vec<String>,
    pub(crate) complexity: String,
    pub(crate) author: String,
    pub(crate) published_at: String,
    pub(crate) reading_time: usize,
    pub image_url: String,
    pub(crate) image: DynamicImage,
}

impl Into<ArticlePreviewData> for ArticleData {
    fn into(self) -> ArticlePreviewData {
        let img_buf = SharedPixelBuffer::clone_from_slice(
            self.image.as_bytes(),
            self.image.width(),
            self.image.height(),
        );
        let image = Image::from_rgb8(img_buf);

        let tags = VecModel::from(
            self.tags
                .iter()
                .map(|t| t.into())
                .collect::<Vec<SharedString>>(),
        );

        ArticlePreviewData {
            id: self.id.into(),
            title: self.title.into(),
            author: self.author.into(),
            published_at: self.published_at.into(),
            reading_time: self.reading_time as i32,
            tags: Rc::new(tags).into(),
            complexity: self.complexity.into(),
            img_url: self.image_url.into(),
            image,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TextType {
    Header(u8),
    Common,
}

impl Into<SlintTextType> for TextType {
    fn into(self) -> SlintTextType {
        match self {
            Self::Common => SlintTextType::Common,
            Self::Header(1) => SlintTextType::H1,
            Self::Header(2) => SlintTextType::H2,
            Self::Header(3) => SlintTextType::H3,
            Self::Header(4) => SlintTextType::H4,
            _ => SlintTextType::Common,
        }
    }
}

pub enum ArticleContent {
    Image(String),
    Text(String, TextType),
}
