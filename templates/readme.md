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
| {{ slide.name }} | [Slide]({{ project.base_url }}{{ slide.name }}) | [PDF]({{ project.base_url }}{{ slide.name }}_v{{ slide.version }}.pdf) | {{ description }} |
{%- else %}
| {{ slide.name }} | - | - | {{ description }} |
{%- endif %}
{%- endfor %}
