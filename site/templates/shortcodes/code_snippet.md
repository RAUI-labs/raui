{% set lines = load_data(path=path, format="plain") | split(pat="\n") -%}
{% set lines_count = lines | length() -%}
{% if not start -%}
    {% set start = 0 -%}
{% else %}
    {% set start = start - 1 %}
{% endif -%}
{% if not end -%}
    {% set end = lines_count -%}
{% endif -%}

{% if not lang -%}
    {% set lang = "rust" -%}
{% endif -%}
```{{lang}}
{{ lines | slice(start=start, end=end) | join(sep="
") | trim_end}}
```
