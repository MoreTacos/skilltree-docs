use glob::glob;
use std::fs;
use std::path::Path;

fn main() {
    let mut skills: Vec<String> = vec![];
    for path in glob("./packages/*/*.svg").expect("Failed at glob") {
        let mut svg = fs::read_to_string(path.unwrap()).unwrap();

        // Remove all <span> tags
        svg = svg.replace(r"<span>", "");
        svg = svg.replace(r"</span>", "");

        let mut sliced = svg.split(r"<rect");

        sliced.next();

        let sliced: Vec<_> = sliced.collect();

        for slice in sliced {
            let slice = slice.to_string();
            // find skill
            let mut search_domain = slice.to_string().clone();

            // closer to answer 1
            let from = search_domain.find("word-wrap").unwrap();
            search_domain = search_domain[from..].to_string();

            let from2 = search_domain.find(">").unwrap();
            let to = search_domain.find("<").unwrap();

            let skill_exact = search_domain[from2 + 1..to].to_string();

            let skill = skill_exact
                .split_whitespace()
                .collect::<String>()
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase();

            // skip if empty
            if skill == "".to_string() {
                println!("Skipped empty box");
                continue;
            }

            skills.push(skill);
        }
    }

    // now i have a vec of skills

    let client = reqwest::blocking::Client::new();

    for skill in skills {
        let path = format!("./pages/{}.md", &skill);

        if !Path::new(&path).exists() {
            let default = fs::read_to_string("./default.md").unwrap();

            fs::create_dir_all("./pages").unwrap();
            fs::write(&path, &default).unwrap();

            let mut options = comrak::ComrakOptions::default();
            options.render.unsafe_ = true;
            let html = comrak::markdown_to_html(&default, &options);

            let content = r###"{% extends "docs" %}

{% block body %}"###
                .to_string()
                + &html
                + r###"<div class="issue"><a href="https://github.com/MoreTacos/skilltree-docs/tree/master/pages/{{ skill }}.md">Add something to the page?</a></div>"###
                + r###"
{% endblock %}"###;
            let _res = client
                .post(format!("https://gymskilltree.com/insert_page?n={}", skill))
                .body(content)
                .send()
                .expect("bad request");
        }
    }
}
