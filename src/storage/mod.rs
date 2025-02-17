use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type Db = Arc<Mutex<HashMap<String, String>>>;
