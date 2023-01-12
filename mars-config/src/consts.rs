/// transform request payload before sending the request
/// request payload has to be json for this to work
/// value has to be json_array (transforomation spec of jolt) for example
/*
    "jolt_request_transform" = [{
        "operation": "default",
        "spec": {
            "before_request": true
        }
    }]
*/
pub const TRANSFORM_JSON_TO_JSON_JOLT_REQUEST: &str = "jolt_request_transform";

/// transform response payload after response is recieved
/// response payload has to be json for this to work
/// value has to be json_array (transforomation spec of jolt) for example
/*
    "jolt_response_transform" = [{
        "operation": "default",
        "spec": {
            "before_request": true
        }
    }]
*/

pub const TRANSFORM_JSON_TO_JSON_JOLT_RESPONSE: &str = "jolt_response_transform";

/// transform request payload from json to yaml
/// example
/*
    "request_json_to_yaml" : true
*/
pub const TRANSFORM_JSON_YAML_REQUEST: &str = "request_json_to_yaml";

/// transform response payload from json to yaml
/// example
/*
    "response_json_to_yaml" : true
*/
pub const TRANSFORM_JSON_TO_YAML_RESPONSE: &str = "response_json_to_yaml";

/// transform request payload from xml to json
/// example
/*
    "request_xml_to_json" : true
*/
pub const TRANSFORM_XML_JSON_REQUEST: &str = "request_xml_to_json";

/// transform response payload from xml to json
/// example
/*
    "response_xml_to_json" : true
*/
pub const TRANSFORM_XML_TO_JSON_RESPONSE: &str = "response_xml_to_json";

/// transform request payload from xml to json
/// example
/*
    "request_yaml_to_json" : true
*/
pub const TRANSFORM_YAML_JSON_REQUEST: &str = "request_yaml_to_json";

/// transform response payload from xml to json
/// example
/*
    "response_yaml_to_json" : true
*/

pub const TRANSFORM_YAML_TO_JSON_RESPONSE: &str = "response_yaml_to_json";

/// request header used for auth token
pub const AVALANCHE_TOKEN: &str = "avalanche-token";

/// max number of requests allowed per second for that service
/// "concurrency_limit": 10
pub const CONCURRENCY_LIMIT: &str = "concurrency_limit";

/// max number of requests allowed per second for that service
/// "rate_limit": 100
pub const RATE_LIMIT: &str = "rate_limit";

/// timeout of requests
/// "timeout": 10
pub const TIMEOUT: &str = "timeout";

/// avalanche trace
///
pub const AVALANCHE_TRACE: &str = "avalanche-trace";
