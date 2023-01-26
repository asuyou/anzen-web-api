use crate::{routes::returns::EventCommandN, model::pipeline::Match};

use mongodb::bson::DateTime;
use crate::ResultT;
use anzen_lib::db_types::{self, User};
use argon2::{self, Config};
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Client, Collection,
};
use rocket::futures::TryStreamExt;

mod helpers;
mod pipeline;

pub struct AnzenDB
{
    users: Collection<db_types::User>,
    plugins: Collection<db_types::Plugin>,
    commands: Collection<db_types::Command>,
    events: Collection<db_types::Event>,
}

impl AnzenDB
{
    pub async fn init(uri: String) -> ResultT<AnzenDB>
    {
        let client = Client::with_uri_str(uri).await?;
        let db = client.database("anzen");
        Ok(AnzenDB {
            users: db.collection("users"),
            plugins: db.collection("plugins"),
            commands: db.collection("commands"),
            events: db.collection("events"),
        })
    }

    pub async fn valid_user(&self, username: &String, password: &String) -> ResultT<bool>
    {
        let user = self.users.find_one(doc! {"name": username}, None).await?;
        match user {
            Some(v) => Ok(argon2::verify_encoded(&v.hash, password.as_bytes())?),
            None => Ok(false),
        }
    }

    pub async fn new_user(&self, username: &String, password: &[u8]) -> ResultT<bool>
    {
        let user = self.users.find_one(doc! { "name": username }, None).await?;
        if user.is_some() {
            return Ok(false);
        }

        let config = Config::default();
        let salt = helpers::gen_salt().await;
        let hash = argon2::hash_encoded(password, salt.as_bytes(), &config)?;
        let new_user = db_types::User {
            _id: ObjectId::new(),
            name: username.to_string(),
            salt,
            hash,
        };
        self.users.insert_one(new_user, None).await?;
        Ok(true)
    }

    pub async fn event_statistics(&self) -> ResultT<Vec<Document>>
    {
        let pipeline = [
            doc! {
                "$match": doc! {
                    "metadata.armed": false
                }
            },
            doc! {
                "$project": doc! {
                    "date": doc! {
                        "$dateToParts": doc! {
                            "date": "$timestamp"
                        }
                    },
                    "data": doc! {
                        "$objectToArray": "$data"
                    }
                }
            },
            doc! {
                "$unwind": doc! {
                    "path": "$data"
                }
            },
            doc! {
                "$group": doc! {
                    "_id": doc! {
                        "date": doc! {
                            "year": "$date.year",
                            "month": "$date.month",
                            "day": "$date.day",
                            "hour": "$date.hour"
                        },
                        "data": "$data.k"
                    },
                    "total_occurences": doc! {
                        "$count": doc! {}
                    },
                    "float_avg": doc! {
                        "$avg": "$data.v.float_value"
                    },
                    "int_avg": doc! {
                        "$avg": "$data.v.int_value"
                    },
                    "binary_avg": doc! {
                        "$avg": "$data.v.binary_value"
                    }
                }
            },
        ];

        let data = self.events.aggregate(pipeline, None).await?;

        let vec_docs: Vec<_> = data.try_collect().await?;

        Ok(vec_docs)
    }

    pub async fn count_status_time(
        &self,
        start: Option<String>,
        end: Option<String>,
        armed: Option<bool>,
        device: Option<String>,
        plugin: Option<String>,
    ) -> ResultT<Vec<Document>>
    {
        let pipeline = pipeline::PipelineBuilder::new()
            .find(None, start, end)?
            .custom(
                doc! {
                    "$project": doc! {
                        "date": doc! {
                            "$dateToParts": doc! {
                                "date": "$timestamp"
                            }
                        },
                        "armed": "$metadata.armed"
                    }
                }
            )?
            .custom(
                    doc! {
                        "$group": doc! {
                            "_id": doc! {
                                "date": doc! {
                                    "year": "$date.year",
                                    "month": "$date.month",
                                    "day": "$date.day",
                                    "hour": "$date.hour"
                                },
                                "armed": "$armed"
                            },
                            "count": doc! {
                                "$count": doc! {}
                            }
                        }
                    }
            )?
            .custom(
                doc! {
                    "$project": doc! {
                        "date": doc! {
                            "$dateFromParts": doc! {
                                "year": "$_id.date.year",
                                "month": "$_id.date.month",
                                "day": "$_id.date.day",
                                "hour": "$_id.date.hour"
                            }
                        },
                        "armed": "$armed",
                        "count": "$count"
                    }
                }
            )?
            .build();

        let data = self.events.aggregate(pipeline, None).await?;

        let vec_docs: Vec<_> = data.try_collect().await?;

        Ok(vec_docs)
    }

    pub async fn get_user(&self, id: &String) -> ResultT<User>
    {
        let data = self
            .users
            .find_one(
                doc! {
                    "name": id
                },
                None,
            )
            .await?;

        match data {
            Some(data) => Ok(data),
            _ => Err("no user".into()),
        }
    }

    pub async fn last_n(&self, n: i64) -> ResultT<EventCommandN>
    {
        if n < 0 {
            return Err("Cannot have less than 0 documents".into());
        }

        let event_pipeline = pipeline::PipelineBuilder::new()
            .custom(doc! {
                "$sort": doc! {
                    "timestamp": -1
                }
            })?
            .limit(n)?
            .lookup("plugins", "metadata.plugin_id", "_id", "plugin")?
            .lookup("devices", "metadata.device_id", "_id", "device")?
            .build();

        let command_pipeline = pipeline::PipelineBuilder::new()
            .custom(doc! {
                "$sort": doc! {
                    "timestamp": -1
                }
            })?
            .limit(n)?
            .lookup("plugins", "metadata.plugin_id", "_id", "plugin")?
            .build();

        let event_data = self.events.aggregate(event_pipeline, None).await?;
        let command_data = self.commands.aggregate(command_pipeline, None).await?;

        let vec_events: Vec<_> = event_data.try_collect().await?;
        let vec_commnads: Vec<_> = command_data.try_collect().await?;

        Ok(EventCommandN {
            events: vec_events,
            commands: vec_commnads,
        })
    }

    pub async fn search(
        &self,
        start: Option<String>,
        end: Option<String>,
        armed: Option<bool>,
        device: Option<String>,
        plugin: Option<String>,
    ) ->  ResultT<EventCommandN>
    {
        let event_pipeline = pipeline::PipelineBuilder::new()
            .limit(50)?
            .find(None, start.clone(), end.clone())?
            .lookup("devices", "metadata.device_id", "_id", "device")?
            .lookup("plugin", "metadata.plugin_id", "_id", "plugin")?
            .replace_field(&["device", "plugin"])?
            .find(Some(&[
                Match::new("metadata.armed", armed),
                Match::new("device.id", device),
                Match::new("plugin.name", plugin),
            ]), None, None)?
            .build();

        let command_pipeline = pipeline::PipelineBuilder::new()
            .limit(50)?
            .find(None, start, end)?
            .lookup("plugin", "metadata.plugin_id", "_id", "plugin")?
            .replace_field(&["plugin"])?
            .build();
        

        let event_data = self.events.aggregate(event_pipeline, None).await?;
        let command_data = self.commands.aggregate(command_pipeline, None).await?;

        let vec_events: Vec<_> = event_data.try_collect().await?;
        let vec_commnads: Vec<_> = command_data.try_collect().await?;

        Ok(EventCommandN {
            events: vec_events,
            commands: vec_commnads,
        })
    }
}
