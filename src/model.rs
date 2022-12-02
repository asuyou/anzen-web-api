use anzen_lib::db_types;
use mongodb::{Collection, Client, bson::{doc, oid::ObjectId, Document}};
use argon2::{self, Config};
use rocket::futures::TryStreamExt;
use crate::ResultT;
use crate::routes::returns::*;

mod helpers;

pub struct AnzenDB {
    users: Collection<db_types::User>,
    plugins: Collection<db_types::Plugin>,
    commands: Collection<db_types::Command>,
    events: Collection<db_types::Event>
}

impl AnzenDB {
    pub async fn init(uri: String) -> ResultT<AnzenDB>
    {
        let client = Client::with_uri_str(uri).await?;
        let db = client.database("anzen");
        Ok(AnzenDB {
            users: db.collection("users"),
            plugins: db.collection("plugins"),
            commands: db.collection("commands"),
            events: db.collection("events")
        })
    }

    pub async fn valid_user(&self, username: &String, password: &String) -> ResultT<bool>
    {
        let user = self.users.find_one(doc! {"name": username}, None).await?;
        match user {
            Some(v) => Ok(argon2::verify_encoded(&v.hash, password.as_bytes())?),
            None => Ok(false)
        }
    }

    pub async fn new_user(&self, username: &String, password: &[u8]) -> ResultT<bool>
    {
        let user = self.users.find_one(doc! { "name": username }, None).await?;
        if user.is_some() {
            return Ok(false)
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

    pub async fn event_statistics(&self) -> ResultT<Vec<Document>> {
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
            }
        ];

        let data = self.events.aggregate(pipeline, None).await?;
        
        let vec_docs: Vec<_> = data.try_collect().await?;

        Ok(vec_docs)
    }

    pub async fn count_status_time(&self) -> ResultT<Vec<Document>> {
        let pipeline = [
            doc! {
                "$project": doc! {
                    "date": doc! {
                        "$dateToParts": doc! {
                            "date": "$timestamp"
                        }
                    },
                    "armed": "$metadata.armed"
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
                        "armed": "$armed"
                    },
                    "count": doc! {
                        "$count": doc! {}
                    }
                }
            },
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
        ];

        let data = self.events.aggregate(pipeline, None).await?;
        
        let vec_docs: Vec<_> = data.try_collect().await?;

        Ok(vec_docs)
    }
}

