use reqwest::Error;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Default)]
struct RobotsTxtRules {
    user_agents: HashMap<String, HashMap<String, Vec<String>>>,
    default_allow: bool,
}
#[derive(Debug, Clone)]
enum RuleType {
    Allow,
    Disallow,
}

#[derive(Debug, Clone)]
struct Rule {
    user_agent: String,
    path: String,
    rule_type: RuleType,
}

#[derive(Debug, Clone)]
pub struct RobotsTxt {
    rules: Vec<Rule>,
    crawl_delay: Option<u64>,
}
async fn fetch_robots_txt(url: &str) -> Result<String, Error> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let robots_txt_url = format!("{}/robots.txt", url);
    let response = client.get(&robots_txt_url).send().await?;

    if response.status().is_success() {
        let content = response.text().await?;
        Ok(content)
    } else {
        Ok(String::new())
    }
}
fn parse_robots_txt(content: &str) -> RobotsTxtRules {
    let mut rules = RobotsTxtRules::default();
    let mut current_user_agent = "default".to_string();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() != 2 {
            continue;
        }

        let key = parts[0].trim().to_lowercase();
        let value = parts[1].trim();

        match key.as_str() {
            "user-agent" => {
                current_user_agent = value.to_string();
                rules
                    .user_agents
                    .entry(current_user_agent.clone())
                    .and_modify(|e| {
                        if !e.contains_key("allow") {
                            e.insert("allow".to_string(), Vec::new());
                        }
                        if !e.contains_key("disallow") {
                            e.insert("disallow".to_string(), Vec::new());
                        }
                    })
                    .or_insert({
                        let mut new_map = HashMap::new();
                        new_map.insert("allow".to_string(), Vec::new());
                        new_map.insert("disallow".to_string(), Vec::new());
                        new_map
                    });
            }
            "allow" => {
                if rules.user_agents.get_mut(&current_user_agent).is_some() {
                    rules
                        .user_agents
                        .get_mut(&current_user_agent)
                        .as_mut()
                        .unwrap()
                        .get_mut("allow")
                        .as_mut()
                        .unwrap()
                        .push(value.to_string());
                }
            }
            "disallow" => {
                if rules.user_agents.get_mut(&current_user_agent).is_some() {
                    rules
                        .user_agents
                        .get_mut(&current_user_agent)
                        .as_mut()
                        .unwrap()
                        .get_mut("disallow")
                        .as_mut()
                        .unwrap()
                        .push(value.to_string());
                }
            }
            _ => continue,
        }
    }

    rules.default_allow = true;
    rules
}
#[test]
fn test_robots_txt_parsing() {
    let robots_txt = r#"
    # This is a comment
    User-agent: *
    Allow: /
    Disallow: /private/

    User-agent: MyCrawler
    Allow: /public/
    Disallow: /public/private/
    "#;

    let rules = parse_robots_txt(robots_txt);
    assert_eq!(rules.user_agents.keys().count(), 2);
    assert_eq!(
        rules.user_agents.get("MyCrawler").unwrap()["allow"].len(),
        1
    );
    assert_eq!(
        rules.user_agents.get("MyCrawler").unwrap()["disallow"].len(),
        1
    );
}
