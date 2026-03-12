# slides

[Slides List]({{ project.base_url }})

| Title | Slide | PDF | Description |
| :---- | :---: | :-: | :---------- |

{%- for slide in slides -%}
{% let description = slide
    .description
    .clone()
    .unwrap_or(String::new())|linebreaksbr
%}
{%- if !slide.draft.unwrap_or(false) -%}
{% let stem = slide.secret.as_ref().unwrap_or(slide.name) %}
| {{ slide.name }} | {% if slide.type_.is_marp() %} [Slide]({{ project.base_url }}{{ stem }}) {% else %} - {% endif %} | [PDF]({{ project.base_url }}{{ stem }}.pdf),{% for v in 1..=slide.version %}[v{{v}}]({{ project.base_url }}{{ stem }}_v{{v}}.pdf){% if !loop.last %},{% endif %}{% endfor %} | {{ description }} |
{%- else %}
| {{ slide.name }} | - | - | {{ description }} |
{%- endif %}
{%- endfor %}
