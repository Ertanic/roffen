use upon::Value;

pub fn render_component(comps: &Value, data: &Value, comp_name: &str) -> Option<String> {
    if let Value::Map(comps) = comps
        && let Some(comp) = comps.get(comp_name)
        && let Value::Map(comp_fields) = comp
        && let Some(html) = comp_fields.get("html")
    {
        let result = render_html(html, data);

        let styles = if let Value::Map(data) = data
            && let (Some(Value::String(col)), Some(Value::String(row))) = (data.get("col"), data.get("row"))
        {
            format!("style=\"grid-row: span {row}; grid-column: span {col}\"")
        }
        else {
            "style=\"grid-row: span 1; grid-column: span 12\"".to_owned()
        };

        let data = if let Value::Map(data) = data {
            data.iter()
                .filter_map(|(k, v)| if let Value::String(v) = v { Some(format!(" data-{k}=\"{v}\"")) } else { None })
                .collect::<String>()
        }
        else {
            Default::default()
        };

        result.map(|html| format!("<div class=\"grid-block\" {styles} {data} data-type=\"{comp_name}\">{html}</div>"))
    }
    else {
        None
    }
}

fn render_html(html: &Value, data: &Value) -> Option<String> {
    if let Value::Map(html) = html {
        if let Some(Value::String(element)) = html.get("element") {
            let attrs = if let Some(attrs) = html.get("attrs")
                && let Value::List(attrs) = attrs
            {
                attrs
                    .iter()
                    .filter_map(|attr| {
                        if let Value::Map(attr) = attr
                            && let (Some(Value::String(key)), Some(Value::String(val))) = (attr.get("name"), attr.get("value"))
                        {
                            Some((parse_binding(key, data), parse_binding(val, data)))
                        }
                        else {
                            None
                        }
                    })
                    .map(|(key, val)| format!(" {key}=\"{val}\""))
                    .collect::<String>()
            }
            else {
                Default::default()
            };

            let children = if let Some(children) = html.get("children")
                && let Value::List(children) = children
            {
                children
                    .iter()
                    .filter_map(|child| {
                        if let Value::Map(child) = child {
                            if let Some(Value::String(ty)) = child.get("type") {
                                match ty.as_str() {
                                    "content" if let Some(Value::String(content)) = child.get("content") => Some(parse_binding(content, data)),
                                    "html" if let Some(html) = child.get("html") => render_html(html, data),
                                    &_ => None,
                                }
                            }
                            else {
                                None
                            }
                        }
                        else {
                            None
                        }
                    })
                    .collect::<String>()
            }
            else {
                Default::default()
            };

            let element = parse_binding(element, data);
            Some(format!("<{element}{attrs}>{children}</{element}>"))
        }
        else if let Some(content) = html.get("content")
            && let Value::String(content) = content
        {
            Some(parse_binding(content, data))
        }
        else {
            None
        }
    }
    else {
        None
    }
}

fn parse_binding(source: &str, data: &Value) -> String {
    if source.contains('#') {
        let pattern = regex::Regex::new(r"#\(?(\w+)\)?").unwrap();
        if let Some(caps) = pattern.captures(source) {
            let rep_pattern = regex::Regex::new(r"(#\(?\w+\)?)").unwrap();
            let mut result = String::new();
            for cap in caps.iter() {
                if let Some(cap) = cap
                    && let Value::Map(val) = data
                    && let Some(val) = val.get(cap.as_str())
                    && let Value::String(str) = val
                {
                    result = rep_pattern.replace(source, str).into();
                }
            }
            result
        }
        else {
            source.to_string()
        }
    }
    else {
        source.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_parse_binding() {
        let content = "some content";
        let data = HashMap::from([("content", content)]);
        let data = upon::to_value(data).unwrap();

        let binding = parse_binding("#content", &data);
        assert_eq!(binding, content);
    }

    #[test]
    fn test_parse_binding_complex() {
        let content = "12";
        let data = HashMap::from([("content", content)]);
        let data = upon::to_value(data).unwrap();

        let binding = parse_binding("h#content", &data);
        assert_eq!(binding, "h".to_owned() + content);
    }

    #[test]
    fn test_parse_binding_complex_postfix() {
        let content = "12";
        let data = HashMap::from([("content", content)]);
        let data = upon::to_value(data).unwrap();

        let binding = parse_binding("h#(content)", &data);
        assert_eq!(binding, "h".to_owned() + content);
    }

    #[test]
    fn test_parse_binding_prefix() {
        let content = "12";
        let data = HashMap::from([("content", content)]);
        let data = upon::to_value(data).unwrap();

        let binding = parse_binding("#(content)_div", &data);
        assert_eq!(binding, content.to_owned() + "_div");
    }
}
