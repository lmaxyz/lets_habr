use std::rc::Rc;
use std::sync::{Arc, Mutex};

// use log::LevelFilter;
// use simple_logger::SimpleLogger;
use image::{DynamicImage, EncodableLayout};
use slint::{ComponentHandle, Image, ModelRc, SharedPixelBuffer, VecModel, Weak};
use tokio::task::AbortHandle;

mod habr_client;
mod utils;

use habr_client::{article::ArticleContent, hub::get_hubs, HabrClient};

use utils::donwload_img;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    // SimpleLogger::new().with_level(LevelFilter::Debug).init().unwrap();
    let tokio_rt = tokio::runtime::Builder::new_multi_thread()
        .thread_name("slint app tokio runtime")
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    let ui = AppWindow::new().unwrap();
    let habr_client = HabrClient::new();

    let hubs_icons_downloading_handlers: Arc<Mutex<Vec<AbortHandle>>> =
        Arc::new(Mutex::new(Vec::new()));

    ui.on_hubs_requested({
        let ui_weak = ui.as_weak();
        let tokio_rt_handle = tokio_rt.handle().clone();

        move |page: i32| {
            let page = page as u8;
            get_hubs_handler(
                ui_weak.clone(),
                page,
                tokio_rt_handle.clone(),
                hubs_icons_downloading_handlers.clone(),
            )
        }
    });

    ui.on_articles_requested({
        let ui_weak = ui.as_weak();
        let habr_client = habr_client.clone();
        let tokio_rt_handle = tokio_rt.handle().clone();

        move |hub_id, page: i32| {
            let page = page as u8;
            let hub_id = hub_id.to_string();
            let ui_clone = ui_weak.clone();
            let habr_client = habr_client.clone();
            let tokio_rt_handle = tokio_rt_handle.clone();
            let tokio_rt_handle_inner = tokio_rt_handle.clone();

            slint::spawn_local(async move {
                tokio_rt_handle.spawn(async move {
                    if let Ok((articles, pages_count)) =
                        habr_client.get_articles(hub_id.as_str(), page).await
                    {
                        let image_urls: Vec<String> =
                            articles.iter().map(|a| a.image_url.clone()).collect();

                        ui_clone
                            .upgrade_in_event_loop(move |app| {
                                let articles: Vec<ArticlePreviewData> =
                                    articles.into_iter().map(|a| a.into()).collect();
                                let articles_model = Rc::new(VecModel::from(articles));

                                app.set_articles_list(articles_model.into());
                                app.set_articles_pages_count(pages_count as i32);
                                app.invoke_articles_loaded();
                            })
                            .expect("Error with articles list data.");

                        for (idx, url) in image_urls.into_iter().enumerate() {
                            let ui_clone = ui_clone.clone();
                            tokio_rt_handle_inner.spawn(async move {
                                if let Some(image) = donwload_img(url.as_str()).await {
                                    let img_height = image.height();
                                    let img_width = image.width();
                                    let img_buf = SharedPixelBuffer::clone_from_slice(
                                        &image.into_rgba8(),
                                        img_width,
                                        img_height,
                                    );
                                    let _ = ui_clone.upgrade_in_event_loop(move |app| {
                                        app.invoke_article_preview_loaded(
                                            idx as i32,
                                            Image::from_rgba8(img_buf),
                                        )
                                    });
                                }
                            });
                        }
                    }
                });
            })
            .unwrap();
        }
    });

    ui.on_article_details_requested({
        let ui_weak = ui.as_weak();
        let habr_client = habr_client.clone();
        let tokio_rt_handle = tokio_rt.handle().clone();

        move |article_id| {
            let ui_clone = ui_weak.clone();
            let habr_client = habr_client.clone();
            let tokio_rt_handle = tokio_rt_handle.clone();
            let tokio_rt_handle_inner = tokio_rt_handle.clone();

            slint::spawn_local(async move {
                tokio_rt_handle.spawn(async move {
                    if let Ok(article_details) =
                        habr_client.get_article_details(article_id.as_str()).await
                    {
                        let title = article_details.0.into();
                        let mut images = Vec::new();
                        for content in article_details.1.iter() {
                            match content {
                                ArticleContent::Image(img_src) => {
                                    let image_url = img_src.clone();
                                    let mut download_image_handlers = Vec::new();
                                    download_image_handlers.push(tokio_rt_handle_inner.spawn(
                                        async move { donwload_img(image_url.as_str()).await },
                                    ));

                                    for download_res in download_image_handlers.into_iter() {
                                        let image_data = match download_res.await {
                                            Ok(res) => {
                                                res.unwrap_or(DynamicImage::new_rgba8(480, 280))
                                            }
                                            Err(_) => DynamicImage::new_rgba8(480, 280),
                                        };

                                        let img_height = image_data.height();
                                        let img_width = image_data.width();
                                        let img_buf = SharedPixelBuffer::clone_from_slice(
                                            &image_data.into_rgba8(),
                                            img_width,
                                            img_height,
                                        );
                                        images.push(img_buf);
                                    }
                                }
                                ArticleContent::Text(..) => {}
                            };
                        }

                        ui_clone
                            .upgrade_in_event_loop(move |w| {
                                let mut images_iter = images.into_iter();
                                let mut content_data_vec = Vec::new();
                                for content in article_details.1.iter() {
                                    let content_data = match content {
                                        ArticleContent::Image(img_src) => ContentData {
                                            content_type: ContentType::ContentImage,
                                            image_content: ContentImage {
                                                data: Image::from_rgba8(
                                                    images_iter.next().unwrap(),
                                                ),
                                                image_url: img_src.into(),
                                            },
                                            text_content: ContentText {
                                                text: "".into(),
                                                text_type: TextType::Common,
                                            },
                                        },
                                        ArticleContent::Text(text, text_type) => ContentData {
                                            content_type: ContentType::ContentText,
                                            image_content: ContentImage {
                                                data: Image::default(),
                                                image_url: "".into(),
                                            },
                                            text_content: ContentText {
                                                text: text.into(),
                                                text_type: (*text_type).into(),
                                            },
                                        },
                                    };
                                    content_data_vec.push(content_data);
                                }
                                let article_details = ArticleDetailsData {
                                    title,
                                    content: ModelRc::from(Rc::new(VecModel::from(
                                        content_data_vec,
                                    ))),
                                };
                                w.set_article_details(article_details);
                            })
                            .unwrap();
                    };
                })
            })
            .unwrap();
        }
    });

    ui.invoke_hubs_requested(1);
    ui.run()
}

fn get_hubs_handler(
    ui_clone: Weak<AppWindow>,
    page: u8,
    tokio_rt_handle: tokio::runtime::Handle,
    hubs_icons_downloading_handlers: Arc<Mutex<Vec<AbortHandle>>>,
) {
    let tokio_rt_handle_inner = tokio_rt_handle.clone();
    let mut handlers_vec = hubs_icons_downloading_handlers.lock().unwrap();

    for handle in handlers_vec.iter() {
        handle.abort();
    }

    handlers_vec.clear();
    drop(handlers_vec);

    slint::spawn_local(async move {
        tokio_rt_handle
            .spawn(async move {
                if let Ok((hubs, pages_count)) = get_hubs(page).await {
                    let mut handlers_vec = hubs_icons_downloading_handlers.lock().unwrap();
                    for (idx, hub_data) in hubs.iter().enumerate() {
                        let full_url = format!("https:{}", hub_data.image_url);
                        let ui_clone = ui_clone.clone();
                        let handle = tokio_rt_handle_inner.spawn(async move {
                            if let Some(image) = donwload_img(full_url.as_str()).await {
                                let img_width = image.width();
                                let img_height = image.height();
                                let img_bytes = image.into_rgba8();
                                let img_buf = SharedPixelBuffer::clone_from_slice(
                                    &img_bytes.as_bytes(),
                                    img_width,
                                    img_height,
                                );
                                ui_clone
                                    .upgrade_in_event_loop(move |app| {
                                        app.invoke_set_hub_icon(
                                            idx as i32,
                                            Image::from_rgba8(img_buf),
                                        )
                                    })
                                    .unwrap();
                            }
                        });
                        handlers_vec.push(handle.abort_handle());
                    }

                    ui_clone
                        .upgrade_in_event_loop(move |app| {
                            let hubs_data: Vec<HubData> =
                                hubs.into_iter().map(|h| h.into()).collect();
                            let hubs_model = Rc::new(VecModel::from(hubs_data));
                            app.set_hubs_pages_count(pages_count as i32);
                            app.set_hubs_list(hubs_model.into());
                            app.invoke_hubs_loaded();
                        })
                        .expect("Error with hubs list data.");
                } else {
                    println!("Error with receiving hubs.")
                }
            })
            .await
            .unwrap()
    })
    .unwrap();
}
