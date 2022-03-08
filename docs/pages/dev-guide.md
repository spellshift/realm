---
layout: page
title: Developer Guide
permalink: /dev-guide/
---

# Developer Guide

Welcome to the {{ site.title }} Developer Guide! Here you can quickly jump to a particular page.

<div class="section-index">
    <hr class="panel-line">
    {% for post in site.docs  %}  
    <!-- Skip unrelated pages -->
        {% if post.url contains 'dev-guide' %}
        {% else %}
            {% continue %}
        {% endif %}         
    <div class="entry">
    <h5><a href="{{ post.url | prepend: site.baseurl }}">{{ post.title }}</a></h5>
    <p>{{ post.description }}</p>
    </div>{% endfor %}
</div>