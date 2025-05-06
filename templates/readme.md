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
| {{ slide.name }} | [Slide]({{ project.base_url }}{{ stem }}) | [PDF]({{ project.base_url }}{{ stem }}.pdf) | {{ description }} |
{%- else %}
| {{ slide.name }} | - | - | {{ description }} |
{%- endif %}
{%- endfor %}
