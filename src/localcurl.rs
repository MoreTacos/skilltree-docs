use std::fs;
use std::path::Path;

fn main() {
    let md_files = fs::read_dir("./pages").unwrap().map(|x| x.unwrap().path().to_str().unwrap().to_string()).filter(|x| x.contains(".md"))
        .collect::<Vec<String>>();
    let mut svg_files = vec![];
    for i in fs::read_dir("./packages").unwrap().filter_map(|x| {
        let path = x.unwrap().path();
        let is_dir = path.is_dir();
        is_dir.then(|| path)
    }) {
        for j in fs::read_dir(i).unwrap().map(|x| x.unwrap().path().to_str().unwrap().to_string()) {
            if j.contains(".svg") {
                svg_files.push(j)
            }
        }
    }

    let client = reqwest::blocking::Client::new();
    md_files.into_iter().for_each(|x| {
        let n = Path::new(&x).file_stem().unwrap().to_str().unwrap().to_string();

        let raw = fs::read_to_string(&x).unwrap();

        let mut options = comrak::ComrakOptions::default();
        options.render.unsafe_ = true;
        let html = comrak::markdown_to_html(&raw, &options);

        let content = r###"{% extends "docs" %}

{% block body %}"###.to_string() + &html + r###"<div class="issue"><a href="https://github.com/MoreTacos/skilltree-docs/tree/master/pages/{{ skill }}.md">Add something to the page?</a></div>"### + r###"
{% endblock %}"###;
        let _res = client.post(format!("http://localhost:8000/insert_page?n={}", n)).body(content).send().expect("bad request");
    });
    svg_files.into_iter().for_each(|x| {
        let p = Path::new(&x)
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        assert_ne!(p, "packages".to_string());
        let n = Path::new(&x)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let content = tabparse(x);
        let _res = client
            .post(format!(
                "http://localhost:8000/insert_tree?p={}&n={}",
                p, n
            ))
            .body(content)
            .send()
            .expect("bad request");
    })
}

fn tabparse(path: String) -> String {
    let mut svg = fs::read_to_string(&path).unwrap();

    // Remove all <span> tags
    svg = svg.replace(r"<span>", "");
    svg = svg.replace(r"</span>", "");

    let mut sliced = svg.split(r"<rect");

    // Removing the first slice, which is irrelevant

    let mut svg = r###"{% extends "user" %}
{% block tree %}
"###
    .to_string()
        + sliced.next().unwrap();

    let sliced: Vec<_> = sliced.collect();

    for slice in sliced {
        let mut slice = slice.to_string();
        if slice.contains("span") {
            println!("Element containing span might not be displayed");
        }

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

        let skill_exact_correct = slice.to_string().clone()[from..from + to].to_string();

        // skip if empty
        if skill == "".to_string() {
            println!("Skipped empty box");
            continue;
        }

        let color = "{% if skills.".to_string()
            + &skill
            + "%}{{ skills."
            + &skill
            + "}}{% else %}0{% endif %}";

        // input slider
        let onchange = format!(
            r###"fetch(`/update?u={{{{ userhash }}}}&s={}&v=${{this.value}}`, {{ method: 'PUT' }})"###,
            &skill
        );
        let oninput = r###"this.closest('g').previousElementSibling.style.fill = `hsl(${this.value}, 50%, 50%)`"###;
        let mut skill_exact_correct_with_input = skill_exact_correct.clone()
            + r###"<input type="range" min="0" max="100" value=""###
            + &color
            + r###"" onchange=""###
            + &onchange
            + r###"" oninput=""###
            + &oninput
            + r###"">"###;
        skill_exact_correct_with_input = skill_exact_correct_with_input.replace(
            &skill_exact,
            &format!(
                r###"<p><a href="/skill?s={}">{}</a></p>"###,
                &skill, &skill_exact
            ),
        );
        slice = slice.replace(&skill_exact_correct, &skill_exact_correct_with_input);

        // Skill value finder and remove (A) | (B) | (C) etc...

        let mut skillvalue: String = "".to_string();

        for c in "ABCDEFGHIabcdefghi".chars() {
            let search = format!("({})", &c);
            if slice.contains(&search) {
                skillvalue = c.to_string();
                slice = slice.replace(&search, "");
            }
        }

        svg =
            svg + r###"<rect fill="hsl("### + &color + r###", 50%, 50%)" class="skill""### + &slice;
    }

    svg = svg
        + r###"
{% endblock %}"###;

    svg = svg.replace(r"<br>", "");
    svg.to_string()
}
