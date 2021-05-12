{% set lines = body | split(pat="\n") -%}
{% for line in lines -%}
{% if not line is starting_with("# ") -%}
{{line | replace(from="```rust,ignore", to="```rust")}}
{% endif -%}
{% endfor -%}
