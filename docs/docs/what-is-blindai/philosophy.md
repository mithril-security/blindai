# Philosophy of Mithril Security

Finding a privacy-by-design tool for AI can be difficult. We have decided to create Mithril Security because we thought that there was no accessible, fast and secure solution to answer the challenges of confidential AI training and deployment.

Having a tool by data scientists for data scientists is primordial to us, and our number one focus is to democratize AI privacy with an easy-to-use solution.

## Easy to use by data scientists
As you can see from the [Getting Started](../../docs/getting-started.md) sample, AI models can be made private in two lines of code, so that any data consumed by these models are never exposed in clear to any third party.

Providing continuity with data scientist current tools, like PyTorch, TensorFlow, ONNX, or Hugging Face Transformers is key for us. We want confidential AI to be as easy to set up as a regular AI deployment solution.

## Transparent security posture
We believe it is key to have transparency when designing security products. This is why we have decided to open source the core of BlindAI to provide scrutiny over our code. We do not believe in security by obscurity and prefer to build secure solutions in the open.

This is especially important for an enclave-based solution, where the code must be audited to fully trust the remote server handling the confidential data.

We provide an in-depth walkthrough of the threat model and security choices we have made in our Advanced security section, to provide all the resources needed by any security engineer to audit our solution.

## Fast and scalable
We think that speed is a key element for data engineers to adopt our solution. Security and privacy must not come at the price of limited performance and usability.

That is why we aim to provide the best tradeoff between security and speed. We provide our benchmarks in the Benchmarking section, with specific details about the slowdown induced by our solution, and how to reproduce the results.
Coverage of state-of-the-art AI models
Many Deep Learning frameworks leveraging Privacy Enhancing Technologies cover only toy models, like few-layers MLP or small CNNs like AlexNet.
We think AI practitioners are far beyond those toy examples and deploy today complex models like GPT, BERT, Whisper, etc. to answer society's current challenges.

That is why we aim to remain up to date with the latest advances in AI and provide an extensive list of models our framework can help make private.
