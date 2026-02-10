---
layout: page
title: Realm Docs
permalink: /
---

<!-- markdownlint-disable MD033 -->
<p align="center">
  <img src="/assets/img/realm_250px.png" alt="realm_250px.png"/>
</p>

## Welcome to Realm

[Realm](https://github.com/spellshift/realm) is an Adversary Emulation Framework with a **focus on scalability, reliability, and automation**. It is highly performant and is designed for engagements of any size (up to many thousands of beacons). [Get started in minutes](user-guide/getting-started).

### Feature Highlights

* **Focus on Reliability:** Realm always prioritizes quality over quantity, enabling operators to focus on the engagement instead of spending hours troubleshooting bugs. Extensive testing and rigorous code review ensure unwavering reliability, while an intuitive design and clear documentation keep the learning curve minimal. After reaching a stable `1.0.0` release, Realm will follow [Semantic Versioning](https://semver.org/), ensuring the stability of older deployments.

* **Eldritch, a Pythonic DSL for Offensive Security:** Ditch clunky scripting and embrace [Eldritch](/user-guide/eldritch), Realm's Pythonic [Domain Specific Language (DSL)](https://en.wikipedia.org/wiki/Domain-specific_language) based on [Google Starlark](https://github.com/bazelbuild/starlark/blob/master/spec.md#starlark-language-specification). Write clear, concise, reusable code that reflects your strategic thinking and streamlines offensive operations. [Eldritch](/user-guide/eldritch) is natively compiled to Rust, providing a performant abstraction for low-level system interactions.

* **Effortless Multi-Host Management:** Juggling tasks across numerous machines during complex engagements? Realm simplifies the process, enabling you to control agents on multiple hosts simultaneously.

* **Native GCP Integration:** Leverage the power and scalability of Google Cloud directly within your red team engagements. Realm seamlessly integrates with GCP services, boosting your attack capabilities without reinventing the wheel.

* **Stateless Server Architecture:** While Realm officially supports GCP, you may deploy its [stateless docker container](https://hub.docker.com/r/spellshift/tavern) to any environment that best fits your needs.

### Documentation Overview

Our three sections of documentation are split by intended audience, but feel free to explore!

* [User Guide](/user-guide) provides documentations for operators using Realm.
* [Admin Guide](/admin-guide) provides documentation for managing a Realm deployment.
* [Developer Guide](/dev-guide) provides documentation for Realm contributors.

### Contact Support

Need a hand? We're here to help! If you're facing an issue with Realm, we're happy to assist! To ensure we can provide the best support, please [create an issue on our GitHub](https://github.com/spellshift/realm/issues/new?labels=bug&template=bug_report.md).

#### Bug Support

When opening your issue, please include:

* A clear and concise description of the problem you're encountering.
* Any relevant error messages or logs.
* Steps to reproduce the issue (if possible).
* Impacted Realm version and operating system.

The more information you provide, the faster we can investigate and help you resolve the issue.

### Feature Requests & Feedback

Realm lives and breathes through its users. Your insights and experiences are crucial in guiding its development and ensuring it continues to empower your mission. Please don't hesitate to reach out!

**Remember:**

* Be respectful and constructive in your feedback ([code of conduct](https://github.com/spellshift/realm/blob/main/CODE_OF_CONDUCT.md)).
* Search for existing discussions or feature requests before creating new ones.
* The more details you provide, the better we can understand your needs and respond effectively.
* Together, we can shape Realm into an incredible framework. **Thank you for being part of the adventure!**

#### Feature Requests

Do you have an idea for a feature that would supercharge your workflow? We're all ears! [Open an issue on GitHub](https://github.com/spellshift/realm/issues/new?labels=feature&projects=&template=feature_request.md&title=%5Bfeature%5D+Something+to+do) and share your detailed proposal. Be sure to explain the problem you're facing, the solution you envision, and how it would benefit other users. The more information you provide, the better we can understand your needs and assess the feasibility of implementing your suggestion.

#### Provide Feedback

Love something about Realm? Feel something could be improved? Let us know! Your feedback, good or bad, helps us make Realm better for everyone. [Open an issue on GitHub](https://github.com/spellshift/realm/issues/new?labels=feedback&projects=&template=feedback.md&title=%5Bfeedback%5D+Something+to+improve) outlining your thoughts, whether it's a praiseworthy feature, a usability concern, or a suggestion for improvement. Every bit of your feedback helps us refine Realm and make it an even more valuable tool in your red teaming toolbox.
