use handlebars::handlebars_helper;
use serde_json::Value;

handlebars_helper!(is_equal_helper: |v1: Value,v2: Value| v1==v2);
