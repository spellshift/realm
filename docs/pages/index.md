---
layout: page
title: Realm Docs
permalink: /
---

# Welcome to Realm!

## User Guide
Realm is a cross platform Red Team engagement platform with a focus on automation and reliability.

Want to get hands on now?  <a href="{{ '/user-guide/getting-started' | prepend: site.baseurl }}">Follow this guide to setup a dev instance of realm.</a>

![realm-logo](/assets/img/realm_create_quest.png)

<h5><a href="{{ '/user-guide/tavern' | prepend: site.baseurl }}">Server (tavern)</a></h5>
<ul>
<li>Web interface.</li>
<li>Group actions.</li>
<li>Graphql backend for easy API access.</li>
<li>OAuth login support.</li>
<li>Cloud native deployment with pre-made terraform for production deployments.</li>
</ul>

<h5><a href="{{ '/user-guide/imix' | prepend: site.baseurl }}">Agent (imix)</a></h5>
<ul>
<li>Written in rust with support for MacOS, Linux, and Windows.</li>
<li>Supports long running tasks by reading output from tasks in real time.</li>
<li>Interval callback times.</li>
<li>Simple file based configuration.</li>
<li>Embedded files.</li>
<li>Built-in interpreter.</li>
</ul>

<h5><a href="{{ '/user-guide/golem' | prepend: site.baseurl }}">Standalone interpreter (golem)</a></h5>
<ul>
<li>Interactive shell for testing hands-on testing.</li>
<li>Embedded files that execute autonomously on execuiton.</li>
</ul>

<h5><a href="{{ '/user-guide/eldritch' | prepend: site.baseurl }}">Built-in interpreter (eldritch)</a></h5>
<ul>
<li>Reflective DLL Loader.</li>
<li>Port scanning.</li>
<li>Remote execution over SSH.</li>
<li>And more.</li>
</ul>

## Developer Guide

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