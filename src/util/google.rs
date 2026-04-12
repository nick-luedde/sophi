use oauth2::url::Url;
use oauth2::{
    AccessToken, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    RedirectUrl, RefreshToken, RevocationUrl, Scope, TokenUrl,
};
use oauth2::{StandardRevocableToken, TokenResponse, basic::BasicClient};
use oauth2::{StandardTokenResponse, reqwest};

use serde_json::{self, Value, json};
use std::collections::HashMap;

// use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::Path;
use std::{env, fs};

use dialoguer::{Select, theme::ColorfulTheme};

use std::time::SystemTime;

use keyring::{Entry};

struct AppsScriptOAuthToken {
    access_token: String,
    refresh_token: Option<String>,
    expiration: usize,
}

fn get_token_keyring_entry() -> Entry {
    Entry::new("sophi_gapi_oauth", "default_account").expect("Could not get keyring Entry!")
}

fn save_token(access_token: String, refresh_token: String, expires_in: u64) {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let json = json!({
        "access_token": access_token,
        "refresh_token": refresh_token,
        "expires_in": expires_in,
        "expiration": now.as_secs() + expires_in
    });

    let entry = get_token_keyring_entry();

    entry
        .set_password(&serde_json::to_string(&json).unwrap())
        .expect("Failed to set_password to keyring Entry!");
}

fn clear_token() {
    match get_token_keyring_entry().delete_credential() {
        Ok(_) => (),
        Err(keyring::Error::NoEntry) => (),
        Err(e) => panic!("{:?}", e),
    }
}

fn load_token() -> Option<AppsScriptOAuthToken> {
    let entry = get_token_keyring_entry();

    let content = match entry.get_password() {
        Ok(json) => json,
        Err(keyring::Error::NoEntry) => String::from(""),
        Err(e) => panic!("{:?}", e),
    };

    if content == "" {
        return None;
    }

    let json: Value = serde_json::from_str(&content).unwrap();
    let obj = json.as_object().unwrap();

    let refresh_token = match obj.get("refresh_token").unwrap().as_str() {
        Some(val) => Some(String::from(val)),
        None => None,
    };

    Some(AppsScriptOAuthToken {
        access_token: String::from(obj.get("access_token").unwrap().as_str().unwrap()),
        refresh_token,
        expiration: obj.get("expiration").unwrap().as_u64().unwrap() as usize,
    })
}

fn get_auth_client() -> oauth2::Client<
    oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
    StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
    oauth2::StandardTokenIntrospectionResponse<
        oauth2::EmptyExtraTokenFields,
        oauth2::basic::BasicTokenType,
    >,
    StandardRevocableToken,
    oauth2::StandardErrorResponse<oauth2::RevocationErrorResponseType>,
    oauth2::EndpointSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointSet,
    oauth2::EndpointSet,
> {
    if Path::new(".sophi.env").exists() {
        dotenv::from_filename(".sophi.env").ok();
    } else {
        let exe_path = env::current_exe().unwrap();
        let src_path = exe_path.parent().unwrap();
        dotenv::from_path(&src_path.join(".env")).ok();
    }

    let google_client_id =
        ClientId::new(env::var("CLIENT_ID").expect("Should be a .env -> CLIENT_ID"));
    let google_client_secret =
        ClientSecret::new(env::var("CLIENT_SECRET").expect("Should be a .env -> CLIENT_SECRET"));
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
        .expect("Invalid token endpoint URL");

    // Set up the config for the Google OAuth2 process.
    let client = BasicClient::new(google_client_id)
        .set_client_secret(google_client_secret)
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        // This example will be running its own server at localhost:8080.
        // See below for the server implementation.
        .set_redirect_uri(
            RedirectUrl::new("http://localhost:8080".to_string()).expect("Invalid redirect URL"),
        )
        // Google supports OAuth 2.0 Token Revocation (RFC-7009)
        .set_revocation_url(
            RevocationUrl::new("https://oauth2.googleapis.com/revoke".to_string())
                .expect("Invalid revocation endpoint URL"),
        );

    client
}

pub fn auth() {
    let cached = load_token();

    if cached.is_none() {
        login();
    } else {
        let token = cached.unwrap();

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        if now >= (token.expiration - 1) {
            refresh_token(&token);
        }
    }
}

fn refresh_token(token: &AppsScriptOAuthToken) {
    if token.refresh_token.is_none() {
        login();
    }

    let client = get_auth_client();

    let http_client = reqwest::blocking::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    let old_refresh_token = RefreshToken::new(String::from(token.refresh_token.as_ref().unwrap()));

    let new_token_result = client
        .exchange_refresh_token(&old_refresh_token)
        .request(&http_client)
        .unwrap();

    save_token(
        new_token_result.access_token().secret().to_string(),
        token.refresh_token.as_ref().unwrap().to_string(),
        new_token_result.expires_in().unwrap().as_secs(),
    );
}

pub fn login() {
    // Set up the config for the Google OAuth2 process.
    let client = get_auth_client();

    let http_client = reqwest::blocking::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    // Google supports Proof Key for Code Exchange (PKCE - https://oauth.net/2/pkce/).
    // Create a PKCE code verifier and SHA-256 encode it as a code challenge.
    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, _csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/script.deployments".to_string(),
        ))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/script.projects".to_string(),
        ))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/script.webapp.deploy".to_string(),
        ))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/script.metrics".to_string(),
        ))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/drive.file".to_string(),
        ))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    println!("Open this URL in your browser:\n{authorize_url}\n");

    let (code, _state) = {
        // A very naive implementation of the redirect server.
        let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

        // The server will terminate itself after collecting the first code.
        let Some(mut stream) = listener.incoming().flatten().next() else {
            panic!("listener terminated without accepting a connection");
        };

        let mut reader = BufReader::new(&stream);

        let mut request_line = String::new();
        reader.read_line(&mut request_line).unwrap();

        let redirect_url = request_line.split_whitespace().nth(1).unwrap();
        let url = Url::parse(&("http://localhost".to_string() + redirect_url)).unwrap();

        let code = url
            .query_pairs()
            .find(|(key, _)| key == "code")
            .map(|(_, code)| AuthorizationCode::new(code.into_owned()))
            .unwrap();

        let state = url
            .query_pairs()
            .find(|(key, _)| key == "state")
            .map(|(_, state)| CsrfToken::new(state.into_owned()))
            .unwrap();

        let message = "Authentication successful, you can close this window!";
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
            message.len(),
            message
        );
        stream.write_all(response.as_bytes()).unwrap();

        (code, state)
    };

    // Exchange the code with a token.
    let token_response = client
        .exchange_code(code)
        .set_pkce_verifier(pkce_code_verifier)
        .request(&http_client);

    // Revoke the obtained token
    let token_response = token_response.unwrap();

    save_token(
        token_response.access_token().secret().to_string(),
        token_response.refresh_token().unwrap().secret().to_string(),
        token_response.expires_in().unwrap().as_secs(),
    );
}

pub fn logout() {
    let cacehd = load_token();

    if cacehd.is_none() {
        return;
    }

    let token = cacehd.unwrap();

    let client = get_auth_client();

    let http_client = reqwest::blocking::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    let token_to_revoke: StandardRevocableToken = match token.refresh_token {
        Some(refresh) => RefreshToken::new(refresh).into(),
        None => AccessToken::new(token.access_token).into(),
    };

    client
        .revoke_token(token_to_revoke)
        .unwrap()
        .request(&http_client)
        .expect("Failed to revoke token");

    clear_token();
}

#[derive(Debug)]
pub struct SophiConfig {
    pub drive_url: String,
    pub apps: Vec<SophiConfigApp>,
    pub script: Vec<SophiConfigSrc>,
    pub html: Vec<SophiConfigSrc>,
}

impl SophiConfig {
    pub fn get_app_or_default(&self, app_name: Option<String>) -> &SophiConfigApp {
        self.apps
            .iter()
            .find(|&a| {
                (app_name.is_some() && Some(a.name.to_string()) == app_name)
                    || (app_name.is_none() && a.default)
            })
            .unwrap_or(&self.apps[0])
    }
}

#[derive(Debug)]
pub struct SophiConfigApp {
    pub default: bool,
    pub name: String,
    pub script_id: String,
    pub dev_url: String,
    #[allow(dead_code)]
    pub deployment: HashMap<String, String>,
}

#[derive(Debug)]
pub struct SophiConfigSrc {
    pub src: String,
    pub to: String,
    pub empty: Option<bool>,
    pub html_wrap: Option<String>,
}

pub fn get_sophi_config() -> SophiConfig {
    let content = fs::read_to_string("./sophi.config.json").expect("Missing sophi.config.json");

    let config: Value = serde_json::from_str(&content).unwrap();

    let drive_url = config
        .get("driveUrl")
        .expect("Missing sophi.config 'driveUrl' property")
        .as_str()
        .expect("Invalid sophi.config 'driveUrl' property")
        .to_string();

    let apps: Vec<SophiConfigApp> = config
        .get("apps")
        .expect("Missing sophi.config 'apps' property")
        .as_array()
        .expect("Invalid sophi.config 'apps' property")
        .iter()
        .map(|v| {
            let app_obj = v
                .as_object()
                .expect("Invalid sophi.config 'apps[n]' object");

            let deploy_obj = app_obj
                .get("deployment")
                .expect("Missing sophi.config 'apps[n].deployment' property")
                .as_object()
                .expect("Invalid sophi.config 'apps[n].deployment' property")
                .to_owned();
            let mut deployment: HashMap<String, String> = HashMap::new();

            deploy_obj.iter().for_each(|(k, v)| {
                deployment.insert(
                    k.to_string(),
                    v.as_str()
                        .expect("Invalid sophi.config 'apps[n].deployment[key]' property")
                        .to_string(),
                );
            });

            let dflt = match app_obj.get("default") {
                Some(v) => v.as_bool().unwrap_or(false),
                None => false,
            };

            SophiConfigApp {
                default: dflt,
                name: app_obj
                    .get("name")
                    .expect("Missing sophi.config 'apps[n].name' property")
                    .as_str()
                    .expect("Invalid sophi.config 'apps[n].name' property")
                    .to_string(),
                script_id: app_obj
                    .get("scriptId")
                    .expect("Missing sophi.config 'apps[n].scriptId' property")
                    .as_str()
                    .expect("Invalid sophi.config 'apps[n].scriptId' property")
                    .to_string(),
                dev_url: app_obj
                    .get("devUrl")
                    .expect("Missing sophi.config 'apps[n].devUrl' property")
                    .as_str()
                    .expect("Invalid sophi.config 'apps[n].devUrl' property")
                    .to_string(),
                deployment,
            }
        })
        .collect();

    let script: Vec<SophiConfigSrc> = config
        .get("script")
        .expect("Missing sophi.config 'script' property")
        .as_array()
        .expect("Invalid sophi.config 'script' property")
        .to_vec()
        .iter()
        .map(|v| {
            let obj = v
                .as_object()
                .expect("Invalid sophi.config 'script[n]' property");
            SophiConfigSrc {
                src: obj
                    .get("src")
                    .expect("Missing sophi.config 'script[n].src' property")
                    .as_str()
                    .expect("Invalid sophi.config 'script[n].src' property")
                    .to_string(),
                to: obj
                    .get("to")
                    .expect("Missing sophi.config 'script[n].to' property")
                    .as_str()
                    .expect("Invalid sophi.config 'script[n].to' property")
                    .to_string(),
                empty: match obj.get("empty") {
                    Some(v) => Some(
                        v.as_bool()
                            .expect("Invalid sophi.config 'script[n].empty' property"),
                    ),
                    None => None,
                },
                html_wrap: None,
            }
        })
        .collect();
    let html: Vec<SophiConfigSrc> = config
        .get("html")
        .expect("Missing sophi.config 'html' property")
        .as_array()
        .expect("Invalid sophi.config 'html' property")
        .to_vec()
        .iter()
        .map(|v| {
            let obj = v
                .as_object()
                .expect("Invalid sophi.config 'html[n]' property");
            SophiConfigSrc {
                src: obj
                    .get("src")
                    .expect("Missing sophi.config 'html[n].src' property")
                    .as_str()
                    .expect("Invalid sophi.config 'html[n].src' property")
                    .to_string(),
                to: obj
                    .get("to")
                    .expect("Missing sophi.config 'html[n].to' property")
                    .as_str()
                    .expect("Invalid sophi.config 'html[n].to' property")
                    .to_string(),
                empty: None,
                html_wrap: match obj.get("htmlWrap") {
                    Some(wrap) => Some(
                        wrap.as_str()
                            .expect("Invalid sophi.config 'html[n].htmlWrap' property")
                            .to_string(),
                    ),
                    None => None,
                },
            }
        })
        .collect();

    SophiConfig {
        apps,
        drive_url,
        script,
        html,
    }
}

#[derive(Debug)]
struct AppsScriptFile {
    name: String,
    r#type: String,
    source: String,
}

pub fn push(app_name: Option<String>) {
    let config = get_sophi_config();

    let app: &SophiConfigApp = config.get_app_or_default(app_name);

    let url = format!(
        "https://script.googleapis.com/v1/projects/{}/content",
        app.script_id
    );

    let scripts = config.script.iter().map(|v| AppsScriptFile {
        name: v.to.to_string(),
        source: match v.empty {
            Some(empty) => {
                if empty {
                    "".to_string()
                } else {
                    fs::read_to_string(v.src.to_string()).unwrap()
                }
            }
            None => fs::read_to_string(v.src.to_string()).unwrap(),
        },
        r#type: "SERVER_JS".to_string(),
    });

    let htmls = config.html.iter().map(|v| AppsScriptFile {
        name: v.to.to_string(),
        source: match &v.html_wrap {
            Some(wrap) => format!(
                "<{}>\n{}\n</{}>",
                wrap,
                fs::read_to_string(v.src.to_string()).unwrap(),
                wrap
            ),
            None => fs::read_to_string(v.src.to_string()).unwrap(),
        },
        r#type: "HTML".to_string(),
    });

    let manifest = AppsScriptFile {
        name: "appsscript".to_string(),
        r#type: "JSON".to_string(),
        source: fs::read_to_string("./appsscript.json").unwrap(),
    };

    let mut files: Vec<AppsScriptFile> = scripts.collect();
    htmls.for_each(|f| files.push(f));
    files.push(manifest);

    files.sort_by_key(|f| f.name.to_string());

    let files_json: Vec<Value> = files
        .iter()
        .map(|f| {
            json!({
                "name": f.name,
                "type": f.r#type,
                "source": f.source
            })
        })
        .collect();

    let body_json = json!({
        "files": files_json
    });

    let body = body_json.to_string();

    let token = load_token().unwrap();

    let client = reqwest::blocking::Client::new();

    let response = client
        .put(url)
        .header("Authorization", format!("Bearer {}", token.access_token))
        .body(body)
        .send()
        .unwrap();

    match response.error_for_status_ref() {
        Ok(_) => (),
        Err(err) => {
            println!("");
            println!(
                "Status [{}]:\n{}",
                err.status().unwrap(),
                response.text().unwrap()
            );
            panic!("{}", err);
        }
    }
}

// pub fn get_test_deployment() -> String {
//     let config = get_sophi.config();

//     let url = format!(
//         "https://script.googleapis.com/v1/projects/{}/deployments?pageSize=1",
//         config.script_id
//     );

//     let token = load_token().unwrap();

//     let client = reqwest::blocking::Client::new();

//     let response = client
//         .get(url)
//         .header("Authorization", format!("Bearer {}", token.access_token))
//         .send()
//         .unwrap();

//     let json: Value = serde_json::from_str(&response.text().unwrap()).unwrap();
//     let deployments = json["deployments"].as_array().unwrap();
//     let test_deployment = deployments[0].as_object().unwrap();
//     let test_deployment_id = test_deployment["deploymentId"].as_str().unwrap();

//     format!("https://script.google.com/a/macros/state.co.us/s/{test_deployment_id}/dev")
// }

#[allow(dead_code)]
pub fn deploy(app_name: Option<String>, environment: &str) {
    let config = get_sophi_config();

    let app = config.get_app_or_default(app_name);

    let dep_id = &app.deployment[environment];

    let url = format!(
        "https://script.googleapis.com/v1/projects/{}/deployments/{}",
        app.script_id, dep_id
    );

    let body = json!({
      "deploymentConfig": {
        "description": "-- testing deployment from sophi"
      }
    });

    let token = load_token().unwrap();

    let client = reqwest::blocking::Client::new();

    let response = client
        .put(url)
        .header("Authorization", format!("Bearer {}", token.access_token))
        .body(body.to_string())
        .send()
        .unwrap();

    match response.error_for_status_ref() {
        Ok(_) => (),
        Err(err) => {
            println!("");
            println!(
                "Status [{}]:\n{}",
                err.status().unwrap(),
                response.text().unwrap()
            );
            panic!("{}", err);
        }
    }
}

pub fn select_app_from_config(cfg: Option<&SophiConfig>) -> String {
    let config = match cfg {
        Some(c) => c,
        None => &get_sophi_config(),
    };

    let options: Vec<String> = config.apps.iter().map(|a| a.name.to_string()).collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Which app?")
        .default(0)
        .items(&options)
        .interact()
        .unwrap();

    let selected = &options[selection];

    return selected.to_string();
}

#[allow(dead_code)]
pub fn metrics(app_name: Option<String>) -> String {
    let config = get_sophi_config();
    let app: &SophiConfigApp = config.get_app_or_default(app_name);

    let url = format!(
        "https://script.googleapis.com/v1/projects/{}/metrics",
        app.script_id
    );

    let token = load_token().unwrap();

    let client = reqwest::blocking::Client::new();

    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token.access_token))
        .send()
        .unwrap();

    return response.text().unwrap();
}
