#![expect(unused)]

use opensearch::{
    auth::Credentials,
    http::transport::{BuildError, SingleNodeConnectionPool, TransportBuilder},
    OpenSearch,
};
use serde_json::json;

const BACKEND_URL: &str = "https://search.nixos.org/backend";
const BACKEND_LOGIN: &str = "aWVSALXpZv";

#[derive(Debug)]
pub(crate) struct Nixpkgs {
    client: OpenSearch,
}

impl Nixpkgs {
    pub(crate) fn new() -> Result<Self, ClientInitError> {
        let pool =
            SingleNodeConnectionPool::new(BACKEND_URL.try_into().expect("url should be correct"));
        let transport = TransportBuilder::new(pool)
            .auth(Credentials::Basic(
                BACKEND_LOGIN.to_string(),
                common::NIXPKGS_PASS.to_string(),
            ))
            .build()?;
        Ok(Self {
            client: OpenSearch::new(transport),
        })
    }
    /// Search by name
    pub(crate) async fn search(
        &self,
        name: &str,
        from: i64,
        size: i64,
    ) -> Result<Vec<HitSource>, SearchError> {
        const CHANNEL: &str = "unstable"; // or 24.05
        let channel = format!("latest-42-nixos-{CHANNEL}");
        let resp = self
            .client
            .search(opensearch::SearchParts::Index(&[channel.as_str()]))
            .from(from)
            .size(size)
            .body(make_search_query_body(name))
            .send()
            .await?;

        let resp = resp.json::<Response>().await?;
        let resp = resp.hits.hits;
        if resp.is_empty() {
            return Err(SearchError::Empty);
        }

        Ok(resp.into_iter().map(|h| h.source).collect())
    }
}

/// Query to search packages
// query copied from https://search.nixos.org/packages
fn make_search_query_body(name: &str) -> impl serde::Serialize {
    json!({
        "query": {
            "bool": {
                "filter": [
                    {
                        "term": {
                            "type": {
                                "value": "package",
                                "_name": "filter_packages"
                            }
                        }
                    },
                ],
                "must": [
                    {
                        "dis_max": {
                            "tie_breaker": 0.7,
                            "queries": [
                                {
                                    "multi_match": {
                                        "type": "cross_fields",
                                        "query": name,
                                        "analyzer": "whitespace",
                                        "auto_generate_synonyms_phrase_query": false,
                                        "operator": "and",
                                        "fields": [
                                            "package_attr_name^9",
                                            "package_attr_name.*^5.3999999999999995",
                                            "package_programs^9",
                                            "package_programs.*^5.3999999999999995",
                                            "package_pname^6",
                                            "package_pname.*^3.5999999999999996",
                                            "package_description^1.3",
                                            "package_description.*^0.78",
                                            "package_longDescription^1",
                                            "package_longDescription.*^0.6",
                                            "flake_name^0.5",
                                            "flake_name.*^0.3"
                                        ]
                                    }
                                },
                                {
                                    "wildcard": {
                                        "package_attr_name": {
                                            "value": format!("*{name}*"),
                                            "case_insensitive": true
                                        }
                                    }
                                },
                            ],
                        },
                    },
                ],
            },
        }
    })
}

/// Query to get package by exact `package_attr_name`
fn make_exact_query_body(name: &str) -> impl serde::Serialize {
    json!({
        "query": {
            "match": {
                "package_attr_name": name,
            },
        },
    })
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ClientInitError {
    #[error("failed to init opensearch client: {0}")]
    Build(#[from] BuildError),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum SearchError {
    #[error("failed to search: {0}")]
    Search(#[from] opensearch::Error),
    #[error("empty response")]
    Empty,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct Response {
    hits: ResponseHits,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ResponseHits {
    hits: Vec<Hit>,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct Hit {
    #[serde(rename = "_source")]
    source: HitSource,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct HitSource {
    // package_attr_name == package_attr_set + . + package_pname
    #[serde(rename = "package_attr_name")]
    name: String,
    #[serde(rename = "package_pversion")]
    version: String,
}
