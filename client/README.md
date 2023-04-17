<a name="readme-top"></a>

[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![Apache License][license-shield]][license-url]


<!-- PROJECT LOGO -->
<br />
<div align="center">
  <a href="https://github.com/mithril-security/blindai">
    <img src="https://github.com/mithril-security/blindai/raw/main/docs/assets/logo.png" alt="Logo" width="80" height="80">
  </a>

<h1 align="center">BlindAI</h1>

[![Website][website-shield]][website-url]
[![Blog][blog-shield]][blog-url]
[![LinkedIn][linkedin-shield]][linkedin-url]

  <p align="center">
    <b>BlindAI</b> is an <b>AI privacy solution</b>, allowing users to query popular AI models or serve their own models whilst ensuring that users' data remains private every step of the way.
	<br /><br />
    <a href="https://blindai.mithrilsecurity.io/en/latest"><strong>Explore the docs »</strong></a>
    <br />
    <br />
    <a href="https://blindai.mithrilsecurity.io/en/latest/docs/getting-started/quick-tour/">Try Demo</a>
    ·
    <a href="https://github.com/mithril-security/blindai/issues">Report Bug</a>
    ·
    <a href="https://github.com/mithril-security/blindai/issues">Request Feature</a>
  </p>
</div>



<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#-about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#-getting-started">Getting Started</a>
      <ul>
        <li><a href="#blindai-api">BlindAI API</a></li>
        <li><a href="#blindai-core">BlindAI Core</a></li>
      </ul>
    </li>
    <li><a href="#-usage">Usage</a></li>
    <li><a href="#-getting-help">Getting Help</a></li>
    <li><a href="#-license">License</a></li>
    <li><a href="#-contact">Contact</a></li>
  </ol>
</details>

<!-- ABOUT THE PROJECT -->
## 🔒 About The Project

**BlindAI** is an **open-source solution** to query and deploy AI models while **guaranteeing data privacy**. The querying of models is done via our **easy-to-use Python library**.

Data sent by users to the AI model is kept **confidential at all times** by hardware-enforced **Trusted Execution Environments**. We explain how they keep data and models safe in detail [here](https://blindai.mithrilsecurity.io/en/latest/docs/getting-started/confidential_computing/).

There are two main scenarios for BlindAI:

- **BlindAI API**: Using BlindAI to query popular AI models hosted by Mithril Security.
- **BlindAI Core**: Using BlindAI's underlying technology to host your own BlindAI server instance to securely deploy your own models.

You can find our more about BlindAI API and BlindAI Core [here](https://blindai.mithrilsecurity.io/en/latest/docs/getting-started/blindai_structure/).

### Built With 

[![Rust][Rust]][Rust-url] [![Python][Python]][Python-url] [![Intel-SGX][Intel-SGX]][Intel-sgx-url] [![Tract][Tract]][tract-url]

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- GETTING STARTED -->
## 🚀 Getting Started

We strongly recommend for you to get started with our [Quick tour](https://blindai.mithrilsecurity.io/en/latest/docs/getting-started/quick-tour/) to discover BlindAI with the open-source model Whisper.

But here’s a taste of what using BlindAI could look like 🍒

### BlindAI API

```py
transcript = blindai.api.Audio.transcribe(
    file="patient_104678.wav"
)
print(transcript)

The patient is a 55-year old male with known coronary artery disease.
```

### BlindAI.Core

#### AI company's side: uploading and deleting models

An AI company AI company want to provide their model as an an easy-to-use service. They upload it to the server, which is assigned a model ID.

```py
response = client_1.upload_model(model="./COVID-Net-CXR-2.onnx")
MODEL_ID = response.model_id
print(MODEL_ID)

8afcdab8-209e-4b93-9403-f3ea2dc0c3ae
```

When collaborating with clients is done, the AI company can delete their model from the server.

```py
# AI company deletes model after use
client_1.delete_model(MODEL_ID)
```

#### Client's side: running a model on confidential data

The client wants to feed their confidential data to the model while protecting it from third-party access. They connect and run the model on the following confidential image.

![](https://github.com/mithril-security/blindai/blob/main/docs/assets/positive_image.png)

```py
pos_ret = client_2.run_model(MODEL_ID, positive)
print("Probability of Covid for positive image is", pos_ret.output[0].as_flat()[0][1])

Probability of Covid for positive image is 0.890598714351654
```

_For more examples, please refer to the [Documentation](https://blindai.mithrilsecurity.io/en/latest/)_

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- GETTING HELP -->
## 🙋 Getting help

* Go to our [Discord](https://discord.com/invite/TxEHagpWd4) #support channel
* Report bugs by [opening an issue on our BlindAI GitHub](https://github.com/mithril-security/blindai/issues)
* [Book a meeting](https://calendly.com/contact-mithril-security/15mins?month=2023-03) with us


<!-- LICENSE -->
## 📜 License

Distributed under the Apache License, version 2.0. See [`LICENSE.md`](https://www.apache.org/licenses/LICENSE-2.0) for more information.


<!-- CONTACT -->
## 📇 Contact

Mithril Security - [@MithrilSecurity](https://twitter.com/MithrilSecurity) - contact@mithrilsecurity.io

Project Link: [https://github.com/mithril-security/blindai](https://github.com/mithril-security/blindai)

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://github.com/alexandresanlim/Badges4-README.md-Profile#-blog- -->
[contributors-shield]: https://img.shields.io/github/contributors/mithril-security/blindai.svg?style=for-the-badge
[contributors-url]: https://github.com/mithril-security/blindai/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/mithril-security/blindai.svg?style=for-the-badge
[forks-url]: https://github.com/mithril-security/blindai/network/members
[stars-shield]: https://img.shields.io/github/stars/mithril-security/blindai.svg?style=for-the-badge
[stars-url]: https://github.com/mithril-security/blindai/stargazers
[issues-shield]: https://img.shields.io/github/issues/mithril-security/blindai.svg?style=for-the-badge
[issues-url]: https://github.com/mithril-security/blindai/issues
[license-shield]: https://img.shields.io/github/license/mithril-security/blindai.svg?style=for-the-badge
[license-url]: https://github.com/mithril-security/blindai/blob/master/LICENSE.txt
[linkedin-shield]: https://img.shields.io/badge/-Jobs-black.svg?style=for-the-badge&logo=linkedin&colorB=555
[linkedin-url]: https://www.linkedin.com/company/mithril-security-company/
[website-url]: https://www.mithrilsecurity.io
[website-shield]: https://img.shields.io/badge/website-000000?style=for-the-badge&colorB=555
[blog-url]: https://blog.mithrilsecurity.io/
[blog-shield]: https://img.shields.io/badge/Blog-000?style=for-the-badge&logo=ghost&logoColor=yellow&colorB=555
[product-screenshot]: images/screenshot.png
[Python]: https://img.shields.io/badge/Python-FFD43B?style=for-the-badge&logo=python&logoColor=blue
[Python-url]: https://www.python.org/
[Rust]: https://img.shields.io/badge/rust-FFD43B?style=for-the-badge&logo=rust&logoColor=black
[Rust-url]: https://www.rust-lang.org/fr
[Intel-SGX]: https://img.shields.io/badge/SGX-FFD43B?style=for-the-badge&logo=intel&logoColor=black
[Intel-sgx-url]: https://www.intel.fr/content/www/fr/fr/architecture-and-technology/software-guard-extensions.html
[Tract]: https://img.shields.io/badge/Tract-FFD43B?style=for-the-badge
[tract-url]: https://github.com/mithril-security/tract/tree/6e4620659837eebeaba40ab3eeda67d33a99c7cf

<!-- Done using https://github.com/othneildrew/Best-README-Template -->
