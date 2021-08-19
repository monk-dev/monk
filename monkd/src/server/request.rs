use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use url::Url;

use crate::metadata::{offline_store::OfflineData, Meta};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Request {
    // POST a Meta to /article/<uuid>/
    // Add {
    //     name: Option<String>,
    //     url: Option<Url>,
    //     comment: Option<String>,
    //     tags: Vec<String>,
    // },

    // POST a Meta to /article/<uuid>/
    // Edit {
    //     id: String,
    //     edit: Edit,
    // },

    // POST empty data to /delete/<uuid>/
    // Delete {
    //     id: String,
    // },

    // GET /article/<uuid>/  returns json encoded Meta
    // Get {
    //     id: String,
    // },

    // POST number of wanted articles to  /articles/
    // List {
    //     count: Option<usize>,
    //     tags: Vec<String>,
    // },

    // GET /article/download/<uuid>/ return OK if download starts properly
    // Download {
    //     id: Option<String>,
    // },

    // GET /articles/<uuid>/serve/  Serve an offline doc, or a redirect to content
    // Open {
    //     id: String,
    //     online: bool, //Get rid of this
    // },

    // POST /search/ Returns a json Vec<(Meta, SnippetDef)>
    // Search {
    //     count: Option<usize>,
    //     query: String,
    // },

    // GET /index/<uuid>/
    // Index {
    //     id: String,
    // },

    // GET /index
    // IndexAll {
    //     tags: Vec<String>,
    // },

    // POST a list of Metas to /import/
    // Import {
    //     metas: Meta,
    // },

    // GET /export/ Return a Json with all of monks Meta data
    // Export,

    // Send a file to the server some how, IDK how uploading stuff
    // works in HTTTP
    // ImportFile {
    //     file: String,
    //     deep_copy: bool,
    // },

    // GET /export-file/ sends a zip with all of monk's data inside
    // ExportFile {
    //     // File to store export
    //     file: PathBuf,
    //     // Export local copies of article as well as metadata
    //     deep_copy: bool,
    // },

    // GET /status/<uuid>/ Returns a JSON:
    // {
    //     Downloaded : (true | false),
    //     Indexed : (true | false),
    //     Size: usize # size on disk to used to store article
    // }
    // IndexStatus {
    //     id: String,
    // },

    // POST of some type of auth to /gtfo/
    // ForceShutdown,

    // POST of some type of auth to /stop/
    // Stop,

    //IDK
    #[serde(skip)]
    UpdateOffline(OfflineData),
    #[serde(skip)]
    UpdateMeta(Meta),
}
