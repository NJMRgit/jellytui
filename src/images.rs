use std::collections::{HashMap, HashSet};

use anyhow::Result;
use image::DynamicImage;
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use tokio::sync::mpsc;

pub struct ImageFetched {
    pub item_id: String,
    pub image: Option<DynamicImage>,
}

enum Entry {
    Loaded(StatefulProtocol),
    Failed,
}

pub struct ImageManager {
    cache: HashMap<String, Entry>,
    pending: HashSet<String>,
    tx: mpsc::UnboundedSender<ImageFetched>,
    picker: Picker,
}

impl ImageManager {
    pub fn new(tx: mpsc::UnboundedSender<ImageFetched>, picker: Picker) -> Self {
        Self {
            cache: HashMap::new(),
            pending: HashSet::new(),
            tx,
            picker,
        }
    }

    pub fn ensure(&mut self, item_id: &str, url: String) {
        if self.cache.contains_key(item_id) || self.pending.contains(item_id) {
            return;
        }
        let id = item_id.to_string();
        self.pending.insert(id.clone());
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let img = fetch_image(&url).await.ok();
            let _ = tx.send(ImageFetched {
                item_id: id,
                image: img,
            });
        });
    }

    pub fn handle_fetched(&mut self, fetched: ImageFetched) {
        self.pending.remove(&fetched.item_id);
        let entry = match fetched.image {
            Some(img) => Entry::Loaded(self.picker.new_resize_protocol(img)),
            None => Entry::Failed,
        };
        self.cache.insert(fetched.item_id, entry);
    }

    pub fn get_protocol(&mut self, item_id: &str) -> Option<&mut StatefulProtocol> {
        match self.cache.get_mut(item_id) {
            Some(Entry::Loaded(protocol)) => Some(protocol),
            _ => None,
        }
    }

    pub fn is_failed(&self, item_id: &str) -> bool {
        matches!(self.cache.get(item_id), Some(Entry::Failed))
    }
}

async fn fetch_image(url: &str) -> Result<DynamicImage> {
    let bytes = reqwest::get(url).await?.error_for_status()?.bytes().await?;
    Ok(image::load_from_memory(&bytes)?)
}
