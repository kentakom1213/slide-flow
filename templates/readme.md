# slides

[Slides List]({{ project.base_url }})

| Title | Slide | PDF | Description |
| :---- | :---: | :-: | :---------- |

{%- for slide in slides -%}
{% let description = slide.description|linebreaksbr %}
{%- if !slide.draft %}
| {{ slide.name }} | {% if slide.is_marp %}[Slide]({{ project.base_url }}{{ slide.slide_path }}){% if !slide.slide_version_paths.is_empty() %},{% endif %}{% for path in slide.slide_version_paths %}[v{{loop.index}}]({{ project.base_url }}{{ path }}){% if !loop.last %},{% endif %}{% endfor %}{% else %} - {% endif %} | [PDF]({{ project.base_url }}{{ slide.pdf_path }}),{% for path in slide.pdf_version_paths %}[v{{loop.index}}]({{ project.base_url }}{{ path }}){% if !loop.last %},{% endif %}{% endfor %} | {{ description }} |
{%- else %}
| {{ slide.name }} | - | - | {{ description }} |
{%- endif %}
{%- endfor %}
