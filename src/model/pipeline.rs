use mongodb::bson::{DateTime, Document, doc, Bson};

use crate::ResultT;

pub struct Match {
    key: &'static str,
    value: Option<Bson>
}

impl Match {
    pub fn new<T: Into<Bson>>(key: &'static str, value: Option<T>) -> Match {
        if let Some(value) = value {
            Match {
                key,
                value: Some(value.into())
            }
        } else {
            Match {
                key,
                value: None
            }
        }
    }
}

pub struct PipelineBuilder {
    pipeline: Vec<Document>
}

impl PipelineBuilder {
    pub fn new() -> PipelineBuilder {
        PipelineBuilder {
            pipeline: Vec::new()
        }
    }

    pub fn custom(&mut self, doc: Document) -> ResultT<&mut Self> {
        self.pipeline.push(doc);
        Ok(self)
    } 

    pub fn find(&mut self, find: Option<&[Match]>, start: Option<String>, end: Option<String>) -> ResultT<&mut Self> {

        let mut match_doc = doc! {};

        let mut range = doc! {};

        if let Some(time) = start {
            range.insert("$lt", DateTime::parse_rfc3339_str(time)?);
        }

        if let Some(time) = end {
            range.insert("$lt", DateTime::parse_rfc3339_str(time)?);
        }

        if !range.is_empty() {
            match_doc.insert("timestamp", range);
        }

        if let Some(find) = find {
            find.iter().for_each(|m| {
                if let Some(value) = &m.value {
                    match_doc.insert(m.key, value.clone());
                }
            });
        }

        self.pipeline.push(doc! {
            "$match": match_doc
        });

        Ok(self)
    }

    pub fn limit(&mut self, limit: i64) -> ResultT<&mut Self> {
        self.pipeline.push(doc! {
            "$limit": limit
        });
        Ok(self)
    }

    pub fn lookup(&mut self, collection: &str, local: &str, foreign: &str, out: &str) -> ResultT<&mut Self> {
        self.pipeline.push(doc! {
            "$lookup": doc! {
                "from": collection,
                "localField": local,
                "foreignField": foreign,
                "as": out
            }
        });
        Ok(self)
    }

    pub fn replace_field(&mut self, names: &[&str]) -> ResultT<&mut Self> {
        let mut replace_fields = doc! {};

        names.iter().for_each(|name| {
            replace_fields.insert(*name, doc! {"$first": "$".to_owned()+&name});
        });

        let replace_doc = doc! {
            "$addFields": replace_fields
        };

        self.pipeline.push(replace_doc);

        Ok(self)
    }

    pub fn build(&mut self) -> Vec<Document> {
        self.pipeline.clone()
    }
}

